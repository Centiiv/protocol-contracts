use crate::{
    error::ContractError,
    liquidity_provider_trait::IGateway,
    storage_types::{DataKey, LpNode, Order, OrderParams, PendingRefund, PendingSettlement},
};
use liquidity_manager::{self, liquidity_manager::LPSettingManagerContractClient};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, Map};

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
            &params.temporary_wallet_address,
            &(params.amount + params.sender_fee),
        );

        let order = Order {
            order_id: params.order_id.clone(),
            sender: params.sender.clone(),
            token: usdc_asset,
            amount: params.amount,
            sender_fee_recipient: params.sender_fee_recipient,
            temporary_wallet_address: params.temporary_wallet_address,
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
        order_id: Bytes,
        liquidity_provider: Address,
        settle_percent: i128,
    ) -> Result<bool, ContractError> {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let relayer: Address = settings_client.get_relayer_address();

        relayer.require_auth();

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
        }

        let pending_settlement = PendingSettlement {
            order_id: order_id.clone(),
            protocol_fee,
            transfer_amount,
            liquidity_provider: liquidity_provider.clone(),
            settle_percent,
        };

        env.storage().persistent().set(
            &DataKey::PendingSettlement(order_id.clone()),
            &pending_settlement,
        );

        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(
            ("OrderSettled", order_id, liquidity_provider),
            settle_percent,
        );

        Ok(true)
    }

    fn execute_settlement_transfer(env: Env, order_id: Bytes) -> Result<(), ContractError> {
        let order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        order.temporary_wallet_address.require_auth();

        let pending_settlement: PendingSettlement = env
            .storage()
            .persistent()
            .get(&DataKey::PendingSettlement(order_id.clone()))
            .ok_or(ContractError::NoPendingSettlement)?;

        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let treasury: Address = settings_client.get_treasury_address();

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();

        let token_client = token::Client::new(&env, &usdc_asset);

        if pending_settlement.protocol_fee > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &treasury,
                &pending_settlement.protocol_fee,
            );
        }

        if pending_settlement.transfer_amount > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &pending_settlement.liquidity_provider,
                &pending_settlement.transfer_amount,
            );
        }

        if order.is_fulfilled && order.sender_fee > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &order.sender_fee_recipient,
                &order.sender_fee,
            );
        }

        env.storage()
            .persistent()
            .remove(&DataKey::PendingSettlement(order_id.clone()));

        env.events().publish(
            ("SettlementTransferred", order_id),
            pending_settlement.settle_percent,
        );

        Ok(())
    }

    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError> {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let relayer: Address = settings_client.get_relayer_address();

        relayer.require_auth();

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

        let pending_refund = PendingRefund {
            order_id: order_id.clone(),
            fee,
            refund_amount: (order.amount + order.sender_fee) - fee,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PendingRefund(order_id.clone()), &pending_refund);

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

    fn execute_refund_transfer(env: Env, order_id: Bytes) -> Result<(), ContractError> {
        let order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        order.temporary_wallet_address.require_auth();

        let pending_refund: PendingRefund = env
            .storage()
            .persistent()
            .get(&DataKey::PendingRefund(order_id.clone()))
            .ok_or(ContractError::NoPendingRefund)?;

        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        let treasury: Address = settings_client.get_treasury_address();

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();

        let token_client = token::Client::new(&env, &usdc_asset);

        if pending_refund.fee > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &treasury,
                &pending_refund.fee,
            );
        }

        token_client.transfer(
            &order.temporary_wallet_address,
            &order.refund_address,
            &pending_refund.refund_amount,
        );

        env.storage()
            .persistent()
            .remove(&DataKey::PendingRefund(order_id.clone()));

        env.events().publish(
            ("RefundTransferred", order_id),
            pending_refund.refund_amount,
        );

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
