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
  getU128Decoder,
  getU128Encoder,
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
  getNullableU64Decoder,
  getNullableU64Encoder,
  type NullableU64,
  type NullableU64Args,
} from '../../hooked';

export type Stake = {
  discriminator: Array<number>;
  amount: bigint;
  deactivationTimestamp: NullableU64;
  deactivatingAmount: bigint;
  inactiveAmount: bigint;
  authority: Address;
  validator: Address;
  lastSeenHolderRewardsPerToken: bigint;
  lastSeenStakeRewardsPerToken: bigint;
};

export type StakeArgs = {
  discriminator: Array<number>;
  amount: number | bigint;
  deactivationTimestamp: NullableU64Args;
  deactivatingAmount: number | bigint;
  inactiveAmount: number | bigint;
  authority: Address;
  validator: Address;
  lastSeenHolderRewardsPerToken: number | bigint;
  lastSeenStakeRewardsPerToken: number | bigint;
};

export function getStakeEncoder(): Encoder<StakeArgs> {
  return getStructEncoder([
    ['discriminator', getArrayEncoder(getU8Encoder(), { size: 8 })],
    ['amount', getU64Encoder()],
    ['deactivationTimestamp', getNullableU64Encoder()],
    ['deactivatingAmount', getU64Encoder()],
    ['inactiveAmount', getU64Encoder()],
    ['authority', getAddressEncoder()],
    ['validator', getAddressEncoder()],
    ['lastSeenHolderRewardsPerToken', getU128Encoder()],
    ['lastSeenStakeRewardsPerToken', getU128Encoder()],
  ]);
}

export function getStakeDecoder(): Decoder<Stake> {
  return getStructDecoder([
    ['discriminator', getArrayDecoder(getU8Decoder(), { size: 8 })],
    ['amount', getU64Decoder()],
    ['deactivationTimestamp', getNullableU64Decoder()],
    ['deactivatingAmount', getU64Decoder()],
    ['inactiveAmount', getU64Decoder()],
    ['authority', getAddressDecoder()],
    ['validator', getAddressDecoder()],
    ['lastSeenHolderRewardsPerToken', getU128Decoder()],
    ['lastSeenStakeRewardsPerToken', getU128Decoder()],
  ]);
}

export function getStakeCodec(): Codec<StakeArgs, Stake> {
  return combineCodec(getStakeEncoder(), getStakeDecoder());
}

export function decodeStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<Stake, TAddress>;
export function decodeStake<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<Stake, TAddress>;
export function decodeStake<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<Stake, TAddress> | MaybeAccount<Stake, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getStakeDecoder()
  );
}

export async function fetchStake<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<Stake, TAddress>> {
  const maybeAccount = await fetchMaybeStake(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeStake<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<Stake, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeStake(maybeAccount);
}

export async function fetchAllStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<Stake>[]> {
  const maybeAccounts = await fetchAllMaybeStake(rpc, addresses, config);
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeStake(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<Stake>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) => decodeStake(maybeAccount));
}

export function getStakeSize(): number {
  return 120;
}
