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

export type SolStakerUpdateAuthorityInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountSolStakerAuthorityOverride extends
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
      TAccountSolStakerStake extends string
        ? WritableAccount<TAccountSolStakerStake>
        : TAccountSolStakerStake,
      TAccountSolStakerAuthorityOverride extends string
        ? ReadonlyAccount<TAccountSolStakerAuthorityOverride>
        : TAccountSolStakerAuthorityOverride,
      ...TRemainingAccounts,
    ]
  >;

export type SolStakerUpdateAuthorityInstructionData = { discriminator: number };

export type SolStakerUpdateAuthorityInstructionDataArgs = {};

export function getSolStakerUpdateAuthorityInstructionDataEncoder(): Encoder<SolStakerUpdateAuthorityInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 17 })
  );
}

export function getSolStakerUpdateAuthorityInstructionDataDecoder(): Decoder<SolStakerUpdateAuthorityInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getSolStakerUpdateAuthorityInstructionDataCodec(): Codec<
  SolStakerUpdateAuthorityInstructionDataArgs,
  SolStakerUpdateAuthorityInstructionData
> {
  return combineCodec(
    getSolStakerUpdateAuthorityInstructionDataEncoder(),
    getSolStakerUpdateAuthorityInstructionDataDecoder()
  );
}

export type SolStakerUpdateAuthorityInput<
  TAccountConfig extends string = string,
  TAccountSolStakerStake extends string = string,
  TAccountSolStakerAuthorityOverride extends string = string,
> = {
  /** Config */
  config: Address<TAccountConfig>;
  /** Sol staker stake */
  solStakerStake: Address<TAccountSolStakerStake>;
  /** Sol staker authority override */
  solStakerAuthorityOverride: Address<TAccountSolStakerAuthorityOverride>;
};

export function getSolStakerUpdateAuthorityInstruction<
  TAccountConfig extends string,
  TAccountSolStakerStake extends string,
  TAccountSolStakerAuthorityOverride extends string,
>(
  input: SolStakerUpdateAuthorityInput<
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountSolStakerAuthorityOverride
  >
): SolStakerUpdateAuthorityInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountSolStakerStake,
  TAccountSolStakerAuthorityOverride
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    solStakerStake: { value: input.solStakerStake ?? null, isWritable: true },
    solStakerAuthorityOverride: {
      value: input.solStakerAuthorityOverride ?? null,
      isWritable: false,
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
      getAccountMeta(accounts.solStakerStake),
      getAccountMeta(accounts.solStakerAuthorityOverride),
    ],
    programAddress,
    data: getSolStakerUpdateAuthorityInstructionDataEncoder().encode({}),
  } as SolStakerUpdateAuthorityInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountSolStakerAuthorityOverride
  >;

  return instruction;
}

export type ParsedSolStakerUpdateAuthorityInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Config */
    config: TAccountMetas[0];
    /** Sol staker stake */
    solStakerStake: TAccountMetas[1];
    /** Sol staker authority override */
    solStakerAuthorityOverride: TAccountMetas[2];
  };
  data: SolStakerUpdateAuthorityInstructionData;
};

export function parseSolStakerUpdateAuthorityInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSolStakerUpdateAuthorityInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 3) {
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
      solStakerAuthorityOverride: getNextAccount(),
    },
    data: getSolStakerUpdateAuthorityInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
