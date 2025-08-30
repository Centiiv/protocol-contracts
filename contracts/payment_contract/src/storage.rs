use soroban_sdk::{contracttype, Address, Bytes};

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
#[derive(Clone, Debug, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Usdc,
}
