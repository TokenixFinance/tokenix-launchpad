use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("2edCULGmDFm3xBZY3n8g8oY33n2NKnBByFFWvwA18Pyz");

#[error_code]
pub enum TokenixError {
    #[msg("Token name is too long")]
    NameTooLong,
    #[msg("Token symbol is too long")]
    SymbolTooLong,
    #[msg("Invalid initial supply")]
    InvalidInitialSupply,
}

#[program]
pub mod tokenix {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        _uri: String,  
        initial_supply: u64,
    ) -> Result<()> {
        require!(name.len() <= 30, TokenixError::NameTooLong);
        require!(symbol.len() <= 10, TokenixError::SymbolTooLong);
        require!(initial_supply == 100_000_000 * 10u64.pow(9), TokenixError::InvalidInitialSupply);

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, initial_supply)?;

        Ok(())
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        initial_price: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.mint = ctx.accounts.mint.key();
        pool.token_account = ctx.accounts.pool_token_account.key();
        pool.current_price = initial_price;
        pool.total_supply = 100_000_000 * 10u64.pow(9);

        let cpi_accounts = token::Transfer {
            from: ctx.accounts.authority_token_account.to_account_info(),
            to: ctx.accounts.pool_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, pool.total_supply)?;

        Ok(())
    }

    pub fn buy_token(
        ctx: Context<BuyToken>,
        amount: u64,
    ) -> Result<()> {
        let price = calculate_price(ctx.accounts.pool.total_supply, amount);

        let buyer_balance = ctx.accounts.buyer.lamports();
        msg!("Buyer balance: {}", buyer_balance);
        msg!("Required price: {}", price);
        
        if buyer_balance < price {
            msg!("Insufficient funds. Buyer needs {} more lamports", price.saturating_sub(buyer_balance));
            return Err(ProgramError::InsufficientFunds.into());
        }

        let cpi_accounts = system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        system_program::transfer(cpi_ctx, price)?;

        let mint_key = ctx.accounts.mint.key();
        let seeds = &[
            b"pool".as_ref(),
            mint_key.as_ref(),
            &[ctx.bumps.pool],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = token::Transfer {
            from: ctx.accounts.pool_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        let pool = &mut ctx.accounts.pool;
        pool.total_supply -= amount;
        pool.current_price = calculate_price(pool.total_supply, 1);

        Ok(())
    }

    pub fn sell_token(
        ctx: Context<SellToken>,
        amount: u64,
    ) -> Result<()> {
        let price = calculate_sell_price(&ctx, amount)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.seller_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            amount,
        )?;

        let pool_seed = b"pool".as_ref();
        let mint_key = ctx.accounts.mint.key();
        let bump = ctx.bumps.pool;
        let seeds = &[pool_seed, mint_key.as_ref(), &[bump]];
        let signer_seeds = &[&seeds[..]];

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.pool.to_account_info(),
                    to: ctx.accounts.seller.to_account_info(),
                },
                signer_seeds,
            ),
            price,
        )?;

        let pool = &mut ctx.accounts.pool;
        pool.total_supply += amount;
        pool.current_price = calculate_price(pool.total_supply, 1);

        Ok(())
    }
}

const BASE_PRICE: u64 = 10_000; // 0.00001 SOL
const LAMPORTS_PER_SOL: u128 = 1_000_000_000;

fn calculate_price(supply: u64, amount: u64) -> u64 {
    let supply_u128 = supply as u128;
    let base_price_u128 = BASE_PRICE as u128;
    let price_per_token = base_price_u128.saturating_add(supply_u128 * base_price_u128 / LAMPORTS_PER_SOL);
    let total_price = price_per_token.saturating_mul(amount as u128);
    
    msg!("Supply: {}", supply);
    msg!("Amount: {}", amount);
    msg!("Price per token: {}", price_per_token);
    msg!("Total price: {}", total_price);
    
    total_price.try_into().unwrap_or(u64::MAX)
}

fn calculate_buy_price(pool: &Account<Pool>, amount: u64) -> Result<u64> {
    let current_supply = pool.total_supply;
    let price = calculate_price(current_supply, amount);
    msg!("Current supply: {}", current_supply);
    msg!("Amount to buy: {}", amount);
    msg!("Calculated price: {}", price);
    msg!("Current pool price: {}", pool.current_price);
    Ok(price)
}

fn calculate_sell_price(ctx: &Context<SellToken>, amount: u64) -> Result<u64> {
    let pool = &ctx.accounts.pool;
    let current_supply = pool.total_supply;
    let price = calculate_price(current_supply - amount, amount);
    Ok(price)
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = authority,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 8,
        seeds = [b"pool".as_ref(), mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = authority_token_account.mint == mint.key(),
        constraint = authority_token_account.owner == authority.key(),
    )]
    pub authority_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = pool,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct SellToken<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        constraint = pool_token_account.mint == mint.key(),
        constraint = pool_token_account.owner == pool.key(),
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = seller_token_account.mint == mint.key(),
        constraint = seller_token_account.owner == seller.key(),
    )]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
pub struct Pool {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub current_price: u64,
    pub total_supply: u64,
}