use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env};

use crate::{
    error::PaymentContractError,
    storage::{DataKey, Payment, PaymentStatus},
};

#[contract]
pub struct PaymentModule;

#[contractimpl]
impl PaymentModule {
    pub fn __constructor(env: Env, admin: Address, usdc_asset: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
    }

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
            (("Processed"), payment.payment_id, payment.invoice_id),
            (payment.amount, payment.receiver),
        );

        Ok(payment_id)
    }

    pub fn get_balance(env: Env, address: Address) -> i128 {
        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.balance(&address)
    }

    pub fn get_payment_status(env: Env, payment_id: Bytes) -> Option<Payment> {
        env.storage().persistent().get(&payment_id)
    }
}
