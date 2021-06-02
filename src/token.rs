use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use super::client::{Client, ClientResult};

pub trait SplToken {
    fn create_token_mint(&mut self, owner: &Pubkey, decimals: u8) -> ClientResult<Keypair>;
    fn create_token_account(&mut self, owner: &Pubkey, token_mint: &Pubkey) -> ClientResult<Keypair>;
    fn mint_to(
        &mut self,
        owner: &Keypair,
        token_mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()>;
    fn transfer_to(
        &mut self,
        owner: &Keypair,
        token_mint: &Pubkey,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()>;
}

impl SplToken for Client {
    fn create_token_mint(&mut self, owner: &Pubkey, decimals: u8) -> ClientResult<Keypair> {
        let token_mint = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &token_mint.pubkey(),
                    self.get_minimum_balance_for_rent_exemption(Mint::LEN)?,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_mint(&spl_token::id(), &token_mint.pubkey(), owner, None, decimals)?,
            ],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &token_mint], self.recent_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(token_mint)
    }

    fn create_token_account(&mut self, owner: &Pubkey, token_mint: &Pubkey) -> ClientResult<Keypair> {
        let token_account = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &token_account.pubkey(),
                    self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &token_account.pubkey(),
                    token_mint,
                    owner,
                )?,
            ],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &token_account], self.recent_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(token_account)
    }

    fn mint_to(
        &mut self,
        owner: &Keypair,
        token_mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::mint_to_checked(
                &spl_token::id(),
                token_mint,
                account,
                &owner.pubkey(),
                &[],
                amount,
                decimals,
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &owner], self.recent_blockhash()?);
        self.process_transaction(&transaction)
    }

    fn transfer_to(
        &mut self,
        owner: &Keypair,
        token_mint: &Pubkey,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::transfer_checked(
                &spl_token::id(),
                source,
                token_mint,
                destination,
                &owner.pubkey(),
                &[],
                amount,
                decimals,
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &owner], self.recent_blockhash()?);
        self.process_transaction(&transaction)
    }
}
