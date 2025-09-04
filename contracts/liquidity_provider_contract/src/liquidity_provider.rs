//! Gateway Main Contract
use crate::{
    error::ContractError,
    liquidity_provider_trait::IGateway,
    storage_types::{DataKey, LpNode, Order, OrderParams},
};
use liquidity_manager::{self, liquidity_manager::GatewaySettingManagerContractClient};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, Map};

#[contract]
pub struct GatewayContract;

#[contractimpl]
impl IGateway for GatewayContract {
    fn create_order(env: Env, params: OrderParams) -> Result<(), ContractError> {
        params.sender.require_auth();
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = GatewaySettingManagerContractClient::new(&env, &settings_contract);

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
        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);
        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
        let protocol_fee = (params.amount * protocol_fee_percent as i128) / max_bps as i128;

        // Transfer USDC to wallet_contract (via EscrowContract in workflow)
        token_client.transfer(
            &params.sender,
            &wallet_contract,
            &(params.amount + params.sender_fee),
        );

        let order = Order {
            sender: params.sender.clone(),
            amount: params.amount,
            sender_fee_recipient: params.sender_fee_recipient,
            sender_fee: params.sender_fee,
            protocol_fee,
            is_fulfilled: false,
            is_refunded: false,
            refund_address: params.refund_address.clone(),
            current_bps: max_bps,
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
        settle_percent: i64,
    ) -> Result<(), ContractError> {
        // Get settings contract and verify aggregator
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = GatewaySettingManagerContractClient::new(&env, &settings_contract);
        let aggregator: Address = settings_client.get_aggregator_address();
        aggregator.require_auth();

        // Validate settle_percent
        if settle_percent <= 0 || settle_percent > 100_000 {
            return Err(ContractError::InvalidSettlePercent);
        }

        // Get order
        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        // Check order state
        if order.is_fulfilled {
            return Err(ContractError::OrderFulfilled);
        }
        if order.is_refunded {
            return Err(ContractError::OrderRefunded);
        }

        // Get necessary addresses and clients
        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
        let treasury: Address = settings_client.get_treasury_address();
        let token_client = token::Client::new(&env, &usdc_asset);

        // Store current BPS before modification
        let current_order_bps = order.current_bps;

        // Update order BPS
        order.current_bps -= settle_percent;

        // Calculate liquidity provider amount based on the original amount and current settle percentage
        let liquidity_provider_amount =
            (order.amount * settle_percent as i128) / current_order_bps as i128;

        // Update remaining order amount
        order.amount -= liquidity_provider_amount;

        // Get protocol fee percentage from settings (returns tuple of fee_percent, max_bps)
        let (protocol_fee_percent, _max_bps) = settings_client.get_fee_details();

        // Calculate protocol fee based on the liquidity provider amount (not the remaining order amount)
        let protocol_fee = (liquidity_provider_amount * protocol_fee_percent as i128) / 100_000i128;

        // Calculate final transfer amount to LP
        let transfer_amount = liquidity_provider_amount - protocol_fee;

        // If order is fully settled, mark as fulfilled and handle sender fee
        if order.current_bps == 0 {
            order.is_fulfilled = true;

            if order.sender_fee > 0 {
                token_client.transfer(
                    &wallet_contract,
                    &order.sender_fee_recipient,
                    &order.sender_fee,
                );

                env.events().publish(
                    ("SenderFeeTransferred", order.sender_fee_recipient.clone()),
                    order.sender_fee,
                );
            }
        }

        // Transfer protocol fee to treasury
        if protocol_fee > 0 {
            token_client.transfer(&wallet_contract, &treasury, &protocol_fee);
        }

        // Transfer remaining amount to liquidity provider
        token_client.transfer(&wallet_contract, &liquidity_provider, &transfer_amount);

        // Save updated order
        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        // Emit settlement event
        env.events().publish(
            ("OrderSettled", split_order_id, order_id, liquidity_provider),
            settle_percent,
        );

        Ok(())
    }

    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError> {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = GatewaySettingManagerContractClient::new(&env, &settings_contract);
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
        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
        let treasury: Address = settings_client.get_treasury_address();
        let token_client = token::Client::new(&env, &usdc_asset);

        if fee > 0 {
            token_client.transfer(&wallet_contract, &treasury, &fee);
        }
        let refund_amount = order.amount + order.sender_fee - fee;
        token_client.transfer(&wallet_contract, &order.refund_address, &refund_amount);

        order.is_refunded = true;
        order.current_bps = 0;
        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(("OrderRefunded", order_id), fee);
        Ok(())
    }

    fn get_order_info(env: Env, order_id: Bytes) -> Result<Order, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(ContractError::OrderNotFound)
    }

    fn get_fee_details(env: Env) -> (i64, i64) {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = GatewaySettingManagerContractClient::new(&env, &settings_contract);
        settings_client.get_fee_details()
    }
}

#[contractimpl]
impl GatewayContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_asset: Address,
        wallet_contract: Address,
        settings_contract: Address,
    ) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
        env.storage()
            .persistent()
            .set(&DataKey::Wallet, &wallet_contract);
        env.storage()
            .persistent()
            .set(&DataKey::SettingsContract, &settings_contract);
    }

    fn register_lp_node(env: Env, lp_node_id: Bytes, capacity: i128) -> Result<(), ContractError> {
        if capacity <= 0 {
            return Err(ContractError::InvalidLpNodeParameters);
        }

        let lp_node = LpNode { capacity };
        let mut node_ids: Map<Bytes, bool> = env
            .storage()
            .persistent()
            .get(&DataKey::LpNode(lp_node_address.clone()))
            .ok_or(ContractError::NotFound)?;

        node.operational_status = LpNodeStatus::Active;
        node.registration_status = RegistrationStatus::Approved;

        env.storage().persistent().set(&lp_node_id, &lp_node);
        node_ids.set(lp_node_id.clone(), true);
        env.storage().persistent().set(&DataKey::NodeIDs, &node_ids);

        env.events()
            .publish(("LpNodeRegistered", lp_node_id), capacity);

        Ok(())
    }
}
