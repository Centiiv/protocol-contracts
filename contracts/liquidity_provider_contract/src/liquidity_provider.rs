use soroban_sdk::{
    contract, contractimpl,
    token::{self, TokenClient},
    xdr::ToXdr,
    Address, Bytes, Env, Map, String,
};

use crate::{
    error::ContractError,
    storage_types::{
        Algorithm, DataKey, LpNode, LpNodeDisbursalStatus, LpNodeRequest, LpNodeStatus,
        RegistrationStatus,
    },
};

#[contract]
pub struct LiquidityProviderContract;

#[contractimpl]
impl LiquidityProviderContract {
    pub fn __constructor(env: Env, admin: Address, usdc_asset: Address, wallet_contract: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
        env.storage()
            .persistent()
            .set(&DataKey::Wallet, &wallet_contract);
        env.storage().persistent().set(&DataKey::LastIdx, &0_i128);
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }

    pub fn register_lp_node(
        env: Env,
        lp_node_address: Address,
        capacity: i128,
        exchange_rate: i128,
        success_rate: i128,
        avg_payout_time: i128,
    ) -> Result<(), ContractError> {
        lp_node_address.require_auth();

        if capacity <= 0 || exchange_rate <= 0 || success_rate < 0 || avg_payout_time <= 0 {
            return Err(ContractError::InvlidLpNodeParameters);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::LpNode(lp_node_address.clone()))
        {
            return Err(ContractError::LpNodeIdAlreadyExists);
        }

        let lp_node = LpNode {
            capacity,
            exchange_rate,
            success_rate,
            avg_payout_time,
            operational_status: LpNodeStatus::AwaitingApproval,
            registration_status: RegistrationStatus::Pending,
        };

        env.storage()
            .persistent()
            .set(&DataKey::LpNode(lp_node_address.clone()), &lp_node);

        Ok(())
    }

    pub fn approve_lp_node(env: Env, lp_node_address: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        let mut node: LpNode = env
            .storage()
            .persistent()
            .get(&DataKey::LpNode(lp_node_address.clone()))
            .ok_or(ContractError::NotFound)?;

        node.operational_status = LpNodeStatus::Active;
        node.registration_status = RegistrationStatus::Approved;

        env.storage()
            .persistent()
            .set(&DataKey::LpNode(lp_node_address), &node);

        Ok(())
    }

    pub fn create_disbursal_request(
        env: Env,
        request_id: Bytes,
        user_id: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        user_id.require_auth();

        if amount <= 0 {
            return Err(ContractError::AmountMustBePositive);
        }

        if env.storage().persistent().has(&request_id) {
            return Err(ContractError::RequestIdAlreadyExists);
        }

        let request = LpNodeRequest {
            user_id: user_id.clone(),
            lp_node_id: Bytes::new(&env),
            amount,
            status: LpNodeDisbursalStatus::Pending,
        };

        env.storage().persistent().set(&request_id, &request);

        env.events()
            .publish(((("Request Created"), request_id), user_id), amount);
        Ok(())
    }

    pub fn update_operational_status(
        env: Env,
        lp_node_address: Address,
        new_status: LpNodeStatus,
    ) -> Result<(), ContractError> {
        lp_node_address.require_auth();

        let mut node: LpNode = env
            .storage()
            .persistent()
            .get(&DataKey::LpNode(lp_node_address.clone()))
            .ok_or(ContractError::NotFound)?;

        if node.registration_status != RegistrationStatus::Approved {
            return Err(ContractError::NotAuthorized);
        }

        node.operational_status = new_status;

        env.storage()
            .persistent()
            .set(&DataKey::LpNode(lp_node_address), &node);

        Ok(())
    }
    //
    //pub fn select_lp_node(
    //    env: Env,
    //    request_id: Bytes,
    //    algorithm: Algorithm,
    //    offchain_node_id: Option<Bytes>,
    //) -> Result<Bytes, ContractError> {
    //    let request: LpNodeRequest = env
    //        .storage()
    //        .persistent()
    //        .get(&request_id)
    //        .ok_or(ContractError::InvalidRequest)?;
    //
    //    let amount = request.amount;
    //    let mut selected_node_id = Bytes::new(&env);
    //
    //    let node_ids: Map<Bytes, bool> = env
    //        .storage()
    //        .persistent()
    //        .get(&DataKey::NodeIDs)
    //        .unwrap_or(Map::new(&env));
    //
    //    let mut nodes: Map<Bytes, (i128, i128, i128, i128, LpNodeStatus)> = Map::new(&env);
    //
    //    for (node_id, _) in node_ids.iter() {
    //        if let Some(node) = env
    //            .storage()
    //            .persistent()
    //            .get::<_, (i128, i128, i128, i128, LpNodeStatus)>(&node_id)
    //        {
    //            if node.4 == LpNodeStatus::Active && node.0 >= amount {
    //                nodes.set(node_id, node);
    //            }
    //        }
    //    }
    //
    //    match algorithm {
    //        Algorithm::Wrr => {
    //            let total_weight = nodes
    //                .iter()
    //                .map(|(_id, node)| node.0 * node.1 / 10_000)
    //                .sum::<i128>();
    //
    //            if total_weight == 0 {
    //                return Err(ContractError::NoSuitableLPNode);
    //            }
    //
    //            let current_index: i128 = env
    //                .storage()
    //                .persistent()
    //                .get(&DataKey::LastIdx)
    //                .unwrap_or(0);
    //
    //            let mut weight_sum = 0;
    //            for (node_id, node) in nodes.iter() {
    //                weight_sum += node.0 * node.1 / 10_000;
    //                if weight_sum > current_index % total_weight {
    //                    selected_node_id = node_id.clone();
    //                    break;
    //                }
    //            }
    //
    //            env.storage()
    //                .persistent()
    //                .set(&DataKey::LastIdx, &((current_index + 1) % total_weight));
    //        }
    //
    //        Algorithm::Greedy => {
    //            if let Some((id, _)) = nodes.iter().max_by(|(_, a), (_, b)| a.1.cmp(&b.1)) {
    //                selected_node_id = id.clone();
    //            }
    //        }
    //
    //        Algorithm::Scoring => {
    //            let max_rate = nodes.iter().map(|(_, node)| node.1).max().unwrap_or(1);
    //            let max_capacity = nodes.iter().map(|(_, node)| node.0).max().unwrap_or(1);
    //            let mut max_score = 0_i128;
    //
    //            for (node_id, node) in nodes.iter() {
    //                let score = (0.4 * (node.1 as f64 / max_rate as f64)
    //                    + 0.3 * (node.0 as f64 / max_capacity as f64)
    //                    + 0.2 * (node.2 as f64 / 10_000.0)
    //                    + 0.1 * (1000.0 / node.3 as f64))
    //                    * 1000.0;
    //
    //                if score as i128 > max_score {
    //                    max_score = score as i128;
    //                    selected_node_id = node_id.clone();
    //                }
    //            }
    //        }
    //
    //        Algorithm::Rl => {
    //            let off_id = offchain_node_id.ok_or(ContractError::InvalidLPNode)?;
    //            if !nodes.contains_key(off_id.clone()) {
    //                return Err(ContractError::InvalidLPNode);
    //            }
    //            selected_node_id = off_id;
    //        }
    //    }
    //
    //    if selected_node_id.is_empty() {
    //        return Err(ContractError::NoSuitableLPNode);
    //    }
    //
    //    env.storage().persistent().set(
    //        &request_id,
    //        &LpNodeRequest {
    //            user_id: request.user_id,
    //            lp_node_id: selected_node_id.clone(),
    //            amount,
    //            status: LpNodeDisbursalStatus::Pending,
    //        },
    //    );
    //
    //    env.events().publish(
    //        (("Node Selected"), request_id, selected_node_id.clone()),
    //        algorithm,
    //    );
    //
    //    Ok(selected_node_id)
    //}

    pub fn accept_disbursal_request(
        env: Env,
        request_id: Bytes,
        lp_node_id: Address,
    ) -> Result<(), ContractError> {
        lp_node_id.require_auth();
        let mut request: LpNodeRequest = env.storage().persistent().get(&request_id).unwrap();

        if request.lp_node_id != lp_node_id.clone().to_xdr(&env) {
            return Err(ContractError::UnauthorizedLpNode);
        }
        if request.status != LpNodeDisbursalStatus::Pending {
            return Err(ContractError::RequestNotPending);
        }
        request.status = LpNodeDisbursalStatus::Accepted;

        env.storage().persistent().set(&request_id, &request);

        env.events().publish(
            (("Request Accepted"), request_id, lp_node_id),
            request.amount,
        );
        Ok(())
    }

    pub fn complete_payout(
        env: Env,
        request_id: Bytes,
        beneficiary: Address,
        amount: i128,
        lp_node_id: Address,
        earnings: i128,
    ) -> Result<(), ContractError> {
        lp_node_id.require_auth();
        if amount <= 0 || earnings < 0 {
            return Err(ContractError::InvalidAmount);
        }

        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);

        token_client.transfer(&wallet_contract, &beneficiary, &amount);

        if earnings > 0 {
            token_client.transfer(&wallet_contract, &lp_node_id, &earnings);
        }

        env.events().publish(
            ("PayoutCompleted", request_id, lp_node_id),
            (amount, earnings),
        );

        Ok(())
    }

    pub fn get_lp_node_status(env: &Env, lp_id: Address) -> LpNodeStatus {
        env.storage()
            .persistent()
            .get::<_, LpNode>(&DataKey::LpNode(lp_id))
            .map(|node| node.operational_status)
            .unwrap_or(LpNodeStatus::AwaitingApproval)
    }

    pub fn get_lp_registration_status(env: &Env, lp_id: Address) -> RegistrationStatus {
        env.storage()
            .persistent()
            .get::<_, LpNode>(&DataKey::LpNode(lp_id))
            .map(|node| node.registration_status)
            .unwrap_or(RegistrationStatus::Unregistered)
    }

    pub fn get_disbursal_status(
        env: Env,
        request_id: Bytes,
    ) -> Option<(Address, Bytes, i128, String)> {
        env.storage().persistent().get(&request_id)
    }

    pub fn get_earnings(env: Env, lp_node_id: Address, request_id: Bytes) -> i128 {
        let node_earnings: Map<Bytes, i128> = env
            .storage()
            .persistent()
            .get(&lp_node_id.to_xdr(&env))
            .unwrap_or(Map::new(&env));
        node_earnings.get(request_id).unwrap_or(0)
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Admin)
    }

    pub fn get_usdc(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Usdc)
    }

    pub fn get_wallet_contract(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Wallet)
    }
}
