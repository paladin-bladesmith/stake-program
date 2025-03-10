/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getAddressDecoder,
  getAddressEncoder,
  getStructDecoder,
  getStructEncoder,
  getU16Decoder,
  getU16Encoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type InitializeConfigInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountVault extends string
        ? ReadonlyAccount<TAccountVault>
        : TAccountVault,
      TAccountVaultHolderRewards extends string
        ? ReadonlyAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeConfigInstructionData = {
  discriminator: number;
  slashAuthority: Address;
  configAuthority: Address;
  cooldownTimeSeconds: bigint;
  maxDeactivationBasisPoints: number;
  syncRewardsLamports: bigint;
};

export type InitializeConfigInstructionDataArgs = {
  slashAuthority: Address;
  configAuthority: Address;
  cooldownTimeSeconds: number | bigint;
  maxDeactivationBasisPoints: number;
  syncRewardsLamports: number | bigint;
};

export function getInitializeConfigInstructionDataEncoder(): Encoder<InitializeConfigInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['slashAuthority', getAddressEncoder()],
      ['configAuthority', getAddressEncoder()],
      ['cooldownTimeSeconds', getU64Encoder()],
      ['maxDeactivationBasisPoints', getU16Encoder()],
      ['syncRewardsLamports', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 0 })
  );
}

export function getInitializeConfigInstructionDataDecoder(): Decoder<InitializeConfigInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['slashAuthority', getAddressDecoder()],
    ['configAuthority', getAddressDecoder()],
    ['cooldownTimeSeconds', getU64Decoder()],
    ['maxDeactivationBasisPoints', getU16Decoder()],
    ['syncRewardsLamports', getU64Decoder()],
  ]);
}

export function getInitializeConfigInstructionDataCodec(): Codec<
  InitializeConfigInstructionDataArgs,
  InitializeConfigInstructionData
> {
  return combineCodec(
    getInitializeConfigInstructionDataEncoder(),
    getInitializeConfigInstructionDataDecoder()
  );
}

export type InitializeConfigInput<
  TAccountConfig extends string = string,
  TAccountMint extends string = string,
  TAccountVault extends string = string,
  TAccountVaultHolderRewards extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Stake token mint */
  mint: Address<TAccountMint>;
  /** Stake vault token account */
  vault: Address<TAccountVault>;
  /** Stake vault holder rewards account */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  slashAuthority: InitializeConfigInstructionDataArgs['slashAuthority'];
  configAuthority: InitializeConfigInstructionDataArgs['configAuthority'];
  cooldownTimeSeconds: InitializeConfigInstructionDataArgs['cooldownTimeSeconds'];
  maxDeactivationBasisPoints: InitializeConfigInstructionDataArgs['maxDeactivationBasisPoints'];
  syncRewardsLamports: InitializeConfigInstructionDataArgs['syncRewardsLamports'];
};

export function getInitializeConfigInstruction<
  TAccountConfig extends string,
  TAccountMint extends string,
  TAccountVault extends string,
  TAccountVaultHolderRewards extends string,
>(
  input: InitializeConfigInput<
    TAccountConfig,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards
  >
): InitializeConfigInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountMint,
  TAccountVault,
  TAccountVaultHolderRewards
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    mint: { value: input.mint ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: false },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: false,
    },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultHolderRewards),
    ],
    programAddress,
    data: getInitializeConfigInstructionDataEncoder().encode(
      args as InitializeConfigInstructionDataArgs
    ),
  } as InitializeConfigInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards
  >;

  return instruction;
}

export type ParsedInitializeConfigInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Stake token mint */
    mint: TAccountMetas[1];
    /** Stake vault token account */
    vault: TAccountMetas[2];
    /** Stake vault holder rewards account */
    vaultHolderRewards: TAccountMetas[3];
  };
  data: InitializeConfigInstructionData;
};

export function parseInitializeConfigInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeConfigInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 4) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      config: getNextAccount(),
      mint: getNextAccount(),
      vault: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
    },
    data: getInitializeConfigInstructionDataDecoder().decode(instruction.data),
  };
}
