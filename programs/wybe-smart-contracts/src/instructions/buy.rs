use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::CustomError;
use crate::state::*;

pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let curve_config = &ctx.accounts.global_configuration_account;

    require!(amount > 0, CustomError::InvalidAmount);
    require!(
        ctx.accounts.pool_token_account.amount >= amount,
        CustomError::InsufficientFunds
    );

    let current_supply: u64 = (990_000_000 * 1_000_000_000) - pool.total_supply;
    // Get current step pricing
    let price = get_buy_price_for_amount(current_supply, amount)?;
    let total_cost = price.checked_mul(amount).ok_or(CustomError::Overflow)?;

    // Apply fee if any
    let fee_bps = (curve_config.fees * 100) as u64; // convert % to basis points
    let fee = total_cost.checked_mul(fee_bps).unwrap_or(0) / 10_000;
    let total_with_fee = total_cost.checked_add(fee).ok_or(CustomError::Overflow)?;

    // Transfer SOL to treasury
    **ctx.accounts.user.try_borrow_mut_lamports()? -= total_with_fee;
    **ctx.accounts.treasury.try_borrow_mut_lamports()? += total_with_fee;

    // Transfer tokens to user from pool
    let token_mint_key = ctx.accounts.token_mint.key();
    let seeds = &[
        LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
        token_mint_key.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    let authority_info = pool.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.pool_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            // authority: ctx.accounts.pool.to_account_info(),
            authority: authority_info,
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)?;

    pool.total_supply = pool
        .total_supply
        .checked_sub(amount)
        .ok_or(CustomError::Overflow)?;
    // pool.reserve_sol = pool.reserve_sol.checked_add(total_cost).ok_or(CustomError::Overflow)?;

    Ok(())
}

fn get_buy_price_for_amount(current_supply: u64, amount: u64) -> Result<u64> {
    let step_size = 1000;
    let base_price = 1000; // 0.000001 SOL in lamports
    let price_increment = 10_000; // 0.00001 SOL in lamports

    let mut cost: u128 = 0;
    let mut remaining = amount;
    let mut supply = current_supply;

    while remaining > 0 {
        let step = supply / step_size;
        let step_price = base_price + step * price_increment;

        let next_step_at = (step + 1) * step_size;
        let tokens_in_this_step = (next_step_at - supply).min(remaining);

        cost = cost
            .checked_add(step_price as u128 * tokens_in_this_step as u128)
            .ok_or(CustomError::Overflow)?;

        remaining -= tokens_in_this_step;
        supply += tokens_in_this_step;
    }

    Ok((cost / amount as u128) as u64)
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(mut)]
    pub global_configuration_account: Box<Account<'info, CurveConfiguration>>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = pool,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
}
