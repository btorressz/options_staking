# options_staking

# Overview
The Options Staking Program is a Solana-based smart contract built using the Anchor framework. This contract allows users to stake option contracts, earn rewards, and claim or compound those rewards. It supports dynamic reward adjustments, auto-restaking, early unstake penalties, and liquidity pool staking.

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
