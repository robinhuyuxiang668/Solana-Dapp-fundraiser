use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        Mint, 
        Token, 
        TokenAccount
    }
};
use crate::{
     state::Fundraiser, FundraiserError, ANCHOR_DISCRIMINATOR, MIN_AMOUNT_TO_RAISE
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,//发起筹款的人
    pub mint_to_raise: Account<'info, Mint>,//制造商想要收到的token
    #[account(
        init,
        payer = maker,
        seeds = [b"fundraiser", maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Fundraiser::INIT_SPACE,
    )]
    pub fundraiser: Account<'info, Fundraiser>,//pda 筹款信息
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_to_raise,
        associated_token::authority = fundraiser,
    )]
    pub vault: Account<'info, TokenAccount>,//金库：我们将初始化一个金库（ATA）来接收捐款。该帐户将源自用户想要接收的铸币以及我们刚刚创建的筹款帐户
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, amount: u64, duration: u16, bumps: &InitializeBumps) -> Result<()> {

        //检查筹集金额是否符合最低金额要求
        require!(
            amount >= MIN_AMOUNT_TO_RAISE.pow(self.mint_to_raise.decimals as u32),
            FundraiserError::InvalidAmount
        );

        // 初始化筹款账户
        self.fundraiser.set_inner(Fundraiser {
            maker: self.maker.key(),
            mint_to_raise: self.mint_to_raise.key(),
            amount_to_raise: amount,
            current_amount: 0,
            time_started: Clock::get()?.unix_timestamp,
            duration,
            bump: bumps.fundraiser
        });
        
        Ok(())
    }
}