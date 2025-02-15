# Paladin Stake Program

The Paladin Stake program manages the delegation of tokens to a particular validator of the system. This allows stakers to earn additional shares of rewards proportional to their share of staked tokens and to participate in governance.

## Overview

A `Config` account essentially represents the staking system and it is the first account that needs to be created. Once a `Config` account is created, `ValidatorStake` accounts can be added to the system. A `ValidatorStake` represents a validator of the network (coupled to a `VoteState` account in the native Solana vote program). Anybody can create the stake account for a validator. For new accounts, the authority is initialized to the validator vote account's withdraw authority.

After `ValidatorStake` accounts are added to the system, `SolStakerStake` accounts can be created. These accounts represent individual SOL stakers (coupled to `StakeState` accounts in the native Solana stake program). Similarly to `ValidatorStake`, anybody can create the stake account for a SOL staker. For new accounts, the authority is initialized to the SOL stake account's withdrawer. When stake is added to a `SolStakerStake`, it will update the total amount of staked tokens both on the corresponding `ValidatorStake` and `Config`. Therefore each `ValidatorStake` tracks the amount of staked tokens that its individual stakers are currently staking, while the `Config` tracks the total of staked tokens on the system. The system also keeps track of the SOL amount staked on the network to determine the share of the rewards and enforce that the proportion of tokens and SOL staked are within the expected limits.

Staking rewards are paid directly to the program via the `DistributeRewards` instruction while holder rewards are accumulated on the Stake program's `vault` token account. For both cases, the program offer instructions for stakers to harvest their rewards.

> [!IMPORTANT]
> There can be only one SOL staker stake account per SOL stake account and config account, since the SOL stake account is part of the SOL staker stake account seeds. Similarly, there can be only one validator stake account since the vote account is part of the validator stake account seeds.

## 🗂️ Accounts

The program makes use of three types of accounts to track staked amounts and manage parameters of the system.

- [`Config`](#config)
- [`SolStakerStake`](#solstakerstake)
- [`ValidatorStake`](#validatorstake)

### `Config`

The `Config` account tracks the total amount of staked tokens and holds the parameters for the staking system:
- `authority`: Authority that can modify any elements in the config.
- `slash authority`: Optional authority that can slash any stake account.
- `cooldown time seconds`: After a deactivation, defines the number of seconds that must pass before the stake is inactive and able to be withdrawn.
- `sync rewards lamports`: Lamports amount paid as a reward for syncing a SOL stake account.
- `maximum deactivation basis points`: The maximum proportion that can be deactivated at once, given as basis points (`1 / 10000`).

Each `Config` account is associated with a particular mint account, determined by the mint of its `vault` token account. The `vault` token account holds all the staked tokens and it is controlled by the `vault authority` of the `Config` account.

> [!NOTE]
> While staked tokens are escrowed by the `Config` account, they still accrue holder rewards in addition to the staking. There are specific instructions on the program that allows holders to claim both their "holder" and "staking" rewards.

### `SolStakerStake`

The `SolStakerStake` accounts hold the delegation information of individual SOL stakers. The delegation holds the amount of staked tokens as well as their SOL stake state.

The maximum amount of tokens that a SOL staker is allowed to stake is currently proportional to the amount of SOL staked, given by `1.3 * SOL amount staked`.

### `ValidatorStake`

The `ValidatorStake` accounts hold the delegation information for the tokens staked by a validator. It also tracks the total amount of SOL and tokens staked by its stakers.

The total amount of SOL staked on a validator is used to determine that maximum amount of tokens that the validator is allowed to stake &mdash; currently the limit is given by `1.3 * SOL amount staked`.
