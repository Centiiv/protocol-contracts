use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Usdc,
    Wallet,
}

#[contracttype]
#[derive(PartialEq, Clone, Debug)]
pub enum EscrowStatus {
    Locked,
    Released,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowData {
    pub user_id: Address,
    pub lp_node_id: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub timeout: u64,
}
