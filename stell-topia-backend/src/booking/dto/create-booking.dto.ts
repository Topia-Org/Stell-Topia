import { IsString, IsInt, IsNotEmpty, Min } from 'class-validator';

export class CreateBookingDto {
  @IsString()
  @IsNotEmpty()
  flightId: string;

  @IsString()
  @IsNotEmpty()
  from: string;

  @IsString()
  @IsNotEmpty()
  to: string;

  @IsString()
  @IsNotEmpty()
  departure_date: string;

  @IsInt()
  @Min(1)
  passengers: number;
}




