use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::{
    error::ContractError,
    storage::{DataKey, ProtocolAddressType},
};

#[contract]
pub struct LPSettingManagerContract;

#[contractimpl]
impl LPSettingManagerContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::ProtocolFeePercent, &1000_i64);
        env.storage()
            .persistent()
            .set(&DataKey::MaxBps, &100_000_i64);
        env.storage().persistent().set(&DataKey::Paused, &false);
    }

    pub fn update_protocol_fee(env: Env, protocol_fee_percent: i64) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if !(0..=100_000).contains(&protocol_fee_percent) {
            return Err(ContractError::InvalidFeePercent);
        }
        env.storage()
            .persistent()
            .set(&DataKey::ProtocolFeePercent, &protocol_fee_percent);
        env.events()
            .publish(("ProtocolFeeUpdated",), protocol_fee_percent);
        Ok(())
    }

    fn update_address(
        env: &Env,
        key: DataKey,
        what: ProtocolAddressType,
        new_value: Address,
    ) -> Result<(), ContractError> {
        if let Some(current) = env.storage().persistent().get::<_, Address>(&key) {
            if current == new_value {
                return Err(ContractError::AddressAlreadySet);
            }
        }
        env.storage().persistent().set(&key, &new_value);
        env.events()
            .publish(("ProtocolAddressUpdated", what), new_value);
        Ok(())
    }

    pub fn update_protocol_address(
        env: Env,
        what: ProtocolAddressType,
        value: Address,
    ) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        match what {
            ProtocolAddressType::Treasury => {
                Self::update_address(&env, DataKey::Treasury, what, value)
            }
            ProtocolAddressType::Aggregator => {
                Self::update_address(&env, DataKey::Aggregator, what, value)
            }
        }
    }

    pub fn pause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused, &true);
        env.events().publish(("Paused",), ());
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.events().publish(("Unpaused",), ());
        Ok(())
    }

    pub fn get_fee_details(env: Env) -> (i64, i64) {
        let protocol_fee_percent: i64 = env
            .storage()
            .persistent()
            .get(&DataKey::ProtocolFeePercent)
            .unwrap_or(0);
        let max_bps: i64 = env
            .storage()
            .persistent()
            .get(&DataKey::MaxBps)
            .unwrap_or(100_000);
        (protocol_fee_percent, max_bps)
    }

    pub fn get_treasury_address(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Treasury).unwrap()
    }

    pub fn get_aggregator_address(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Aggregator)
            .unwrap()
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn is_token_supported(env: Env, token: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::TokenSupported(token))
            .unwrap_or(false)
    }

    pub fn upgrade_lp_manager(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
