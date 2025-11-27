use crate::{
    error::ContractError,
    liquidity_provider_trait::IGateway,
    storage_types::{DataKey, LpNode, Order, OrderParams, PendingRefund, PendingSettlement},
};
use liquidity_manager::{self, liquidity_manager::LPSettingManagerContractClient};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, Map};

/// # Liquidity Provider Contract
///
/// ## Overview
/// A two-step settlement system for cross-chain liquidity provisioning.
///
/// ## Key Features:
/// - **Two-Step Settlement**: Separate state updates from token transfers
/// - **Non-Custodial**: Temporary wallets hold funds, not the contract
/// - **Partial Settlements**: Orders can be settled in multiple chunks
/// - **Secure Refunds**: Guaranteed refund mechanism for failed orders
///
/// ## Security Model:
/// - **Sender**: Authorizes order creation and fund transfer to temporary wallet
/// - **Relayer**: Authorizes settlement/refund state changes
/// - **Temporary Wallet**: Authorizes actual token transfers
///
/// ## State Transitions:
/// - Order → Settled (via settle() + execute_settlement_transfer())
/// - Order → Refunded (via refund() + execute_refund_transfer())
/// - States are one-way: cannot revert from Settled/Refunded
#[contract]
pub struct LPContract;

#[contractimpl]
impl IGateway for LPContract {
    /// # Create a new liquidity order
    ///
    /// ## Description:
    /// Creates a new order and transfers funds from sender to temporary wallet.
    /// The order enters a pending state until settled or refunded.
    ///
    /// ## Authorization:
    /// - `params.sender`: Must authorize the token transfer
    ///
    /// ## Validation:
    /// - Contract must not be paused
    /// - Amount must be positive
    /// - Message hash must not be empty
    /// - Order ID must not already exist
    ///
    /// ## Events:
    /// - `("OrderCreated", order_id, sender)` with order details
    ///
    /// ## Parameters:
    /// - `env`: Soroban environment
    /// - `params`: Order creation parameters
    ///
    /// ## Returns:
    /// - `Ok(())` on success
    /// - `Err(ContractError)` on failure
    fn create_order(env: Env, params: OrderParams) -> Result<(), ContractError> {
        params.sender.require_auth();

        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();

        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        // Check if contract is paused
        if settings_client.is_paused() {
            return Err(ContractError::Paused);
        }

        // Validate input parameters
        if params.amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if params.message_hash.is_empty() {
            return Err(ContractError::InvalidMessageHash);
        }

        // Ensure order doesn't already exist
        let order_exists: Option<Order> = env
            .storage()
            .persistent()
            .get(&DataKey::Order(params.order_id.clone()));

        if order_exists.is_some() {
            return Err(ContractError::OrderAlreadyExists);
        }

        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);

        // Calculate protocol fee (currently 1%)
        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
        let protocol_fee = (params.amount * protocol_fee_percent as i128) / max_bps as i128;

        // Transfer funds from sender to temporary wallet
        token_client.transfer(
            &params.sender,
            &params.temporary_wallet_address,
            &(params.amount),
        );

        // Create and store order
        let order = Order {
            order_id: params.order_id.clone(),
            sender: params.sender.clone(),
            token: usdc_asset,
            amount: params.amount,
            temporary_wallet_address: params.temporary_wallet_address,
            protocol_fee,
            is_fulfilled: false,
            is_refunded: false,
            refund_address: params.refund_address.clone(),
            current_bps: max_bps as i128, // 100,000 = 100%
            rate: params.rate,
            message_hash: params.message_hash.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Order(params.order_id.clone()), &order);

        // Update sender nonce for replay protection
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

        // Emit creation event
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

    /// # Settle an order (Step 1: State Update)
    ///
    /// ## Description:
    /// Updates order state and calculates settlement amounts. This is the first step
    /// in the two-step settlement process. No tokens are transferred here.
    ///
    /// ## Authorization:
    /// - `relayer`: Must authorize the state change
    ///
    /// ## Validation:
    /// - Order must exist and not be fulfilled/refunded
    /// - Settle percent must be valid (0 < percent ≤ 100,000)
    /// - Order must have sufficient remaining BPS
    ///
    /// ## State Changes:
    /// - Updates order amount and current_bps
    /// - Marks order as fulfilled if current_bps reaches 0
    /// - Stores pending settlement for transfer execution
    ///
    /// ## Events:
    /// - `("OrderSettled", order_id, liquidity_provider)` with settle_percent
    ///
    /// ## Parameters:
    /// - `order_id`: Unique identifier for the order
    /// - `liquidity_provider`: Address to receive settled funds
    /// - `settle_percent`: Percentage to settle (in basis points, 100,000 = 100%)
    ///
    /// ## Returns:
    /// - `Ok(true)` on successful state update
    /// - `Err(ContractError)` on failure
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

        if settings_client.is_paused() {
            return Err(ContractError::Paused);
        }

        let relayer: Address = settings_client.get_relayer_address();
        relayer.require_auth();

        // Validate settle percentage (0 < percent ≤ 100,000)
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

        if order.current_bps == 0 {
            return Err(ContractError::OrderFulfilled);
        }

        // Prevent double-settlement and settlement of refunded orders
        if order.is_fulfilled {
            return Err(ContractError::OrderFulfilled);
        }

        if order.is_refunded {
            return Err(ContractError::OrderRefunded);
        }

        // Calculate settlement amounts
        let current_order_bps = order.current_bps;
        order.current_bps -= settle_percent;

        let liquidity_provider_amount = (order.amount * settle_percent) / current_order_bps;
        order.amount -= liquidity_provider_amount;

        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
        let protocol_fee =
            (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);
        let transfer_amount = liquidity_provider_amount - protocol_fee;

        // Mark as fulfilled if completely settled
        if order.current_bps == 0 {
            order.is_fulfilled = true;
        }

        // Store pending settlement for transfer execution
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

        // Update order state
        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(
            ("OrderSettled", order_id, liquidity_provider),
            settle_percent,
        );

        Ok(true)
    }

    /// # Execute Settlement Transfers (Step 2: Token Transfer)
    ///
    /// ## Description:
    /// Executes the actual token transfers for a previously settled order.
    /// This is the second step in the two-step settlement process.
    ///
    /// ## Authorization:
    /// - `order.temporary_wallet_address`: Must authorize the token transfers
    ///
    /// ## Transfers:
    /// 1. Protocol fee to treasury (if any)
    /// 2. Remaining amount to liquidity provider
    ///
    /// ## Events:
    /// - `("SettlementTransferred", order_id)` with settle_percent
    ///
    /// ## Note:
    /// - Only executes if pending settlement exists
    /// - Clears pending settlement after execution
    /// - Temporary wallet maintains control of funds until this point
    fn execute_settlement_transfer(env: Env, order_id: Bytes) -> Result<(), ContractError> {
        let order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        // Temporary wallet must authorize the transfers
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

        // Transfer protocol fee to treasury
        if pending_settlement.protocol_fee > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &treasury,
                &pending_settlement.protocol_fee,
            );
        }

        // Transfer remaining amount to liquidity provider
        if pending_settlement.transfer_amount > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &pending_settlement.liquidity_provider,
                &pending_settlement.transfer_amount,
            );
        }

        // Clear pending settlement to prevent re-execution
        env.storage()
            .persistent()
            .remove(&DataKey::PendingSettlement(order_id.clone()));

        env.events().publish(
            ("SettlementTransferred", order_id),
            pending_settlement.settle_percent,
        );

        Ok(())
    }

    /// # Initiate Order Refund (Step 1: State Update)
    ///
    /// ## Description:
    /// Marks an order for refund and calculates refund amounts.
    /// This is the first step in the two-step refund process.
    ///
    /// ## Authorization:
    /// - `relayer`: Must authorize the state change
    ///
    /// ## Validation:
    /// - Order must exist and not be fulfilled/refunded
    /// - Fee must not exceed accumulated protocol fee
    ///
    /// ## State Changes:
    /// - Marks order as refunded
    /// - Zeros out order amounts
    /// - Stores pending refund for transfer execution
    ///
    /// ## Events:
    /// - `("OrderRefunded", order_id)` with fee amount
    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError> {
        let settings_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::SettingsContract)
            .unwrap();
        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);

        if settings_client.is_paused() {
            return Err(ContractError::Paused);
        }

        let relayer: Address = settings_client.get_relayer_address();
        relayer.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id.clone()))
            .ok_or(ContractError::OrderNotFound)?;

        // Prevent refund of fulfilled or already refunded orders
        if order.is_fulfilled {
            return Err(ContractError::OrderFulfilled);
        }

        if order.is_refunded {
            return Err(ContractError::OrderRefunded);
        }

        // Validate refund fee doesn't exceed protocol fee
        if fee > order.protocol_fee {
            return Err(ContractError::FeeExceedsProtocolFee);
        }

        // Calculate and store pending refund
        let pending_refund = PendingRefund {
            order_id: order_id.clone(),
            fee,
            refund_amount: (order.amount) - fee,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PendingRefund(order_id.clone()), &pending_refund);

        // Update order state to refunded
        order.is_refunded = true;
        order.current_bps = 0;
        order.amount = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id.clone()), &order);

        env.events().publish(("OrderRefunded", order_id), fee);

        Ok(())
    }

    /// # Execute Refund Transfers (Step 2: Token Transfer)
    ///
    /// ## Description:
    /// Executes the actual token transfers for a previously refunded order.
    /// This is the second step in the two-step refund process.
    ///
    /// ## Authorization:
    /// - `order.temporary_wallet_address`: Must authorize the token transfers
    ///
    /// ## Transfers:
    /// 1. Protocol fee to treasury (if any)
    /// 2. Remaining amount to refund address
    ///
    /// ## Note:
    /// - Only executes if pending refund exists
    /// - Clears pending refund after execution
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

        // Transfer protocol fee to treasury
        if pending_refund.fee > 0 {
            token_client.transfer(
                &order.temporary_wallet_address,
                &treasury,
                &pending_refund.fee,
            );
        }

        // Transfer remaining amount to refund address
        token_client.transfer(
            &order.temporary_wallet_address,
            &order.refund_address,
            &pending_refund.refund_amount,
        );

        // Clear pending refund to prevent re-execution
        env.storage()
            .persistent()
            .remove(&DataKey::PendingRefund(order_id.clone()));

        env.events().publish(
            ("RefundTransferred", order_id),
            pending_refund.refund_amount,
        );

        Ok(())
    }

    /// # Get token balance for a user
    ///
    /// ## Returns:
    /// - USDC balance of the specified user
    fn get_token_balance(env: Env, user: Address) -> i128 {
        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.balance(&user)
    }

    /// # Get order ID (validation function)
    ///
    /// ## Returns:
    /// - Order ID if order exists
    /// - Error if order not found
    fn get_order_id(env: Env, order_id: Bytes) -> Result<Bytes, ContractError> {
        let order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(ContractError::OrderNotFound)?;

        Ok(order.order_id)
    }

    /// # Get complete order information
    ///
    /// ## Returns:
    /// - Full Order struct if order exists
    /// - Error if order not found
    fn get_order_info(env: Env, order_id: Bytes) -> Result<Order, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(ContractError::OrderNotFound)
    }

    /// # Get current fee details from settings
    ///
    /// ## Returns:
    /// - Tuple of (protocol_fee_percent, max_bps)
    /// - Current values: (1000, 100000) = 1% fee
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
    /// # Initialize the LP Contract
    ///
    /// ## Description:
    /// Sets up the contract with required addresses and configuration.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the initialization
    ///
    /// ## Parameters:
    /// - `usdc_asset`: Address of the USDC token contract
    /// - `settings_contract`: Address of the settings manager contract
    pub fn init(env: Env, admin: Address, usdc_asset: Address, settings_contract: Address) {
        let storage = env.storage().persistent();

        if storage.get::<DataKey, Address>(&DataKey::Admin).is_some() {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        storage.set(&DataKey::Admin, &admin);

        storage.set(&DataKey::Usdc, &usdc_asset);
        storage.set(&DataKey::SettingsContract, &settings_contract);
    }

    /// # Register a new Liquidity Provider Node
    ///
    /// ## Description:
    /// Registers an LP node with specified capacity for order fulfillment.
    ///
    /// ## Validation:
    /// - Capacity must be positive
    /// - LP node ID must not already exist
    ///
    /// ## Events:
    /// - `("LpNodeRegistered", lp_node_id)` with capacity
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

    /// # Upgrade Contract WASM
    ///
    /// ## Description:
    /// Updates the contract's WASM code for upgrades and bug fixes.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the upgrade
    ///
    /// ## Note:
    /// - Only admin can upgrade the contract
    /// - Ensures contract upgradeability while maintaining state
    pub fn upgrade_lp(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
