//! Usage example:
//!
//! ```
//! use solana_client::rpc_client::RpcClient;
//! use solana_client_helpers::{Client, ClientResult, SplToken};
//! use solana_sdk::{
//!     commitment_config::CommitmentConfig,
//!     signature::{Keypair, Signer},
//! };
//!
//! fn main() -> ClientResult<()> {
//!     let payer = Keypair::new();
//!     let sender = Keypair::new();
//!     let recipient = Keypair::new();
//!
//!     let client = RpcClient::new_with_commitment("http://localhost:8899".into(), CommitmentConfig::confirmed());
//!     let client = Client { client, payer };
//!
//!     client.airdrop(&client.payer_pubkey(), 10_000_000_000)?;
//!     assert_eq!(client.get_balance(&client.payer_pubkey())?, 10_000_000_000);
//!
//!     let token_mint = client.create_token_mint(&sender.pubkey(), 2)?;
//!     let sender_token_account = client.create_associated_token_account_by_payer(&sender.pubkey(), &token_mint.pubkey())?;
//!     let recipient_token_account = client.create_associated_token_account_by_payer(&recipient.pubkey(), &token_mint.pubkey())?;
//!
//!     client.mint_to(&sender, &token_mint.pubkey(), &sender_token_account, 1000, 2)?;
//!
//!     let sender_balance = client.get_token_account_balance(&sender_token_account)?;
//!     assert_eq!(sender_balance.ui_amount, Some(10.00));
//!
//!     client.transfer_to(&sender, &token_mint.pubkey(), &sender_token_account, &recipient_token_account, 500, 2)?;
//!
//!     let sender_balance = client.get_token_account_balance(&sender_token_account)?;
//!     assert_eq!(sender_balance.ui_amount, Some(5.00));
//!     
//!     let recipient_balance = client.get_token_account_balance(&recipient_token_account)?;
//!     assert_eq!(recipient_balance.ui_amount, Some(5.00));
//!
//!     Ok(())
//! }
//! ```

pub use crate::{client::*, print::*, token::*};

pub mod client;
pub mod print;
pub mod token;
