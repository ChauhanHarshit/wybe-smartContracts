use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use crate::instructions::*;

declare_id!("Brh4Paf4JNBDN3NBwoNeU6LBZqM4MRx6QyTVruknTXza");

#[program]
pub mod wybe_smart_contracts {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCurveConfiguration>, fee: u64) -> Result<()> {
        instructions::init_global(ctx, fee)
    }

    pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn buy_tokens(ctx: Context<BuyTokens> , amount:u64) -> Result<()>{
        instructions::buy_tokens(ctx, amount)
    }

    pub fn sell_tokens(ctx: Context<SellTokens> , amount : u64) -> Result<()>{
        instructions::sell_tokens(ctx, amount)
    }

}
