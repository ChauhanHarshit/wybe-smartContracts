use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Initialize the LiquidityPool account state
    pool.set_inner(LiquidityPool::new(
        ctx.accounts.payer.key(),
        ctx.accounts.token_mint.key(),
        ctx.bumps.pool,
    ));

    // Mint 10 billion tokens (10_000_000_000 * 10^9 = 10^19 assuming 9 decimals)
    let cpi_accounts = MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        authority: ctx.accounts.pool.to_account_info(), // PDA is mint authority
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let token_mint_key = ctx.accounts.token_mint.key(); 
    let seeds = &[
        LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
        token_mint_key.as_ref(),
        &[ctx.bumps.pool],
    ];
    let signer_seeds = &[&seeds[..]];

    let amount: u64 = 10_000_000_000u64 * 1_000_000_000u64;
    token::mint_to(
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
        amount, // 10 billion tokens with 9 decimals
    )?;

    Ok(())
}


#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(
        init,
        space = LiquidityPool::ACCOUNT_SIZE,
        payer = payer,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = pool,
        mint::freeze_authority = pool,
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
