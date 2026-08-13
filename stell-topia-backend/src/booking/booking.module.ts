import { Module } from '@nestjs/common';
import { BookingService } from './booking.service';
import { BookingController } from './booking.controller';
import { FaresModule } from '../fares/fares.module';
import { StellarModule } from '../stellar/stellar.module';
import { AuthModule } from '../auth/auth.module';

@Module({
  imports: [FaresModule, StellarModule, AuthModule],
  controllers: [BookingController],
  providers: [BookingService],
  exports: [BookingService],
})
export class BookingModule {}



