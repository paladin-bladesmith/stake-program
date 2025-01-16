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
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';
import {
  getConfigFieldDecoder,
  getConfigFieldEncoder,
  type ConfigField,
  type ConfigFieldArgs,
} from '../types';

export type UpdateConfigInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountConfigAuthority extends string | IAccountMeta<string> = string,
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
      ...TRemainingAccounts,
    ]
  >;

export type UpdateConfigInstructionData = {
  discriminator: number;
  configField: ConfigField;
};

export type UpdateConfigInstructionDataArgs = { configField: ConfigFieldArgs };

export function getUpdateConfigInstructionDataEncoder(): Encoder<UpdateConfigInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['configField', getConfigFieldEncoder()],
    ]),
    (value) => ({ ...value, discriminator: 7 })
  );
}

export function getUpdateConfigInstructionDataDecoder(): Decoder<UpdateConfigInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['configField', getConfigFieldDecoder()],
  ]);
}

export function getUpdateConfigInstructionDataCodec(): Codec<
  UpdateConfigInstructionDataArgs,
  UpdateConfigInstructionData
> {
  return combineCodec(
    getUpdateConfigInstructionDataEncoder(),
    getUpdateConfigInstructionDataDecoder()
  );
}

export type UpdateConfigInput<
  TAccountConfig extends string = string,
  TAccountConfigAuthority extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Stake config authority */
  configAuthority: TransactionSigner<TAccountConfigAuthority>;
  configField: UpdateConfigInstructionDataArgs['configField'];
};

export function getUpdateConfigInstruction<
  TAccountConfig extends string,
  TAccountConfigAuthority extends string,
>(
  input: UpdateConfigInput<TAccountConfig, TAccountConfigAuthority>
): UpdateConfigInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountConfigAuthority
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
    ],
    programAddress,
    data: getUpdateConfigInstructionDataEncoder().encode(
      args as UpdateConfigInstructionDataArgs
    ),
  } as UpdateConfigInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountConfigAuthority
  >;

  return instruction;
}

export type ParsedUpdateConfigInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Stake config authority */
    configAuthority: TAccountMetas[1];
  };
  data: UpdateConfigInstructionData;
};

export function parseUpdateConfigInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedUpdateConfigInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 2) {
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
    },
    data: getUpdateConfigInstructionDataDecoder().decode(instruction.data),
  };
}
