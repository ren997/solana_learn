use anchor_lang::prelude::*;

declare_id!("5e44g7KZvJuhEPEuYX6S8tHWtb2FEyCg41HvDYwwV7z5");

const SEED: &[u8] = b"data";

#[program]
pub mod lesson_5_pxsol_ss_anchor {
    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        let account_user = &ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;
        account_user_pda.auth = account_user.key();
        account_user_pda.bump = ctx.bumps.user_pda;
        account_user_pda.data = Vec::new();
        Ok(())
    }

    pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
        let account_user = &ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;

        // Update the data field with the new data.
        account_user_pda.data = data;

        // If the account was shrunk, Anchor won't automatically refund excess lamports. Refund any surplus (over the
        // new rent-exempt minimum) back to the user.
        let account_user_pda_info = account_user_pda.to_account_info();
        let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());
        let hold = **account_user_pda_info.lamports.borrow();
        if hold > rent_exemption {
            let refund = hold.saturating_sub(rent_exemption);
            **account_user_pda_info.lamports.borrow_mut() = rent_exemption;
            **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
        }
        Ok(())
    }

    pub fn close(_ctx: Context<Close>) -> Result<()> {
        // Close the PDA account and refund all lamports to the user
        // The actual closing is handled by Anchor's #[account(close = user)] attribute
        Ok(())
    }
}

#[account]
pub struct Data {
    pub auth: Pubkey,  // The owner of this pda account
    pub bump: u8,      // The bump to generate the PDA
    pub data: Vec<u8>  // The content, arbitrary bytes
}

impl Data {
    pub fn space_for(data_len: usize) -> usize {
        // 8 (discriminator) + 32 (auth) + 1 (bump) + 4 (vec len) + data_len
        8 + 32 + 1 + 4 + data_len
    }
}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [SEED, user.key().as_ref()],
        bump,
        space = Data::space_for(0)
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_data: Vec<u8>)]
pub struct Update<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED, user.key().as_ref()],
        bump = user_pda.bump,
        realloc = Data::space_for(new_data.len()),
        realloc::payer = user,
        realloc::zero = false,
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED, user.key().as_ref()],
        bump = user_pda.bump,
        close = user,  // Close account and send lamports to user
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,
    )]
    pub user_pda: Account<'info, Data>,
}

#[error_code]
pub enum PxsolError {
    #[msg("Unauthorized: only the account owner can update data")]
    Unauthorized,
}
