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

export type WithdrawInactiveStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountStake extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultHolderRewards extends string | IAccountMeta<string> = string,
  TAccountVaultAuthority extends string | IAccountMeta<string> = string,
  TAccountDestinationTokenAccount extends
    | string
    | IAccountMeta<string> = string,
  TAccountStakeAuthority extends string | IAccountMeta<string> = string,
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
      TAccountStake extends string
        ? WritableAccount<TAccountStake>
        : TAccountStake,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
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
      TAccountDestinationTokenAccount extends string
        ? WritableAccount<TAccountDestinationTokenAccount>
        : TAccountDestinationTokenAccount,
      TAccountStakeAuthority extends string
        ? ReadonlySignerAccount<TAccountStakeAuthority> &
            IAccountSignerMeta<TAccountStakeAuthority>
        : TAccountStakeAuthority,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type WithdrawInactiveStakeInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type WithdrawInactiveStakeInstructionDataArgs = {
  amount: number | bigint;
};

export function getWithdrawInactiveStakeInstructionDataEncoder(): Encoder<WithdrawInactiveStakeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 5 })
  );
}

export function getWithdrawInactiveStakeInstructionDataDecoder(): Decoder<WithdrawInactiveStakeInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getWithdrawInactiveStakeInstructionDataCodec(): Codec<
  WithdrawInactiveStakeInstructionDataArgs,
  WithdrawInactiveStakeInstructionData
> {
  return combineCodec(
    getWithdrawInactiveStakeInstructionDataEncoder(),
    getWithdrawInactiveStakeInstructionDataDecoder()
  );
}

export type WithdrawInactiveStakeInput<
  TAccountConfig extends string = string,
  TAccountStake extends string = string,
  TAccountMint extends string = string,
  TAccountVault extends string = string,
  TAccountVaultHolderRewards extends string = string,
  TAccountVaultAuthority extends string = string,
  TAccountDestinationTokenAccount extends string = string,
  TAccountStakeAuthority extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Validator or SOL staker stake account */
  stake: Address<TAccountStake>;
  /** Stake Token Mint */
  mint: Address<TAccountMint>;
  /** Vault token account */
  vault: Address<TAccountVault>;
  /** Vault holder rewards */
  vaultHolderRewards: Address<TAccountVaultHolderRewards>;
  /** Vault authority */
  vaultAuthority: Address<TAccountVaultAuthority>;
  /** Destination token account */
  destinationTokenAccount: Address<TAccountDestinationTokenAccount>;
  /** Stake authority */
  stakeAuthority: TransactionSigner<TAccountStakeAuthority>;
  /** Token program */
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: WithdrawInactiveStakeInstructionDataArgs['amount'];
};

export function getWithdrawInactiveStakeInstruction<
  TAccountConfig extends string,
  TAccountStake extends string,
  TAccountMint extends string,
  TAccountVault extends string,
  TAccountVaultHolderRewards extends string,
  TAccountVaultAuthority extends string,
  TAccountDestinationTokenAccount extends string,
  TAccountStakeAuthority extends string,
  TAccountTokenProgram extends string,
>(
  input: WithdrawInactiveStakeInput<
    TAccountConfig,
    TAccountStake,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountVaultAuthority,
    TAccountDestinationTokenAccount,
    TAccountStakeAuthority,
    TAccountTokenProgram
  >
): WithdrawInactiveStakeInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountStake,
  TAccountMint,
  TAccountVault,
  TAccountVaultHolderRewards,
  TAccountVaultAuthority,
  TAccountDestinationTokenAccount,
  TAccountStakeAuthority,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    stake: { value: input.stake ?? null, isWritable: true },
    mint: { value: input.mint ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    vaultHolderRewards: {
      value: input.vaultHolderRewards ?? null,
      isWritable: false,
    },
    vaultAuthority: { value: input.vaultAuthority ?? null, isWritable: false },
    destinationTokenAccount: {
      value: input.destinationTokenAccount ?? null,
      isWritable: true,
    },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false },
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
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultHolderRewards),
      getAccountMeta(accounts.vaultAuthority),
      getAccountMeta(accounts.destinationTokenAccount),
      getAccountMeta(accounts.stakeAuthority),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getWithdrawInactiveStakeInstructionDataEncoder().encode(
      args as WithdrawInactiveStakeInstructionDataArgs
    ),
  } as WithdrawInactiveStakeInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountStake,
    TAccountMint,
    TAccountVault,
    TAccountVaultHolderRewards,
    TAccountVaultAuthority,
    TAccountDestinationTokenAccount,
    TAccountStakeAuthority,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedWithdrawInactiveStakeInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Validator or SOL staker stake account */
    stake: TAccountMetas[1];
    /** Stake Token Mint */
    mint: TAccountMetas[2];
    /** Vault token account */
    vault: TAccountMetas[3];
    /** Vault holder rewards */
    vaultHolderRewards: TAccountMetas[4];
    /** Vault authority */
    vaultAuthority: TAccountMetas[5];
    /** Destination token account */
    destinationTokenAccount: TAccountMetas[6];
    /** Stake authority */
    stakeAuthority: TAccountMetas[7];
    /** Token program */
    tokenProgram: TAccountMetas[8];
  };
  data: WithdrawInactiveStakeInstructionData;
};

export function parseWithdrawInactiveStakeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedWithdrawInactiveStakeInstruction<TProgram, TAccountMetas> {
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
      stake: getNextAccount(),
      mint: getNextAccount(),
      vault: getNextAccount(),
      vaultHolderRewards: getNextAccount(),
      vaultAuthority: getNextAccount(),
      destinationTokenAccount: getNextAccount(),
      stakeAuthority: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getWithdrawInactiveStakeInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
