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
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type ValidatorOverrideStakedLamportsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountConfigAuthority extends string | IAccountMeta<string> = string,
  TAccountValidatorStake extends string | IAccountMeta<string> = string,
  TAccountValidatorStakeAuthority extends
    | string
    | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountConfigAuthority extends string
        ? ReadonlySignerAccount<TAccountConfigAuthority> &
            IAccountSignerMeta<TAccountConfigAuthority>
        : TAccountConfigAuthority,
      TAccountValidatorStake extends string
        ? WritableAccount<TAccountValidatorStake>
        : TAccountValidatorStake,
      TAccountValidatorStakeAuthority extends string
        ? WritableAccount<TAccountValidatorStakeAuthority>
        : TAccountValidatorStakeAuthority,
      TAccountVaultHolderRewards extends string
        ? WritableAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      ...TRemainingAccounts,
    ]
  >;

export type ValidatorOverrideStakedLamportsInstructionData = {
  discriminator: number;
  amountMin: bigint;
};

export type ValidatorOverrideStakedLamportsInstructionDataArgs = {
  amountMin: number | bigint;
};

export function getValidatorOverrideStakedLamportsInstructionDataEncoder(): Encoder<ValidatorOverrideStakedLamportsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amountMin', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 16 })
  );
}

export function getValidatorOverrideStakedLamportsInstructionDataDecoder(): Decoder<ValidatorOverrideStakedLamportsInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amountMin', getU64Decoder()],
  ]);
}

export function getValidatorOverrideStakedLamportsInstructionDataCodec(): Codec<
  ValidatorOverrideStakedLamportsInstructionDataArgs,
  ValidatorOverrideStakedLamportsInstructionData
> {
  return combineCodec(
    getValidatorOverrideStakedLamportsInstructionDataEncoder(),
    getValidatorOverrideStakedLamportsInstructionDataDecoder()
  );
}

export type ValidatorOverrideStakedLamportsInput<
  TAccountConfig extends string = string,
  TAccountConfigAuthority extends string = string,
  TAccountValidatorStake extends string = string,
  TAccountValidatorStakeAuthority extends string = string,
  TAccountVaultHolderRewards extends string = string,
> = {
  /** Config */
  config: Address<TAccountConfig>;
  /** Config authority */
  configAuthority: TransactionSigner<TAccountConfigAuthority>;
  /** Validator stake */
  validatorStake: Address<TAccountValidatorStake>;
  /** Validator stake authority */
  validatorStakeAuthority: Address<TAccountValidatorStakeAuthority>;
  /** Vault holder rewards */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  amountMin: ValidatorOverrideStakedLamportsInstructionDataArgs['amountMin'];
};

export function getValidatorOverrideStakedLamportsInstruction<
  TAccountConfig extends string,
  TAccountConfigAuthority extends string,
  TAccountValidatorStake extends string,
  TAccountValidatorStakeAuthority extends string,
  TAccountVaultHolderRewards extends string,
>(
  input: ValidatorOverrideStakedLamportsInput<
    TAccountConfig,
    TAccountConfigAuthority,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority,
    TAccountVaultHolderRewards
  >
): ValidatorOverrideStakedLamportsInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountConfigAuthority,
  TAccountValidatorStake,
  TAccountValidatorStakeAuthority,
  TAccountVaultHolderRewards
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    configAuthority: {
      value: input.configAuthority ?? null,
      isWritable: false,
    },
    validatorStake: { value: input.validatorStake ?? null, isWritable: true },
    validatorStakeAuthority: {
      value: input.validatorStakeAuthority ?? null,
      isWritable: true,
    },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
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
      getAccountMeta(accounts.configAuthority),
      getAccountMeta(accounts.validatorStake),
      getAccountMeta(accounts.validatorStakeAuthority),
      getAccountMeta(accounts.vaultHolderRewards),
    ],
    programAddress,
    data: getValidatorOverrideStakedLamportsInstructionDataEncoder().encode(
      args as ValidatorOverrideStakedLamportsInstructionDataArgs
    ),
  } as ValidatorOverrideStakedLamportsInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountConfigAuthority,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority,
    TAccountVaultHolderRewards
  >;

  return instruction;
}

export type ParsedValidatorOverrideStakedLamportsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Config */
    config: TAccountMetas[0];
    /** Config authority */
    configAuthority: TAccountMetas[1];
    /** Validator stake */
    validatorStake: TAccountMetas[2];
    /** Validator stake authority */
    validatorStakeAuthority: TAccountMetas[3];
    /** Vault holder rewards */
    vaultHolderRewards: TAccountMetas[4];
  };
  data: ValidatorOverrideStakedLamportsInstructionData;
};

export function parseValidatorOverrideStakedLamportsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedValidatorOverrideStakedLamportsInstruction<TProgram, TAccountMetas> {
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
      configAuthority: getNextAccount(),
      validatorStake: getNextAccount(),
      validatorStakeAuthority: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
    },
    data: getValidatorOverrideStakedLamportsInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
