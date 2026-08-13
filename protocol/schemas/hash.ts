import { z } from "zod";

export const hash32Schema = z
  .string()
  .transform((val) => val.toLowerCase().trim())
  .refine((val) => val.length === 64, {
    message: "Expected a 32-byte lowercase hexadecimal hash",
  })
  .refine((val) => /^[a-f0-9]+$/.test(val), {
    message: "Expected a 32-byte lowercase hexadecimal hash",
  });

export type Hash32 = z.infer<typeof hash32Schema>;
