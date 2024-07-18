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

export type HarvestHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountStake extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountHolderRewards extends string | IAccountMeta<string> = string,
  TAccountDestination extends string | IAccountMeta<string> = string,
  TAccountStakeAuthority extends string | IAccountMeta<string> = string,
  TAccountVaultAuthority extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? ReadonlyAccount<TAccountConfig>
        : TAccountConfig,
      TAccountStake extends string
        ? WritableAccount<TAccountStake>
        : TAccountStake,
      TAccountVault extends string
        ? WritableAccount<TAccountVault>
        : TAccountVault,
      TAccountHolderRewards extends string
        ? ReadonlyAccount<TAccountHolderRewards>
        : TAccountHolderRewards,
      TAccountDestination extends string
        ? WritableAccount<TAccountDestination>
        : TAccountDestination,
      TAccountStakeAuthority extends string
        ? ReadonlySignerAccount<TAccountStakeAuthority> &
            IAccountSignerMeta<TAccountStakeAuthority>
        : TAccountStakeAuthority,
      TAccountVaultAuthority extends string
        ? ReadonlyAccount<TAccountVaultAuthority>
        : TAccountVaultAuthority,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type HarvestHolderRewardsInstructionData = { discriminator: number };

export type HarvestHolderRewardsInstructionDataArgs = {};

export function getHarvestHolderRewardsInstructionDataEncoder(): Encoder<HarvestHolderRewardsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 6 })
  );
}

export function getHarvestHolderRewardsInstructionDataDecoder(): Decoder<HarvestHolderRewardsInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getHarvestHolderRewardsInstructionDataCodec(): Codec<
  HarvestHolderRewardsInstructionDataArgs,
  HarvestHolderRewardsInstructionData
> {
  return combineCodec(
    getHarvestHolderRewardsInstructionDataEncoder(),
    getHarvestHolderRewardsInstructionDataDecoder()
  );
}

export type HarvestHolderRewardsInput<
  TAccountConfig extends string = string,
  TAccountStake extends string = string,
  TAccountVault extends string = string,
  TAccountHolderRewards extends string = string,
  TAccountDestination extends string = string,
  TAccountStakeAuthority extends string = string,
  TAccountVaultAuthority extends string = string,
  TAccountMint extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  /** Stake config account */
  config: Address<TAccountConfig>;
  /** Validator stake account (pda of `['stake::state::stake', validator, config]`) */
  stake: Address<TAccountStake>;
  /** Vault token account */
  vault: Address<TAccountVault>;
  /** Holder rewards account for vault token account */
  holderRewards: Address<TAccountHolderRewards>;
  /** Destination account for withdrawn lamports */
  destination: Address<TAccountDestination>;
  /** Stake authority */
  stakeAuthority: TransactionSigner<TAccountStakeAuthority>;
  /** Vault authority (pda of `['token-owner', config]`) */
  vaultAuthority: Address<TAccountVaultAuthority>;
  /** Stake token mint */
  mint: Address<TAccountMint>;
  /** Token program */
  tokenProgram?: Address<TAccountTokenProgram>;
};

export function getHarvestHolderRewardsInstruction<
  TAccountConfig extends string,
  TAccountStake extends string,
  TAccountVault extends string,
  TAccountHolderRewards extends string,
  TAccountDestination extends string,
  TAccountStakeAuthority extends string,
  TAccountVaultAuthority extends string,
  TAccountMint extends string,
  TAccountTokenProgram extends string,
>(
  input: HarvestHolderRewardsInput<
    TAccountConfig,
    TAccountStake,
    TAccountVault,
    TAccountHolderRewards,
    TAccountDestination,
    TAccountStakeAuthority,
    TAccountVaultAuthority,
    TAccountMint,
    TAccountTokenProgram
  >
): HarvestHolderRewardsInstruction<
  typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountStake,
  TAccountVault,
  TAccountHolderRewards,
  TAccountDestination,
  TAccountStakeAuthority,
  TAccountVaultAuthority,
  TAccountMint,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    stake: { value: input.stake ?? null, isWritable: true },
    vault: { value: input.vault ?? null, isWritable: true },
    holderRewards: { value: input.holderRewards ?? null, isWritable: false },
    destination: { value: input.destination ?? null, isWritable: true },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false },
    vaultAuthority: { value: input.vaultAuthority ?? null, isWritable: false },
    mint: { value: input.mint ?? null, isWritable: false },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' as Address<'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.holderRewards),
      getAccountMeta(accounts.destination),
      getAccountMeta(accounts.stakeAuthority),
      getAccountMeta(accounts.vaultAuthority),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getHarvestHolderRewardsInstructionDataEncoder().encode({}),
  } as HarvestHolderRewardsInstruction<
    typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountStake,
    TAccountVault,
    TAccountHolderRewards,
    TAccountDestination,
    TAccountStakeAuthority,
    TAccountVaultAuthority,
    TAccountMint,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedHarvestHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    config: TAccountMetas[0];
    /** Validator stake account (pda of `['stake::state::stake', validator, config]`) */
    stake: TAccountMetas[1];
    /** Vault token account */
    vault: TAccountMetas[2];
    /** Holder rewards account for vault token account */
    holderRewards: TAccountMetas[3];
    /** Destination account for withdrawn lamports */
    destination: TAccountMetas[4];
    /** Stake authority */
    stakeAuthority: TAccountMetas[5];
    /** Vault authority (pda of `['token-owner', config]`) */
    vaultAuthority: TAccountMetas[6];
    /** Stake token mint */
    mint: TAccountMetas[7];
    /** Token program */
    tokenProgram: TAccountMetas[8];
  };
  data: HarvestHolderRewardsInstructionData;
};

export function parseHarvestHolderRewardsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedHarvestHolderRewardsInstruction<TProgram, TAccountMetas> {
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
      vault: getNextAccount(),
      holderRewards: getNextAccount(),
      destination: getNextAccount(),
      stakeAuthority: getNextAccount(),
      vaultAuthority: getNextAccount(),
      mint: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getHarvestHolderRewardsInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
