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

export type HarvestValidatorRewardsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountValidatorStake extends string | IAccountMeta<string> = string,
  TAccountValidatorStakeAuthority extends
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
      TAccountValidatorStake extends string
        ? WritableAccount<TAccountValidatorStake>
        : TAccountValidatorStake,
      TAccountValidatorStakeAuthority extends string
        ? WritableAccount<TAccountValidatorStakeAuthority>
        : TAccountValidatorStakeAuthority,
      ...TRemainingAccounts,
    ]
  >;

export type HarvestValidatorRewardsInstructionData = { discriminator: number };

export type HarvestValidatorRewardsInstructionDataArgs = {};

export function getHarvestValidatorRewardsInstructionDataEncoder(): Encoder<HarvestValidatorRewardsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 7 })
  );
}

export function getHarvestValidatorRewardsInstructionDataDecoder(): Decoder<HarvestValidatorRewardsInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getHarvestValidatorRewardsInstructionDataCodec(): Codec<
  HarvestValidatorRewardsInstructionDataArgs,
  HarvestValidatorRewardsInstructionData
> {
  return combineCodec(
    getHarvestValidatorRewardsInstructionDataEncoder(),
    getHarvestValidatorRewardsInstructionDataDecoder()
  );
}

export type HarvestValidatorRewardsInput<
  TAccountConfig extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountValidatorStake extends string = string,
  TAccountValidatorStakeAuthority extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Holder rewards account */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Validator stake account */
  validatorStake: Address<TAccountValidatorStake>;
  /** Validator stake authority */
  validatorStakeAuthority: Address<TAccountValidatorStakeAuthority>;
};

export function getHarvestValidatorRewardsInstruction<
  TAccountConfig extends string,
  TAccountVaultHolderRewards extends string,
  TAccountValidatorStake extends string,
  TAccountValidatorStakeAuthority extends string,
>(
  input: HarvestValidatorRewardsInput<
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority
  >
): HarvestValidatorRewardsInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVaultHolderRewards,
  TAccountValidatorStake,
  TAccountValidatorStakeAuthority
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
    validatorStake: { value: input.validatorStake ?? null, isWritable: true },
    validatorStakeAuthority: {
      value: input.validatorStakeAuthority ?? null,
      isWritable: true,
    },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.validatorStake),
      getAccountMeta(accounts.validatorStakeAuthority),
    ],
    programAddress,
    data: getHarvestValidatorRewardsInstructionDataEncoder().encode({}),
  } as HarvestValidatorRewardsInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVaultHolderRewards,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority
  >;

  return instruction;
}

export type ParsedHarvestValidatorRewardsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Holder rewards account */
    vaultHolderRewards: TAccountMetas[1];
    /** Validator stake account */
    validatorStake: TAccountMetas[2];
    /** Validator stake authority */
    validatorStakeAuthority: TAccountMetas[3];
  };
  data: HarvestValidatorRewardsInstructionData;
};

export function parseHarvestValidatorRewardsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedHarvestValidatorRewardsInstruction<TProgram, TAccountMetas> {
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
      vaultHolderRewards: getNextAccount(),
      validatorStake: getNextAccount(),
      validatorStakeAuthority: getNextAccount(),
    },
    data: getHarvestValidatorRewardsInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
