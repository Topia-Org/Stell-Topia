import { Injectable } from '@nestjs/common';
import { FaresService } from '../fares/fares.service';
import { StellarService } from '../stellar/stellar.service';

@Injectable()
export class BookingService {
  constructor(
    private readonly faresService: FaresService,
    private readonly stellarService: StellarService,
  ) {}

  async create(createBookingDto: Record<string, any>, user: any) {
    const flight = await this.faresService.search({
      from: createBookingDto.from,
      to: createBookingDto.to,
      departure_date: createBookingDto.departure_date,
      passengers: createBookingDto.passengers,
    });

    const selectedFlight = flight.data[0];
    if (!selectedFlight) {
      throw new Error('No flights available');
    }

    const txPayload = await this.stellarService.prepareEscrowTx({
      flightId: selectedFlight.id,
      amount: selectedFlight.price_xlm,
      passenger: user.sub,
    });

    return {
      flight: selectedFlight,
      txPayload,
      expiresAt: new Date(Date.now() + 5 * 60 * 1000).toISOString(),
    };
  }

  findOne(id: string) {
    return {
      id,
      status: 'PENDING',
      createdAt: new Date().toISOString(),
    };
  }
}




