import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';

@Injectable()
export class AuthService {
  constructor(private readonly jwtService: JwtService) {}

  async validateUser(email: string, password: string): Promise<any> {
    if (email === 'demo@example.com' && password === 'demopass') {
      return { email, sub: email };
    }
    return null;
  }

  async login(user: any) {
    const payload = { email: user.email, sub: user.sub };
    return {
      access_token: this.jwtService.sign(payload),
      token_type: 'bearer',
      expires_in: 3600,
    };
  }
}



