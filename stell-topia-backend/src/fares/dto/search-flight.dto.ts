import { IsString, IsOptional, IsInt, Min, Max } from 'class-validator';

export class SearchFlightDto {
  @IsString()
  from: string;

  @IsString()
  to: string;

  @IsString()
  departure_date: string;

  @IsOptional()
  @IsString()
  return_date?: string;

  @IsInt()
  @Min(1)
  @Max(9)
  passengers: number;
}



