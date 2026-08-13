import { z } from "zod";

export const stellarAddressSchema = z
  .string()
  .regex(/^G[A-Z2-7]{55}$/, "Expected a Stellar G-address");

export type StellarAddress = z.infer<typeof stellarAddressSchema>;
