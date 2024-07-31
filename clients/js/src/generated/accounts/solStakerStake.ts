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
  getAddressDecoder,
  getAddressEncoder,
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

export type SolStakerStake = {
  discriminator: Array<number>;
  delegation: Delegation;
  lamportsAmount: bigint;
  stakeState: Address;
};

export type SolStakerStakeArgs = {
  discriminator: Array<number>;
  delegation: DelegationArgs;
  lamportsAmount: number | bigint;
  stakeState: Address;
};

export function getSolStakerStakeEncoder(): Encoder<SolStakerStakeArgs> {
  return getStructEncoder([
    ['discriminator', getArrayEncoder(getU8Encoder(), { size: 8 })],
    ['delegation', getDelegationEncoder()],
    ['lamportsAmount', getU64Encoder()],
    ['stakeState', getAddressEncoder()],
  ]);
}

export function getSolStakerStakeDecoder(): Decoder<SolStakerStake> {
  return getStructDecoder([
    ['discriminator', getArrayDecoder(getU8Decoder(), { size: 8 })],
    ['delegation', getDelegationDecoder()],
    ['lamportsAmount', getU64Decoder()],
    ['stakeState', getAddressDecoder()],
  ]);
}

export function getSolStakerStakeCodec(): Codec<
  SolStakerStakeArgs,
  SolStakerStake
> {
  return combineCodec(getSolStakerStakeEncoder(), getSolStakerStakeDecoder());
}

export function decodeSolStakerStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<SolStakerStake, TAddress>;
export function decodeSolStakerStake<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<SolStakerStake, TAddress>;
export function decodeSolStakerStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<SolStakerStake, TAddress> | MaybeAccount<SolStakerStake, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getSolStakerStakeDecoder()
  );
}

export async function fetchSolStakerStake<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<SolStakerStake, TAddress>> {
  const maybeAccount = await fetchMaybeSolStakerStake(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeSolStakerStake<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<SolStakerStake, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeSolStakerStake(maybeAccount);
}

export async function fetchAllSolStakerStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<SolStakerStake>[]> {
  const maybeAccounts = await fetchAllMaybeSolStakerStake(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeSolStakerStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<SolStakerStake>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeSolStakerStake(maybeAccount)
  );
}

export function getSolStakerStakeSize(): number {
  return 176;
}