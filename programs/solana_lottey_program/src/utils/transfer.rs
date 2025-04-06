use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub fn transfer_token_to_pool<'info>(
    from: &Account<'info, TokenAccount>,
    to: AccountInfo<'info>,
    amount: u64,
    token_program: &Program<'info, Token>,
    authority: AccountInfo<'info>,
) -> Result<()> {
    anchor_spl::token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

pub fn transfer_token_from_pool<'info>(
    from: &Account<'info, TokenAccount>,
    to: AccountInfo<'info>,
    amount: u64,
    token_program: &Program<'info, Token>,
    authority: &AccountInfo<'info>,
    bump: u8,
) -> Result<()> {
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
            &[&["GLOBAL".as_bytes(), &[bump]]],
        ),
        amount,
    )?;

    Ok(())
}
