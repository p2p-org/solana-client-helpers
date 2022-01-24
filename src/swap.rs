use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::state::Account as TokenAccount;
pub use spl_token_swap::curve::fees::Fees;
use spl_token_swap::{curve::base::SwapCurve, instruction::Swap};

use crate::{Client, ClientResult, SplToken};

pub struct SwapKeys {
    pub swap: Keypair,
    pub authority_address: Pubkey,
    pub authority_nonce: u8,
    pub token_a: Keypair,
    pub token_b: Keypair,
    pub pool_token_mint: Keypair,
    pub fee_account: Keypair,
    pub pool_token_initial_supply_account: Keypair,
}

pub trait SplSwap {
    #[allow(clippy::too_many_arguments)]
    fn create_swap(
        &self,
        swap_program_id: &Pubkey,
        swap_account: &Keypair,
        swap_authority_address: &Pubkey,
        swap_authority_nonce: u8,
        pool_token_mint_address: &Pubkey,
        token_a_address: &Pubkey,
        token_b_address: &Pubkey,
        owner_address: &Pubkey,
        fees: Fees,
        fee_owner_address: &Pubkey,
    ) -> ClientResult<(Keypair, Keypair)>;

    #[allow(clippy::too_many_arguments)]
    fn create_swap_and_init(
        &self,
        swap_program_id: &Pubkey,
        owner: &Keypair,
        token_a_mint_address: &Pubkey,
        token_a_maker: Option<impl Fn(&Pubkey) -> ClientResult<Keypair>>,
        token_b_mint_address: &Pubkey,
        token_b_maker: Option<impl Fn(&Pubkey) -> ClientResult<Keypair>>,
        pool_token_decimals: u8,
        fees: Fees,
        fee_owner_address: &Pubkey,
    ) -> ClientResult<SwapKeys>;

    #[allow(clippy::too_many_arguments)]
    fn swap(
        &self,
        swap_program_id: &Pubkey,
        swap_account_address: &Pubkey,
        swap_authority_address: &Pubkey,
        user_transfer_authority: &Keypair,
        source_address: &Pubkey,
        pool_source_address: &Pubkey,
        pool_destination_address: &Pubkey,
        destination_address: &Pubkey,
        pool_token_mint_address: &Pubkey,
        fee_account_address: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ClientResult<()>;
}

impl SplSwap for Client {
    fn create_swap(
        &self,
        swap_program_id: &Pubkey,
        swap_account: &Keypair,
        swap_authority_address: &Pubkey,
        swap_authority_nonce: u8,
        pool_token_mint_address: &Pubkey,
        token_a_address: &Pubkey,
        token_b_address: &Pubkey,
        owner_address: &Pubkey,
        fees: Fees,
        fee_account_owner_address: &Pubkey,
    ) -> ClientResult<(Keypair, Keypair)> {
        let fee_account = Keypair::new();
        let pool_token_initial_supply_account = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &fee_account.pubkey(),
                    self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &fee_account.pubkey(),
                    pool_token_mint_address,
                    fee_account_owner_address,
                )?,
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &pool_token_initial_supply_account.pubkey(),
                    self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &pool_token_initial_supply_account.pubkey(),
                    pool_token_mint_address,
                    owner_address,
                )?,
                spl_token_swap::instruction::initialize(
                    swap_program_id,
                    &spl_token::id(),
                    &swap_account.pubkey(),
                    swap_authority_address,
                    token_a_address,
                    token_b_address,
                    pool_token_mint_address,
                    &fee_account.pubkey(),
                    &pool_token_initial_supply_account.pubkey(),
                    swap_authority_nonce,
                    fees,
                    SwapCurve::default(),
                )?,
            ],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(
            &vec![
                self.payer(),
                swap_account,
                &fee_account,
                &pool_token_initial_supply_account,
            ],
            self.latest_blockhash()?,
        );
        self.process_transaction(&transaction)?;

        Ok((fee_account, pool_token_initial_supply_account))
    }

    fn create_swap_and_init<'a>(
        &self,
        swap_program_id: &Pubkey,
        owner: &Keypair,
        token_a_mint_address: &Pubkey,
        token_a_maker: Option<impl Fn(&Pubkey) -> ClientResult<Keypair>>,
        token_b_mint_address: &Pubkey,
        token_b_maker: Option<impl Fn(&Pubkey) -> ClientResult<Keypair>>,
        pool_token_decimals: u8,
        fees: Fees,
        fee_owner_address: &Pubkey,
    ) -> ClientResult<SwapKeys> {
        let swap_account =
            self.create_account(swap_program_id, spl_token_swap::state::SwapVersion::LATEST_LEN, None)?;

        let (swap_authority_address, swap_authority_nonce) =
            Pubkey::find_program_address(&[swap_account.pubkey().as_ref()], swap_program_id);

        let token_a = if let Some(maker) = token_a_maker {
            maker(&swap_authority_address)?
        } else {
            self.create_token_account(&swap_authority_address, token_a_mint_address)?
        };

        let token_b = if let Some(maker) = token_b_maker {
            maker(&swap_authority_address)?
        } else {
            self.create_token_account(&swap_authority_address, token_b_mint_address)?
        };

        let pool_token_mint = self.create_token_mint(&swap_authority_address, pool_token_decimals)?;

        let (fee_account, pool_token_initial_supply_account) = self.create_swap(
            swap_program_id,
            &swap_account,
            &swap_authority_address,
            swap_authority_nonce,
            &pool_token_mint.pubkey(),
            &token_a.pubkey(),
            &token_b.pubkey(),
            &owner.pubkey(),
            fees,
            fee_owner_address,
        )?;

        Ok(SwapKeys {
            swap: swap_account,
            authority_address: swap_authority_address,
            authority_nonce: swap_authority_nonce,
            token_a,
            token_b,
            pool_token_mint,
            fee_account,
            pool_token_initial_supply_account,
        })
    }

    fn swap(
        &self,
        swap_program_id: &Pubkey,
        swap_account_address: &Pubkey,
        swap_authority_address: &Pubkey,
        user_transfer_authority: &Keypair,
        source_address: &Pubkey,
        pool_source_address: &Pubkey,
        pool_destination_address: &Pubkey,
        destination_address: &Pubkey,
        pool_token_mint_address: &Pubkey,
        fee_account_address: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> ClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token_swap::instruction::swap(
                swap_program_id,
                &spl_token::id(),
                swap_account_address,
                swap_authority_address,
                &user_transfer_authority.pubkey(),
                source_address,
                pool_source_address,
                pool_destination_address,
                destination_address,
                pool_token_mint_address,
                fee_account_address,
                None,
                Swap {
                    amount_in,
                    minimum_amount_out,
                },
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&vec![self.payer(), user_transfer_authority], self.latest_blockhash()?);

        self.process_transaction(&transaction)
    }
}
