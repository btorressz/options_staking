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

 
# Program Instructions

## 1. **Stake Options**

This feature allows users to lock an option contract for staking and track its staking period.

- The option being staked must not already be locked.
- The option type is validated (0 for Call, 1 for Put).
- The option is added to the user's staking list and locked for the specified period.
- The staking pool statistics and reward rate are updated to reflect the new stake.

**Accounts involved:**
- `user_account`: The user's account that is staking the option.
- `staking_pool`: The pool where all the stakes are tracked.
- `option_contract`: The option contract that is being staked.
- `system_program`: The program that handles system-level operations.

---

## 2. **Unstake Options**

This feature allows users to unstake their options and claim rewards. If a user decides to unstake early, a penalty is applied to the rewards.

- If unstaked early, the system applies a penalty to the accumulated rewards.
- If unstaked after the lock period, the option is unlocked and removed from the user's staked list.
- The system calculates staking rewards based on the duration the option was staked and the Total Value Locked (TVL) in the pool.
- The staking pool statistics are updated and rewards are recalculated.

**Accounts involved:**
- `user_account`: The user's account that is unstaking the option.
- `staking_pool`: The pool from which the option is being unstaked.
- `option_contract`: The option contract that is being unstaked.
- `system_program`: The program that handles system-level operations.

---

## 3. **Claim Rewards**

Users can claim their accumulated rewards or choose to compound them back into the staking pool for further staking.

- The accumulated rewards are transferred to the user’s account.
- If the user chooses to compound rewards, they are added back into the staking pool.
- The rewards are minted using a pool authority derived from a Program Derived Address (PDA).

**Accounts involved:**
- `user_account`: The account where rewards are sent or from which rewards are compounded.
- `staking_pool`: The staking pool from which the rewards are calculated.
- `reward_token_account`: The account holding the reward tokens.
- `reward_mint`: The mint for the reward token.
- `token_program`: The token program used for handling tokens.
- `pool_authority`: The derived PDA representing the staking pool authority.

---

## 4. **Auto-Restake**

This feature automatically restakes an option after its lock period has expired.

- The system ensures that the lock period for the option has passed before it restakes.
- The staking start time is reset to reflect the new staking period.
- The staking pool statistics are updated, and the reward rate is recalculated based on the new stake.

**Accounts involved:**
- `user_account`: The user's account that will be restaking the option.
- `staking_pool`: The pool where the restake will happen.
- `option_contract`: The option contract that is being restaked.
- `system_program`: The program that handles system-level operations.

---

## 5. **Stake Multiple Options in Pool**

This feature allows users to stake multiple options with different lock periods in a single staking pool.

- The system ensures that the input vectors for the options, lock periods, and option types are of matching lengths.
- Each option type is validated and locked in the staking pool.
- The reward rate for the pool is updated dynamically based on the Total Value Locked (TVL) in the pool.

**Accounts involved:**
- `user_account`: The user's account that is staking multiple options.
- `staking_pool`: The pool where all the options are being staked.
- `user_authority`: The authority managing the user's staking options.
- `system_program`: The program that handles system-level operations.
- Additional accounts for each of the option contracts being staked.

---

## Reward Mechanism

### **Tiered Rewards**

Rewards scale based on the staking duration and lock period, incentivizing longer lock periods with higher rewards.

**Bonus Multipliers:**
- **3x Rewards** → Lock period ≥ 180 days.
- **2x Rewards** → Lock period ≥ 30 days.
- **1x Rewards** → Default rate (for shorter lock periods).

### **Dynamic Reward Adjustments**

Reward rates are dynamically adjusted based on the Total Value Locked (TVL) in the staking pool.

- If the TVL in the staking pool is less than 1000 options, rewards are reduced based on the total amount staked.
- If the TVL is greater than or equal to 1000 options, the base reward rate is halved.

### **Early Unstake Penalty**

If users unstake their options before the lock period has ended, they receive a penalty that reduces their rewards.

**Penalty Calculation:**
- The penalty is applied to the rewards balance using a specified penalty rate.
-  The formula for the early unstake penalty is: penalty = reward_balance * penalty_rate / 100

---
## TECH STACK 
- Rust
- Typescript
- Anchor
- Solana
- Solana Playground IDE
  
---

## License
- ***MIT LICENSE***


