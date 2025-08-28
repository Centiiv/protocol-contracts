use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, IntoVal};

use crate::{
    error::EscrowError,
    storage_types::{DataKey, EscrowData, EscrowStatus},
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn __constructor(env: Env, admin: Address, usdc_asset: Address, wallet_contract: Address) {
        admin.require_auth();

        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);

        env.storage()
            .persistent()
            .set(&DataKey::Wallet, &wallet_contract);
    }

    pub fn lock_funds(
        env: Env,
        request_id: Bytes,
        user_id: Address,
        lp_node_id: Address,
        amount: i128,
        timeout: u64,
    ) -> Result<(), EscrowError> {
        user_id.require_auth();

        if env.storage().persistent().has(&request_id) {
            return Err(EscrowError::RequestIDAlreadyUsed);
        }

        lp_node_id.require_auth();

        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();

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

        let escrow = EscrowData {
            user_id: user_id.clone(),
            lp_node_id: lp_node_id.clone(),
            amount,
            status: EscrowStatus::Locked,
            timeout: env.ledger().timestamp() + timeout,
        };

        env.storage().persistent().set(&request_id, &escrow);

        env.events()
            .publish((("Locked"), request_id, user_id, lp_node_id), amount);

        Ok(())
    }

    pub fn release_funds(
        env: Env,
        request_id: Bytes,
        lp_node_id: Address,
    ) -> Result<(), EscrowError> {
        lp_node_id.require_auth();

        let mut escrow: EscrowData = env.storage().persistent().get(&request_id).unwrap();

        if escrow.lp_node_id != lp_node_id || escrow.status != EscrowStatus::Locked {
            return Err(EscrowError::InvalidEscrowState);
        }

        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();

        env.invoke_contract::<()>(
            &wallet_contract,
            &symbol_short!("transfer"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                lp_node_id.into_val(&env),
                escrow.amount.into_val(&env),
            ],
        );

        escrow.status = EscrowStatus::Released;

        env.storage().persistent().set(&request_id, &escrow);

        env.events()
            .publish((("Funds Released"), request_id, lp_node_id), escrow.amount);

        Ok(())
    }

    pub fn refund_funds(env: Env, request_id: Bytes, user_id: Address) -> Result<(), EscrowError> {
        user_id.require_auth();

        let escrow: EscrowData = env.storage().persistent().get(&request_id).unwrap();

        if escrow.user_id != user_id || escrow.status != EscrowStatus::Locked {
            return Err(EscrowError::InvalidEscrowState);
        }

        if env.ledger().timestamp() < escrow.timeout {
            return Err(EscrowError::TimeoutNotReached);
        }

        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();

        env.invoke_contract::<()>(
            &wallet_contract,
            &symbol_short!("transfer"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                user_id.into_val(&env),
                escrow.amount.into_val(&env),
            ],
        );

        env.storage().persistent().set(
            &request_id,
            &(
                escrow.user_id,
                escrow.lp_node_id,
                escrow.amount,
                EscrowStatus::Refunded,
                escrow.timeout,
            ),
        );

        env.events()
            .publish((("Funds Refunded"), request_id, user_id), escrow.amount);

        Ok(())
    }

    pub fn get_escrow_status(env: Env, request_id: Bytes) -> Option<EscrowData> {
        env.storage().persistent().get(&request_id)
    }
}
