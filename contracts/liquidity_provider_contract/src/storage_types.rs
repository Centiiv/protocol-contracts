use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, IntoVal, Map, TryFromVal, Val};
use soroban_sdk::{contracttype, String};
#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    SettingsContract,
    NodeIDs,
    Nonces,
    Order(Bytes),
    Usdc,
    PendingSettlement(Bytes),
    PendingRefund(Bytes),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LpNode {
    pub capacity: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingSettlement {
    pub order_id: Bytes,
    pub protocol_fee: i128,
    pub transfer_amount: i128,
    pub liquidity_provider: Address,
    pub settle_percent: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRefund {
    pub order_id: Bytes,
    pub fee: i128,
    pub refund_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct OrderParams {
    pub order_id: Bytes,
    pub token: Address,
    pub sender: Address,
    pub amount: i128,
    pub rate: i64,
    pub sender_fee_recipient: Address,
    pub temporary_wallet_address: Address,
    pub sender_fee: i128,
    pub refund_address: Address,
    pub message_hash: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Order {
    pub order_id: Bytes,
    pub sender: Address,
    pub token: Address,
    pub sender_fee_recipient: Address,
    pub temporary_wallet_address: Address,
    pub sender_fee: i128,
    pub protocol_fee: i128,
    pub is_fulfilled: bool,
    pub is_refunded: bool,
    pub refund_address: Address,
    pub current_bps: i128,
    pub amount: i128,
    pub rate: i64,
    pub message_hash: String,
}
