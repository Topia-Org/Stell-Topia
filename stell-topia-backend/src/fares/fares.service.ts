import { Injectable } from '@nestjs/common';
import { HttpService } from '@nestjs/axios';
import { firstValueFrom } from 'rxjs';

@Injectable()
export class FaresService {
  private readonly pythonApiUrl: string;

  constructor(private readonly httpService: HttpService) {
    this.pythonApiUrl = process.env.PYTHON_API_URL || 'http://localhost:8000/api/v1';
  }

  async search(query: Record<string, any>) {
    const url = `${this.pythonApiUrl}/flights/search`;
    const response = await firstValueFrom(
      this.httpService.post(url, query, {
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    return response.data as any;
  }
}




