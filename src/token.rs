use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use super::client::{Client, ClientResult};

pub trait SplToken {
    fn create_token_mint(&self, owner: &Pubkey, decimals: u8) -> ClientResult<Keypair>;
    fn create_token_account(&self, owner: &Pubkey, token_mint: &Pubkey) -> ClientResult<Keypair>;
    fn create_token_account_with_lamports(
        &self,
        owner: &Pubkey,
        token_mint: &Pubkey,
        lamports: u64,
    ) -> ClientResult<Keypair>;
    fn mint_to(
        &self,
        owner: &Keypair,
        token_mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()>;
    fn transfer_to(
        &self,
        owner: &Keypair,
        token_mint: &Pubkey,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> ClientResult<()>;
    fn get_associated_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey;
    fn create_associated_token_account(
        &self,
        funder: &Keypair,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> ClientResult<Pubkey>;
    fn create_associated_token_account_by_payer(&self, recipient: &Pubkey, token_mint: &Pubkey)
        -> ClientResult<Pubkey>;
    fn close_token_account(&self, owner: &Keypair, account: &Pubkey, destination: &Pubkey) -> ClientResult<()>;
}

impl SplToken for Client {
    fn create_token_mint(&self, owner: &Pubkey, decimals: u8) -> ClientResult<Keypair> {
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

    fn create_token_account(&self, owner: &Pubkey, token_mint: &Pubkey) -> ClientResult<Keypair> {
        self.create_token_account_with_lamports(
            owner,
            token_mint,
            self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
        )
    }

    fn create_token_account_with_lamports(
        &self,
        owner: &Pubkey,
        token_mint: &Pubkey,
        lamports: u64,
    ) -> ClientResult<Keypair> {
        let token_account = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &token_account.pubkey(),
                    lamports,
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
        &self,
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
        &self,
        authority: &Keypair,
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
                &authority.pubkey(),
                &[],
                amount,
                decimals,
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &authority], self.recent_blockhash()?);
        self.process_transaction(&transaction)
    }

    fn get_associated_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(wallet_address, token_mint)
    }

    fn create_associated_token_account(
        &self,
        funder: &Keypair,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> ClientResult<Pubkey> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_associated_token_account::create_associated_token_account(
                &funder.pubkey(),
                recipient,
                token_mint,
            )],
            Some(&self.payer_pubkey()),
        );
        if funder.pubkey() == self.payer_pubkey() {
            transaction.sign(&[self.payer()], self.recent_blockhash()?);
        } else {
            transaction.sign(&[self.payer(), funder], self.recent_blockhash()?);
        };
        self.process_transaction(&transaction)?;

        Ok(Self::get_associated_token_address(recipient, token_mint))
    }

    fn create_associated_token_account_by_payer(
        &self,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> ClientResult<Pubkey> {
        self.create_associated_token_account(self.payer(), recipient, token_mint)
    }

    fn close_token_account(&self, owner: &Keypair, account: &Pubkey, destination: &Pubkey) -> ClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::close_account(
                &spl_token::id(),
                account,
                destination,
                &owner.pubkey(),
                &[],
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &owner], self.recent_blockhash()?);
        self.process_transaction(&transaction)
    }
}
