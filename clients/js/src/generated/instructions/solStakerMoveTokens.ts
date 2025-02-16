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
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type SolStakerMoveTokensInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountSolStakerAuthority extends string | IAccountMeta<string> = string,
  TAccountSourceValidatorStake extends string | IAccountMeta<string> = string,
  TAccountSourceSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountDestinationValidatorStake extends
    | string
    | IAccountMeta<string> = string,
  TAccountDestinationSolStakerStake extends
    | string
    | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountVaultHolderRewards extends string
        ? ReadonlyAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      TAccountSolStakerAuthority extends string
        ? ReadonlySignerAccount<TAccountSolStakerAuthority> &
            IAccountSignerMeta<TAccountSolStakerAuthority>
        : TAccountSolStakerAuthority,
      TAccountSourceValidatorStake extends string
        ? WritableAccount<TAccountSourceValidatorStake>
        : TAccountSourceValidatorStake,
      TAccountSourceSolStakerStake extends string
        ? WritableAccount<TAccountSourceSolStakerStake>
        : TAccountSourceSolStakerStake,
      TAccountDestinationValidatorStake extends string
        ? WritableAccount<TAccountDestinationValidatorStake>
        : TAccountDestinationValidatorStake,
      TAccountDestinationSolStakerStake extends string
        ? WritableAccount<TAccountDestinationSolStakerStake>
        : TAccountDestinationSolStakerStake,
      ...TRemainingAccounts,
    ]
  >;

export type SolStakerMoveTokensInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type SolStakerMoveTokensInstructionDataArgs = {
  amount: number | bigint;
};

export function getSolStakerMoveTokensInstructionDataEncoder(): Encoder<SolStakerMoveTokensInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 13 })
  );
}

export function getSolStakerMoveTokensInstructionDataDecoder(): Decoder<SolStakerMoveTokensInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getSolStakerMoveTokensInstructionDataCodec(): Codec<
  SolStakerMoveTokensInstructionDataArgs,
  SolStakerMoveTokensInstructionData
> {
  return combineCodec(
    getSolStakerMoveTokensInstructionDataEncoder(),
    getSolStakerMoveTokensInstructionDataDecoder()
  );
}

export type SolStakerMoveTokensInput<
  TAccountConfig extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountSolStakerAuthority extends string = string,
  TAccountSourceValidatorStake extends string = string,
  TAccountSourceSolStakerStake extends string = string,
  TAccountDestinationValidatorStake extends string = string,
  TAccountDestinationSolStakerStake extends string = string,
> = {
  /** Staking config */
  config: Address<TAccountConfig>;
  /** Vault holder rewards */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Sol staker authority */
  solStakerAuthority: TransactionSigner<TAccountSolStakerAuthority>;
  /** Source validator stake */
  sourceValidatorStake: Address<TAccountSourceValidatorStake>;
  /** Source sol staker stake */
  sourceSolStakerStake: Address<TAccountSourceSolStakerStake>;
  /** Destination validator stake */
  destinationValidatorStake: Address<TAccountDestinationValidatorStake>;
  /** Destination sol staker stake */
  destinationSolStakerStake: Address<TAccountDestinationSolStakerStake>;
  amount: SolStakerMoveTokensInstructionDataArgs['amount'];
};

export function getSolStakerMoveTokensInstruction<
  TAccountConfig extends string,
  TAccountVaultHolderRewards extends string,
  TAccountSolStakerAuthority extends string,
  TAccountSourceValidatorStake extends string,
  TAccountSourceSolStakerStake extends string,
  TAccountDestinationValidatorStake extends string,
  TAccountDestinationSolStakerStake extends string,
>(
  input: SolStakerMoveTokensInput<
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountSolStakerAuthority,
    TAccountSourceValidatorStake,
    TAccountSourceSolStakerStake,
    TAccountDestinationValidatorStake,
    TAccountDestinationSolStakerStake
  >
): SolStakerMoveTokensInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVaultHolderRewards,
  TAccountSolStakerAuthority,
  TAccountSourceValidatorStake,
  TAccountSourceSolStakerStake,
  TAccountDestinationValidatorStake,
  TAccountDestinationSolStakerStake
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: false,
    },
    solStakerAuthority: {
      value: input.solStakerAuthority ?? null,
      isWritable: false,
    },
    sourceValidatorStake: {
      value: input.sourceValidatorStake ?? null,
      isWritable: true,
    },
    sourceSolStakerStake: {
      value: input.sourceSolStakerStake ?? null,
      isWritable: true,
    },
    destinationValidatorStake: {
      value: input.destinationValidatorStake ?? null,
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
      getAccountMeta(accounts.sourceValidatorStake),
      getAccountMeta(accounts.sourceSolStakerStake),
      getAccountMeta(accounts.destinationValidatorStake),
      getAccountMeta(accounts.destinationSolStakerStake),
    ],
    programAddress,
    data: getSolStakerMoveTokensInstructionDataEncoder().encode(
      args as SolStakerMoveTokensInstructionDataArgs
    ),
  } as SolStakerMoveTokensInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountSolStakerAuthority,
    TAccountSourceValidatorStake,
    TAccountSourceSolStakerStake,
    TAccountDestinationValidatorStake,
    TAccountDestinationSolStakerStake
  >;

  return instruction;
}

export type ParsedSolStakerMoveTokensInstruction<
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
    /** Source validator stake */
    sourceValidatorStake: TAccountMetas[3];
    /** Source sol staker stake */
    sourceSolStakerStake: TAccountMetas[4];
    /** Destination validator stake */
    destinationValidatorStake: TAccountMetas[5];
    /** Destination sol staker stake */
    destinationSolStakerStake: TAccountMetas[6];
  };
  data: SolStakerMoveTokensInstructionData;
};

export function parseSolStakerMoveTokensInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSolStakerMoveTokensInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 7) {
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
      sourceValidatorStake: getNextAccount(),
      sourceSolStakerStake: getNextAccount(),
      destinationValidatorStake: getNextAccount(),
      destinationSolStakerStake: getNextAccount(),
    },
    data: getSolStakerMoveTokensInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
