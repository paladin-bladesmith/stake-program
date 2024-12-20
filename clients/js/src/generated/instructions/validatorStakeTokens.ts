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

export type ValidatorStakeTokensInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountValidatorStake extends string | IAccountMeta<string> = string,
  TAccountValidatorStakeAuthority extends
    | string
    | IAccountMeta<string> = string,
  TAccountSourceTokenAccount extends string | IAccountMeta<string> = string,
  TAccountSourceTokenAccountAuthority extends
    | string
    | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountValidatorStake extends string
        ? WritableAccount<TAccountValidatorStake>
        : TAccountValidatorStake,
      TAccountValidatorStakeAuthority extends string
        ? WritableAccount<TAccountValidatorStakeAuthority>
        : TAccountValidatorStakeAuthority,
      TAccountSourceTokenAccount extends string
        ? WritableAccount<TAccountSourceTokenAccount>
        : TAccountSourceTokenAccount,
      TAccountSourceTokenAccountAuthority extends string
        ? ReadonlySignerAccount<TAccountSourceTokenAccountAuthority> &
            IAccountSignerMeta<TAccountSourceTokenAccountAuthority>
        : TAccountSourceTokenAccountAuthority,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountVault extends string
        ? WritableAccount<TAccountVault>
        : TAccountVault,
      TAccountVaultHolderRewards extends string
        ? WritableAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type ValidatorStakeTokensInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type ValidatorStakeTokensInstructionDataArgs = {
  amount: number | bigint;
};

export function getValidatorStakeTokensInstructionDataEncoder(): Encoder<ValidatorStakeTokensInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 2 })
  );
}

export function getValidatorStakeTokensInstructionDataDecoder(): Decoder<ValidatorStakeTokensInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getValidatorStakeTokensInstructionDataCodec(): Codec<
  ValidatorStakeTokensInstructionDataArgs,
  ValidatorStakeTokensInstructionData
> {
  return combineCodec(
    getValidatorStakeTokensInstructionDataEncoder(),
    getValidatorStakeTokensInstructionDataDecoder()
  );
}

export type ValidatorStakeTokensInput<
  TAccountConfig extends string = string,
  TAccountValidatorStake extends string = string,
  TAccountValidatorStakeAuthority extends string = string,
  TAccountSourceTokenAccount extends string = string,
  TAccountSourceTokenAccountAuthority extends string = string,
  TAccountMint extends string = string,
  TAccountVault extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Validator stake account */
  validatorStake: Address<TAccountValidatorStake>;
  /** Validator stake account */
  validatorStakeAuthority: Address<TAccountValidatorStakeAuthority>;
  /** Token account */
  sourceTokenAccount: Address<TAccountSourceTokenAccount>;
  /** Owner or delegate of the token account */
  sourceTokenAccountAuthority: TransactionSigner<TAccountSourceTokenAccountAuthority>;
  /** Stake Token Mint */
  mint: Address<TAccountMint>;
  /** Stake token Vault */
  vault: Address<TAccountVault>;
  /** Holder rewards for the vault account (to facilitate harvest) */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Token program */
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: ValidatorStakeTokensInstructionDataArgs['amount'];
};

export function getValidatorStakeTokensInstruction<
  TAccountConfig extends string,
  TAccountValidatorStake extends string,
  TAccountValidatorStakeAuthority extends string,
  TAccountSourceTokenAccount extends string,
  TAccountSourceTokenAccountAuthority extends string,
  TAccountMint extends string,
  TAccountVault extends string,
  TAccountVaultHolderRewards extends string,
  TAccountTokenProgram extends string,
>(
  input: ValidatorStakeTokensInput<
    TAccountConfig,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority,
    TAccountSourceTokenAccount,
    TAccountSourceTokenAccountAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountTokenProgram
  >
): ValidatorStakeTokensInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountValidatorStake,
  TAccountValidatorStakeAuthority,
  TAccountSourceTokenAccount,
  TAccountSourceTokenAccountAuthority,
  TAccountMint,
  TAccountVault,
  TAccountVaultHolderRewards,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    validatorStake: { value: input.validatorStake ?? null, isWritable: true },
    validatorStakeAuthority: {
      value: input.validatorStakeAuthority ?? null,
      isWritable: true,
    },
    sourceTokenAccount: {
      value: input.sourceTokenAccount ?? null,
      isWritable: true,
    },
    sourceTokenAccountAuthority: {
      value: input.sourceTokenAccountAuthority ?? null,
      isWritable: false,
    },
    mint: { value: input.mint ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: true,
    },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb' as Address<'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.validatorStake),
      getAccountMeta(accounts.validatorStakeAuthority),
      getAccountMeta(accounts.sourceTokenAccount),
      getAccountMeta(accounts.sourceTokenAccountAuthority),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getValidatorStakeTokensInstructionDataEncoder().encode(
      args as ValidatorStakeTokensInstructionDataArgs
    ),
  } as ValidatorStakeTokensInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountValidatorStake,
    TAccountValidatorStakeAuthority,
    TAccountSourceTokenAccount,
    TAccountSourceTokenAccountAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedValidatorStakeTokensInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Validator stake account */
    validatorStake: TAccountMetas[1];
    /** Validator stake account */
    validatorStakeAuthority: TAccountMetas[2];
    /** Token account */
    sourceTokenAccount: TAccountMetas[3];
    /** Owner or delegate of the token account */
    sourceTokenAccountAuthority: TAccountMetas[4];
    /** Stake Token Mint */
    mint: TAccountMetas[5];
    /** Stake token Vault */
    vault: TAccountMetas[6];
    /** Holder rewards for the vault account (to facilitate harvest) */
    vaultHolderRewards: TAccountMetas[7];
    /** Token program */
    tokenProgram: TAccountMetas[8];
  };
  data: ValidatorStakeTokensInstructionData;
};

export function parseValidatorStakeTokensInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedValidatorStakeTokensInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 9) {
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
      validatorStake: getNextAccount(),
      validatorStakeAuthority: getNextAccount(),
      sourceTokenAccount: getNextAccount(),
      sourceTokenAccountAuthority: getNextAccount(),
      mint: getNextAccount(),
      vault: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getValidatorStakeTokensInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
