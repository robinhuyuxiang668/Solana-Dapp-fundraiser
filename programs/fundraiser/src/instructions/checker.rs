use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        transfer, 
        Mint, 
        Token, 
        TokenAccount, 
        Transfer
    }
};

use crate::{
    state::Fundraiser, 
    FundraiserError
};

#[derive(Accounts)]
pub struct CheckContributions<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_to_raise: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"fundraiser".as_ref(), maker.key().as_ref()],
        bump = fundraiser.bump,
        close = maker,
    )]
    pub fundraiser: Account<'info, Fundraiser>,
    #[account(
        mut,
        associated_token::mint = mint_to_raise,
        associated_token::authority = fundraiser,
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_to_raise,
        associated_token::authority = maker,
    )]
    pub maker_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CheckContributions<'info> {
    pub fn check_contributions(&self) -> Result<()> {
        
        // 检查是否达到目标金额
        require!(
            self.vault.amount >= self.fundraiser.amount_to_raise,
            FundraiserError::TargetNotMet
        );

        // 将资金转移给制造商 CPI 向代币程序转移资金
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };
     
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"fundraiser".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.fundraiser.bump],
        ]];

        // 由于筹款账户是 PDA，因此 CPI 与签名者相关
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);
        transfer(cpi_ctx, self.vault.amount)?;

        Ok(())
    }
}