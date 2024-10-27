/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
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

export type MoveSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountSolStakerAuthority extends string | IAccountMeta<string> = string,
  TAccountSourceSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountDestinationSolStakerStake extends
    | string
    | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? ReadonlyAccount<TAccountConfig>
        : TAccountConfig,
      TAccountVaultHolderRewards extends string
        ? ReadonlyAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      TAccountSolStakerAuthority extends string
        ? WritableAccount<TAccountSolStakerAuthority>
        : TAccountSolStakerAuthority,
      TAccountSourceSolStakerStake extends string
        ? WritableAccount<TAccountSourceSolStakerStake>
        : TAccountSourceSolStakerStake,
      TAccountDestinationSolStakerStake extends string
        ? WritableAccount<TAccountDestinationSolStakerStake>
        : TAccountDestinationSolStakerStake,
      ...TRemainingAccounts,
    ]
  >;

export type MoveSolStakerStakeInstructionData = {
  discriminator: number;
  args: bigint;
};

export type MoveSolStakerStakeInstructionDataArgs = { args: number | bigint };

export function getMoveSolStakerStakeInstructionDataEncoder(): Encoder<MoveSolStakerStakeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['args', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 16 })
  );
}

export function getMoveSolStakerStakeInstructionDataDecoder(): Decoder<MoveSolStakerStakeInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['args', getU64Decoder()],
  ]);
}

export function getMoveSolStakerStakeInstructionDataCodec(): Codec<
  MoveSolStakerStakeInstructionDataArgs,
  MoveSolStakerStakeInstructionData
> {
  return combineCodec(
    getMoveSolStakerStakeInstructionDataEncoder(),
    getMoveSolStakerStakeInstructionDataDecoder()
  );
}

export type MoveSolStakerStakeInput<
  TAccountConfig extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountSolStakerAuthority extends string = string,
  TAccountSourceSolStakerStake extends string = string,
  TAccountDestinationSolStakerStake extends string = string,
> = {
  /** Staking config */
  config: Address<TAccountConfig>;
  /** Vault holder rewards */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Sol staker authority */
  solStakerAuthority: Address<TAccountSolStakerAuthority>;
  /** Source sol staker stake */
  sourceSolStakerStake: Address<TAccountSourceSolStakerStake>;
  /** Destination sol staker stake */
  destinationSolStakerStake: Address<TAccountDestinationSolStakerStake>;
  args: MoveSolStakerStakeInstructionDataArgs['args'];
};

export function getMoveSolStakerStakeInstruction<
  TAccountConfig extends string,
  TAccountVaultHolderRewards extends string,
  TAccountSolStakerAuthority extends string,
  TAccountSourceSolStakerStake extends string,
  TAccountDestinationSolStakerStake extends string,
>(
  input: MoveSolStakerStakeInput<
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountSolStakerAuthority,
    TAccountSourceSolStakerStake,
    TAccountDestinationSolStakerStake
  >
): MoveSolStakerStakeInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVaultHolderRewards,
  TAccountSolStakerAuthority,
  TAccountSourceSolStakerStake,
  TAccountDestinationSolStakerStake
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: false,
    },
    solStakerAuthority: {
      value: input.solStakerAuthority ?? null,
      isWritable: true,
    },
    sourceSolStakerStake: {
      value: input.sourceSolStakerStake ?? null,
      isWritable: true,
    },
    destinationSolStakerStake: {
      value: input.destinationSolStakerStake ?? null,
      isWritable: true,
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
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.solStakerAuthority),
      getAccountMeta(accounts.sourceSolStakerStake),
      getAccountMeta(accounts.destinationSolStakerStake),
    ],
    programAddress,
    data: getMoveSolStakerStakeInstructionDataEncoder().encode(
      args as MoveSolStakerStakeInstructionDataArgs
    ),
  } as MoveSolStakerStakeInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountSolStakerAuthority,
    TAccountSourceSolStakerStake,
    TAccountDestinationSolStakerStake
  >;

  return instruction;
}

export type ParsedMoveSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Staking config */
    config: TAccountMetas[0];
    /** Vault holder rewards */
    vaultHolderRewards: TAccountMetas[1];
    /** Sol staker authority */
    solStakerAuthority: TAccountMetas[2];
    /** Source sol staker stake */
    sourceSolStakerStake: TAccountMetas[3];
    /** Destination sol staker stake */
    destinationSolStakerStake: TAccountMetas[4];
  };
  data: MoveSolStakerStakeInstructionData;
};

export function parseMoveSolStakerStakeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedMoveSolStakerStakeInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 5) {
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
      vaultHolderRewards: getNextAccount(),
      solStakerAuthority: getNextAccount(),
      sourceSolStakerStake: getNextAccount(),
      destinationSolStakerStake: getNextAccount(),
    },
    data: getMoveSolStakerStakeInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
