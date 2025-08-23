use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Bytes, Env, String,
};

use crate::error::PaymentContractError;

#[contracttype]
pub struct Payment {
    // UUID from off-chain
    pub payment_id: Bytes,
    // UUID from Invoice table
    pub invoice_id: Bytes,
    // Customer Stellar address
    pub sender: Address,
    // Merchant/aggregator Stellar address
    pub receiver: Address,
    // USDC amount (in smallest unit)
    pub amount: i128,
    // pending, completed, failed
    pub status: PaymentStatus,
    // Stellar transaction hash
    pub stellar_tx_id: Bytes,
}

#[contracttype]
#[derive(PartialEq, Clone, Debug)]
pub enum PaymentStatus {
    Completed,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Usdc,
}
#[contract]
pub struct PaymentModule;

#[contractimpl]
impl PaymentModule {
    // Initialize contract with USDC asset
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
    }

    // Process a USDC payment
    pub fn make_payment(
        env: Env,
        payment_id: Bytes,
        invoice_id: Bytes,
        sender: Address,
        receiver: Address,
        amount: i128,
    ) -> Result<Bytes, PaymentContractError> {
        sender.require_auth();

        if amount <= 0 {
            return Err(PaymentContractError::AmountMustBePositive);
        }

        if env.storage().persistent().has(&payment_id) {
            return Err(PaymentContractError::PaymentIDAlreadyExists);
        }

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();

        let token_client = token::Client::new(&env, &usdc_asset);

        token_client.transfer(&sender, &receiver, &amount);

        let stellar_tx_id = Bytes::from_array(&env, &[0u8; 32]);

        let payment = Payment {
            payment_id: payment_id.clone(),
            invoice_id: invoice_id.clone(),
            sender,
            receiver: receiver.clone(),
            amount,
            status: PaymentStatus::Completed,
            stellar_tx_id,
        };

        env.storage().persistent().set(&payment_id, &payment);

        env.events().publish(
            (("Payment Processed"), payment_id.clone(), invoice_id),
            (amount, receiver),
        );

        Ok(payment_id)
    }

    pub fn get_payment_status(
        env: Env,
        payment_id: Bytes,
    ) -> Option<(Bytes, Address, Address, i128, String)> {
        env.storage().persistent().get(&payment_id)
    }
}
