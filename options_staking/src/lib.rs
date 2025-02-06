use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::borsh::try_from_slice_unchecked;

declare_id!("6skdVTguzuPFbCeGju4Mg6uvdS82AcxBqTUpRzdVMRaz");

#[program]
pub mod options_staking {
    use super::*;

    /// Stake an option contract with a specified lock period.
    pub fn stake_options(
        ctx: Context<StakeOptions>, 
        option_pubkey: Pubkey, 
        lock_period: u64, 
        option_type: u8
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let option_contract = &mut ctx.accounts.option_contract;

        // Ensure the option is not already locked.
        require!(!option_contract.locked, CustomError::OptionAlreadyLocked);

        // Validate the option type.
        require!(option_type == 0 || option_type == 1, CustomError::InvalidOptionType);

        // Lock the option and record it in the user's account.
        option_contract.locked = true;
        user_account.staked_options.push(option_pubkey);
        user_account.option_types.push(option_type);

        // Record the lock period and staking start time.
        user_account.lock_period = lock_period;
        user_account.staking_start_time = Clock::get()?.unix_timestamp as u64;

        // Update pool state.
        staking_pool.total_staked = staking_pool
            .total_staked
            .checked_add(1)
            .ok_or(CustomError::ArithmeticError)?;
        update_reward_rate(staking_pool)?;

        Ok(())
    }

    /// Unstake an option contract 
    // and claim rewards—with an early unstake penalty if applicable.
    pub fn unstake_options(ctx: Context<UnstakeOptions>, early_unstake: bool) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let option_contract = &mut ctx.accounts.option_contract;
        let current_time = Clock::get()?.unix_timestamp as u64;

        if early_unstake {
            // Apply an early unstake penalty.
            user_account.reward_balance = apply_early_unstaking_penalty(
                user_account.reward_balance, 
                staking_pool.early_unstake_penalty_rate
            );
        } else {
            // Ensure the lock period has passed.
            require!(
                current_time >= user_account.staking_start_time
                    .checked_add(user_account.lock_period)
                    .ok_or(CustomError::ArithmeticError)?,
                CustomError::LockPeriodNotOver
            );
        }

        // Unlock the option and remove it from the user's records.
        option_contract.locked = false;
        user_account.staked_options.retain(|&x| x != option_contract.key());
        user_account.option_types.retain(|&x| x != option_contract.option_type);

        // Calculate and add rewards.
        let duration = current_time
            .checked_sub(user_account.staking_start_time)
            .ok_or(CustomError::ArithmeticError)?;
        let reward = calculate_tiered_rewards(duration, staking_pool.reward_rate, user_account.lock_period);
        user_account.reward_balance = user_account
            .reward_balance
            .checked_add(reward)
            .ok_or(CustomError::ArithmeticError)?;

        // Update pool state.
        staking_pool.total_staked = staking_pool
            .total_staked
            .checked_sub(1)
            .ok_or(CustomError::ArithmeticError)?;
        update_reward_rate(staking_pool)?;

        Ok(())
    }

    /// Claim accumulated rewards. If `compound_rewards` is false, reward tokens are minted to the user's token account.
    pub fn claim_rewards(ctx: Context<ClaimRewards>, compound_rewards: bool) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let amount_to_claim = user_account.reward_balance;
        require!(amount_to_claim > 0, CustomError::NoRewardsToClaim);

        if compound_rewards {
            // Compound rewards by adding them back into the pool.
            staking_pool.reward_pool = staking_pool
                .reward_pool
                .checked_add(amount_to_claim)
                .ok_or(CustomError::ArithmeticError)?;
            user_account.reward_balance = 0;
        } else {
            // Mint reward tokens to the user's token account.
            let cpi_accounts = MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.reward_token_account.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            // Bind the pool key to a local variable so it lives long enough.
            let pool_key = staking_pool.key();
            let seeds = &[b"pool_authority", pool_key.as_ref(), &[staking_pool.pool_authority_bump]];
            let signer = &[&seeds[..]];
            let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::mint_to(cpi_context, amount_to_claim)?;
            user_account.reward_balance = 0;
        }

        Ok(())
    }

    /// Auto-restake: When the lock period ends, 
    //restake the option by resetting the staking start time.
    pub fn auto_restake(ctx: Context<AutoRestake>, option_pubkey: Pubkey) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let option_contract = &mut ctx.accounts.option_contract;
        let current_time = Clock::get()?.unix_timestamp as u64;

        require!(
            current_time >= user_account.staking_start_time
                .checked_add(user_account.lock_period)
                .ok_or(CustomError::ArithmeticError)?,
            CustomError::LockPeriodNotOver
        );

        // Reset staking time and update pool.
        user_account.staking_start_time = current_time;
        staking_pool.total_staked = staking_pool
            .total_staked
            .checked_add(1)
            .ok_or(CustomError::ArithmeticError)?;
        update_reward_rate(staking_pool)?;

        Ok(())
    }

    /// Liquidity pool staking: Stake multiple options with variable lock periods.
    pub fn stake_options_in_pool(
        ctx: Context<StakeOptionsInPool>,
        option_pubkeys: Vec<Pubkey>,
        lock_periods: Vec<u64>,
        option_types: Vec<u8>,
    ) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;

        require!(
            option_pubkeys.len() == lock_periods.len() && option_pubkeys.len() == option_types.len(),
            CustomError::InvalidInput
        );

        let remaining_accounts = ctx.remaining_accounts;

        for (index, option_pubkey) in option_pubkeys.iter().enumerate() {
            let option_contract_info = &remaining_accounts[index];
            require!(
                option_contract_info.key() == *option_pubkey,
                CustomError::InvalidOptionAccount
            );

            let mut option_contract = try_from_slice_unchecked::<OptionContract>(
                &option_contract_info.try_borrow_data()?
            )?;
            require!(!option_contract.locked, CustomError::OptionAlreadyLocked);
            require!(
                option_types[index] == 0 || option_types[index] == 1,
                CustomError::InvalidOptionType
            );

            option_contract.locked = true;
            user_account.staked_options.push(*option_pubkey);
            user_account.lock_periods.push(lock_periods[index]);
            user_account.option_types.push(option_types[index]);

            let mut data = option_contract_info.try_borrow_mut_data()?;
            let serialized = option_contract.try_to_vec()?;
            data[..serialized.len()].copy_from_slice(&serialized);
        }

        staking_pool.total_staked = staking_pool
            .total_staked
            .checked_add(option_pubkeys.len() as u64)
            .ok_or(CustomError::ArithmeticError)?;
        update_reward_rate(staking_pool)?;

        Ok(())
    }

    /// Emergency unstake: Unstake an option without any penalty
    // (useful if the protocol needs to halt normal operations).
    pub fn emergency_unstake(ctx: Context<EmergencyUnstake>) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let option_contract = &mut ctx.accounts.option_contract;

        option_contract.locked = false;
        user_account.staked_options.retain(|&x| x != option_contract.key());
        user_account.option_types.retain(|&x| x != option_contract.option_type);

        staking_pool.total_staked = staking_pool
            .total_staked
            .checked_sub(1)
            .ok_or(CustomError::ArithmeticError)?;
        update_reward_rate(staking_pool)?;

        Ok(())
    }

    /// Update staking parameters (admin-only function).
    pub fn update_staking_params(
        ctx: Context<UpdateStakingParams>,
        new_reward_rate: u64,
        new_penalty_rate: u64
    ) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;
        require!(ctx.accounts.admin.key() == staking_pool.admin, CustomError::Unauthorized);
        
        staking_pool.reward_rate = new_reward_rate;
        staking_pool.early_unstake_penalty_rate = new_penalty_rate;

        Ok(())
    }
}

#[account]
pub struct UserAccount {
    pub user_pubkey: Pubkey,
    pub staked_options: Vec<Pubkey>, // List of staked options.
    pub reward_balance: u64,         // Accumulated reward balance.
    pub staking_start_time: u64,     // Timestamp when staking began.
    pub lock_period: u64,            // Lock period duration (in seconds).
    pub lock_periods: Vec<u64>,      // Lock periods for liquidity pool staking.
    pub option_types: Vec<u8>,       // Option types (0 = Call, 1 = Put).
}

#[account]
pub struct StakingPool {
    pub total_staked: u64,               // Total number of staked options.
    pub reward_pool: u64,                // Reward tokens available for compounding.
    pub reward_rate: u64,                // Base reward rate per option.
    pub lock_period: u64,                // Default lock period (in seconds).
    pub early_unstake_penalty_rate: u64, // Penalty rate for early unstaking.
    pub admin: Pubkey,                 // Admin authority for updating parameters.
    pub pool_authority_bump: u8,         // Bump seed for the pool authority PDA.
}

#[account]
pub struct OptionContract {
    pub option_type: u8,   // 0 for Call, 1 for Put.
    pub strike_price: u64, // Strike price for the option.
    pub expiry: u64,       // Expiry date of the option.
    pub locked: bool,      // True if locked for staking.
}

/// Context for staking a single option.
#[derive(Accounts)]
pub struct StakeOptions<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub option_contract: Account<'info, OptionContract>,
    pub system_program: Program<'info, System>,
}

/// Context for unstaking a single option.
#[derive(Accounts)]
pub struct UnstakeOptions<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub option_contract: Account<'info, OptionContract>,
    pub system_program: Program<'info, System>,
}

/// Context for claiming rewards.
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub reward_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This account is the PDA derived for pool authority.
    #[account(
        seeds = [b"pool_authority", staking_pool.key().as_ref()],
        bump = staking_pool.pool_authority_bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,
}

/// Context for auto-restaking.
#[derive(Accounts)]
pub struct AutoRestake<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub option_contract: Account<'info, OptionContract>,
    pub system_program: Program<'info, System>,
}

/// Context for staking multiple options in a pool.
#[derive(Accounts)]
pub struct StakeOptionsInPool<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(signer)]
    pub user_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Context for emergency unstaking.
#[derive(Accounts)]
pub struct EmergencyUnstake<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub option_contract: Account<'info, OptionContract>,
    pub system_program: Program<'info, System>,
}

/// Context for updating staking parameters.
#[derive(Accounts)]
pub struct UpdateStakingParams<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(signer)]
    pub admin: AccountInfo<'info>,
}

/// Calculate tiered rewards using time-based escalation (bonus multiplier based on months staked).
pub fn calculate_tiered_rewards(stake_duration: u64, reward_rate: u64, _lock_period: u64) -> u64 {
    let seconds_in_month = 30 * 24 * 60 * 60;
    let months_staked = stake_duration / seconds_in_month;
    // Bonus multiplier: 1 + min(months_staked, 6)
    let bonus_multiplier = 1 + months_staked.min(6);
    stake_duration
        .checked_mul(reward_rate)
        .and_then(|v| v.checked_mul(bonus_multiplier))
        .unwrap_or(0)
}

/// Update the reward rate dynamically 
// based on the total staked options.
fn update_reward_rate(staking_pool: &mut StakingPool) -> Result<()> {
    let base_rate: u64 = 100;
    let max_staked_threshold: u64 = 1000;
    
    staking_pool.reward_rate = if staking_pool.total_staked < max_staked_threshold {
        base_rate
            .checked_sub(staking_pool.total_staked / 10)
            .ok_or(CustomError::ArithmeticError)?
    } else {
        base_rate / 2
    };
    
    Ok(())
}

/// Apply a penalty for early unstaking.
fn apply_early_unstaking_penalty(reward_balance: u64, penalty_rate: u64) -> u64 {
    reward_balance * (100 - penalty_rate) / 100
}

#[error_code]
pub enum CustomError {
    #[msg("Option is already locked.")]
    OptionAlreadyLocked,
    #[msg("Lock period has not expired.")]
    LockPeriodNotOver,
    #[msg("No rewards to claim.")]
    NoRewardsToClaim,
    #[msg("Invalid option type.")]
    InvalidOptionType,
    #[msg("Invalid option account provided.")]
    InvalidOptionAccount,
    #[msg("Input length mismatch.")]
    InvalidInput,
    #[msg("Arithmetic operation overflow or underflow.")]
    ArithmeticError,
    #[msg("Unauthorized.")]
    Unauthorized,
}
