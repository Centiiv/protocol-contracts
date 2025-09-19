use crate::{
    error::ContractError,
    liquidity_provider_trait::IGateway,
    storage_types::{DataKey, LpNode, Order, OrderParams},
};
use liquidity_manager::{self, liquidity_manager::LPSettingManagerContractClient};
use soroban_sdk::{contract, contractimpl, log, token, Address, Bytes, BytesN, Env, Map};

#[contract]
pub struct LPContract;

#[contractimpl]
impl IGateway for LPContract {
    fn create_order(env: Env, params: OrderParams) -> Result<(), ContractError> {
        params.sender.require_auth();

        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        if settings_client.is_paused() {
            return Err(ContractError::Paused);
        }

        if params.amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if params.message_hash.is_empty() {
            return Err(ContractError::InvalidMessageHash);
        }

        let order_exists: Option<Order> = env
            .storage()
            .persistent()
            .get(&DataKey::Order(params.order_id.clone()));

        if order_exists.is_some() {
            return Err(ContractError::OrderAlreadyExists);
        }

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);
        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
        let protocol_fee = (params.amount * protocol_fee_percent as i128) / max_bps as i128;

        token_client.transfer(
            &params.sender,
            &env.current_contract_address(),
            &(params.amount + params.sender_fee),
        );

        let order = Order {
            order_id: params.order_id.clone(),
            sender: params.sender.clone(),
            token: usdc_asset,
            amount: params.amount,
            sender_fee_recipient: params.sender_fee_recipient,
            sender_fee: params.sender_fee,
            protocol_fee,
            is_fulfilled: false,
            is_refunded: false,
            refund_address: params.refund_address.clone(),
            current_bps: max_bps as i128,
            rate: params.rate,
            message_hash: params.message_hash.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Order(params.order_id.clone()), &order);

        let mut nonces: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&DataKey::Nonces)
            .unwrap_or(Map::new(&env));

        nonces.set(
            params.sender.clone(),
            nonces.get(params.sender.clone()).unwrap_or(0) + 1,
        );
        env.storage().persistent().set(&DataKey::Nonces, &nonces);

        env.events().publish(
            ("OrderCreated", params.order_id, params.sender),
            (
                params.refund_address,
                params.amount,
                protocol_fee,
                params.rate,
                params.message_hash,
            ),
        );
        Ok(())
    }

    fn settle(
        env: Env,
        split_order_id: Bytes,
        order_id: Bytes,
        liquidity_provider: Address,
        settle_percent: i128,
    ) -> Result<bool, ContractError> {
        let settings_contract: Address =
            match env.storage().persistent().get(&DataKey::SettingsContract) {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::SettingsContractNotSet);
                }
            };

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let aggregator: Address = settings_client.get_aggregator_address();

        aggregator.require_auth();

        if settle_percent <= 0 || settle_percent > 100_000 {
            return Err(ContractError::InvalidSettlePercent);
        }

        let order_option: Option<Order> = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()));

        if order_option.is_none() {
            return Err(ContractError::OrderNotFound);
        }

        let mut order: Order = order_option.unwrap();

        if order.is_fulfilled {
            return Err(ContractError::OrderFulfilled);
        }

        if order.is_refunded {
            return Err(ContractError::OrderRefunded);
        }

        let usdc_asset: Address = match env.storage().persistent().get(&DataKey::Usdc) {
            Some(addr) => addr,
            None => {
                return Err(ContractError::UsdcNotSet);
            }
        };

        let treasury: Address = settings_client.get_treasury_address();

        let token_client = token::Client::new(&env, &usdc_asset);

        let current_order_bps = order.current_bps;

        order.current_bps -= settle_percent;

        let liquidity_provider_amount = (order.amount * settle_percent) / current_order_bps;

        order.amount -= liquidity_provider_amount;

        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();

        let protocol_fee =
            (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);

        let transfer_amount = liquidity_provider_amount - protocol_fee;

        if order.current_bps == 0 {
            order.is_fulfilled = true;

            if order.sender_fee > 0 {
                match token_client.try_transfer(
                    &env.current_contract_address(),
                    &order.sender_fee_recipient,
                    &order.sender_fee,
                ) {
                    Ok(_) => log!(&env, "Sender fee transferred successfully"),
                    Err(_) => {
                        return Err(ContractError::TransferFailed);
                    }
                }
            }
        }

        if protocol_fee > 0 {
            match token_client.try_transfer(
                &env.current_contract_address(),
                &treasury,
                &protocol_fee,
            ) {
                Ok(_) => log!(&env, "Protocol fee transferred successfully"),
                Err(_) => {
                    return Err(ContractError::TransferFailed);
                }
            }
        }

        if transfer_amount > 0 {
            match token_client.try_transfer(
                &env.current_contract_address(),
                &liquidity_provider,
                &transfer_amount,
            ) {
                Ok(_) => log!(&env, "Liquidity provider amount transferred successfully"),
                Err(_) => {
                    return Err(ContractError::TransferFailed);
                }
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(
            ("OrderSettled", split_order_id, order_id, liquidity_provider),
            settle_percent,
        );

        Ok(true)
    }

    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError> {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let aggregator: Address = settings_client.get_aggregator_address();
        aggregator.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        if order.is_fulfilled {
            return Err(ContractError::OrderFulfilled);
        }

        if order.is_refunded {
            return Err(ContractError::OrderRefunded);
        }

        if fee > order.protocol_fee {
            return Err(ContractError::FeeExceedsProtocolFee);
        }

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();

        let treasury: Address = settings_client.get_treasury_address();

        let token_client = token::Client::new(&env, &usdc_asset);

        if fee > 0 {
            token_client.transfer(&env.current_contract_address(), &treasury, &fee);
        }

        let refund_amount = (order.amount + order.sender_fee) - fee;

        token_client.transfer(
            &env.current_contract_address(),
            &order.refund_address,
            &refund_amount,
        );

        order.is_refunded = true;

        order.current_bps = 0;

        order.amount = 0;

        order.sender_fee = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(("OrderRefunded", order_id), fee);

        Ok(())
    }

    fn get_token_balance(env: Env, user: Address) -> i128 {
        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.balance(&user)
    }

    fn get_order_id(env: Env, order_id: Bytes) -> Result<Bytes, ContractError> {
        let order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(ContractError::OrderNotFound)?;

        Ok(order.order_id)
    }

    fn get_order_info(env: Env, order_id: Bytes) -> Result<Order, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(ContractError::OrderNotFound)
    }

    fn get_lp_fee_details(env: Env) -> (i64, i64) {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
        settings_client.get_fee_details()
    }
}

#[contractimpl]
impl LPContract {
    pub fn init(env: Env, admin: Address, usdc_asset: Address, settings_contract: Address) {
        admin.require_auth();

        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
        env.storage()
            .persistent()
            .set(&DataKey::SettingsContract, &settings_contract);
    }

    pub fn register_lp_node(
        env: Env,
        lp_node_id: Bytes,
        capacity: i128,
    ) -> Result<(), ContractError> {
        if capacity <= 0 {
            return Err(ContractError::InvalidLpNodeParameters);
        }

        let lp_node = LpNode { capacity };
        let mut node_ids: Map<Bytes, bool> = env
            .storage()
            .persistent()
            .get(&DataKey::NodeIDs)
            .unwrap_or(Map::new(&env));

        if node_ids.contains_key(lp_node_id.clone()) {
            return Err(ContractError::LpNodeIdAlreadyExists);
        }

        env.storage().persistent().set(&lp_node_id, &lp_node);
        node_ids.set(lp_node_id.clone(), true);
        env.storage().persistent().set(&DataKey::NodeIDs, &node_ids);

        env.events()
            .publish(("LpNodeRegistered", lp_node_id), capacity);

        Ok(())
    }

    pub fn upgrade_lp(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
