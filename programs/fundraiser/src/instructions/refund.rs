use anchor_lang::prelude::*;
use anchor_spl::token::{
    transfer, 
    Mint, 
    Token, 
    TokenAccount, 
    Transfer
};

use crate::{
    state::{
        Contributor, 
        Fundraiser
    }, 
    SECONDS_TO_DAYS
};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub mint_to_raise: Account<'info, Mint>,
    #[account(
        mut,
        has_one = mint_to_raise,
        seeds = [b"fundraiser", maker.key().as_ref()],
        bump = fundraiser.bump,
    )]
    pub fundraiser: Account<'info, Fundraiser>,
    #[account(
        mut,
        seeds = [b"contributor", fundraiser.key().as_ref(), contributor.key().as_ref()],
        bump,
        close = contributor,
    )]
    pub contributor_account: Account<'info, Contributor>,//一个初始化的贡献者帐户，将存储特定贡献者迄今为止贡献的总金额
    #[account(
        mut,
        associated_token::mint = mint_to_raise,
        associated_token::authority = contributor
    )]
    pub contributor_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_to_raise,
        associated_token::authority = fundraiser
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {

        // 检查募集期限是否已达到
        let current_time = Clock::get()?.unix_timestamp;
 
        require!(
            self.fundraiser.duration >= ((current_time - self.fundraiser.time_started) / SECONDS_TO_DAYS) as u16,
            crate::FundraiserError::FundraiserNotEnded
        );

        require!(
            self.vault.amount < self.fundraiser.amount_to_raise,
            crate::FundraiserError::TargetMet
        );

        //将资金转回给贡献者
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.contributor_ata.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"fundraiser".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.fundraiser.bump],
        ]];

        //由于筹款账户是 PDA，因此 CPI 与签名者相关
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);
        transfer(cpi_ctx, self.contributor_account.amount)?;
        self.fundraiser.current_amount -= self.contributor_account.amount;

        Ok(())
    }
}