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

    let price = get_sell_price_for_amount(total_supply - amount, amount)?;
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
    let step_size = 1000;
    let base_price = 1_000_000;       // 0.001 SOL
    let price_increment = 1_000_000;  // 0.001 SOL per step

    let end_supply = supply_after + amount - 1;
    let start_step = end_supply / step_size;
    let end_step = supply_after / step_size;

    if start_step == end_step {
        let price = base_price + start_step * price_increment;
        return Ok(price);
    }

    let mut total_cost: u128 = 0;
    let mut remaining = amount;
    let mut step = start_step;
    let mut current_supply = supply_after + amount;

    while remaining > 0 {
        let step_start_supply = (step * step_size).max(supply_after);
        let tokens_in_step = current_supply - step_start_supply;

        let price = base_price + step * price_increment;
        total_cost += tokens_in_step as u128 * price as u128;

        remaining -= tokens_in_step;
        current_supply -= tokens_in_step;
        if step == 0 { break; } // Prevent underflow
        step -= 1;
    }

    let avg_price = total_cost / amount as u128;
    Ok(avg_price as u64)
}
