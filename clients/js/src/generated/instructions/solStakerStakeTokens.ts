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

export type SolStakerStakeTokensInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountValidatorStake extends string | IAccountMeta<string> = string,
  TAccountSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountSolStakerStakeAuthority extends
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
      TAccountSolStakerStake extends string
        ? WritableAccount<TAccountSolStakerStake>
        : TAccountSolStakerStake,
      TAccountSolStakerStakeAuthority extends string
        ? WritableAccount<TAccountSolStakerStakeAuthority>
        : TAccountSolStakerStakeAuthority,
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
        ? ReadonlyAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type SolStakerStakeTokensInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type SolStakerStakeTokensInstructionDataArgs = {
  amount: number | bigint;
};

export function getSolStakerStakeTokensInstructionDataEncoder(): Encoder<SolStakerStakeTokensInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 9 })
  );
}

export function getSolStakerStakeTokensInstructionDataDecoder(): Decoder<SolStakerStakeTokensInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getSolStakerStakeTokensInstructionDataCodec(): Codec<
  SolStakerStakeTokensInstructionDataArgs,
  SolStakerStakeTokensInstructionData
> {
  return combineCodec(
    getSolStakerStakeTokensInstructionDataEncoder(),
    getSolStakerStakeTokensInstructionDataDecoder()
  );
}

export type SolStakerStakeTokensInput<
  TAccountConfig extends string = string,
  TAccountValidatorStake extends string = string,
  TAccountSolStakerStake extends string = string,
  TAccountSolStakerStakeAuthority extends string = string,
  TAccountSourceTokenAccount extends string = string,
  TAccountSourceTokenAccountAuthority extends string = string,
  TAccountMint extends string = string,
  TAccountVault extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Validator stake */
  validatorStake: Address<TAccountValidatorStake>;
  /** SOL staker stake account */
  solStakerStake: Address<TAccountSolStakerStake>;
  /** SOL staker stake authority account */
  solStakerStakeAuthority: Address<TAccountSolStakerStakeAuthority>;
  /** Token account */
  sourceTokenAccount: Address<TAccountSourceTokenAccount>;
  /** Owner or delegate of the token account */
  sourceTokenAccountAuthority: TransactionSigner<TAccountSourceTokenAccountAuthority>;
  /** Stake Token Mint */
  mint: Address<TAccountMint>;
  /** Stake token Vault */
  vault: Address<TAccountVault>;
  /** Stake token Vault */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Token program */
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: SolStakerStakeTokensInstructionDataArgs['amount'];
};

export function getSolStakerStakeTokensInstruction<
  TAccountConfig extends string,
  TAccountValidatorStake extends string,
  TAccountSolStakerStake extends string,
  TAccountSolStakerStakeAuthority extends string,
  TAccountSourceTokenAccount extends string,
  TAccountSourceTokenAccountAuthority extends string,
  TAccountMint extends string,
  TAccountVault extends string,
  TAccountVaultHolderRewards extends string,
  TAccountTokenProgram extends string,
>(
  input: SolStakerStakeTokensInput<
    TAccountConfig,
    TAccountValidatorStake,
    TAccountSolStakerStake,
    TAccountSolStakerStakeAuthority,
    TAccountSourceTokenAccount,
    TAccountSourceTokenAccountAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountTokenProgram
  >
): SolStakerStakeTokensInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountValidatorStake,
  TAccountSolStakerStake,
  TAccountSolStakerStakeAuthority,
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
    solStakerStake: { value: input.solStakerStake ?? null, isWritable: true },
    solStakerStakeAuthority: {
      value: input.solStakerStakeAuthority ?? null,
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
      isWritable: false,
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
      getAccountMeta(accounts.solStakerStake),
      getAccountMeta(accounts.solStakerStakeAuthority),
      getAccountMeta(accounts.sourceTokenAccount),
      getAccountMeta(accounts.sourceTokenAccountAuthority),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getSolStakerStakeTokensInstructionDataEncoder().encode(
      args as SolStakerStakeTokensInstructionDataArgs
    ),
  } as SolStakerStakeTokensInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountValidatorStake,
    TAccountSolStakerStake,
    TAccountSolStakerStakeAuthority,
    TAccountSourceTokenAccount,
    TAccountSourceTokenAccountAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedSolStakerStakeTokensInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Validator stake */
    validatorStake: TAccountMetas[1];
    /** SOL staker stake account */
    solStakerStake: TAccountMetas[2];
    /** SOL staker stake authority account */
    solStakerStakeAuthority: TAccountMetas[3];
    /** Token account */
    sourceTokenAccount: TAccountMetas[4];
    /** Owner or delegate of the token account */
    sourceTokenAccountAuthority: TAccountMetas[5];
    /** Stake Token Mint */
    mint: TAccountMetas[6];
    /** Stake token Vault */
    vault: TAccountMetas[7];
    /** Stake token Vault */
    vaultHolderRewards: TAccountMetas[8];
    /** Token program */
    tokenProgram: TAccountMetas[9];
  };
  data: SolStakerStakeTokensInstructionData;
};

export function parseSolStakerStakeTokensInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSolStakerStakeTokensInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 10) {
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
      solStakerStake: getNextAccount(),
      solStakerStakeAuthority: getNextAccount(),
      sourceTokenAccount: getNextAccount(),
      sourceTokenAccountAuthority: getNextAccount(),
      mint: getNextAccount(),
      vault: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getSolStakerStakeTokensInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
