use anchor_lang::{prelude::*, system_program};
use mpl_token_metadata::state::*;
use anchor_spl::token::{self};
use anchor_lang::prelude::Clock;
use std::collections::HashMap;

declare_id!("8jF63DJNhnsEJQwwwWx9ZnWAXg1He1squ2Wre7mbDL6V");

#[program]
pub mod nft_marketplace {

    use anchor_spl::token::TokenAccount;

    use super::*;

    pub fn initialize_marketplace(
        ctx: Context<InitializeMarketplace>,
        extra_seed: u64,
        fee_percentage: u64,
    ) -> Result<()> {
        let state_account = &mut ctx.accounts.state_account;
        
        state_account.initializer = ctx.accounts.initializer.key();
        state_account.total_listed_count_sol = 0;
        state_account.total_listed_count_spl = 0;
        
        state_account.total_volume_all_time_sol = 0;

        state_account.total_listed_count_spl = 0;
        state_account.all_time_sale_count_sol = 0;
        state_account.marketplace_fee_percentage = fee_percentage;

        state_account.extra_seed = extra_seed;
        state_account.bump = ctx.bumps.state_account;

        Ok(())
    }

    pub fn list_nft(ctx: Context<ListNft>, price: u64) -> Result<()> {             
        let metadata: Metadata = Metadata::from_account_info(&&ctx.accounts.nft_metadata.to_account_info()).expect("Invalid metadata");
        let creators_array = metadata.data.creators.unwrap();
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_token_account.to_account_info(),
                to: ctx.accounts.nft_holder_address.to_account_info(),
                authority: ctx.accounts.initializer.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, 1)?;

        let listing_account = &mut ctx.accounts.listing_account;
        listing_account.global_state_address = ctx.accounts.global_state_account.key();
        listing_account.initializer = ctx.accounts.initializer.key();
        listing_account.nft_mint_address = ctx.accounts.nft_mint.key();
        listing_account.nft_holder_address = ctx.accounts.nft_holder_address.key();
        listing_account.price = price;
        let clock = Clock::get()?;
        listing_account.creation_time = clock.unix_timestamp;
        listing_account.updated_at = listing_account.creation_time;
        listing_account.is_spl_listing = false;

        listing_account.bump = ctx.bumps.listing_account;
        
        let marketplace = &mut ctx.accounts.global_state_account;
        marketplace.total_listed_count_sol += 1;

        Ok(())
    }

    pub fn list_nft_in_spl(ctx: Context<ListNftInSpl>, price: u64) -> Result<()> {
        let metadata: Metadata = Metadata::from_account_info(&&ctx.accounts.nft_metadata.to_account_info()).expect("Invalid metadata");
        let creators_array = metadata.data.creators.unwrap();
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_token_account.to_account_info(),
                to: ctx.accounts.nft_holder_address.to_account_info(),
                authority: ctx.accounts.initializer.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, 1)?;

        let listing_account = &mut ctx.accounts.listing_account;
        listing_account.global_state_address = ctx.accounts.global_state_account.key();
        listing_account.initializer = ctx.accounts.initializer.key();
        listing_account.nft_mint_address = ctx.accounts.nft_mint.key();
        listing_account.nft_holder_address = ctx.accounts.nft_holder_address.key();
        listing_account.price = price;

        let clock = Clock::get()?;
        listing_account.creation_time = clock.unix_timestamp;
        listing_account.updated_at = listing_account.creation_time;
        listing_account.is_spl_listing = true;
        listing_account.trade_spl_token_mint_address = ctx.accounts.trade_nft_mint.key();
        listing_account.trade_spl_token_seller_account_address = ctx.accounts.trade_nft_token_account.key();

        listing_account.bump = ctx.bumps.listing_account;

        let marketplace = &mut ctx.accounts.global_state_account;
        marketplace.total_listed_count_spl += 1;

        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
        let listing_account = &mut ctx.accounts.listing_account;
        listing_account.price = new_price;
        
        let clock = Clock::get()?;
        listing_account.updated_at = clock.unix_timestamp;

        Ok(())
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
         
        let init_k = ctx.accounts.initializer.key().clone();
        let global_k = ctx.accounts.global_state_account.key().clone();
        let mint_k = ctx.accounts.nft_mint.key().clone();
        
        let bmp = &[ctx.bumps.nft_holder_address];
        let inner = vec![
                b"nft_holder".as_ref(),
                init_k.as_ref(),
                global_k.as_ref(),
                mint_k.as_ref(),
                bmp
            ];
        
        let slc = inner.as_slice();
        let outer = vec![slc];
        let signer_seeds = outer.as_slice();

         let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_holder_address.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.nft_holder_address.to_account_info(),
            },
            signer_seeds
        );

        token::transfer(cpi_ctx, 1)?;

        let cpi_accounts = token::CloseAccount{
            account: ctx.accounts.nft_holder_address.to_account_info(),
            destination: ctx.accounts.initializer.to_account_info(),
            authority: ctx.accounts.nft_holder_address.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();

            
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
       
        token::close_account(cpi_ctx)?;
        
        let listing_account = &mut ctx.accounts.listing_account;
        let marketplace = &mut ctx.accounts.global_state_account;

        if listing_account.is_spl_listing {
            marketplace.total_listed_count_spl -= 1;
        }else {
            marketplace.total_listed_count_sol -= 1;
        }

        Ok(())
    }

    pub fn buy_nft<'info>(ctx: Context<'_, '_, '_, 'info, BuyNft<'info>>) -> Result<()> {
        let creator_accounts = ctx.remaining_accounts;
        let metadata: Metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata_account.to_account_info())?;
        let listing_account = &mut ctx.accounts.listing_account;
        
        let marketplace_account = &mut ctx.accounts.global_state_account;

        let creators_array = metadata.data.creators.unwrap();
        let mut acc_map: HashMap<Pubkey, Creator> = HashMap::new();
        for creator in creators_array.iter(){
            acc_map.insert(creator.address.clone() , creator.clone());
        }


        let total_share_points = metadata.data.seller_fee_basis_points;
        let creators_total_share = listing_account.price * (total_share_points as u64) / 10000;

        let marketplace_fee = listing_account.price * marketplace_account.marketplace_fee_percentage / 100;

        let seller_amount = listing_account.price - creators_total_share - marketplace_fee;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.listing_account_initializer.to_account_info(),
                }
            ),
            seller_amount
        )?;


        for creator_account in creator_accounts.iter(){
            if acc_map.contains_key(creator_account.key) {
                let curr_share = creators_total_share*(acc_map[creator_account.key].share as u64)/100;
                let ix = system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: creator_account.to_account_info(),
                };
                let transfer_ctx = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    ix,
                );

                system_program::transfer(
                    transfer_ctx, 
                    curr_share
                )?;

                acc_map.remove(creator_account.key);
            }
        }

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.global_state_initializer.to_account_info(),
                }
            ),
            marketplace_fee
        )?;


        let init_k = ctx.accounts.listing_account_initializer.key().clone();
        let global_k = ctx.accounts.global_state_account.key().clone();
        let mint_k = ctx.accounts.nft_mint.key().clone();

        let bmp = &[ctx.bumps.nft_holder_address];
        let inner = vec![
            b"nft_holder".as_ref(),
            init_k.as_ref(),
            global_k.as_ref(),
            mint_k.as_ref(),
            bmp
        ];

        let outer = vec![inner.as_slice()];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_holder_address.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.nft_holder_address.to_account_info(),
            },
            outer.as_slice()
        );
        token::transfer(cpi_ctx, 1)?;


        let ca = token::CloseAccount{
            account: ctx.accounts.nft_holder_address.to_account_info(),
            destination: ctx.accounts.listing_account_initializer.to_account_info(),
            authority: ctx.accounts.nft_holder_address.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            ca,
            outer.as_slice()
        );
        token::close_account(cpi_ctx)?;

        let marketplace = &mut ctx.accounts.global_state_account;
        marketplace.total_listed_count_sol -= 1;
        marketplace.all_time_sale_count_sol += 1;

        marketplace.total_volume_all_time_sol += listing_account.price as u128;

        Ok(())
    }

    pub fn buy_nft_with_spl<'info> (ctx: Context<'_, '_, 'info, 'info, BuyNftInSpl<'info>>) -> Result<()> {
        let creator_accounts = ctx.remaining_accounts;
        let metadata: Metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata_account.to_account_info())?;
        let listing_account = &mut ctx.accounts.listing_account;
        
        let marketplace_account = &mut ctx.accounts.global_state_account;

        let creators_array = metadata.data.creators.unwrap();
        let mut acc_map: HashMap<Pubkey, Creator> = HashMap::new();
        for creator in creators_array.iter(){
            acc_map.insert(creator.address.clone() , creator.clone());
        }

        let total_share_points = metadata.data.seller_fee_basis_points;
        let creators_total_share = listing_account.price * (total_share_points as u64) / 10000;

        let marketplace_fee = listing_account.price * marketplace_account.marketplace_fee_percentage / 100;

        let seller_amount = listing_account.price - creators_total_share - marketplace_fee;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.buyer_trade_token_account.to_account_info(),
                    to: ctx.accounts.seller_trade_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                }
            ), 
        seller_amount)?;

        for creator_account in creator_accounts.iter(){

            let creator_copy = creator_account;
            
            let x:Account<'info, TokenAccount> = Account::try_from(&creator_copy)?;
           
            if acc_map.contains_key(&x.owner) { 
                let curr_share = creators_total_share*(acc_map[&creator_copy.owner].share as u64)/100;
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.clone().to_account_info(),
                        token::Transfer {
                            from: ctx.accounts.buyer_trade_token_account.clone().to_account_info(),
                            to: creator_account.clone().to_account_info(),
                            authority: ctx.accounts.buyer.clone().to_account_info(),
                        }
                    ), 
                curr_share.clone())?;

                acc_map.remove(&x.owner);
            }
        }

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.buyer_trade_token_account.clone().to_account_info(),
                    to: ctx.accounts.global_state_initializer.clone().to_account_info(),
                    authority: ctx.accounts.buyer.clone().to_account_info(),
                }
            ),
            marketplace_fee
        )?;

        let init_k = ctx.accounts.listing_account_initializer.key().clone();
        let global_k = ctx.accounts.global_state_account.key().clone();
        let mint_k = ctx.accounts.nft_mint.key().clone();

        let bmp = &[ctx.bumps.nft_holder_address];
        let inner = vec![
            b"nft_holder".as_ref(),
            init_k.as_ref(),
            global_k.as_ref(),
            mint_k.as_ref(),
            bmp
        ];

        let outer = vec![inner.as_slice()];


        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.nft_holder_address.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.nft_holder_address.to_account_info(),
                },
                outer.as_slice()
            ), 
        1)?;
        

        token::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::CloseAccount{
                    account: ctx.accounts.nft_holder_address.to_account_info(),
                    destination: ctx.accounts.listing_account_initializer.to_account_info(),
                    authority: ctx.accounts.nft_holder_address.to_account_info(),
                },
                outer.as_slice()
            )
        )?;

        let marketplace = &mut ctx.accounts.global_state_account;
        marketplace.total_listed_count_spl -= 1;
        marketplace.all_time_sale_count_spl += 1;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(extra_seed: u64)]
pub struct InitializeMarketplace<'info>  {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(
        init,
        payer = initializer,
        space = GLOBAL_STATE_SIZE + 8,
        seeds = [
            initializer.key().as_ref(),
            b"state_account",
            &extra_seed.to_be_bytes(),
        ],
        bump
    )]
    pub state_account: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>

}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,

    #[account(mut,
        constraint = (nft_token_account.mint == nft_mint.key()  &&
                     nft_token_account.owner == initializer.key())
    )]
    pub nft_token_account: Account<'info, token::TokenAccount>,

    #[account(
        init,
        payer = initializer,
        space = LISTING_SIZE + 8,
        seeds = [
            b"listing_account",
            initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub listing_account: Account<'info, Listing>,

     #[account(
        init,
        payer = initializer,
        token::mint  = nft_mint,
        token::authority = nft_holder_address,
        seeds = [
            b"nft_holder",
            initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_holder_address: Account<'info, token::TokenAccount>,

    #[account(mut)]
    /// CHECK: checking in instruction
    pub nft_metadata: AccountInfo<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ListNftInSpl<'info>{
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,

    #[account(mut,
        constraint = (nft_token_account.mint == nft_mint.key()  &&
                     nft_token_account.owner == initializer.key())
    )]
    pub nft_token_account: Account<'info, token::TokenAccount>,

    #[account(
        init,
        payer = initializer,
        space = 8 + LISTING_SIZE,
        seeds = [
            b"listing_account",
            initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub listing_account: Account<'info, Listing>,

     #[account(
        init,
        payer = initializer,
        token::mint  = nft_mint,
        token::authority = nft_holder_address,
        seeds = [
            b"nft_holder",
            initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_holder_address: Account<'info, token::TokenAccount>,


    #[account(mut)]
    pub trade_nft_mint: Account<'info, token::Mint>,

    #[account(mut,
        constraint = (trade_nft_token_account.mint == trade_nft_mint.key()  &&
                    trade_nft_token_account.owner == initializer.key())
    )]
    pub trade_nft_token_account: Account<'info, token::TokenAccount>,


    #[account(mut)]
    /// CHECK: checking in instruction
    pub nft_metadata: AccountInfo<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,

    #[account(mut, 
        seeds = [
            b"listing_account", 
            initializer.key().as_ref(),
            global_state_account.key().as_ref(), 
            nft_mint.key().as_ref()
            ],
        bump = listing_account.bump
    )]
    pub listing_account: Account<'info, Listing>,

}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut, 
        constraint = listing_account.initializer == initializer.key(),
        close = initializer
    )]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,

    #[account(mut,
        constraint = (nft_token_account.mint == nft_mint.key()  &&
                     nft_token_account.owner == initializer.key())
    )]
    pub nft_token_account: Account<'info, token::TokenAccount>,

    #[account(
        mut,
        constraint = listing_account.nft_holder_address == nft_holder_address.key(),
        seeds = [
            b"nft_holder",
            initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_holder_address: Account<'info, token::TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut, constraint = listing_account_initializer.key() == listing_account.initializer)]
    /// CHECK: checking in instruction
    pub listing_account_initializer: AccountInfo<'info>, 

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut, constraint = global_state_initializer.key() == global_state_account.initializer)]
     /// CHECK: checking in instruction
    pub global_state_initializer: AccountInfo<'info>,

    #[account(mut, 
        constraint = listing_account.global_state_address == global_state_account.key(),
        close = listing_account_initializer
    )]
    pub listing_account: Account<'info, Listing>,

    #[account(mut, 
        constraint = listing_account.nft_holder_address == nft_holder_address.key() && 
                    nft_holder_address.mint == nft_mint.key(),
        seeds = [
            b"nft_holder",
            listing_account_initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_holder_address: Account<'info, token::TokenAccount>,

    
    #[account(mut,
        constraint = (buyer_token_account.mint == nft_mint.key()  &&
        buyer_token_account.owner == buyer.key())
    )]
    pub buyer_token_account: Account<'info, token::TokenAccount>,


    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,

    #[account(mut)]
    /// CHECK: checking in instruction
    pub nft_metadata_account: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>

}

#[derive(Accounts)]
pub struct BuyNftInSpl<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut,
        constraint = (buyer_trade_token_account.mint == trade_spl_token_mint.key()  &&
        buyer_trade_token_account.owner == buyer.key())
    )]
    pub buyer_trade_token_account: Account<'info, token::TokenAccount>,

    #[account(mut, constraint = listing_account_initializer.key() == listing_account.initializer)]
     /// CHECK: checking in instruction
    pub listing_account_initializer: AccountInfo<'info>, 

    #[account(mut,
        constraint = (seller_trade_token_account.mint == trade_spl_token_mint.key()  &&
        seller_trade_token_account.owner == listing_account_initializer.key() &&  
        seller_trade_token_account.key() == listing_account.trade_spl_token_seller_account_address)
    )]
    pub seller_trade_token_account: Account<'info, token::TokenAccount>,

    #[account(mut)]
    pub global_state_account: Account<'info, GlobalState>,

    #[account(mut, constraint = global_state_initializer.owner == global_state_account.initializer && 
                                global_state_initializer.mint ==  trade_spl_token_mint.key())]
    pub global_state_initializer: Account<'info, token::TokenAccount>,


    #[account(mut, 
        constraint = listing_account.global_state_address == global_state_account.key(),
        close = listing_account_initializer
    )]
    pub listing_account: Account<'info, Listing>,

    #[account(mut, 
        constraint = listing_account.nft_holder_address == nft_holder_address.key() && 
                    nft_holder_address.mint == nft_mint.key(),
        seeds = [
            b"nft_holder",
            listing_account_initializer.key().as_ref(),
            global_state_account.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub nft_holder_address: Account<'info, token::TokenAccount>,

    
    #[account(mut,
        constraint = (buyer_token_account.mint == nft_mint.key()  &&
        buyer_token_account.owner == buyer.key())
    )]
    pub buyer_token_account: Account<'info, token::TokenAccount>,


    #[account(mut)]
    pub nft_mint: Account<'info, token::Mint>,


    #[account(mut, constraint = trade_spl_token_mint.key() == listing_account.trade_spl_token_mint_address)]
    pub trade_spl_token_mint: Account<'info, token::Mint>,

    #[account(mut)]
    /// CHECK: checking in instruction
    pub nft_metadata_account: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>
}


#[account]
#[derive(Default)]
pub struct GlobalState {
	pub initializer: Pubkey,
	pub total_listed_count_sol: u64,
	pub total_listed_count_spl: u64,

	pub total_volume_all_time_sol: u128,

	pub all_time_sale_count_spl: u64,
	pub all_time_sale_count_sol: u64,
	pub marketplace_fee_percentage: u64,

    pub extra_seed: u64,
    pub bump: u8,
}

const GLOBAL_STATE_SIZE :usize = 32 + 8 + 8 + 16 + 8 + 8 + 8 + 8 + 1; 

#[account]
#[derive(Default)]
pub struct Listing {
    // Marketplace instance global state address
    pub global_state_address: Pubkey,

    // User who listed this nft
    pub initializer: Pubkey,
    // NFT mint address
    pub nft_mint_address: Pubkey,
    // Program PDA account address, who holds NFT now
    pub nft_holder_address: Pubkey,
    // Price of this NFT.
    pub price: u64,

    // listing creation time
    pub creation_time: i64,
    pub updated_at: i64,

    // if trade payment is in spl token currency
    pub is_spl_listing: bool,
    // trade spl token address
    pub trade_spl_token_mint_address: Pubkey,
    pub trade_spl_token_seller_account_address: Pubkey,

    pub bump: u8,
}

const LISTING_SIZE :usize = 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 32 + 32 + 1;