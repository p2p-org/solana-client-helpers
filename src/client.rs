use std::{net::SocketAddr, ops::Deref, thread, time::Duration};

use solana_client::{client_error::Result as ClientResult, rpc_client::RpcClient};
use solana_program::{hash::Hash, pubkey::Pubkey, system_instruction};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

pub struct Client {
    pub client: RpcClient,
    pub payer: Keypair,
}

impl Client {
    pub fn payer(&self) -> &Keypair {
        &self.payer
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    #[track_caller]
    pub fn recent_blockhash(&self) -> Hash {
        self.client.get_recent_blockhash().unwrap().0
    }

    #[track_caller]
    pub fn rent_minimum_balance(&self, data_len: usize) -> u64 {
        self.client.get_minimum_balance_for_rent_exemption(data_len).unwrap()
    }

    #[track_caller]
    pub fn process_transaction(&mut self, transaction: &Transaction) {
        self.client.send_and_confirm_transaction(transaction).unwrap();
    }

    #[track_caller]
    pub fn create_account(&mut self, owner: &Pubkey, account_data_len: usize, lamports: Option<u64>) -> Keypair {
        let account = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &self.payer_pubkey(),
                &account.pubkey(),
                lamports.unwrap_or_else(|| self.rent_minimum_balance(account_data_len)),
                account_data_len as u64,
                owner,
            )],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &account], self.recent_blockhash());
        self.process_transaction(&transaction);
        account
    }

    #[track_caller]
    pub fn create_associated_token_account(&mut self, funder: &Keypair, recipient: &Pubkey, token_mint: &Pubkey) {
        let mut transaction = Transaction::new_with_payer(
            &[spl_associated_token_account::create_associated_token_account(
                &funder.pubkey(),
                recipient,
                token_mint,
            )],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), funder], self.recent_blockhash());
        self.process_transaction(&transaction);
    }

    #[track_caller]
    pub fn create_associated_token_account_by_payer(&mut self, recipient: &Pubkey, token_mint: &Pubkey) {
        let mut transaction = Transaction::new_with_payer(
            &[spl_associated_token_account::create_associated_token_account(
                &self.payer_pubkey(),
                recipient,
                token_mint,
            )],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer()], self.recent_blockhash());
        self.process_transaction(&transaction);
    }

    pub fn airdrop(&self, to_pubkey: &Pubkey, lamports: u64) -> ClientResult<Signature> {
        let (blockhash, _fee_calculator) = self.client.get_recent_blockhash()?;
        let signature = self
            .client
            .request_airdrop_with_blockhash(to_pubkey, lamports, &blockhash)?;
        self.client
            .confirm_transaction_with_spinner(&signature, &blockhash, self.client.commitment())?;

        Ok(signature)
    }
}

impl Deref for Client {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
