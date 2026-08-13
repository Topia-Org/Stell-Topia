import { Injectable } from '@nestjs/common';

@Injectable()
export class StellarService {
  private readonly horizonUrl: string;
  private readonly contractId: string | null;

  constructor() {
    this.horizonUrl = process.env.STELLAR_HORIZON_URL || 'https://horizon.stellar.org';
    this.contractId = process.env.SOROBAN_CONTRACT_ID || null;
  }

  async prepareEscrowTx(payload: Record<string, any>) {
    return {
      networkPassphrase: process.env.STELLAR_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015',
      contractId: this.contractId,
      function: 'create_booking',
      args: payload,
    };
  }
}




