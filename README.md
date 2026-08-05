

## ✈️ What is Stell-Topia?

**Stell-Topia** is a decentralized flight-booking protocol built on the **Stellar network**. It lets travelers search, reserve, and pay for plane tickets directly in **XLM** — no card fees, no cross-border friction, no middleman holding your money hostage between "booking" and "confirmed."

Think of it as the airport check-in counter, rebuilt as a smart contract: fast settlement, transparent pricing, and a ledger that never loses your seat.

## 🪐 Why Stellar?

Stellar was built for cross-border value movement — which is exactly what buying a flight already is. Sub-5-second finality, near-zero fees, and native asset support make XLM (and Stellar-issued stablecoins) a natural fit for a ticket that might be bought in Lagos and flown out of Lisbon.

## 🧭 Core Features

- **Book with XLM** — reserve and pay for flights natively in Lumens, priced live against fiat routes.
- **Escrow-style holds** — funds are locked in a Soroban contract on booking and only released to the carrier pool on confirmed issuance; automatic refund path on failure.
- **Verifiable itineraries** — every booking reference is anchored on-chain, so a ticket's status (`PENDING → CONFIRMED → ISSUED`) can't be quietly rewritten.
- **Wallet-first auth** — connect a Stellar wallet (Freighter, Albedo, etc.) instead of creating yet another password.
- **Fare aggregation layer** — the Python API stitches together external fare/inventory sources into a single searchable feed.

## 🏗️ Architecture

```
                ┌────────────────────┐
                │   Client / Wallet   │
                └─────────┬───────────┘
                          │
                ┌─────────▼───────────┐
                │   NestJS Gateway     │   auth, booking orchestration,
                │   (TypeScript)       │   request validation, webhooks
                └─────────┬───────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                                   │
┌───────▼────────┐                ┌─────────▼──────────┐
│   Python API     │                │  Soroban Contract   │
│  fare search &    │◄──────────────►│  (Rust)             │
│  inventory sync    │   tx payload   │  escrow · booking   │
└────────────────┘                │  state · settlement  │
                                    └─────────┬──────────┘
                                              │
                                    ┌─────────▼──────────┐
                                    │   Stellar Network    │
                                    │   (Horizon / RPC)    │
                                    └───────────────────┘
```

## 🛠️ Tech Stack

| Layer                | Technology                          |
|-----------------------|--------------------------------------|
| Fare & inventory API  | Python (FastAPI)                     |
| Booking orchestration | NestJS (TypeScript)                  |
| Smart contracts       | Rust (Soroban on Stellar)            |
| Ledger / settlement   | Stellar Network (XLM)                |
| Wallets               | Freighter, Albedo, WalletConnect      |

## 🚀 Getting Started

```bash
# 1. Clone the repo
git clone https://github.com/<your-org>/stell-topia.git
cd stell-topia

# 2. Spin up the Python fare API
cd services/fare-api
pip install -r requirements.txt
uvicorn main:app --reload

# 3. Start the NestJS gateway
cd ../gateway
npm install
npm run start:dev

# 4. Build & deploy the Soroban contract
cd ../../contracts/booking
soroban contract build
soroban contract deploy --network testnet
```

> Point the gateway's `.env` at your deployed contract ID and a Stellar Horizon/RPC endpoint (testnet to start).

## 🗺️ Roadmap

- [ ] Testnet booking flow (search → hold → pay → issue)
- [ ] Multi-carrier fare aggregation
- [ ] Stellar-issued stablecoin support alongside native XLM
- [ ] Refund/dispute contract logic
- [ ] Mainnet audit + launch

## 🤝 Contributing

Issues and PRs are welcome. Open a discussion first for anything that touches the contract's escrow logic — that part flies (pun intended) with real money.

## 📄 License

MIT — build on it, fork it, take it further.

---

<p align="center"><sub>Stell-Topia — where every itinerary starts on the ledger.</sub></p>
