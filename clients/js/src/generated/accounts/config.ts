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
  getI64Decoder,
  getI64Encoder,
  getStructDecoder,
  getStructEncoder,
  getU16Decoder,
  getU16Encoder,
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
  getNullableAddressDecoder,
  getNullableAddressEncoder,
  type NullableAddress,
  type NullableAddressArgs,
} from '../../hooked';

export type Config = {
  discriminator: Array<number>;
  authority: NullableAddress;
  slashAuthority: NullableAddress;
  vault: Address;
  cooldownTimeSeconds: bigint;
  tokenAmountDelegated: bigint;
  totalStakeRewards: bigint;
  maxDeactivationBasisPoints: number;
  padding: Array<number>;
};

export type ConfigArgs = {
  discriminator: Array<number>;
  authority: NullableAddressArgs;
  slashAuthority: NullableAddressArgs;
  vault: Address;
  cooldownTimeSeconds: number | bigint;
  tokenAmountDelegated: number | bigint;
  totalStakeRewards: number | bigint;
  maxDeactivationBasisPoints: number;
  padding: Array<number>;
};

export function getConfigEncoder(): Encoder<ConfigArgs> {
  return getStructEncoder([
    ['discriminator', getArrayEncoder(getU8Encoder(), { size: 8 })],
    ['authority', getNullableAddressEncoder()],
    ['slashAuthority', getNullableAddressEncoder()],
    ['vault', getAddressEncoder()],
    ['cooldownTimeSeconds', getI64Encoder()],
    ['tokenAmountDelegated', getU64Encoder()],
    ['totalStakeRewards', getU64Encoder()],
    ['maxDeactivationBasisPoints', getU16Encoder()],
    ['padding', getArrayEncoder(getU8Encoder(), { size: 6 })],
  ]);
}

export function getConfigDecoder(): Decoder<Config> {
  return getStructDecoder([
    ['discriminator', getArrayDecoder(getU8Decoder(), { size: 8 })],
    ['authority', getNullableAddressDecoder()],
    ['slashAuthority', getNullableAddressDecoder()],
    ['vault', getAddressDecoder()],
    ['cooldownTimeSeconds', getI64Decoder()],
    ['tokenAmountDelegated', getU64Decoder()],
    ['totalStakeRewards', getU64Decoder()],
    ['maxDeactivationBasisPoints', getU16Decoder()],
    ['padding', getArrayDecoder(getU8Decoder(), { size: 6 })],
  ]);
}

export function getConfigCodec(): Codec<ConfigArgs, Config> {
  return combineCodec(getConfigEncoder(), getConfigDecoder());
}

export function decodeConfig<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<Config, TAddress>;
export function decodeConfig<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<Config, TAddress>;
export function decodeConfig<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<Config, TAddress> | MaybeAccount<Config, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getConfigDecoder()
  );
}

export async function fetchConfig<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<Config, TAddress>> {
  const maybeAccount = await fetchMaybeConfig(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeConfig<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<Config, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeConfig(maybeAccount);
}

export async function fetchAllConfig(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<Config>[]> {
  const maybeAccounts = await fetchAllMaybeConfig(rpc, addresses, config);
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeConfig(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<Config>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) => decodeConfig(maybeAccount));
}

export function getConfigSize(): number {
  return 136;
}
