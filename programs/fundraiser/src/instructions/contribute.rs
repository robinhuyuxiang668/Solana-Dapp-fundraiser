use anchor_lang::prelude::*;
use anchor_spl::token::{
    Mint, 
    transfer, 
    Token, 
    TokenAccount, 
    Transfer
};

use crate::{
     state::{Contributor, Fundraiser}, FundraiserError, ANCHOR_DISCRIMINATOR, MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS
};

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,//贡献者的地址
    pub mint_to_raise: Account<'info, Mint>,
    #[account(
        mut,
        has_one = mint_to_raise,
        seeds = [b"fundraiser".as_ref(), fundraiser.maker.as_ref()],
        bump = fundraiser.bump,
    )]
    pub fundraiser: Account<'info, Fundraiser>,
    #[account(
        init_if_needed,
        payer = contributor,
        seeds = [b"contributor", fundraiser.key().as_ref(), contributor.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Contributor::INIT_SPACE,
    )]
    pub contributor_account: Account<'info, Contributor>,//存储特定贡献者迄今为止贡献的总金额
    #[account(
        mut,
        associated_token::mint = mint_to_raise,
        associated_token::authority = contributor
    )]
    pub contributor_ata: Account<'info, TokenAccount>,//贡献者 TokenAccount
    #[account(
        mut,
        associated_token::mint = fundraiser.mint_to_raise,
        associated_token::authority = fundraiser
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn contribute(&mut self, amount: u64) -> Result<()> {

        //检查用户是否存入至少一个代币
        require!(
            amount >= 1_u64.pow(self.mint_to_raise.decimals as u32), 
            FundraiserError::ContributionTooSmall
        );

        let max = (self.fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
        // 用户的贡献没有超过目标金额的 10%
        require!(
            amount <= max, 
            FundraiserError::ContributionTooBig
        );

        // 筹款期限是否已过
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            self.fundraiser.duration <= ((current_time - self.fundraiser.time_started) / SECONDS_TO_DAYS) as u16,
            FundraiserError::FundraiserEnded
        );

        // 已达到每位贡献者的最大捐款额
        require!((self.contributor_account.amount <= max) && 
                 (self.contributor_account.amount + amount <= max),
            FundraiserError::MaximumContributionsReached
        );

        let cpi_program = self.token_program.to_account_info();
        // 将一定数量的 SPL 代币从贡献者 ATA 转移到金库
        let cpi_accounts = Transfer {
            from: self.contributor_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.contributor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        // Update the fundraiser and contributor accounts with the new amounts
        self.fundraiser.current_amount += amount;

        self.contributor_account.amount += amount;

        Ok(())
    }
}