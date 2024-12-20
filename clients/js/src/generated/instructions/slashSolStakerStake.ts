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

export type SlashSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountSolStakerStake extends string | IAccountMeta<string> = string,
  TAccountSolStakerStakeAuthority extends
    | string
    | IAccountMeta<string> = string,
  TAccountSlashAuthority extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountVaultAuthority extends string | IAccountMeta<string> = string,
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
      TAccountSolStakerStake extends string
        ? WritableAccount<TAccountSolStakerStake>
        : TAccountSolStakerStake,
      TAccountSolStakerStakeAuthority extends string
        ? WritableAccount<TAccountSolStakerStakeAuthority>
        : TAccountSolStakerStakeAuthority,
      TAccountSlashAuthority extends string
        ? ReadonlySignerAccount<TAccountSlashAuthority> &
            IAccountSignerMeta<TAccountSlashAuthority>
        : TAccountSlashAuthority,
      TAccountMint extends string
        ? WritableAccount<TAccountMint>
        : TAccountMint,
      TAccountVault extends string
        ? WritableAccount<TAccountVault>
        : TAccountVault,
      TAccountVaultHolderRewards extends string
        ? ReadonlyAccount<TAccountVaultHolderRewards>
        : TAccountVaultHolderRewards,
      TAccountVaultAuthority extends string
        ? ReadonlyAccount<TAccountVaultAuthority>
        : TAccountVaultAuthority,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type SlashSolStakerStakeInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type SlashSolStakerStakeInstructionDataArgs = {
  amount: number | bigint;
};

export function getSlashSolStakerStakeInstructionDataEncoder(): Encoder<SlashSolStakerStakeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 15 })
  );
}

export function getSlashSolStakerStakeInstructionDataDecoder(): Decoder<SlashSolStakerStakeInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getSlashSolStakerStakeInstructionDataCodec(): Codec<
  SlashSolStakerStakeInstructionDataArgs,
  SlashSolStakerStakeInstructionData
> {
  return combineCodec(
    getSlashSolStakerStakeInstructionDataEncoder(),
    getSlashSolStakerStakeInstructionDataDecoder()
  );
}

export type SlashSolStakerStakeInput<
  TAccountConfig extends string = string,
  TAccountSolStakerStake extends string = string,
  TAccountSolStakerStakeAuthority extends string = string,
  TAccountSlashAuthority extends string = string,
  TAccountMint extends string = string,
  TAccountVault extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountVaultAuthority extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** SOL staker stake account */
  solStakerStake: Address<TAccountSolStakerStake>;
  /** SOL staker stake authority account */
  solStakerStakeAuthority: Address<TAccountSolStakerStakeAuthority>;
  /** Config slash authority */
  slashAuthority: TransactionSigner<TAccountSlashAuthority>;
  /** Vault token mint */
  mint: Address<TAccountMint>;
  /** Vault token account */
  vault: Address<TAccountVault>;
  /** Vault holder rewards account */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Vault authority */
  vaultAuthority: Address<TAccountVaultAuthority>;
  /** Token program */
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: SlashSolStakerStakeInstructionDataArgs['amount'];
};

export function getSlashSolStakerStakeInstruction<
  TAccountConfig extends string,
  TAccountSolStakerStake extends string,
  TAccountSolStakerStakeAuthority extends string,
  TAccountSlashAuthority extends string,
  TAccountMint extends string,
  TAccountVault extends string,
  TAccountVaultHolderRewards extends string,
  TAccountVaultAuthority extends string,
  TAccountTokenProgram extends string,
>(
  input: SlashSolStakerStakeInput<
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountSolStakerStakeAuthority,
    TAccountSlashAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountVaultAuthority,
    TAccountTokenProgram
  >
): SlashSolStakerStakeInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountSolStakerStake,
  TAccountSolStakerStakeAuthority,
  TAccountSlashAuthority,
  TAccountMint,
  TAccountVault,
  TAccountVaultHolderRewards,
  TAccountVaultAuthority,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    solStakerStake: { value: input.solStakerStake ?? null, isWritable: true },
    solStakerStakeAuthority: {
      value: input.solStakerStakeAuthority ?? null,
      isWritable: true,
    },
    slashAuthority: { value: input.slashAuthority ?? null, isWritable: false },
    mint: { value: input.mint ?? null, isWritable: true },
    vault: { value: input.vault ?? null, isWritable: true },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: false,
    },
    vaultAuthority: { value: input.vaultAuthority ?? null, isWritable: false },
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
      getAccountMeta(accounts.solStakerStake),
      getAccountMeta(accounts.solStakerStakeAuthority),
      getAccountMeta(accounts.slashAuthority),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.vaultAuthority),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getSlashSolStakerStakeInstructionDataEncoder().encode(
      args as SlashSolStakerStakeInstructionDataArgs
    ),
  } as SlashSolStakerStakeInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountSolStakerStake,
    TAccountSolStakerStakeAuthority,
    TAccountSlashAuthority,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountVaultAuthority,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedSlashSolStakerStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** SOL staker stake account */
    solStakerStake: TAccountMetas[1];
    /** SOL staker stake authority account */
    solStakerStakeAuthority: TAccountMetas[2];
    /** Config slash authority */
    slashAuthority: TAccountMetas[3];
    /** Vault token mint */
    mint: TAccountMetas[4];
    /** Vault token account */
    vault: TAccountMetas[5];
    /** Vault holder rewards account */
    vaultHolderRewards: TAccountMetas[6];
    /** Vault authority */
    vaultAuthority: TAccountMetas[7];
    /** Token program */
    tokenProgram: TAccountMetas[8];
  };
  data: SlashSolStakerStakeInstructionData;
};

export function parseSlashSolStakerStakeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSlashSolStakerStakeInstruction<TProgram, TAccountMetas> {
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
      solStakerStake: getNextAccount(),
      solStakerStakeAuthority: getNextAccount(),
      slashAuthority: getNextAccount(),
      mint: getNextAccount(),
      vault: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
      vaultAuthority: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getSlashSolStakerStakeInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
