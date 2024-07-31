/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  getArrayDecoder,
  getArrayEncoder,
  getStructDecoder,
  getStructEncoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  type Account,
  type Address,
  type Codec,
  type Decoder,
  type EncodedAccount,
  type Encoder,
  type FetchAccountConfig,
  type FetchAccountsConfig,
  type MaybeAccount,
  type MaybeEncodedAccount,
} from '@solana/web3.js';
import {
  getDelegationDecoder,
  getDelegationEncoder,
  type Delegation,
  type DelegationArgs,
} from '../types';

export type ValidatorStake = {
  discriminator: Array<number>;
  delegation: Delegation;
  totalStakedPalAmount: bigint;
  totalStakedLamportsAmount: bigint;
};

export type ValidatorStakeArgs = {
  discriminator: Array<number>;
  delegation: DelegationArgs;
  totalStakedPalAmount: number | bigint;
  totalStakedLamportsAmount: number | bigint;
};

export function getValidatorStakeEncoder(): Encoder<ValidatorStakeArgs> {
  return getStructEncoder([
    ['discriminator', getArrayEncoder(getU8Encoder(), { size: 8 })],
    ['delegation', getDelegationEncoder()],
    ['totalStakedPalAmount', getU64Encoder()],
    ['totalStakedLamportsAmount', getU64Encoder()],
  ]);
}

export function getValidatorStakeDecoder(): Decoder<ValidatorStake> {
  return getStructDecoder([
    ['discriminator', getArrayDecoder(getU8Decoder(), { size: 8 })],
    ['delegation', getDelegationDecoder()],
    ['totalStakedPalAmount', getU64Decoder()],
    ['totalStakedLamportsAmount', getU64Decoder()],
  ]);
}

export function getValidatorStakeCodec(): Codec<
  ValidatorStakeArgs,
  ValidatorStake
> {
  return combineCodec(getValidatorStakeEncoder(), getValidatorStakeDecoder());
}

export function decodeValidatorStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<ValidatorStake, TAddress>;
export function decodeValidatorStake<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<ValidatorStake, TAddress>;
export function decodeValidatorStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<ValidatorStake, TAddress> | MaybeAccount<ValidatorStake, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getValidatorStakeDecoder()
  );
}

export async function fetchValidatorStake<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<ValidatorStake, TAddress>> {
  const maybeAccount = await fetchMaybeValidatorStake(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeValidatorStake<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<ValidatorStake, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeValidatorStake(maybeAccount);
}

export async function fetchAllValidatorStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<ValidatorStake>[]> {
  const maybeAccounts = await fetchAllMaybeValidatorStake(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeValidatorStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<ValidatorStake>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeValidatorStake(maybeAccount)
  );
}

export function getValidatorStakeSize(): number {
  return 152;
}
