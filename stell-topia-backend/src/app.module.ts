import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { FaresModule } from './fares/fares.module';
import { BookingModule } from './booking/booking.module';
import { AuthModule } from './auth/auth.module';
import { StellarModule } from './stellar/stellar.module';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true }),
    FaresModule,
    BookingModule,
    AuthModule,
    StellarModule,
  ],
})
export class AppModule {}




