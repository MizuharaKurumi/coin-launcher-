
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Locker111111111111111111111111111111111111");

#[program]
pub mod eggs_lp_locker {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.locker_data.owner = ctx.accounts.owner.key();
        Ok(())
    }

    pub fn lock(ctx: Context<Lock>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_lp_account.to_account_info(),
            to: ctx.accounts.lock_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn unlock(ctx: Context<Unlock>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.owner.key == ctx.accounts.locker_data.owner,
            LockerError::Unauthorized
        );

        let seeds = &[b"locker".as_ref(), ctx.accounts.owner.key.as_ref()];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.lock_vault.to_account_info(),
            to: ctx.accounts.user_lp_account.to_account_info(),
            authority: ctx.accounts.locker_data.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 32, seeds = [b"locker", owner.key().as_ref()], bump)]
    pub locker_data: Account<'info, LockerData>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Lock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub user_lp_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lock_vault: Account<'info, TokenAccount>,
    #[account(seeds = [b"locker", owner.key().as_ref()], bump)]
    pub locker_data: Account<'info, LockerData>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unlock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub user_lp_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lock_vault: Account<'info, TokenAccount>,
    #[account(seeds = [b"locker", owner.key().as_ref()], bump)]
    pub locker_data: Account<'info, LockerData>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct LockerData {
    pub owner: Pubkey,
}

#[error_code]
pub enum LockerError {
    #[msg("Unauthorized: only the locker owner can unlock.")]
    Unauthorized,
}
