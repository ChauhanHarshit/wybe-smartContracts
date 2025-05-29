use crate::{errors::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount}, token_interface::TokenInterface,
};

pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let required_lamports: u64 = 5_700_000;
    
    let received_lamports = ctx.accounts.payer.lamports();
    require!(
        received_lamports >= required_lamports + Rent::get()?.minimum_balance(LiquidityPool::ACCOUNT_SIZE),
        CustomError::InsufficientPayment
    );

    // Transfer the SOL to a treasury or keep it in PDA (optional)
    **ctx.accounts.payer.try_borrow_mut_lamports()? -= required_lamports;
    **ctx.accounts.treasury.try_borrow_mut_lamports()? += required_lamports;

    msg!(
        "Transferred {} lamports (≈ 0.0057 SOL) from user {} to treasury {}",
        required_lamports,
        ctx.accounts.payer.key(),
        ctx.accounts.treasury.key()
    );

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

    let total_amount: u64 = 1_000_000_000u64 * 1_000_000_000u64;
    token::mint_to(
        CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer_seeds),
        total_amount, // 10 billion tokens with 9 decimals
    )?;

    // Transfer 1% to treasury ATA
    let transfer_accounts = token::Transfer {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.pool.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(cpi_program, transfer_accounts, signer_seeds),
        total_amount / 100, // 1% = 100M tokens
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

    #[account(
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: SystemAccount<'info>, // Just a PDA placeholder for ATA authority

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = treasury
    )]
    pub treasury_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
