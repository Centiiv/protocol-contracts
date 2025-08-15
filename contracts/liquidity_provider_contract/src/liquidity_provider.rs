use soroban_sdk::{
    contract, contractimpl, symbol_short, xdr::ToXdr, Address, Bytes, Env, Map, String, Symbol,
};

#[contract]
pub struct LiquidityProviderContract;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl LiquidityProviderContract {
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address, wallet_contract: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("usdc"), &usdc_asset);
        env.storage()
            .persistent()
            .set(&symbol_short!("wallet"), &wallet_contract);
        env.storage()
            .persistent()
            .set(&symbol_short!("last_idx"), &0_i128);
        env.storage().persistent().set(&ADMIN, &admin);
    }

    pub fn register_lp_node(
        env: Env,
        lp_node_id: Bytes,
        capacity: i128,
        exchange_rate: i128,
        success_rate: i128,
        avg_payout_time: i128,
    ) {
        let admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        admin.require_auth();
        let lp_node = (
            capacity,
            exchange_rate,
            success_rate,
            avg_payout_time,
            String::from_str(&env, "active"),
        );
        env.storage().persistent().set(&lp_node_id, &lp_node);

        // Track LP node IDs
        let mut node_ids: Map<Bytes, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("node_ids"))
            .unwrap_or(Map::new(&env));
        node_ids.set(lp_node_id.clone(), true);
        env.storage()
            .persistent()
            .set(&symbol_short!("node_ids"), &node_ids);
    }

    pub fn create_disbursal_request(env: Env, request_id: Bytes, user_id: Address, amount: i128) {
        user_id.require_auth();
        let request = (
            user_id.clone(),
            Bytes::new(&env),
            amount,
            String::from_str(&env, "pending"),
        );
        env.storage().persistent().set(&request_id, &request);
        env.events()
            .publish((symbol_short!("Created"), request_id, user_id), amount);
    }

    pub fn select_lp_node(
        env: Env,
        request_id: Bytes,
        algorithm: String,
        offchain_node_id: Option<Bytes>,
    ) -> Bytes {
        let request: (Address, Bytes, i128, String) =
            env.storage().persistent().get(&request_id).unwrap();
        let amount = request.2;
        let mut selected_node_id = Bytes::new(&env);

        // Fetch active LP nodes
        let node_ids: Map<Bytes, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("node_ids"))
            .unwrap_or(Map::new(&env));
        let mut nodes: Map<Bytes, (i128, i128, i128, i128, String)> = Map::new(&env);
        for (node_id, _) in node_ids.iter() {
            if let Some(node) = env
                .storage()
                .persistent()
                .get::<_, (i128, i128, i128, i128, String)>(&node_id)
            {
                if node.4 == String::from_str(&env, "active") && node.0 >= amount {
                    nodes.set(node_id, node);
                }
            }
        }

        if algorithm == String::from_str(&env, "wrr") {
            let total_weight = nodes
                .iter()
                .map(|(_id, node)| node.0 * node.1 / 10000)
                .sum::<i128>();
            if total_weight == 0 {
                panic!("No suitable LP node");
            }
            let current_index: i128 = env
                .storage()
                .persistent()
                .get(&symbol_short!("last_idx"))
                .unwrap_or(0);
            let mut weight_sum = 0;
            for (node_id, node) in nodes.iter() {
                weight_sum += node.0 * node.1 / 10000;
                if weight_sum > current_index % total_weight {
                    selected_node_id = node_id;
                    break;
                }
            }
            env.storage().persistent().set(
                &symbol_short!("last_idx"),
                &((current_index + 1) % total_weight),
            );
        } else if algorithm == String::from_str(&env, "greedy") {
            selected_node_id = nodes
                .iter()
                .max_by(|(_, a), (_, b)| a.1.cmp(&b.1))
                .map(|(id, _)| id)
                .unwrap_or(Bytes::new(&env));
        } else if algorithm == String::from_str(&env, "scoring") {
            let max_rate = nodes.iter().map(|(_, node)| node.1).max().unwrap_or(1);
            let max_capacity = nodes.iter().map(|(_, node)| node.0).max().unwrap_or(1);
            let mut max_score = 0_i128;
            for (node_id, node) in nodes.iter() {
                let score = (0.4 * (node.1 as f64 / max_rate as f64)
                    + 0.3 * (node.0 as f64 / max_capacity as f64)
                    + 0.2 * (node.2 as f64 / 10000.0)
                    + 0.1 * (1000.0 / node.3 as f64))
                    * 1000.0;
                if score as i128 > max_score {
                    max_score = score as i128;
                    selected_node_id = node_id;
                }
            }
        } else if algorithm == String::from_str(&env, "rl") {
            selected_node_id = offchain_node_id.unwrap();
            if !nodes.contains_key(selected_node_id.clone()) {
                panic!("Invalid LP node");
            }
        } else {
            panic!("Unsupported algorithm");
        }

        if selected_node_id.is_empty() {
            panic!("No suitable LP node");
        }

        env.storage().persistent().set(
            &request_id,
            &(
                request.0,
                selected_node_id.clone(),
                amount,
                String::from_str(&env, "pending"),
            ),
        );
        env.events().publish(
            (
                symbol_short!("NodSelect"),
                request_id,
                selected_node_id.clone(),
            ),
            algorithm,
        );
        selected_node_id
    }

    pub fn accept_disbursal_request(env: Env, request_id: Bytes, lp_node_id: Address) {
        lp_node_id.require_auth();
        let mut request: (Address, Bytes, i128, String) =
            env.storage().persistent().get(&request_id).unwrap();
        if request.1 != lp_node_id.clone().to_xdr(&env) {
            panic!("Unauthorized LP node");
        }
        request.3 = String::from_str(&env, "accepted");
        env.storage().persistent().set(&request_id, &request);

        env.events().publish(
            (symbol_short!("Accepted"), request_id, lp_node_id),
            request.2,
        );
    }

    pub fn complete_payout(env: Env, request_id: Bytes, lp_node_id: Address, earnings: i128) {
        lp_node_id.require_auth();
        let mut request: (Address, Bytes, i128, String) =
            env.storage().persistent().get(&request_id).unwrap();
        if request.1 != lp_node_id.clone().to_xdr(&env) {
            panic!("Unauthorized LP node");
        }
        request.3 = String::from_str(&env, "completed");
        env.storage().persistent().set(&request_id, &request);

        // Record earnings
        let mut node_earnings: Map<Bytes, i128> = env
            .storage()
            .persistent()
            .get(&lp_node_id.clone().to_xdr(&env))
            .unwrap_or(Map::new(&env));
        node_earnings.set(request_id.clone(), earnings);
        env.storage()
            .persistent()
            .set(&lp_node_id.clone().to_xdr(&env), &node_earnings);

        env.events().publish(
            (symbol_short!("PytCmplt"), request_id, lp_node_id),
            earnings,
        );
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
}
