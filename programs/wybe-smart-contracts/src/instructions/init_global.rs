use crate::{errors::CustomError, state::*};
use anchor_lang::prelude::*;

pub fn init_global(
    ctx: Context<InitializeCurveConfiguration>,
    fees: f64,
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_configuration_account;

    if fees < 0_f64 || fees > 100_f64 {
        return err!(CustomError::InvalidFee);
    }

    global_config.set_inner(CurveConfiguration::new(fees));

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeCurveConfiguration<'info> {
    #[account(
        init,
        space = CurveConfiguration::ACCOUNT_SIZE,
        payer = admin,
        seeds = [CurveConfiguration::SEED.as_bytes()],
        bump,
    )]
    pub global_configuration_account: Box<Account<'info, CurveConfiguration>>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
