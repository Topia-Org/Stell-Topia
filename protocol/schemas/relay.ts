import { z } from "zod";

export const RelayRequestSchema = z.object({
  amount: z.string(),
  messageId: z.string(),
  paymentHash: z.string(),
  recipient: z.string(),
  sender: z.string(),
});

export type RelayRequest = z.infer<typeof RelayRequestSchema>;
