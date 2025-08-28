use soroban_sdk::{contracttype, Address, Bytes};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Usdc,
    Wallet,
    LastIdx,
    NodeIDs,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum LpNodeDisbursalStatus {
    Pending,
    Accepted,
    Completed,
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
#[derive(Clone)]
pub struct LpNode {
    pub capacity: i128,
    pub exchange_rate: i128,
    pub success_rate: i128,
    pub avg_payout_time: i128,
    pub s_active: LpNodeStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum LpNodeStatus {
    Active,
    Inactive,
}

#[contracttype]
#[derive(PartialEq, Clone, Debug)]
pub enum Algorithm {
    Wrr,
    Greedy,
    Scoring,
    Rl,
}
