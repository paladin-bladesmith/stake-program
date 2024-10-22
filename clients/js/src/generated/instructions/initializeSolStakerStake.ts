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

export type InitializeSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountValidatorStake extends string | IAccountMeta<string> = string,
  TAccountSolStakerNativeStake extends string | IAccountMeta<string> = string,
  TAccountSysvarStakeHistory extends
    | string
    | IAccountMeta<string> = 'SysvarStakeHistory1111111111111111111111111',
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TAccountSolStakeViewProgram extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? ReadonlyAccount<TAccountConfig>
        : TAccountConfig,
      TAccountSolStakerStake extends string
        ? WritableAccount<TAccountSolStakerStake>
        : TAccountSolStakerStake,
      TAccountValidatorStake extends string
        ? WritableAccount<TAccountValidatorStake>
        : TAccountValidatorStake,
      TAccountSolStakerNativeStake extends string
        ? ReadonlyAccount<TAccountSolStakerNativeStake>
        : TAccountSolStakerNativeStake,
      TAccountSysvarStakeHistory extends string
        ? ReadonlyAccount<TAccountSysvarStakeHistory>
        : TAccountSysvarStakeHistory,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      TAccountSolStakeViewProgram extends string
        ? ReadonlyAccount<TAccountSolStakeViewProgram>
        : TAccountSolStakeViewProgram,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeSolStakerStakeInstructionData = { discriminator: number };

export type InitializeSolStakerStakeInstructionDataArgs = {};

export function getInitializeSolStakerStakeInstructionDataEncoder(): Encoder<InitializeSolStakerStakeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 11 })
  );
}

export function getInitializeSolStakerStakeInstructionDataDecoder(): Decoder<InitializeSolStakerStakeInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getInitializeSolStakerStakeInstructionDataCodec(): Codec<
  InitializeSolStakerStakeInstructionDataArgs,
  InitializeSolStakerStakeInstructionData
> {
  return combineCodec(
    getInitializeSolStakerStakeInstructionDataEncoder(),
    getInitializeSolStakerStakeInstructionDataDecoder()
  );
}

export type InitializeSolStakerStakeInput<
  TAccountConfig extends string = string,
  TAccountSolStakerStake extends string = string,
  TAccountValidatorStake extends string = string,
  TAccountSolStakerNativeStake extends string = string,
  TAccountSysvarStakeHistory extends string = string,
  TAccountSystemProgram extends string = string,
  TAccountSolStakeViewProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`) */
  solStakerStake: Address<TAccountSolStakerStake>;
  /** Validator stake account (pda of `['stake::state::validator_stake', validator, config]`) */
  validatorStake: Address<TAccountValidatorStake>;
  /** Sol staker native stake */
  solStakerNativeStake: Address<TAccountSolStakerNativeStake>;
  /** Sysvar stake history */
  sysvarStakeHistory?: Address<TAccountSysvarStakeHistory>;
  /** System program */
  systemProgram?: Address<TAccountSystemProgram>;
  /** Paladin SOL Stake View program */
  solStakeViewProgram: Address<TAccountSolStakeViewProgram>;
};

export function getInitializeSolStakerStakeInstruction<
  TAccountConfig extends string,
  TAccountSolStakerStake extends string,
  TAccountValidatorStake extends string,
  TAccountSolStakerNativeStake extends string,
  TAccountSysvarStakeHistory extends string,
  TAccountSystemProgram extends string,
  TAccountSolStakeViewProgram extends string,
>(
  input: InitializeSolStakerStakeInput<
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountValidatorStake,
    TAccountSolStakerNativeStake,
    TAccountSysvarStakeHistory,
    TAccountSystemProgram,
    TAccountSolStakeViewProgram
  >
): InitializeSolStakerStakeInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountSolStakerStake,
  TAccountValidatorStake,
  TAccountSolStakerNativeStake,
  TAccountSysvarStakeHistory,
  TAccountSystemProgram,
  TAccountSolStakeViewProgram
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    solStakerStake: { value: input.solStakerStake ?? null, isWritable: true },
    validatorStake: { value: input.validatorStake ?? null, isWritable: true },
    solStakerNativeStake: {
      value: input.solStakerNativeStake ?? null,
      isWritable: false,
    },
    sysvarStakeHistory: {
      value: input.sysvarStakeHistory ?? null,
      isWritable: false,
    },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    solStakeViewProgram: {
      value: input.solStakeViewProgram ?? null,
      isWritable: false,
    },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.sysvarStakeHistory.value) {
    accounts.sysvarStakeHistory.value =
      'SysvarStakeHistory1111111111111111111111111' as Address<'SysvarStakeHistory1111111111111111111111111'>;
  }
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.solStakerStake),
      getAccountMeta(accounts.validatorStake),
      getAccountMeta(accounts.solStakerNativeStake),
      getAccountMeta(accounts.sysvarStakeHistory),
      getAccountMeta(accounts.systemProgram),
      getAccountMeta(accounts.solStakeViewProgram),
    ],
    programAddress,
    data: getInitializeSolStakerStakeInstructionDataEncoder().encode({}),
  } as InitializeSolStakerStakeInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountValidatorStake,
    TAccountSolStakerNativeStake,
    TAccountSysvarStakeHistory,
    TAccountSystemProgram,
    TAccountSolStakeViewProgram
  >;

  return instruction;
}

export type ParsedInitializeSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`) */
    solStakerStake: TAccountMetas[1];
    /** Validator stake account (pda of `['stake::state::validator_stake', validator, config]`) */
    validatorStake: TAccountMetas[2];
    /** Sol staker native stake */
    solStakerNativeStake: TAccountMetas[3];
    /** Sysvar stake history */
    sysvarStakeHistory: TAccountMetas[4];
    /** System program */
    systemProgram: TAccountMetas[5];
    /** Paladin SOL Stake View program */
    solStakeViewProgram: TAccountMetas[6];
  };
  data: InitializeSolStakerStakeInstructionData;
};

export function parseInitializeSolStakerStakeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeSolStakerStakeInstruction<TProgram, TAccountMetas> {
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
      solStakerStake: getNextAccount(),
      validatorStake: getNextAccount(),
      solStakerNativeStake: getNextAccount(),
      sysvarStakeHistory: getNextAccount(),
      systemProgram: getNextAccount(),
      solStakeViewProgram: getNextAccount(),
    },
    data: getInitializeSolStakerStakeInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
