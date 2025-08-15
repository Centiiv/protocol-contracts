use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, IntoVal, String};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address, wallet_contract: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("usdc"), &usdc_asset);
        env.storage()
            .persistent()
            .set(&symbol_short!("wallet"), &wallet_contract);
    }

    pub fn lock_funds(
        env: Env,
        request_id: Bytes,
        user_id: Address,
        lp_node_id: Address,
        amount: i128,
        timeout: u64,
    ) {
        user_id.require_auth();
        lp_node_id.require_auth();
        let wallet_contract: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("wallet"))
            .unwrap();

        // Transfer USDC to escrow
        env.invoke_contract::<()>(
            &wallet_contract,
            &symbol_short!("transfer"),
            soroban_sdk::vec![
                &env,
                user_id.into_val(&env),
                env.current_contract_address().into_val(&env),
                amount.into_val(&env),
            ],
        );

        // Record escrow
        let escrow = (
            user_id.clone(),
            lp_node_id.clone(),
            amount,
            String::from_str(&env, "locked"),
            env.ledger().timestamp() + timeout,
        );
        env.storage().persistent().set(&request_id, &escrow);

        env.events().publish(
            (symbol_short!("Locked"), request_id, user_id, lp_node_id),
            amount,
        );
    }

    pub fn release_funds(env: Env, request_id: Bytes, lp_node_id: Address) {
        lp_node_id.require_auth();
        let escrow: (Address, Address, i128, String, u64) =
            env.storage().persistent().get(&request_id).unwrap();
        if escrow.1 != lp_node_id || escrow.3 != String::from_str(&env, "locked") {
            panic!("Invalid escrow state");
        }

        let wallet_contract: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("wallet"))
            .unwrap();
        env.invoke_contract::<()>(
            &wallet_contract,
            &symbol_short!("transfer"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                lp_node_id.into_val(&env),
                escrow.2.into_val(&env),
            ],
        );

        env.storage().persistent().set(
            &request_id,
            &(
                escrow.0,
                escrow.1,
                escrow.2,
                String::from_str(&env, "released"),
                escrow.4,
            ),
        );
        env.events().publish(
            (symbol_short!("FndsRlsed"), request_id, lp_node_id),
            escrow.2,
        );
    }

    pub fn refund_funds(env: Env, request_id: Bytes, user_id: Address) {
        user_id.require_auth();
        let escrow: (Address, Address, i128, String, u64) =
            env.storage().persistent().get(&request_id).unwrap();
        if escrow.0 != user_id || escrow.3 != String::from_str(&env, "locked") {
            panic!("Invalid escrow state");
        }
        if env.ledger().timestamp() < escrow.4 {
            panic!("Timeout not reached");
        }

        let wallet_contract: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("wallet"))
            .unwrap();
        env.invoke_contract::<()>(
            &wallet_contract,
            &symbol_short!("transfer"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                user_id.into_val(&env),
                escrow.2.into_val(&env),
            ],
        );

        env.storage().persistent().set(
            &request_id,
            &(
                escrow.0,
                escrow.1,
                escrow.2,
                String::from_str(&env, "refunded"),
                escrow.4,
            ),
        );
        env.events()
            .publish((symbol_short!("FndsRefnd"), request_id, user_id), escrow.2);
    }

    pub fn get_escrow_status(
        env: Env,
        request_id: Bytes,
    ) -> Option<(Address, Address, i128, String, u64)> {
        env.storage().persistent().get(&request_id)
    }
}
