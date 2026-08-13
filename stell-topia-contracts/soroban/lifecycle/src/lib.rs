#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address,
    BytesN, Env, Symbol,
};
use stealth_policies::{PoliciesContractClient, PolicyDecision, PolicyReason};

#[cfg(feature = "contract")]
#[contract]
pub struct LifecycleContract;

#[cfg(not(feature = "contract"))]
#[contractclient(name = "LifecycleContractClient")]
pub trait LifecycleContractInterface {
    fn initialize(policies: Address, postage: Address, receipts: Address) -> Result<(), Error>;
    fn config() -> Result<LifecycleConfig, Error>;
    fn bind(
        message_id: BytesN<32>,
        owner: Address,
        sender: Address,
        recipient: Address,
        amount: i128,
        verified: bool,
        receipt_required: bool,
    ) -> Result<LifecycleRecord, Error>;
    fn verify_settle(message_id: BytesN<32>, postage: Postage) -> Result<LifecycleRecord, Error>;
    fn verify_refund(message_id: BytesN<32>, postage: Postage) -> Result<LifecycleRecord, Error>;
    fn verify_dispute(message_id: BytesN<32>, postage: Postage) -> Result<LifecycleRecord, Error>;
    fn verify_expire(message_id: BytesN<32>, postage: Postage) -> Result<LifecycleRecord, Error>;
    fn verify_reclaim(message_id: BytesN<32>, postage: Postage) -> Result<LifecycleRecord, Error>;
    fn verify_delivered(
        message_id: BytesN<32>,
        receipt: ReceiptState,
    ) -> Result<LifecycleRecord, Error>;
    fn verify_read(message_id: BytesN<32>, receipt: ReceiptState)
        -> Result<LifecycleRecord, Error>;
    fn get(message_id: BytesN<32>) -> Result<LifecycleRecord, Error>;
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleConfig {
    pub policies: Address,
    pub postage: Address,
    pub receipts: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Postage {
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub fee: i128,
    pub created_at: u64,
    pub expires_at: u64,
    pub dispute_until: u64,
    pub status: PostageStatus,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostageStatus {
    Pending,
    Expired,
    Disputed,
    Settled,
    Refunded,
    Reclaimed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptState {
    pub message_id: BytesN<32>,
    pub payload_hash: BytesN<32>,
    pub protocol_version: u32,
    pub sender: Address,
    pub recipient: Address,
    pub delivered_at: u64,
    pub read_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleTerminal {
    Open,
    Delivered,
    Read,
    Settled,
    Refunded,
    Disputed,
    Expired,
    Reclaimed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleRecord {
    pub message_id: BytesN<32>,
    pub owner: Address,
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub verified: bool,
    pub receipt_required: bool,
    pub policy_version: u32,
    pub decision_reason: PolicyReason,
    pub payload_hash: Option<BytesN<32>>,
    pub protocol_version: Option<u32>,
    pub delivered_at: Option<u64>,
    pub read_at: Option<u64>,
    pub terminal: LifecycleTerminal,
    pub bound_at: u64,
}

#[contractevent(topics = ["lifecycle"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleEvent {
    #[topic]
    pub action: Symbol,
    #[topic]
    pub message_id: BytesN<32>,
    pub record: LifecycleRecord,
}

#[contracttype]
enum DataKey {
    Config,
    Record(BytesN<32>),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    UnauthorizedContract = 3,
    PolicyRejected = 4,
    PolicyVersionMismatch = 5,
    PostageMismatch = 6,
    ReceiptMismatch = 7,
    MissingLifecycle = 8,
    TerminalStateMismatch = 9,
    DuplicateLifecycle = 10,
    AlreadyDelivered = 11,
    AlreadyRead = 12,
}

#[cfg(feature = "contract")]
#[contractimpl]
impl LifecycleContract {
    pub fn initialize(
        env: Env,
        policies: Address,
        postage: Address,
        receipts: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(
            &DataKey::Config,
            &LifecycleConfig {
                policies,
                postage,
                receipts,
            },
        );
        Ok(())
    }

    pub fn config(env: Env) -> Result<LifecycleConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bind(
        env: Env,
        message_id: BytesN<32>,
        owner: Address,
        sender: Address,
        recipient: Address,
        amount: i128,
        verified: bool,
        receipt_required: bool,
    ) -> Result<LifecycleRecord, Error> {
        if owner != recipient {
            return Err(Error::PostageMismatch);
        }
        sender.require_auth();

        let key = DataKey::Record(message_id.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::DuplicateLifecycle);
        }

        let decision = Self::evaluate_policy(
            &env,
            owner.clone(),
            sender.clone(),
            verified,
            amount,
            receipt_required,
        )?;

        let record = LifecycleRecord {
            message_id: message_id.clone(),
            owner,
            sender,
            recipient,
            amount,
            verified,
            receipt_required,
            policy_version: decision.version,
            decision_reason: decision.reason,
            payload_hash: None,
            protocol_version: None,
            delivered_at: None,
            read_at: None,
            terminal: LifecycleTerminal::Open,
            bound_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&key, &record);
        Self::publish_event(&env, symbol_short!("bind"), message_id, record.clone());
        Ok(record)
    }

    pub fn verify_settle(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
    ) -> Result<LifecycleRecord, Error> {
        Self::verify_terminal(env, message_id, postage, LifecycleTerminal::Settled)
    }

    pub fn verify_refund(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
    ) -> Result<LifecycleRecord, Error> {
        Self::verify_terminal(env, message_id, postage, LifecycleTerminal::Refunded)
    }

    pub fn verify_dispute(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
    ) -> Result<LifecycleRecord, Error> {
        if !matches!(
            postage.status,
            PostageStatus::Pending | PostageStatus::Expired
        ) {
            return Err(Error::PostageMismatch);
        }
        Self::verify_terminal(env, message_id, postage, LifecycleTerminal::Disputed)
    }

    pub fn verify_expire(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
    ) -> Result<LifecycleRecord, Error> {
        if postage.status != PostageStatus::Pending {
            return Err(Error::PostageMismatch);
        }
        Self::verify_terminal(env, message_id, postage, LifecycleTerminal::Expired)
    }

    pub fn verify_reclaim(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
    ) -> Result<LifecycleRecord, Error> {
        if matches!(
            postage.status,
            PostageStatus::Settled | PostageStatus::Refunded | PostageStatus::Reclaimed
        ) {
            return Err(Error::PostageMismatch);
        }
        Self::verify_terminal(env, message_id, postage, LifecycleTerminal::Reclaimed)
    }

    pub fn verify_delivered(
        env: Env,
        message_id: BytesN<32>,
        receipt: ReceiptState,
    ) -> Result<LifecycleRecord, Error> {
        Self::require_receipts_contract(&env)?;
        let mut record = Self::read_record(&env, &message_id)?;
        Self::assert_core_match(&record, &message_id, &receipt.sender, &receipt.recipient)?;
        Self::assert_receipt_match(&record, &receipt)?;

        if record.delivered_at.is_some() {
            return Err(Error::AlreadyDelivered);
        }
        if record.terminal == LifecycleTerminal::Read {
            return Err(Error::AlreadyRead);
        }
        if !matches!(
            record.terminal,
            LifecycleTerminal::Open | LifecycleTerminal::Delivered
        ) {
            return Err(Error::TerminalStateMismatch);
        }

        record.delivered_at = Some(receipt.delivered_at);
        record.payload_hash = Some(receipt.payload_hash.clone());
        record.protocol_version = Some(receipt.protocol_version);
        record.terminal = LifecycleTerminal::Delivered;

        env.storage()
            .persistent()
            .set(&DataKey::Record(message_id.clone()), &record);
        Self::publish_event(&env, symbol_short!("delivered"), message_id, record.clone());
        Ok(record)
    }

    pub fn verify_read(
        env: Env,
        message_id: BytesN<32>,
        receipt: ReceiptState,
    ) -> Result<LifecycleRecord, Error> {
        Self::require_receipts_contract(&env)?;
        let mut record = Self::read_record(&env, &message_id)?;
        Self::assert_core_match(&record, &message_id, &receipt.sender, &receipt.recipient)?;
        Self::assert_receipt_match(&record, &receipt)?;

        if record.delivered_at.is_none() {
            return Err(Error::TerminalStateMismatch);
        }
        if record.read_at.is_some() {
            return Err(Error::AlreadyRead);
        }
        if !matches!(
            record.terminal,
            LifecycleTerminal::Open | LifecycleTerminal::Delivered | LifecycleTerminal::Read
        ) {
            return Err(Error::TerminalStateMismatch);
        }

        record.read_at = Some(env.ledger().timestamp());
        record.terminal = LifecycleTerminal::Read;

        env.storage()
            .persistent()
            .set(&DataKey::Record(message_id.clone()), &record);
        Self::publish_event(&env, symbol_short!("read"), message_id, record.clone());
        Ok(record)
    }

    pub fn get(env: Env, message_id: BytesN<32>) -> Result<LifecycleRecord, Error> {
        Self::read_record(&env, &message_id)
    }

    fn verify_terminal(
        env: Env,
        message_id: BytesN<32>,
        postage: Postage,
        terminal: LifecycleTerminal,
    ) -> Result<LifecycleRecord, Error> {
        Self::require_postage_contract(&env)?;
        let mut record = Self::read_record(&env, &message_id)?;
        Self::assert_core_match(&record, &message_id, &postage.sender, &postage.recipient)?;
        if record.amount != postage.amount {
            return Err(Error::PostageMismatch);
        }
        if !Self::can_transition(record.terminal, terminal) {
            return Err(Error::TerminalStateMismatch);
        }
        if record.receipt_required && record.delivered_at.is_none() {
            return Err(Error::TerminalStateMismatch);
        }

        record.terminal = terminal;
        env.storage()
            .persistent()
            .set(&DataKey::Record(message_id.clone()), &record);
        Self::publish_event(
            &env,
            Self::terminal_symbol(terminal),
            message_id,
            record.clone(),
        );
        Ok(record)
    }

    fn can_transition(current: LifecycleTerminal, next: LifecycleTerminal) -> bool {
        match next {
            LifecycleTerminal::Settled => matches!(
                current,
                LifecycleTerminal::Open | LifecycleTerminal::Delivered | LifecycleTerminal::Read
            ),
            LifecycleTerminal::Refunded => matches!(
                current,
                LifecycleTerminal::Open
                    | LifecycleTerminal::Delivered
                    | LifecycleTerminal::Read
                    | LifecycleTerminal::Disputed
            ),
            LifecycleTerminal::Disputed => matches!(
                current,
                LifecycleTerminal::Open
                    | LifecycleTerminal::Delivered
                    | LifecycleTerminal::Read
                    | LifecycleTerminal::Expired
            ),
            LifecycleTerminal::Expired => matches!(
                current,
                LifecycleTerminal::Open | LifecycleTerminal::Delivered | LifecycleTerminal::Read
            ),
            LifecycleTerminal::Reclaimed => matches!(
                current,
                LifecycleTerminal::Open
                    | LifecycleTerminal::Delivered
                    | LifecycleTerminal::Read
                    | LifecycleTerminal::Expired
                    | LifecycleTerminal::Disputed
            ),
            LifecycleTerminal::Open | LifecycleTerminal::Delivered | LifecycleTerminal::Read => {
                false
            }
        }
    }

    fn evaluate_policy(
        env: &Env,
        owner: Address,
        sender: Address,
        verified: bool,
        postage: i128,
        receipt_required: bool,
    ) -> Result<PolicyDecision, Error> {
        let config = Self::read_config(env)?;
        let decision = PoliciesContractClient::new(env, &config.policies).evaluate(
            &owner,
            &sender,
            &verified,
            &postage,
            &receipt_required,
        );

        if !decision.allowed {
            return Err(Error::PolicyRejected);
        }
        if decision.required_postage > postage {
            return Err(Error::PolicyRejected);
        }
        Ok(decision)
    }

    fn read_config(env: &Env) -> Result<LifecycleConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn read_record(env: &Env, message_id: &BytesN<32>) -> Result<LifecycleRecord, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Record(message_id.clone()))
            .ok_or(Error::MissingLifecycle)
    }

    fn require_postage_contract(env: &Env) -> Result<(), Error> {
        let config = Self::read_config(env)?;
        config.postage.require_auth();
        Ok(())
    }

    fn require_receipts_contract(env: &Env) -> Result<(), Error> {
        let config = Self::read_config(env)?;
        config.receipts.require_auth();
        Ok(())
    }

    fn assert_core_match(
        record: &LifecycleRecord,
        message_id: &BytesN<32>,
        sender: &Address,
        recipient: &Address,
    ) -> Result<(), Error> {
        if record.message_id != *message_id
            || record.sender != *sender
            || record.recipient != *recipient
        {
            return Err(Error::PostageMismatch);
        }
        Ok(())
    }

    fn assert_receipt_match(record: &LifecycleRecord, receipt: &ReceiptState) -> Result<(), Error> {
        if record.message_id != receipt.message_id
            || record.sender != receipt.sender
            || record.recipient != receipt.recipient
        {
            return Err(Error::ReceiptMismatch);
        }
        if let Some(payload_hash) = &record.payload_hash {
            if payload_hash != &receipt.payload_hash {
                return Err(Error::ReceiptMismatch);
            }
        }
        if let Some(protocol_version) = record.protocol_version {
            if protocol_version != receipt.protocol_version {
                return Err(Error::ReceiptMismatch);
            }
        }
        Ok(())
    }

    fn publish_event(env: &Env, action: Symbol, message_id: BytesN<32>, record: LifecycleRecord) {
        LifecycleEvent {
            action,
            message_id,
            record,
        }
        .publish(env);
    }

    fn terminal_symbol(terminal: LifecycleTerminal) -> Symbol {
        match terminal {
            LifecycleTerminal::Open => symbol_short!("open"),
            LifecycleTerminal::Delivered => symbol_short!("delivered"),
            LifecycleTerminal::Read => symbol_short!("read"),
            LifecycleTerminal::Settled => symbol_short!("settle"),
            LifecycleTerminal::Refunded => symbol_short!("refund"),
            LifecycleTerminal::Disputed => symbol_short!("dispute"),
            LifecycleTerminal::Expired => symbol_short!("expire"),
            LifecycleTerminal::Reclaimed => symbol_short!("reclaim"),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events, Ledger},
        xdr::{ContractEventBody, ScSymbol, ScVal},
        TryFromVal, Val, Vec as SorobanVec,
    };
    use std::string::ToString;
    use stealth_policies::{MailboxPolicy, PoliciesContract, PoliciesContractClient};

    fn hash(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    fn configure_policies(env: &Env, owner: &Address) -> Address {
        let policies = env.register(PoliciesContract, ());
        let policies_client = PoliciesContractClient::new(env, &policies);
        policies_client.set_policy(
            owner,
            &MailboxPolicy {
                allow_unknown: true,
                require_verified: false,
                require_receipt: false,
                minimum_postage: 0,
            },
        );
        policies
    }

    fn setup() -> (Env, Address, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(42);
        env.ledger().set_sequence_number(10);

        let owner = Address::generate(&env);
        let sender = Address::generate(&env);
        // `bind` requires the mailbox owner to be the recipient (see the
        // `owner != recipient` guard), so valid test data must share the address.
        let recipient = owner.clone();

        let policies = configure_policies(&env, &owner);
        let postage = Address::generate(&env);
        let receipts = Address::generate(&env);

        let contract_id = env.register(LifecycleContract, ());
        let client = LifecycleContractClient::new(&env, &contract_id);
        client.initialize(&policies, &postage, &receipts);

        (env, contract_id, owner, sender, recipient, postage)
    }

    #[test]
    fn initialize_sets_config() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let policies = configure_policies(&env, &owner);
        let postage = Address::generate(&env);
        let receipts = Address::generate(&env);

        let contract_id = env.register(LifecycleContract, ());
        let client = LifecycleContractClient::new(&env, &contract_id);
        client.initialize(&policies, &postage, &receipts);

        let config = client.config();
        assert_eq!(config.policies, policies);
        assert_eq!(config.postage, postage);
        assert_eq!(config.receipts, receipts);
    }

    #[test]
    fn bind_creates_open_record() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        let record = client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );
        assert_eq!(record.message_id, message_id);
        assert_eq!(record.owner, owner);
        assert_eq!(record.sender, sender);
        assert_eq!(record.recipient, recipient);
        assert_eq!(record.amount, 100);
        assert!(record.verified);
        assert!(!record.receipt_required);
        assert_eq!(record.terminal, LifecycleTerminal::Open);
        assert_eq!(record.bound_at, 42);
    }

    #[test]
    fn bind_emits_event() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let events = env.events().all().filter_by_contract(&contract_id);
        assert_eq!(events.events().len(), 1);
        let body = &events.events()[0].body;
        let ContractEventBody::V0(v0) = body;
        assert_eq!(v0.topics.len(), 3);
        assert_eq!(
            v0.topics[0],
            ScVal::Symbol(ScSymbol("lifecycle".try_into().unwrap()))
        );
        assert_eq!(
            v0.topics[1],
            ScVal::Symbol(ScSymbol("bind".try_into().unwrap()))
        );
        assert_eq!(
            v0.topics[2],
            ScVal::Bytes(message_id.to_array().to_vec().try_into().unwrap())
        );
    }

    #[test]
    fn verify_delivered_transitions_to_delivered() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let receipt = ReceiptState {
            message_id: message_id.clone(),
            payload_hash: hash(&env, 2),
            protocol_version: 1,
            sender: sender.clone(),
            recipient: recipient.clone(),
            delivered_at: 100,
            read_at: None,
        };

        let record = client.verify_delivered(&message_id, &receipt);
        assert_eq!(record.terminal, LifecycleTerminal::Delivered);
        assert_eq!(record.delivered_at, Some(100));
    }

    #[test]
    fn verify_read_transitions_to_read() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let receipt = ReceiptState {
            message_id: message_id.clone(),
            payload_hash: hash(&env, 2),
            protocol_version: 1,
            sender: sender.clone(),
            recipient: recipient.clone(),
            delivered_at: 100,
            read_at: None,
        };

        client.verify_delivered(&message_id, &receipt);

        env.ledger().set_timestamp(200);
        let record = client.verify_read(&message_id, &receipt);
        assert_eq!(record.terminal, LifecycleTerminal::Read);
        assert_eq!(record.read_at, Some(200));
    }

    #[test]
    fn verify_settle_transitions_to_settled() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let postage = Postage {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: 100,
            fee: 10,
            created_at: 42,
            expires_at: 100,
            dispute_until: 100,
            status: PostageStatus::Pending,
        };

        let record = client.verify_settle(&message_id, &postage);
        assert_eq!(record.terminal, LifecycleTerminal::Settled);
    }

    #[test]
    fn duplicate_bind_fails() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );
        assert_eq!(
            client
                .try_bind(
                    &message_id,
                    &owner,
                    &sender,
                    &recipient,
                    &100,
                    &true,
                    &false
                )
                .unwrap_err()
                .unwrap(),
            Error::DuplicateLifecycle
        );
    }

    #[test]
    fn get_returns_record() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        let created = client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let fetched = client.get(&message_id);
        assert_eq!(fetched, created);
    }

    #[test]
    fn event_schema_is_stable() {
        let (env, contract_id, owner, sender, recipient, _) = setup();
        let client = LifecycleContractClient::new(&env, &contract_id);
        let message_id = hash(&env, 1);

        client.bind(
            &message_id,
            &owner,
            &sender,
            &recipient,
            &100,
            &true,
            &false,
        );

        let events = env.events().all().filter_by_contract(&contract_id);
        let body = &events.events()[0].body;
        let ContractEventBody::V0(v0) = body;

        assert_eq!(v0.topics.len(), 3);
        assert_eq!(
            v0.topics[0],
            ScVal::Symbol(ScSymbol("lifecycle".try_into().unwrap()))
        );
        assert_eq!(
            v0.topics[1],
            ScVal::Symbol(ScSymbol("bind".try_into().unwrap()))
        );
        assert_eq!(
            v0.topics[2],
            ScVal::Bytes(message_id.to_array().to_vec().try_into().unwrap())
        );
    }

    #[test]
    fn storage_keys_are_pinned() {
        let env = Env::default();
        let config_key = DataKey::Config;
        let record_key = DataKey::Record(hash(&env, 1));

        let config_val = Val::try_from_val(&env, &config_key).unwrap();
        let record_val = Val::try_from_val(&env, &record_key).unwrap();

        let config_vec: SorobanVec<Val> = SorobanVec::try_from_val(&env, &config_val).unwrap();
        let record_vec: SorobanVec<Val> = SorobanVec::try_from_val(&env, &record_val).unwrap();

        let config_symbol: Symbol =
            Symbol::try_from_val(&env, &config_vec.get(0).unwrap()).unwrap();
        let record_symbol: Symbol =
            Symbol::try_from_val(&env, &record_vec.get(0).unwrap()).unwrap();

        assert_eq!(config_symbol.to_string(), "Config");
        assert_eq!(record_symbol.to_string(), "Record");
    }
}
