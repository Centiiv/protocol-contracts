use soroban_sdk::{contracttype, Address, Bytes, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Usdc,
    Wallet,
    SettingsContract,
    NodeIDs,
    Nonces,
    Order(Bytes),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LpNode {
    pub capacity: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum LpNodeDisbursalStatus {
    Pending,
    Accepted,
    Completed,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RegistrationStatus {
    Unregistered,
    Pending,
    Approved,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LpNodeRequest {
    pub user_id: Address,
    pub lp_node_id: Bytes,
    pub amount: i128,
    pub status: LpNodeDisbursalStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum LpNodeStatus {
    Active,
    Inactive,
    NoLiquidity,
    Suspended,
    AwaitingApproval,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct OrderParams {
    pub order_id: Bytes,
    pub sender: Address,
    pub amount: i128,
    pub rate: i64,
    pub sender_fee_recipient: Address,
    pub sender_fee: i128,
    pub refund_address: Address,
    pub message_hash: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Order {
    pub sender: Address,
    pub amount: i128,
    pub sender_fee_recipient: Address,
    pub sender_fee: i128,
    pub protocol_fee: i128,
    pub is_fulfilled: bool,
    pub is_refunded: bool,
    pub refund_address: Address,
    pub current_bps: i64,
}
