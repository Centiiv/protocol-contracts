use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

#[contracttype]
pub struct Escrow {
    request_id: Symbol,
    amount: i128,
    sender: Address,
    lp_node: Address,
    status: Symbol,
    created_at: u64,
}

pub struct EscrowModule;

impl EscrowModule {}
