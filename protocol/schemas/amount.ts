import { z } from "zod";

const MAX_I128 = "170141183460469231731687303715884105727";

export const stroopAmountSchema = z
  .string()
  .trim()
  .refine((val) => /^[0-9]+$/.test(val), {
    message: "Expected a non-negative integer string",
  })
  .refine((val) => !(val.length > 1 && val.startsWith("0")), {
    message: "Expected a non-negative integer string",
  })
  .refine((val) => val <= MAX_I128, {
    message: "Amount exceeds Soroban i128",
  })
  .transform((val) => BigInt(val));

export type StroopAmount = z.infer<typeof stroopAmountSchema>;
