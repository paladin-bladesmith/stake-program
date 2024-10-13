/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import { containsBytes, getU8Encoder, type Address } from '@solana/web3.js';
import {
  type ParsedDeactivateStakeInstruction,
  type ParsedDistributeRewardsInstruction,
  type ParsedHarvestHolderRewardsInstruction,
  type ParsedHarvestSolStakerRewardsInstruction,
  type ParsedHarvestValidatorRewardsInstruction,
  type ParsedInactivateSolStakerStakeInstruction,
  type ParsedInactivateValidatorStakeInstruction,
  type ParsedInitializeConfigInstruction,
  type ParsedInitializeSolStakerStakeInstruction,
  type ParsedInitializeValidatorStakeInstruction,
  type ParsedSetAuthorityInstruction,
  type ParsedSlashSolStakerStakeInstruction,
  type ParsedSlashValidatorStakeInstruction,
  type ParsedSolStakerStakeTokensInstruction,
  type ParsedUpdateConfigInstruction,
  type ParsedValidatorStakeTokensInstruction,
  type ParsedWithdrawInactiveStakeInstruction,
} from '../instructions';

export const PALADIN_STAKE_PROGRAM_PROGRAM_ADDRESS =
  'GQurxHCYQCNfYR37nHNb6ZiLWg3jpbh2fWv2RpzwGqRK' as Address<'GQurxHCYQCNfYR37nHNb6ZiLWg3jpbh2fWv2RpzwGqRK'>;

export enum PaladinStakeProgramAccount {
  Config,
  SolStakerStake,
  ValidatorStake,
}

export enum PaladinStakeProgramInstruction {
  InitializeConfig,
  InitializeValidatorStake,
  ValidatorStakeTokens,
  DeactivateStake,
  InactivateValidatorStake,
  WithdrawInactiveStake,
  HarvestHolderRewards,
  HarvestValidatorRewards,
  SlashValidatorStake,
  SetAuthority,
  UpdateConfig,
  DistributeRewards,
  InitializeSolStakerStake,
  SolStakerStakeTokens,
  HarvestSolStakerRewards,
  InactivateSolStakerStake,
  SlashSolStakerStake,
}

export function identifyPaladinStakeProgramInstruction(
  instruction: { data: Uint8Array } | Uint8Array
): PaladinStakeProgramInstruction {
  const data =
    instruction instanceof Uint8Array ? instruction : instruction.data;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return PaladinStakeProgramInstruction.InitializeConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return PaladinStakeProgramInstruction.InitializeValidatorStake;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return PaladinStakeProgramInstruction.ValidatorStakeTokens;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return PaladinStakeProgramInstruction.DeactivateStake;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return PaladinStakeProgramInstruction.InactivateValidatorStake;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return PaladinStakeProgramInstruction.WithdrawInactiveStake;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return PaladinStakeProgramInstruction.HarvestHolderRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return PaladinStakeProgramInstruction.HarvestValidatorRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return PaladinStakeProgramInstruction.SlashValidatorStake;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return PaladinStakeProgramInstruction.SetAuthority;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return PaladinStakeProgramInstruction.UpdateConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(11), 0)) {
    return PaladinStakeProgramInstruction.DistributeRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(12), 0)) {
    return PaladinStakeProgramInstruction.InitializeSolStakerStake;
  }
  if (containsBytes(data, getU8Encoder().encode(13), 0)) {
    return PaladinStakeProgramInstruction.SolStakerStakeTokens;
  }
  if (containsBytes(data, getU8Encoder().encode(14), 0)) {
    return PaladinStakeProgramInstruction.HarvestSolStakerRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(15), 0)) {
    return PaladinStakeProgramInstruction.InactivateSolStakerStake;
  }
  if (containsBytes(data, getU8Encoder().encode(16), 0)) {
    return PaladinStakeProgramInstruction.SlashSolStakerStake;
  }
  throw new Error(
    'The provided instruction could not be identified as a paladinStakeProgram instruction.'
  );
}

export type ParsedPaladinStakeProgramInstruction<
  TProgram extends string = 'GQurxHCYQCNfYR37nHNb6ZiLWg3jpbh2fWv2RpzwGqRK',
> =
  | ({
      instructionType: PaladinStakeProgramInstruction.InitializeConfig;
    } & ParsedInitializeConfigInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.InitializeValidatorStake;
    } & ParsedInitializeValidatorStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.ValidatorStakeTokens;
    } & ParsedValidatorStakeTokensInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.DeactivateStake;
    } & ParsedDeactivateStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.InactivateValidatorStake;
    } & ParsedInactivateValidatorStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.WithdrawInactiveStake;
    } & ParsedWithdrawInactiveStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.HarvestHolderRewards;
    } & ParsedHarvestHolderRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.HarvestValidatorRewards;
    } & ParsedHarvestValidatorRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.SlashValidatorStake;
    } & ParsedSlashValidatorStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.SetAuthority;
    } & ParsedSetAuthorityInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.UpdateConfig;
    } & ParsedUpdateConfigInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.DistributeRewards;
    } & ParsedDistributeRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.InitializeSolStakerStake;
    } & ParsedInitializeSolStakerStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.SolStakerStakeTokens;
    } & ParsedSolStakerStakeTokensInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.HarvestSolStakerRewards;
    } & ParsedHarvestSolStakerRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.InactivateSolStakerStake;
    } & ParsedInactivateSolStakerStakeInstruction<TProgram>)
  | ({
      instructionType: PaladinStakeProgramInstruction.SlashSolStakerStake;
    } & ParsedSlashSolStakerStakeInstruction<TProgram>);
