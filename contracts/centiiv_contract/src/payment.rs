use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

#[contracttype]
pub struct Payment {
    invoice_id: Symbol,
    amount: i128,
    sender: Address,
    receiver: Address,
    status: Symbol,
    beneficiary_ids: Vec<Symbol>,
}

pub struct PaymentModule;

impl PaymentModule {}
