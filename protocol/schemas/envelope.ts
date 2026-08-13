import { z } from "zod";
import { stellarAddressSchema } from "./address";
import { hash32Schema } from "./hash";

const EncryptionMetadataSchema = z.object({
  algorithm: z.string(),
  ephemeral_public_key: stellarAddressSchema,
  nonce: z.string(),
  mac: z.string(),
});

const PayloadSchema = z.object({
  version: z.string(),
  sender: stellarAddressSchema,
  recipient: stellarAddressSchema,
  timestamp: z.string().datetime(),
  encryption_metadata: EncryptionMetadataSchema,
  content_commitment: hash32Schema,
  attachments: z.array(z.unknown()),
});

const SignatureSchema = z.object({
  scheme: z.string(),
  value: z.string(),
});

export const EnvelopeSchema = z.object({
  payload: PayloadSchema,
  signature: SignatureSchema,
});

export type Envelope = z.infer<typeof EnvelopeSchema>;
export type Payload = z.infer<typeof PayloadSchema>;
export type Signature = z.infer<typeof SignatureSchema>;
export type EncryptionMetadata = z.infer<typeof EncryptionMetadataSchema>;
