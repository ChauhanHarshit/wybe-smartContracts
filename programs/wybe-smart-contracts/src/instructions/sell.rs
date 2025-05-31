use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Token, TokenAccount};
use crate::state::*;
use crate::errors::CustomError;

pub fn sell_tokens(ctx: Context<SellTokens>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(amount > 0, CustomError::InvalidAmount);
    require!(pool.reserve_sol > 0, CustomError::InsufficientFunds);

    let total_supply = pool.total_supply;
    require!(total_supply >= amount, CustomError::InsufficientFunds);

    let current_supply: u64 = 990_000_000 - pool.total_supply;


    let price = get_sell_price_for_amount(current_supply - amount, amount)?;
    let total_payout = price.checked_mul(amount).ok_or(CustomError::Overflow)?;

    require!(
        pool.reserve_sol >= total_payout,
        CustomError::InsufficientFunds
    );

    // Burn the tokens from the user's token account
    let cpi_accounts = Burn {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    token::burn(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        amount,
    )?;

    // Send SOL from treasury to user
    **ctx.accounts.treasury.try_borrow_mut_lamports()? -= total_payout;
    **ctx.accounts.user.try_borrow_mut_lamports()? += total_payout;

    // Update pool state
    pool.total_supply = pool.total_supply.checked_sub(amount).ok_or(CustomError::Overflow)?;
    pool.reserve_sol = pool.reserve_sol.checked_sub(total_payout).ok_or(CustomError::Overflow)?;

    Ok(())
}

#[derive(Accounts)]
pub struct SellTokens<'info> {
    #[account(mut, seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()], bump = pool.bump)]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(mut)]
    pub token_mint: Account<'info, token::Mint>,

    #[account(mut, associated_token::mint = token_mint, associated_token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
}


fn get_sell_price_for_amount(supply_after: u64, amount: u64) -> Result<u64> {
    let step_size = 1_000;
    let base_price = 1_000;       // 0.000001 SOL
    let price_increment = 10_000; // 0.00001 SOL per step

    let mut total_revenue: u128 = 0;
    let mut remaining = amount;
    let mut current_supply = supply_after + amount;

    while remaining > 0 {
        let step = (current_supply - 1) / step_size;
        let step_price = base_price + step * price_increment;

        let step_start = step * step_size;
        let tokens_in_this_step = (current_supply - step_start).min(remaining);

        total_revenue += tokens_in_this_step as u128 * step_price as u128;

        remaining -= tokens_in_this_step;
        current_supply -= tokens_in_this_step;
    }

    let avg_price = total_revenue / amount as u128;
    Ok(avg_price as u64)
}
