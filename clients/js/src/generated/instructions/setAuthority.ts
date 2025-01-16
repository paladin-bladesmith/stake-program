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
import {
  getAuthorityTypeDecoder,
  getAuthorityTypeEncoder,
  type AuthorityType,
  type AuthorityTypeArgs,
} from '../types';

export type SetAuthorityInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountAccount extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TAccountNewAuthority extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountAccount extends string
        ? WritableAccount<TAccountAccount>
        : TAccountAccount,
      TAccountAuthority extends string
        ? ReadonlySignerAccount<TAccountAuthority> &
            IAccountSignerMeta<TAccountAuthority>
        : TAccountAuthority,
      TAccountNewAuthority extends string
        ? ReadonlyAccount<TAccountNewAuthority>
        : TAccountNewAuthority,
      ...TRemainingAccounts,
    ]
  >;

export type SetAuthorityInstructionData = {
  discriminator: number;
  authorityType: AuthorityType;
};

export type SetAuthorityInstructionDataArgs = {
  authorityType: AuthorityTypeArgs;
};

export function getSetAuthorityInstructionDataEncoder(): Encoder<SetAuthorityInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['authorityType', getAuthorityTypeEncoder()],
    ]),
    (value) => ({ ...value, discriminator: 6 })
  );
}

export function getSetAuthorityInstructionDataDecoder(): Decoder<SetAuthorityInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['authorityType', getAuthorityTypeDecoder()],
  ]);
}

export function getSetAuthorityInstructionDataCodec(): Codec<
  SetAuthorityInstructionDataArgs,
  SetAuthorityInstructionData
> {
  return combineCodec(
    getSetAuthorityInstructionDataEncoder(),
    getSetAuthorityInstructionDataDecoder()
  );
}

export type SetAuthorityInput<
  TAccountAccount extends string = string,
  TAccountAuthority extends string = string,
  TAccountNewAuthority extends string = string,
> = {
  /** Config or Stake account */
  account: Address<TAccountAccount>;
  /** Current authority on the account */
  authority: TransactionSigner<TAccountAuthority>;
  /** Authority to set */
  newAuthority: Address<TAccountNewAuthority>;
  authorityType: SetAuthorityInstructionDataArgs['authorityType'];
};

export function getSetAuthorityInstruction<
  TAccountAccount extends string,
  TAccountAuthority extends string,
  TAccountNewAuthority extends string,
>(
  input: SetAuthorityInput<
    TAccountAccount,
    TAccountAuthority,
    TAccountNewAuthority
  >
): SetAuthorityInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountAccount,
  TAccountAuthority,
  TAccountNewAuthority
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    account: { value: input.account ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    newAuthority: { value: input.newAuthority ?? null, isWritable: false },
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
      getAccountMeta(accounts.account),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.newAuthority),
    ],
    programAddress,
    data: getSetAuthorityInstructionDataEncoder().encode(
      args as SetAuthorityInstructionDataArgs
    ),
  } as SetAuthorityInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountAccount,
    TAccountAuthority,
    TAccountNewAuthority
  >;

  return instruction;
}

export type ParsedSetAuthorityInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Config or Stake account */
    account: TAccountMetas[0];
    /** Current authority on the account */
    authority: TAccountMetas[1];
    /** Authority to set */
    newAuthority: TAccountMetas[2];
  };
  data: SetAuthorityInstructionData;
};

export function parseSetAuthorityInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSetAuthorityInstruction<TProgram, TAccountMetas> {
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
      account: getNextAccount(),
      authority: getNextAccount(),
      newAuthority: getNextAccount(),
    },
    data: getSetAuthorityInstructionDataDecoder().decode(instruction.data),
  };
}
