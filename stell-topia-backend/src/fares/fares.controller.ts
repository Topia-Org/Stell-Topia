import { Controller, Get, Param, Query, UseGuards } from '@nestjs/common';
import { FaresService } from './fares.service';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';

@Controller('api/v1/fares')
@UseGuards(JwtAuthGuard)
export class FaresController {
  constructor(private readonly faresService: FaresService) {}

  @Get('search')
  search(@Query() query: Record<string, any>) {
    return this.faresService.search(query);
  }
}




