use anchor_lang::prelude::*;
use anchor_spl::associated_token::{AssociatedToken};
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::sysvar::instructions::ID as IX_ID;
pub mod utils;
pub mod errors;

declare_id!("GCcYJLJuYu8GVwgzZcEopC1sHx5f6J1u5PjGgrsEpNa9");
#[program]
pub mod solana_lottery_program {
    use anchor_lang::solana_program::keccak;
    use crate::errors::{ClaimErrorCode, StakeErrorCode};
    use crate::utils::{transfer_token_from_pool, transfer_token_to_pool};
    use super::*;

    pub fn init_config(
        ctx: Context<InitConfig>,
        signer_address: Pubkey,
        token_mint_address: Pubkey,
        lottery_fees: u64,
    ) -> Result<()> {
        let global_config = &mut ctx.accounts.global_config.load_init()?;
        const ADMIN_PUBKEY:&str = "9nBEAzgig4PCbY2jyNfKLQM7uX51EpLsvg6ptGoHRPxW" ; // owner
        require!(ctx.accounts.signer.key().to_string() == ADMIN_PUBKEY, StakeErrorCode::NotSigner);
        global_config.signer_address = signer_address;
        global_config.token_mint_address = token_mint_address;
        global_config.lottery_fees = lottery_fees;
        Ok(())
    }

    pub fn lottery(ctx: Context<Lottery>, nft_name:String) -> Result<()> {
        let block_number = Clock::get()?.slot;
        let global_config = &mut ctx.accounts.global_config.load_mut()?;

        msg!("transfer to pool {}", (global_config.lottery_fees));
        transfer_token_to_pool(
            &ctx.accounts.user_token_account,
            ctx.accounts.pool_token_account.to_account_info(),
            global_config.lottery_fees,
            &ctx.accounts.token_program,
            ctx.accounts.payer.to_account_info(),
        )?;

        // 3. Emit the event
        emit!(
            LotteryEvent {
                payer: ctx.accounts.payer.key(),
                nft_mint: ctx.accounts.nft_mint.key(),
                block_number,
                nft_name,
                lottery_fee:global_config.lottery_fees
            }
        );

        Ok(())
    }

    pub fn claim(
        ctx: Context<Claim>,
        nonce:u64,
        amount_one:u64,
        amount_two:u64,
        amount_three:u64,
        order_type:u64,
        timestamp: u64,
        signature: [u8; 64],
    ) -> Result<()>{
        let config = &mut ctx.accounts.global_config.load()?;
        if amount_one + amount_two + amount_three > ctx.accounts.pool_token_account.amount {
            return Err(ClaimErrorCode::InsufficientBalance.into());
        }

        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        if (timestamp + 300) < current_timestamp {
            return Err(ClaimErrorCode::InvalidTimestamp.into());
        }

        let addr_nonce = ctx.accounts.address_manager.nonce + 1;


        if nonce != addr_nonce {
            return Err(ClaimErrorCode::InvalidNonce.into())
        }
        ctx.accounts.address_manager.nonce = nonce;
        msg!("msg params {} {} {} {} {}", amount_one, amount_two, amount_three, timestamp, addr_nonce);

        let mut msg = vec![];
        msg.extend(ctx.accounts.payer.key().to_bytes());
        msg.extend(ctx.accounts.output_one_account.key().to_bytes());
        msg.extend(ctx.accounts.output_sec_account.key().to_bytes());
        msg.extend(ctx.accounts.output_third_account.key().to_bytes());
        msg.extend(order_type.to_le_bytes());
        msg.extend(amount_one.to_le_bytes());
        msg.extend(amount_two.to_le_bytes());
        msg.extend(amount_three.to_le_bytes());
        msg.extend(timestamp.to_le_bytes());
        msg.extend(addr_nonce.to_le_bytes());

        let hash = keccak::hash(&msg).to_bytes();
        msg!("hash {}", Pubkey::new_from_array(hash));
        let ix: Instruction = load_instruction_at_checked(2, &ctx.accounts.ix_sysvar)?;
        utils::verify_ed25519_ix(&ix, &config.signer_address.to_bytes(), &hash, &signature)?;
        msg!("signature verified");

        msg!("transfer from pool");
        transfer_token_from_pool(
            &ctx.accounts.pool_token_account,
            ctx.accounts.output_one_account.to_account_info(),
            amount_one,
            &ctx.accounts.token_program,
            &ctx.accounts.global_account.to_account_info(),
            ctx.bumps.global_account,
        )?;

        transfer_token_from_pool(
            &ctx.accounts.pool_token_account,
            ctx.accounts.output_sec_account.to_account_info(),
            amount_two,
            &ctx.accounts.token_program,
            &ctx.accounts.global_account.to_account_info(),
            ctx.bumps.global_account,
        )?;

        transfer_token_from_pool(
            &ctx.accounts.pool_token_account,
            ctx.accounts.output_third_account.to_account_info(),
            amount_three,
            &ctx.accounts.token_program,
            &ctx.accounts.global_account.to_account_info(),
            ctx.bumps.global_account,
        )?;

        emit!(ClaimEvent{
            payer:ctx.accounts.payer.key(),
            nonce:addr_nonce,
            order_type,
            amount_one,
            amount_two,
            amount_three,
            output_one:ctx.accounts.payer.key(),
            output_two:ctx.accounts.output_sec_origin_account.key(),
            output_three:ctx.accounts.output_third_origin_account.key(),
            timestamp
        });

        Ok(())
    }


    pub fn claim_rank(
        ctx: Context<ClaimRank>,
        nonce:u64,
        amount:u64,
        timestamp: u64,
        start_time: u64,
        end_time: u64,
        signature: [u8; 64],
    ) -> Result<()>{
        let config = &mut ctx.accounts.global_config.load()?;
        if amount > ctx.accounts.pool_token_account.amount {
            return Err(ClaimErrorCode::InsufficientBalance.into());
        }

        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        if (timestamp + 300) < current_timestamp {
            return Err(ClaimErrorCode::InvalidTimestamp.into());
        }

        let addr_nonce = ctx.accounts.address_rank_manager.nonce + 1;


        if nonce != addr_nonce {
            return Err(ClaimErrorCode::InvalidNonce.into())
        }
        ctx.accounts.address_rank_manager.nonce = nonce;
        msg!("msg params {} {} {} {} {}", timestamp, addr_nonce, amount, start_time, end_time);

        let mut msg = vec![];
        msg.extend(ctx.accounts.payer.key().to_bytes());
        msg.extend(ctx.accounts.output_one_account.key().to_bytes());
        msg.extend(start_time.to_le_bytes());
        msg.extend(end_time.to_le_bytes());
        msg.extend(amount.to_le_bytes());
        msg.extend(timestamp.to_le_bytes());
        msg.extend(addr_nonce.to_le_bytes());

        let hash = keccak::hash(&msg).to_bytes();
        msg!("hash {}", Pubkey::new_from_array(hash));
        let ix: Instruction = load_instruction_at_checked(2, &ctx.accounts.ix_sysvar)?;
        utils::verify_ed25519_ix(&ix, &config.signer_address.to_bytes(), &hash, &signature)?;
        msg!("signature verified");

        msg!("transfer from pool");
        transfer_token_from_pool(
            &ctx.accounts.pool_token_account,
            ctx.accounts.output_one_account.to_account_info(),
            amount,
            &ctx.accounts.token_program,
            &ctx.accounts.global_account.to_account_info(),
            ctx.bumps.global_account,
        )?;

        emit!(ClaimRankEvent{
            payer:ctx.accounts.payer.key(),
            nonce:addr_nonce,
            amount,
            start_time,
            end_time,
            timestamp
        });

        Ok(())
    }


    pub fn init_address_manager(ctx: Context<InitAddressManager>) -> Result<()>{
        msg!("init address manager");
        Ok(())
    }

    pub fn init_rank_address_manager(ctx: Context<InitRankAddressManager>) -> Result<()>{
        msg!("init rank address manager");
        Ok(())
    }

    pub fn burn_nft(ctx: Context<BurnNft>, nft_name:String) -> Result<()>{
        msg!("Start Burn NFT");
        emit!(BurnEvent{
           payer:ctx.accounts.payer.key(),
           nft_mint:ctx.accounts.nft_mint.key(),
           timestamp: Clock::get()?.unix_timestamp as u64,
           nft_name
        });

        Ok(())
    }

}

#[derive(Accounts)]
pub struct Config {}

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(init, payer = signer, space = 8 + 32 + 32 + 32 + 8,seeds=[b"global_config"], bump)]
    pub global_config: AccountLoader<'info, GlobalConfig>,

    /// CHECK: This is the PDA derived from [b"GLOBAL"] and is safe to use as authority
    #[account(
        init,
        payer = signer,
        space = 8,
        seeds = [b"GLOBAL"],
        bump
    )]
    pub global_account: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(nft_name: String)]
pub struct Lottery<'info> {
    #[account(mut)]
    pub global_config: AccountLoader<'info, GlobalConfig>,

    /// CHECK: This is the PDA derived from [b"GLOBAL"] and is safe to use as authority
    #[account(
        mut,
        seeds = [b"GLOBAL"],
        bump
    )]
    pub global_account: AccountInfo<'info>,

    #[account(mut,address = global_config.load().unwrap().token_mint_address)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = global_account,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = payer,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK This mint use Token_2022_Program
    pub nft_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Claim<'info>{

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Use 'GLOBAL' PDA
    #[account(
        mut,
        seeds = [b"GLOBAL"],
        bump
    )]
    pub global_account: AccountInfo<'info>,

    #[account(
       mut,
        seeds = [
            b"ADDRESS",
            payer.key().as_ref()
        ],
        bump
    )]
    pub address_manager: Box<Account<'info, AccountManager>>,

    #[account(mut)]
    pub global_config: AccountLoader<'info, GlobalConfig>,

    #[account(mut,address = global_config.load().unwrap().token_mint_address)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = global_account,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub output_one_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub output_sec_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub output_third_account: Box<Account<'info, TokenAccount>>,

    ///CHECK: Checked in origin address
    pub output_sec_origin_account: UncheckedAccount<'info>,
    ///CHECK: Checked in origin address
    pub output_third_origin_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: ix sign check
    #[account(address = IX_ID)]
    pub ix_sysvar: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimRank<'info>{

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Use 'GLOBAL' PDA
    #[account(
        mut,
        seeds = [b"GLOBAL"],
        bump
    )]
    pub global_account: AccountInfo<'info>,

    #[account(
       mut,
        seeds = [
            b"ADDRESS_RANK",
            payer.key().as_ref()
        ],
        bump
    )]
    pub address_rank_manager: Box<Account<'info, AccountManager>>,

    #[account(mut)]
    pub global_config: AccountLoader<'info, GlobalConfig>,

    #[account(mut,address = global_config.load().unwrap().token_mint_address)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = global_account,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub output_one_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: ix sign check
    #[account(address = IX_ID)]
    pub ix_sysvar: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitAddressManager<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8+8,
        seeds = [b"ADDRESS", payer.key().as_ref()],
        bump
    )]
    pub address_manager: Box<Account<'info, AccountManager>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitRankAddressManager<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8+8,
        seeds = [b"ADDRESS_RANK", payer.key().as_ref()],
        bump
    )]
    pub address_manager: Box<Account<'info, AccountManager>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nft_name: String)]
pub struct BurnNft<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK
    pub nft_mint: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}


#[account]
pub struct AccountManager {
    pub nonce: u64,
}


#[account(zero_copy(unsafe))]
#[repr(C)]
pub struct GlobalConfig {
    pub signer_address: Pubkey,
    pub token_mint_address: Pubkey,
    pub lottery_fees: u64,
}

#[event]
pub struct LotteryEvent {
    pub payer: Pubkey,
    pub nft_mint: Pubkey,
    pub block_number: u64,
    pub nft_name: String,
    pub lottery_fee:u64
}

#[event]
pub struct ClaimEvent {
    pub payer: Pubkey,
    pub nonce: u64,
    pub order_type: u64,
    pub amount_one: u64,
    pub amount_two: u64,
    pub amount_three: u64,
    pub output_one: Pubkey,
    pub output_two: Pubkey,
    pub output_three: Pubkey,
    pub timestamp: u64,
}

#[event]
pub struct ClaimRankEvent {
    pub payer: Pubkey,
    pub nonce: u64,
    pub amount: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub timestamp: u64,
}

#[event]
pub struct BurnEvent {
    pub payer: Pubkey,
    pub nft_mint:Pubkey,
    pub timestamp: u64,
    pub nft_name: String,
}
