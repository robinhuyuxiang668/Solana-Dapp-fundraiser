# Token Fundraiser

此示例演示了如何为 SPL 代币创建筹款活动。

在此示例中，用户将能够创建一个筹款帐户，在其中他将指定他想要收集的铸币和筹款目标

## Let's walk through the architecture:

A fundraising account consists of:

```rust
#[account]
#[derive(InitSpace)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
}
```

### In this state account, we will store:

- Maker：发起筹款的人

- mint_to_raise：制造商想要收到的薄荷

- amount_to_raise：制造商试图筹集的目标金额

- current_amount：当前捐赠的总金额

- time_started：账户创建时间

- 持续时间：收集所有捐款的时间范围（以天为单位）

- 凹凸：由于我们的筹款帐户将是一个 PDA（程序派生地址），因此我们将存储该帐户的凹凸

## 我们使用 InitSpace 派生宏来实现空间 triat，它将计算我们的帐户将在链上使用的空间量（不考虑锚鉴别器）

### 用户将能够创建新的筹款活动帐户。为此，我们创建以下上下文：

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_to_raise: Account<'info, Mint>,
    #[account(
        init,
        payer = maker,
        seeds = [b"fundraiser", maker.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Fundraiser::INIT_SPACE,
    )]
    pub fundraiser: Account<'info, Fundraiser>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_to_raise,
        associated_token::authority = fundraiser,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
```

让我们仔细看看我们在这种情况下传递的帐户：

- 发起人：将是发起筹款的人。他将成为交易的签名者，我们将他的帐户标记为可变，因为我们将从该帐户中扣除 lamports

- mint_to_raise：用户想要接收的薄荷。这将是一个铸币账户，我们将用它来存储铸币地址

- 筹款者：将是我们将初始化的状态帐户，制造商将支付该帐户的初始化费用。
  我们从单词“fundraiser”的字节表示和制造商公钥的引用中推导出 Fundraiser PDA。 Anchor 将计算规范凹凸（将该地址抛出 ed25519 椭圆曲线的第一个凹凸）并将其保存在一个结构体中

- 金库：我们将初始化一个金库（ATA）来接收捐款。该帐户将源自用户想要接收的铸币以及我们刚刚创建的筹款帐户

- system_program：负责初始化任何新帐户的程序

- token_program 和 Associated_token_program：我们正在创建新的 ATA

### 然后我们为初始化上下文实现一些功能：

```rust
impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, amount: u64, duration: u8, bumps: &InitializeBumps) -> Result<()> {

        // Check if the amount to raise meets the minimum amount required
        require!(
            amount > MIN_AMOUNT_TO_RAISE.pow(self.mint_to_raise.decimals as u32),
            FundraiserError::InvalidAmount
        );

        // Initialize the fundraiser account
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
```

在这里，我们基本上只是设置筹款账户的数据，如果筹款金额大于 3（最低金额）

---

### 用户将能够为筹款活动做出贡献

缴款账户包括：

````rust
#[account]
#[derive(InitSpace)]
pub struct Contributor {
    pub amount: u64,
}
```rust

在此帐户中，我们将仅存储特定贡献者贡献的总金额

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,
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
    pub contributor_account: Account<'info, Contributor>,
    #[account(
        mut,
        associated_token::mint = mint_to_raise,
        associated_token::authority = contributor
    )]
    pub contributor_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = fundraiser.mint_to_raise,
        associated_token::authority = fundraiser
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
````

在这种情况下，我们将传递为筹款活动捐款所需的所有帐户：

- 贡献者：贡献者的地址

- mint_to_raise：制造商期望收到的贡献的薄荷

- 筹款活动：初始化的筹款活动帐户，将在其中执行适当的检查，例如适当的薄荷、种子和筹款活动 PDA 的碰撞

- 贡献者帐户：我们初始化（如果需要）一个贡献者帐户，该帐户将存储特定贡献者迄今为止贡献的总金额

-contributor_ata：我们将从中转移代币的 ata。我们确保 ATA 的权限和铸币正确（mint_to_raise 和贡献者地址），并且我们将其标记为可变，因为我们将从该帐户中扣除代币

- 金库：我们将向其存入代币的 ata。我们确保 ATA 的授权和铸币正确（mint_to_raise 和 Fundraiser 账户），并且我们将其标记为可变，因为我们将在该账户中存入代币

- token_program：我们将对代币程序执行 CPI（跨程序调用）来转移代币

### 然后我们为 Contribute 上下文实现一些功能：

```rust
impl<'info> Contribute<'info> {
    pub fn contribute(&mut self, amount: u64) -> Result<()> {

        // Check if the amount to contribute meets the minimum amount required
        require!(
            amount > 1_u8.pow(self.mint_to_raise.decimals as u32) as u64,
            FundraiserError::ContributionTooSmall
        );

        // Check if the amount to contribute is less than the maximum allowed contribution
        require!(
            amount <= (self.fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER,
            FundraiserError::ContributionTooBig
        );

        // Check if the maximum contributions per contributor have been reached
        require!(
            (self.contributor_account.amount <= (self.fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER)
                && (self.contributor_account.amount + amount <= (self.fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER),
            FundraiserError::MaximumContributionsReached
        );

        // Check if the fundraising duration has been reached
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            self.fundraiser.duration <= ((current_time - self.fundraiser.time_started) / SECONDS_TO_DAYS) as u8,
            crate::FundraiserError::FundraisingEnded
        );

        // Transfer the funds to the vault
        // CPI to the token program to transfer the funds
        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the contributor to the vault
        let cpi_accounts = Transfer {
            from: self.contributor_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.contributor.to_account_info(),
        };

        // Crete a CPI context
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Transfer the funds from the contributor to the vault
        transfer(cpi_ctx, amount)?;

        // Update the fundraiser and contributor accounts with the new amounts
        self.fundraiser.current_amount += amount;

        self.contributor_account.amount += amount;

        Ok(())
    }
}
```

在这里，我们进行一些检查：

- 我们检查用户是否存入至少一个代币

- 我们检查用户的贡献没有超过目标金额的 10%

- 我们检查用户的贡献总额不超过目标金额的 10%

- 我们检查筹款期限是否已过

之后，我们为代币程序创建一个 CPI，将一定数量的 SPL 代币从贡献者 ATA 转移到金库。
我们传递从中扣除代币的帐户的权限（在本例中是贡献者，因为他是贡献者 ata 的权限）。

最后，我们用正确的金额更新我们的州账户

---

```rust
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
```

```rust
impl<'info> CheckContributions<'info> {
    pub fn check_contributions(&self) -> Result<()> {

        // Check if the target amount has been met
        require!(
            self.vault.amount >= self.fundraiser.amount_to_raise,
            FundraiserError::TargetNotMet
        );

        // Transfer the funds to the maker
        // CPI to the token program to transfer the funds
        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the vault to the maker
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };

        // Signer seeds to sign the CPI on behalf of the fundraiser account
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"fundraiser".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.fundraiser.bump],
        ]];

        // CPI context with signer since the fundraiser account is a PDA
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        // Transfer the funds from the vault to the maker
        transfer(cpi_ctx, self.vault.amount)?;

        Ok(())
    }
}
```

## 最后，我们关闭筹款账户，并将租金中的 lamports 发送回制造商（通过筹款账户中的“关闭”约束完成）。

### 如果筹款期限已过且未达到目标，用户将可以退还其捐款

```rust
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
    pub contributor_account: Account<'info, Contributor>,
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
```

### 然后，我们为退款上下文实现一些功能：

```rust
impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {

        // Check if the fundraising duration has been reached
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            self.fundraiser.duration <= ((current_time - self.fundraiser.time_started) / SECONDS_TO_DAYS) as u8,
            crate::FundraiserError::FundraiserNotEnded
        );

        require!(
            self.vault.amount < self.fundraiser.amount_to_raise,
            crate::FundraiserError::TargetMet
        );

        // Transfer the funds back to the contributor
        // CPI to the token program to transfer the funds
        let cpi_program = self.token_program.to_account_info();

        // Transfer the funds from the vault to the contributor
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.contributor_ata.to_account_info(),
            authority: self.fundraiser.to_account_info(),
        };

        // Signer seeds to sign the CPI on behalf of the fundraiser account
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"fundraiser".as_ref(),
            self.maker.to_account_info().key.as_ref(),
            &[self.fundraiser.bump],
        ]];

        // CPI context with signer since the fundraiser account is a PDA
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        // Transfer the funds from the vault to the contributor
        transfer(cpi_ctx, self.contributor_account.amount)?;

        // Update the fundraiser state by reducing the amount contributed
        self.fundraiser.current_amount -= self.contributor_account.amount;

        Ok(())
    }
}
```

在这里，我们将检查筹款是否已经达到目标以及是否超过了持续时间。
进行适当的检查后，我们将捐赠的资金从金库转回给贡献者
