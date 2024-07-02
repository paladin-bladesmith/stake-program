import { combineCodec, createDecoder, createEncoder } from '@solana/web3.js';
import { getU64Decoder, getU64Encoder } from '@solana/web3.js';

export type NullableU64 = number | bigint | null;
export type NullableU64Args = NullableU64;

const DEFAULT_VALUE = BigInt(0);

export const getNullableU64Encoder = () =>
  createEncoder<NullableU64>({
    fixedSize: 8,
    write(value, bytes, offset) {
      if (value === null) {
        bytes.set(getU64Encoder().encode(DEFAULT_VALUE), offset);
      } else {
        bytes.set(getU64Encoder().encode(value), offset);
      }
      return offset + 8;
    },
  });

export const getNullableU64Decoder = () =>
  createDecoder<NullableU64>({
    fixedSize: 32,
    read(bytes, offset) {
      if (getU64Decoder().decode(bytes, offset) === DEFAULT_VALUE) {
        return [null, offset + 8];
      } else {
        return [getU64Decoder().decode(bytes, offset), offset + 8];
      }
    },
  });

export const getNullableU64Codec = () =>
  combineCodec(getNullableU64Encoder(), getNullableU64Decoder());
