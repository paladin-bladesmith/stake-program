# Paladin Stake Program

<a href="https://explorer.solana.com/address/PStake1111111111111111111111111111111111111"><img src="https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fpaladin%2Fstake%2Fmain%2Fprogram%2Fidl.json&query=%24.version&label=program&logo=data:image/svg%2bxml;base64,PHN2ZyB3aWR0aD0iMzEzIiBoZWlnaHQ9IjI4MSIgdmlld0JveD0iMCAwIDMxMyAyODEiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxnIGNsaXAtcGF0aD0idXJsKCNjbGlwMF80NzZfMjQzMCkiPgo8cGF0aCBkPSJNMzExLjMxOCAyMjEuMDU3TDI1OS42NiAyNzYuNTU4QzI1OC41MzcgMjc3Ljc2NCAyNTcuMTc4IDI3OC43MjUgMjU1LjY2OSAyNzkuMzgyQzI1NC4xNTkgMjgwLjAzOSAyNTIuNTMgMjgwLjM3OCAyNTAuODg0IDI4MC4zNzdINS45OTcxOUM0LjgyODcgMjgwLjM3NyAzLjY4NTY4IDI4MC4wMzUgMi43MDg1NSAyNzkuMzkzQzEuNzMxNDMgMjc4Ljc1MSAwLjk2Mjc3MSAyNzcuODM3IDAuNDk3MDIgMjc2Ljc2NEMwLjAzMTI2OTEgMjc1LjY5IC0wLjExMTI4NiAyNzQuNTA0IDAuMDg2ODcxMiAyNzMuMzVDMC4yODUwMjggMjcyLjE5NiAwLjgxNTI2NSAyNzEuMTI2IDEuNjEyNDMgMjcwLjI3TDUzLjMwOTkgMjE0Ljc2OUM1NC40Mjk5IDIxMy41NjYgNTUuNzg0MyAyMTIuNjA3IDU3LjI4OTMgMjExLjk1QzU4Ljc5NDMgMjExLjI5MyA2MC40MTc4IDIxMC45NTMgNjIuMDU5NSAyMTAuOTVIMzA2LjkzM0MzMDguMTAxIDIxMC45NSAzMDkuMjQ0IDIxMS4yOTIgMzEwLjIyMSAyMTEuOTM0QzMxMS4xOTkgMjEyLjU3NiAzMTEuOTY3IDIxMy40OSAzMTIuNDMzIDIxNC41NjRDMzEyLjg5OSAyMTUuNjM3IDMxMy4wNDEgMjE2LjgyNCAzMTIuODQzIDIxNy45NzdDMzEyLjY0NSAyMTkuMTMxIDMxMi4xMTUgMjIwLjIwMSAzMTEuMzE4IDIyMS4wNTdaTTI1OS42NiAxMDkuMjk0QzI1OC41MzcgMTA4LjA4OCAyNTcuMTc4IDEwNy4xMjcgMjU1LjY2OSAxMDYuNDdDMjU0LjE1OSAxMDUuODEzIDI1Mi41MyAxMDUuNDc0IDI1MC44ODQgMTA1LjQ3NUg1Ljk5NzE5QzQuODI4NyAxMDUuNDc1IDMuNjg1NjggMTA1LjgxNyAyLjcwODU1IDEwNi40NTlDMS43MzE0MyAxMDcuMTAxIDAuOTYyNzcxIDEwOC4wMTUgMC40OTcwMiAxMDkuMDg4QzAuMDMxMjY5MSAxMTAuMTYyIC0wLjExMTI4NiAxMTEuMzQ4IDAuMDg2ODcxMiAxMTIuNTAyQzAuMjg1MDI4IDExMy42NTYgMC44MTUyNjUgMTE0LjcyNiAxLjYxMjQzIDExNS41ODJMNTMuMzA5OSAxNzEuMDgzQzU0LjQyOTkgMTcyLjI4NiA1NS43ODQzIDE3My4yNDUgNTcuMjg5MyAxNzMuOTAyQzU4Ljc5NDMgMTc0LjU1OSA2MC40MTc4IDE3NC44OTkgNjIuMDU5NSAxNzQuOTAySDMwNi45MzNDMzA4LjEwMSAxNzQuOTAyIDMwOS4yNDQgMTc0LjU2IDMxMC4yMjEgMTczLjkxOEMzMTEuMTk5IDE3My4yNzYgMzExLjk2NyAxNzIuMzYyIDMxMi40MzMgMTcxLjI4OEMzMTIuODk5IDE3MC4yMTUgMzEzLjA0MSAxNjkuMDI4IDMxMi44NDMgMTY3Ljg3NUMzMTIuNjQ1IDE2Ni43MjEgMzEyLjExNSAxNjUuNjUxIDMxMS4zMTggMTY0Ljc5NUwyNTkuNjYgMTA5LjI5NFpNNS45OTcxOSA2OS40MjY3SDI1MC44ODRDMjUyLjUzIDY5LjQyNzUgMjU0LjE1OSA2OS4wODkgMjU1LjY2OSA2OC40MzJDMjU3LjE3OCA2Ny43NzUxIDI1OC41MzcgNjYuODEzOSAyNTkuNjYgNjUuNjA4MkwzMTEuMzE4IDEwLjEwNjlDMzEyLjExNSA5LjI1MTA3IDMxMi42NDUgOC4xODA1NiAzMTIuODQzIDcuMDI2OTVDMzEzLjA0MSA1Ljg3MzM0IDMxMi44OTkgNC42ODY4NiAzMTIuNDMzIDMuNjEzM0MzMTEuOTY3IDIuNTM5NzQgMzExLjE5OSAxLjYyNTg2IDMxMC4yMjEgMC45ODM5NDFDMzA5LjI0NCAwLjM0MjAyNiAzMDguMTAxIDMuOTUzMTRlLTA1IDMwNi45MzMgMEw2Mi4wNTk1IDBDNjAuNDE3OCAwLjAwMjc5ODY2IDU4Ljc5NDMgMC4zNDMxNCA1Ny4yODkzIDAuOTk5OTUzQzU1Ljc4NDMgMS42NTY3NyA1NC40Mjk5IDIuNjE2MDcgNTMuMzA5OSAzLjgxODQ3TDEuNjI1NzYgNTkuMzE5N0MwLjgyOTM2MSA2MC4xNzQ4IDAuMjk5MzU5IDYxLjI0NCAwLjEwMDc1MiA2Mi4zOTY0Qy0wLjA5Nzg1MzkgNjMuNTQ4OCAwLjA0MzU2OTggNjQuNzM0MiAwLjUwNzY3OSA2NS44MDczQzAuOTcxNzg5IDY2Ljg4MDMgMS43Mzg0MSA2Ny43OTQzIDIuNzEzNTIgNjguNDM3MkMzLjY4ODYzIDY5LjA4MDIgNC44Mjk4NCA2OS40MjQgNS45OTcxOSA2OS40MjY3WiIgZmlsbD0idXJsKCNwYWludDBfbGluZWFyXzQ3Nl8yNDMwKSIvPgo8L2c+CjxkZWZzPgo8bGluZWFyR3JhZGllbnQgaWQ9InBhaW50MF9saW5lYXJfNDc2XzI0MzAiIHgxPSIyNi40MTUiIHkxPSIyODcuMDU5IiB4Mj0iMjgzLjczNSIgeTI9Ii0yLjQ5NTc0IiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+CjxzdG9wIG9mZnNldD0iMC4wOCIgc3RvcC1jb2xvcj0iIzk5NDVGRiIvPgo8c3RvcCBvZmZzZXQ9IjAuMyIgc3RvcC1jb2xvcj0iIzg3NTJGMyIvPgo8c3RvcCBvZmZzZXQ9IjAuNSIgc3RvcC1jb2xvcj0iIzU0OTdENSIvPgo8c3RvcCBvZmZzZXQ9IjAuNiIgc3RvcC1jb2xvcj0iIzQzQjRDQSIvPgo8c3RvcCBvZmZzZXQ9IjAuNzIiIHN0b3AtY29sb3I9IiMyOEUwQjkiLz4KPHN0b3Agb2Zmc2V0PSIwLjk3IiBzdG9wLWNvbG9yPSIjMTlGQjlCIi8+CjwvbGluZWFyR3JhZGllbnQ+CjxjbGlwUGF0aCBpZD0iY2xpcDBfNDc2XzI0MzAiPgo8cmVjdCB3aWR0aD0iMzEyLjkzIiBoZWlnaHQ9IjI4MC4zNzciIGZpbGw9IndoaXRlIi8+CjwvY2xpcFBhdGg+CjwvZGVmcz4KPC9zdmc+Cg==&color=9945FF" /></a>
<a href="https://www.npmjs.com/package/@paladin/stake"><img src="https://img.shields.io/npm/v/%40paladin%2Fstake?logo=npm&color=377CC0" /></a>
<a href="https://crates.io/crates/paladin-stake"><img src="https://img.shields.io/crates/v/paladin-stake?logo=rust" /></a>

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
* `authority`: Authority that can modify any elements in the config.
* `slash authority`: Optional authority that can slash any stake account.
* `cooldown time seconds`: After a deactivation, defines the number of seconds that must pass before the stake is inactive and able to be withdrawn.
* `sync rewards lamports`: Lamports amount paid as a reward for syncing a SOL stake account.
* `maximum deactivation basis points`: The maximum proportion that can be deactivated at once, given as basis points (`1 / 10000`).

Each `Config` account is associated with a particular mint account, determined by the mint of its `vault` token account. The `vault` token account holds all the staked tokens and it is controlled by the `vault authority` of the `Config` account.

> [!NOTE]
> While staked tokens are escrowed by the `Config` account, they still accrue holder rewards in addition to the staking. There are specific instructions on the program that allows holders to claim both their "holder" and "staking" rewards.

### `SolStakerStake`

The `SolStakerStake` accounts hold the delegation information of individual SOL stakers. The delegation holds the amount of staked tokens as well as their SOL stake state.

The maximum amount of tokens that a SOL staker is allowed to stake is currently proportional to the amount of SOL staked, given by `1.3 * SOL amount staked`.


### `ValidatorStake`

The `ValidatorStake` accounts hold the delegation information for the tokens staked by a validator. It also tracks the total amount of SOL and tokens staked by its stakers.

The total amount of SOL staked on a validator is used to determine that maximum amount of tokens that the validator is allowed to stake &mdash; currently the limit is given by `1.3 * SOL amount staked`.

## 📋 Instructions

- [`DeactivateStake`](#deactivatestake)
- [`DistributeRewards`](#distributerewards)
- [`InactivateSolStakerStake`](#inactivatesolstakerstake)
- [`InactivateValidatorStake`](#inactivatevalidatorstake)
- [`InitializeConfig`](#initializeconfig)
- [`InitializeSolStakerStake`](#initializesolstakerstake)
- [`InitializeValidatorStake`](#initializevalidatorstake)
- [`HarvestHolderRewards`](#harvestholderrewards)
- [`HarvestSolStakerRewards`](#harvestsolstakerrewards)
- [`HarvestSyncRewards`](#harvestsyncrewards)
- [`HarvestValidatorRewards`](#harvestvalidatorrewards)
- [`SetAuthority`](#setauthority)
- [`SlashSolStakerStake`](#slashsolstakerstake)
- [`SlashValidatorStake`](#slashvalidatorstake)
- [`SolStakerStakeTokens`](#solstakerstaketokens)
- [`SyncSolStake`](#syncsolstake)
- [`UpdateConfig`](#updateconfig)
- [`ValidatorStakeTokens`](#validatorstaketokens)
- [`WithdrawInactiveStake`](#withdrawinactivestake)

### `DeactivateStake`

Deactivate staked tokens for a stake delegation, either `ValidatorStake` or `SolStakerStake`. Only one deactivation may be in-flight at once, so if this is called with an active deactivation, it will succeed, but reset the amount and timestamp. An active deactivation can be cancelled by executing this instruction with a `0` (zero) amount.

### `DistributeRewards`

Moves SOL rewards to the `Config` and updates the stake rewards total. This intruction increments the staking rewards on the system.

### `InactivateSolStakerStake`

Move tokens from deactivating to inactive. This effectively reduces the total voting power for the SOL staker stake account, the total staked amount on the corresponding validator stake and config accounts. This instruction is used prior to withdraw staked tokens.

> [!NOTE]
> This instruction is permissionless, so anybody can finish deactivating someone's tokens, preparing them to be withdrawn.

### `InactivateValidatorStake`

Move tokens from deactivating to inactive. Reduces the total voting power for the validator stake account and the total staked amount on the system. This instruction is used prior to withdraw staked tokens.

> [!NOTE]
> This instruction is permissionless, so anybody can finish deactivating validator's tokens, preparing them to be withdrawn.

### `InitializeConfig`

Creates stake `Config` account which controls staking parameters. This is the first instruction required to set up the staking system. In addition to the staking configuration, the instruction expects the `mint` and `vault` accounts. The `mint` determines the type of tokens to be staked while the `vault` is the escrow token account to hold the staked tokens.

### `InitializeSolStakerStake`

Initializes `SolStakerStake` account data for a SOL staker. This instruction can be used multiple times to add a new staker to the stake system. Stakers are uniquely identified by their `StakeState`, i.e., there is only one `SolStakerStake` account for each (`StakeState`, `Config`) pair.

The `SolStakerStake` serves the purpose of managing the stake amount of an individual staker, tracking the SOL amount staked on the network to determine the allowed limit of tokens staked.

> [!NOTE]
> Anybody can create the stake account for a SOL staker. For new accounts, the authority is initialized to the stake state account's withdrawer.

### `InitializeValidatorStake`

Initializes `ValidatorStake` account data for a validator. This instruction can be used multiple times to add validators to the stake system. Validators are uniquely identified by their `VoteState`, i.e., there is only one `ValidatorStake` account for each (`VoteState`, `Config`) pair.

The `ValidatorStake` serves two purposes on the staking system: (1) it allows individual staker (`SolStakerStake` account) to stake tokens on the system; and (2) it allows validators to stake tokens on the system. Each validator tracks the SOL amount staked on the network of its stakers, which in turn determines the amount of tokens that a validator and its stakers are allowed to stake.

> [!NOTE]
>  Anybody can create the stake account for a validator. For new accounts, the authority is initialized to the validator vote account's withdraw authority.

### `HarvestHolderRewards`

Harvests holder SOL rewards earned by the given stake account. This instruction supports claiming rewards for both `ValidatorStake` and `SolStakerStake` accounts.

### `HarvestSolStakerRewards`

Harvests staker SOL rewards earned by the given SOL staker stake account.

### `HarvestSyncRewards`

Harvest the rewards from syncing the SOL stake balance with a validator and SOL staker stake accounts.

The staking system requires the SOL staked amount to be up to date with the `StakeState` network delegation amount. This instruction is used to sync their amounts, rewarding the address that executes the instrution.

> [!NOTE]
> This is a permissionless instruction, anybody can sync the balance of a SOL stake account. Rewards are only paid when the balances are out-of-sync.

### `HarvestValidatorRewards`

Harvests staker SOL rewards earned by the given validator stake account.

### `SetAuthority`

Sets new authority on a config or stake account.

### `SlashSolStakerStake`

Slashes a `SolStakerStake` account for the given amount. Burns the given amount of tokens from the vault account, and reduces the amount in the stake account. This instruction is executed by the `Config`'s slash authority, usually determined by a governance proposal.

### `SlashValidatorStake`

Slashes a `ValidatorStake` account for the given amount. Burns the given amount of tokens from the vault account, and reduces the amount in the stake account. This instruction is executed by the `Config`'s slash authority, usually determined by a governance proposal.

### `SolStakerStakeTokens`

Stakes tokens with the given config. This instruction is used by SOL staker stake accounts. The total amount of staked tokens is limited to the `1.3 * current amount of SOL` staked by the SOL staker.

### `SyncSolStake`

Sync the SOL stake balance with a validator and SOL staker stake accounts.

> [!NOTE]
> This is a permissionless instruction. Anybody can sync the balance of a SOL stake account.

### `UpdateConfig`

Updates configuration parameters of the stake system.

### `ValidatorStakeTokens`

Stakes tokens with the given config. This instruction is used by validator stake accounts. The total amount of staked tokens is currently limited to the `1.3 * current amount of SOL` staked to the validator.

### `WithdrawInactiveStake`

Withdraw inactive staked tokens from the vault. After a deactivation has gone through the cooldown period and been "inactivated", the authority may move the tokens out of the vault. This instruction support both `ValidatorStake` and `SolStakerStake` accounts.
