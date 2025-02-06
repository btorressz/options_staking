# options_staking

# Overview
The Options Staking Program is a Solana-based smart contract(program) built using the Anchor framework. This contract allows users to stake option contracts, earn rewards, and claim or compound those rewards. It supports dynamic reward adjustments, auto-restaking, early unstake penalties, and liquidity pool staking. 

devnet:(https://explorer.solana.com/address/6skdVTguzuPFbCeGju4Mg6uvdS82AcxBqTUpRzdVMRaz?cluster=devnet)

## Features

- ✅ Stake option contracts with different lock periods.
- ✅ Earn dynamic rewards based on lock period, staking duration, and total value locked (TVL)
- ✅ Unstake options early with penalties or at maturity with full rewards.
- ✅ Compound rewards back into the staking pool.
- ✅ Auto-restake options after the lock period expires.
- ✅ Liquidity pool staking for multiple options with variable lock periods.
- ✅ Dynamic reward rate adjustments based on total staked TVL.
- ✅ Emergency unstake function for system failures or protocol halts.
- ✅ Admin functionality to adjust reward rates and penalties.
- ✅ PDA-based pool authority for automated reward minting.

## Program Architecture

- **UserAccount** : Stores a user's staked options, reward balance, and lock period data.

- **StakingPool** : Manages total staked options, reward pools, staking policies, and admin-controlled parameters.

- **OptionContract** : Represents a staked option contract, including type, strike price, expiry, and lock status.

  ## Program Instructions

### 1. **Stake Options**

Locks an option contract for staking and tracks its staking period.

```rust
pub fn stake_options(ctx: Context<StakeOptions>, option_pubkey: Pubkey, lock_period: u64, option_type: u8) -> Result<()>




