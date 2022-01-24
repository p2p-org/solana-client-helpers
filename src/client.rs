use std::ops::{Deref, DerefMut};

pub use solana_client::{client_error, rpc_client::RpcClient};
use solana_sdk::{
    hash::Hash,
    program_error::ProgramError,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Client(#[from] client_error::ClientError),

    #[error(transparent)]
    Program(#[from] ProgramError),
}

pub type ClientResult<T> = Result<T, ClientError>;

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

    pub fn latest_blockhash(&self) -> ClientResult<Hash> {
        Ok(self.client.get_latest_blockhash()?)
    }

    pub fn process_transaction(&self, transaction: &Transaction) -> ClientResult<()> {
        self.send_and_confirm_transaction(transaction)?;
        Ok(())
    }

    pub fn create_account(
        &self,
        owner: &Pubkey,
        account_data_len: usize,
        lamports: Option<u64>,
    ) -> ClientResult<Keypair> {
        let account = Keypair::new();
        let lamports = if let Some(lamports) = lamports {
            lamports
        } else {
            self.get_minimum_balance_for_rent_exemption(account_data_len)?
        };

        let mut transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &self.payer_pubkey(),
                &account.pubkey(),
                lamports,
                account_data_len as u64,
                owner,
            )],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &account], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(account)
    }

    pub fn airdrop(&self, to_pubkey: &Pubkey, lamports: u64) -> ClientResult<Signature> {
        let blockhash = self.client.get_latest_blockhash()?;
        let signature = self.request_airdrop_with_blockhash(to_pubkey, lamports, &blockhash)?;
        self.confirm_transaction_with_spinner(&signature, &blockhash, self.commitment())?;

        Ok(signature)
    }
}

impl Deref for Client {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
