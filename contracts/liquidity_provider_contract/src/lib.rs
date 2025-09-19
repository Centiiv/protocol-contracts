#![no_std]
mod error;
pub mod liquidity_provider;
pub mod liquidity_provider_trait;
mod storage_types;
mod test;

//use crate::{
//    error::ContractError,
//    liquidity_provider_trait::IGateway,
//    storage_types::{DataKey, LpNode, Order, OrderParams},
//};
//use liquidity_manager::{self, liquidity_manager::LPSettingManagerContractClient};
//use soroban_sdk::{contract, contractimpl, log, token, Address, Bytes, Env, Map};
//
//#[contract]
//pub struct LPContract;
//
//#[contractimpl]
//impl IGateway for LPContract {
//    fn create_order(env: Env, params: OrderParams) -> Result<(), ContractError> {
//        params.sender.require_auth();
//
//        let settings_contract: Address = env
//            .storage()
//            .persistent()
//            .get(&DataKey::SettingsContract)
//            .unwrap();
//
//        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//
//        if settings_client.is_paused() {
//            return Err(ContractError::Paused);
//        }
//
//        if params.amount <= 0 {
//            return Err(ContractError::InvalidAmount);
//        }
//
//        if params.message_hash.is_empty() {
//            return Err(ContractError::InvalidMessageHash);
//        }
//
//        let order_exists: Option<Order> = env
//            .storage()
//            .persistent()
//            .get(&DataKey::Order(params.order_id.clone()));
//
//        if order_exists.is_some() {
//            return Err(ContractError::OrderAlreadyExists);
//        }
//
//        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//        let token_client = token::Client::new(&env, &usdc_asset);
//        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//        let protocol_fee = (params.amount * protocol_fee_percent as i128) / max_bps as i128;
//
//        // Transfer tokens to THIS CONTRACT (gateway) instead of wallet contract
//        token_client.transfer(
//            &params.sender,
//            &env.current_contract_address(), // Gateway itself becomes the token holder
//            &(params.amount + params.sender_fee),
//        );
//
//        let order = Order {
//            order_id: params.order_id.clone(),
//            sender: params.sender.clone(),
//            token: usdc_asset,
//            amount: params.amount,
//            sender_fee_recipient: params.sender_fee_recipient,
//            sender_fee: params.sender_fee,
//            protocol_fee,
//            is_fulfilled: false,
//            is_refunded: false,
//            refund_address: params.refund_address.clone(),
//            current_bps: max_bps as i128,
//            rate: params.rate,
//            message_hash: params.message_hash.clone(),
//        };
//
//        env.storage()
//            .persistent()
//            .set(&DataKey::Order(params.order_id.clone()), &order);
//
//        let mut nonces: Map<Address, i128> = env
//            .storage()
//            .persistent()
//            .get(&DataKey::Nonces)
//            .unwrap_or(Map::new(&env));
//
//        nonces.set(
//            params.sender.clone(),
//            nonces.get(params.sender.clone()).unwrap_or(0) + 1,
//        );
//        env.storage().persistent().set(&DataKey::Nonces, &nonces);
//
//        env.events().publish(
//            ("OrderCreated", params.order_id, params.sender),
//            (
//                params.refund_address,
//                params.amount,
//                protocol_fee,
//                params.rate,
//                params.message_hash,
//            ),
//        );
//        Ok(())
//    }
//
//    fn settle(
//        env: Env,
//        split_order_id: Bytes,
//        order_id: Bytes,
//        liquidity_provider: Address,
//        settle_percent: i128,
//    ) -> Result<bool, ContractError> {
//        log!(&env, "=== START SETTLE FUNCTION ===");
//
//        let settings_contract: Address = env
//            .storage()
//            .persistent()
//            .get(&DataKey::SettingsContract)
//            .unwrap();
//
//        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//        let aggregator: Address = settings_client.get_aggregator_address();
//        aggregator.require_auth();
//
//        if settle_percent <= 0 || settle_percent > 100_000 {
//            return Err(ContractError::InvalidSettlePercent);
//        }
//
//        let order_option: Option<Order> = env
//            .storage()
//            .persistent()
//            .get(&DataKey::Order(order_id.clone()));
//
//        if order_option.is_none() {
//            return Err(ContractError::OrderNotFound);
//        }
//
//        let mut order: Order = order_option.unwrap();
//
//        if order.is_fulfilled {
//            return Err(ContractError::OrderFulfilled);
//        }
//
//        if order.is_refunded {
//            return Err(ContractError::OrderRefunded);
//        }
//
//        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//        let treasury: Address = settings_client.get_treasury_address();
//        let token_client = token::Client::new(&env, &usdc_asset);
//
//        let current_order_bps = order.current_bps;
//        order.current_bps -= settle_percent;
//
//        let liquidity_provider_amount = (order.amount * settle_percent) / current_order_bps;
//        order.amount -= liquidity_provider_amount;
//
//        let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//        let protocol_fee =
//            (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);
//        let transfer_amount = liquidity_provider_amount - protocol_fee;
//
//        // Check if the order is fully settled
//        if order.current_bps == 0 {
//            order.is_fulfilled = true;
//
//            // Transfer sender fee from THIS CONTRACT (gateway)
//            if order.sender_fee > 0 {
//                token_client.transfer(
//                    &env.current_contract_address(), // From gateway itself
//                    &order.sender_fee_recipient,
//                    &order.sender_fee,
//                );
//            }
//        }
//
//        // Transfer protocol fee from THIS CONTRACT (gateway)
//        if protocol_fee > 0 {
//            token_client.transfer(
//                &env.current_contract_address(), // From gateway itself
//                &treasury,
//                &protocol_fee,
//            );
//        }
//
//        // Transfer to liquidity provider from THIS CONTRACT (gateway)
//        if transfer_amount > 0 {
//            token_client.transfer(
//                &env.current_contract_address(), // From gateway itself
//                &liquidity_provider,
//                &transfer_amount,
//            );
//        }
//
//        // Save the updated order
//        env.storage()
//            .persistent()
//            .set(&DataKey::Order(order_id.clone()), &order);
//
//        env.events().publish(
//            ("OrderSettled", split_order_id, order_id, liquidity_provider),
//            settle_percent,
//        );
//
//        Ok(true)
//    }
//    //fn create_order(env: Env, params: OrderParams) -> Result<(), ContractError> {
//    //    params.sender.require_auth();
//    //
//    //    let settings_contract: Address = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::SettingsContract)
//    //        .unwrap();
//    //
//    //    let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//    //
//    //    if settings_client.is_paused() {
//    //        return Err(ContractError::Paused);
//    //    }
//    //
//    //    if params.amount <= 0 {
//    //        return Err(ContractError::InvalidAmount);
//    //    }
//    //
//    //    if params.message_hash.is_empty() {
//    //        return Err(ContractError::InvalidMessageHash);
//    //    }
//    //
//    //    let order_exists: Option<Order> = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::Order(params.order_id.clone()));
//    //
//    //    if order_exists.is_some() {
//    //        return Err(ContractError::OrderAlreadyExists);
//    //    }
//    //
//    //    let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//    //    let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
//    //    let token_client = token::Client::new(&env, &usdc_asset);
//    //    let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//    //    let protocol_fee = (params.amount * protocol_fee_percent as i128) / max_bps as i128;
//    //
//    //    token_client.transfer(
//    //        &params.sender,
//    //        &wallet_contract,
//    //        &(params.amount + params.sender_fee),
//    //    );
//    //
//    //    let order = Order {
//    //        order_id: params.order_id.clone(),
//    //        sender: params.sender.clone(),
//    //        token: usdc_asset,
//    //        amount: params.amount,
//    //        sender_fee_recipient: params.sender_fee_recipient,
//    //        sender_fee: params.sender_fee,
//    //        protocol_fee,
//    //        is_fulfilled: false,
//    //        is_refunded: false,
//    //        refund_address: params.refund_address.clone(),
//    //        current_bps: max_bps as i128,
//    //        rate: params.rate,                         // Add this
//    //        message_hash: params.message_hash.clone(), // Add this
//    //    };
//    //
//    //    env.storage()
//    //        .persistent()
//    //        .set(&DataKey::Order(params.order_id.clone()), &order);
//    //
//    //    let mut nonces: Map<Address, i128> = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::Nonces)
//    //        .unwrap_or(Map::new(&env));
//    //
//    //    nonces.set(
//    //        params.sender.clone(),
//    //        nonces.get(params.sender.clone()).unwrap_or(0) + 1,
//    //    );
//    //    env.storage().persistent().set(&DataKey::Nonces, &nonces);
//    //
//    //    env.events().publish(
//    //        ("OrderCreated", params.order_id, params.sender),
//    //        (
//    //            params.refund_address,
//    //            params.amount,
//    //            protocol_fee,
//    //            params.rate,
//    //            params.message_hash,
//    //        ),
//    //    );
//    //    Ok(())
//    //}
//    //fn settle(
//    //    env: Env,
//    //    split_order_id: Bytes,
//    //    order_id: Bytes,
//    //    liquidity_provider: Address,
//    //    settle_percent: i128,
//    //) -> Result<bool, ContractError> {
//    //    log!(&env, "=== START SETTLE FUNCTION ===");
//    //    log!(&env, "split_order_id:", split_order_id.clone());
//    //    log!(&env, "order_id:", order_id.clone());
//    //    log!(&env, "liquidity_provider:", liquidity_provider.clone());
//    //    log!(&env, "settle_percent:", settle_percent);
//    //
//    //    // Get settings contract
//    //    log!(&env, "Getting settings contract...");
//    //    let settings_contract: Address =
//    //        match env.storage().persistent().get(&DataKey::SettingsContract) {
//    //            Some(addr) => {
//    //                log!(&env, "Settings contract found:", addr);
//    //                addr
//    //            }
//    //            None => {
//    //                log!(&env, "ERROR: Settings contract not found!");
//    //                return Err(ContractError::SettingsContractNotSet);
//    //            }
//    //        };
//    //
//    //    log!(&env, "Creating settings client...");
//    //    let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//    //
//    //    log!(&env, "Getting aggregator address...");
//    //    let aggregator: Address = settings_client.get_aggregator_address();
//    //    log!(&env, "Aggregator address:", aggregator);
//    //
//    //    log!(&env, "Requiring auth for aggregator...");
//    //    //aggregator.require_auth();
//    //    log!(&env, "Auth passed for aggregator");
//    //
//    //    // Validate settle percent
//    //    log!(&env, "Validating settle_percent:", settle_percent);
//    //    if settle_percent <= 0 || settle_percent > 100_000 {
//    //        log!(&env, "ERROR: Invalid settle percent");
//    //        return Err(ContractError::InvalidSettlePercent);
//    //    }
//    //    log!(&env, "Settle percent validation passed");
//    //
//    //    // Retrieve order
//    //    log!(&env, "Retrieving order from storage...");
//    //    let order_option: Option<Order> = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::Order(order_id.clone()));
//    //
//    //    if order_option.is_none() {
//    //        log!(&env, "ERROR: Order not found for ID:", order_id);
//    //        return Err(ContractError::OrderNotFound);
//    //    }
//    //
//    //    let mut order: Order = order_option.unwrap();
//    //    log!(&env, "Order retrieved successfully");
//    //    log!(
//    //        &env,
//    //        "Order details - amount:",
//    //        order.amount,
//    //        "current_bps:",
//    //        order.current_bps,
//    //        "is_fulfilled:",
//    //        order.is_fulfilled,
//    //        "is_refunded:",
//    //        order.is_refunded
//    //    );
//    //
//    //    // Check order status
//    //    if order.is_fulfilled {
//    //        log!(&env, "ERROR: Order already fulfilled");
//    //        return Err(ContractError::OrderFulfilled);
//    //    }
//    //
//    //    if order.is_refunded {
//    //        log!(&env, "ERROR: Order already refunded");
//    //        return Err(ContractError::OrderRefunded);
//    //    }
//    //    log!(&env, "Order status validation passed");
//    //
//    //    // Get USDC asset
//    //    log!(&env, "Getting USDC asset...");
//    //    let usdc_asset: Address = match env.storage().persistent().get(&DataKey::Usdc) {
//    //        Some(addr) => {
//    //            log!(&env, "USDC asset found:", addr);
//    //            addr
//    //        }
//    //        None => {
//    //            log!(&env, "ERROR: USDC asset not found!");
//    //            return Err(ContractError::UsdcNotSet);
//    //        }
//    //    };
//    //
//    //    // Get wallet contract
//    //    log!(&env, "Getting wallet contract...");
//    //    let wallet_contract: Address = match env.storage().persistent().get(&DataKey::Wallet) {
//    //        Some(addr) => {
//    //            log!(&env, "Wallet contract found:", addr);
//    //            addr
//    //        }
//    //        None => {
//    //            log!(&env, "ERROR: Wallet contract not found!");
//    //            return Err(ContractError::WalletNotSet);
//    //        }
//    //    };
//    //
//    //    // Get treasury address
//    //    log!(&env, "Getting treasury address...");
//    //    let treasury: Address = settings_client.get_treasury_address();
//    //    log!(&env, "Treasury address:", treasury);
//    //
//    //    // Create token client
//    //    log!(&env, "Creating token client...");
//    //    let token_client = token::Client::new(&env, &usdc_asset);
//    //    log!(&env, "Token client created");
//    //
//    //    // Store the current BPS before modifying it
//    //    let current_order_bps = order.current_bps;
//    //    log!(
//    //        &env,
//    //        "Current BPS:",
//    //        current_order_bps,
//    //        "Settle percent:",
//    //        settle_percent
//    //    );
//    //
//    //    // Reduce the remaining BPS by the settle percentage
//    //    order.current_bps -= settle_percent;
//    //    log!(&env, "New current_bps:", order.current_bps);
//    //
//    //    // Calculate liquidity provider amount
//    //    log!(&env, "Calculating liquidity provider amount...");
//    //    let liquidity_provider_amount = (order.amount * settle_percent) / current_order_bps;
//    //    log!(
//    //        &env,
//    //        "Liquidity provider amount:",
//    //        liquidity_provider_amount
//    //    );
//    //
//    //    // Reduce the remaining order amount
//    //    order.amount -= liquidity_provider_amount;
//    //    log!(&env, "New order amount:", order.amount);
//    //
//    //    // Calculate protocol fee
//    //    log!(&env, "Getting fee details...");
//    //    let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//    //    log!(
//    //        &env,
//    //        "Fee details - percent:",
//    //        protocol_fee_percent,
//    //        "max_bps:",
//    //        max_bps
//    //    );
//    //
//    //    let protocol_fee =
//    //        (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);
//    //    let transfer_amount = liquidity_provider_amount - protocol_fee;
//    //
//    //    log!(
//    //        &env,
//    //        "Protocol fee:",
//    //        protocol_fee,
//    //        "Transfer amount:",
//    //        transfer_amount
//    //    );
//    //
//    //    // Check if the order is fully settled
//    //    if order.current_bps == 0 {
//    //        log!(&env, "Order fully fulfilled - setting is_fulfilled to true");
//    //        order.is_fulfilled = true;
//    //        if order.sender_fee > 0 {
//    //            log!(
//    //                &env,
//    //                "Transferring sender fee:",
//    //                order.sender_fee,
//    //                "to:",
//    //                order.sender_fee_recipient.clone()
//    //            );
//    //            // TODO: Fix wallet contract authorization for this transfer
//    //            // token_client.transfer(
//    //            //     &wallet_contract,
//    //            //     &order.sender_fee_recipient,
//    //            //     &order.sender_fee,
//    //            // );
//    //            log!(&env, "Sender fee transferred successfully");
//    //        } else {
//    //            log!(&env, "No sender fee to transfer");
//    //        }
//    //        // Transfer sender fee only when the order is completely fulfilled
//    //        //if order.sender_fee > 0 {
//    //        //    log!(
//    //        //        &env,
//    //        //        "Transferring sender fee:",
//    //        //        order.sender_fee,
//    //        //        "to:",
//    //        //        order.sender_fee_recipient.clone()
//    //        //    );
//    //        //    token_client.transfer(
//    //        //        &wallet_contract,
//    //        //        &order.sender_fee_recipient,
//    //        //        &order.sender_fee,
//    //        //    );
//    //        //    log!(&env, "Sender fee transferred successfully");
//    //        //} else {
//    //        //    log!(&env, "No sender fee to transfer");
//    //        //}
//    //    } else {
//    //        log!(
//    //            &env,
//    //            "Order partially fulfilled - remaining BPS:",
//    //            order.current_bps
//    //        );
//    //    }
//    //    // Transfer protocol fee to treasury
//    //    if protocol_fee > 0 {
//    //        log!(
//    //            &env,
//    //            "Transferring protocol fee to treasury:",
//    //            protocol_fee,
//    //            "to:",
//    //            treasury.clone()
//    //        );
//    //        // TODO: Fix wallet contract authorization for this transfer
//    //        // token_client.transfer(&wallet_contract, &treasury, &protocol_fee);
//    //        log!(&env, "Protocol fee transferred successfully");
//    //    } else {
//    //        log!(&env, "No protocol fee to transfer (zero amount)");
//    //    }
//    //
//    //    // Transfer remaining amount to liquidity provider
//    //    if transfer_amount > 0 {
//    //        log!(
//    //            &env,
//    //            "Transferring to liquidity provider:",
//    //            transfer_amount,
//    //            "to:",
//    //            liquidity_provider.clone()
//    //        );
//    //        // TODO: Fix wallet contract authorization for this transfer
//    //        // token_client.transfer(&wallet_contract, &liquidity_provider, &transfer_amount);
//    //        log!(&env, "Liquidity provider amount transferred successfully");
//    //    } else {
//    //        log!(
//    //            &env,
//    //            "No transfer amount to liquidity provider (zero amount)"
//    //        );
//    //    }
//    //    // Transfer protocol fee to treasury
//    //    //if protocol_fee > 0 {
//    //    //    log!(
//    //    //        &env,
//    //    //        "Transferring protocol fee to treasury:",
//    //    //        protocol_fee,
//    //    //        "to:",
//    //    //        treasury.clone()
//    //    //    );
//    //    //    token_client.transfer(&wallet_contract, &treasury, &protocol_fee);
//    //    //    log!(&env, "Protocol fee transferred successfully");
//    //    //} else {
//    //    //    log!(&env, "No protocol fee to transfer (zero amount)");
//    //    //}
//    //    //
//    //    //// Transfer remaining amount to liquidity provider
//    //    //if transfer_amount > 0 {
//    //    //    log!(
//    //    //        &env,
//    //    //        "Transferring to liquidity provider:",
//    //    //        transfer_amount,
//    //    //        "to:",
//    //    //        liquidity_provider.clone()
//    //    //    );
//    //    //    token_client.transfer(&wallet_contract, &liquidity_provider, &transfer_amount);
//    //    //    log!(&env, "Liquidity provider amount transferred successfully");
//    //    //} else {
//    //    //    log!(
//    //    //        &env,
//    //    //        "No transfer amount to liquidity provider (zero amount)"
//    //    //    );
//    //    //}
//    //    //
//    //    // Save the updated order
//    //    log!(&env, "Saving updated order to storage...");
//    //    env.storage()
//    //        .persistent()
//    //        .set(&DataKey::Order(order_id.clone()), &order);
//    //    log!(&env, "Order saved successfully");
//    //
//    //    // Publish event
//    //    log!(&env, "Publishing settlement event...");
//    //    env.events().publish(
//    //        ("OrderSettled", split_order_id, order_id, liquidity_provider),
//    //        settle_percent,
//    //    );
//    //    log!(&env, "Event published");
//    //
//    //    log!(&env, "=== SETTLE FUNCTION COMPLETED SUCCESSFULLY ===");
//    //    Ok(true)
//    //}
//    //fn settle(
//    //    env: Env,
//    //    split_order_id: Bytes,
//    //    order_id: Bytes,
//    //    liquidity_provider: Address,
//    //    settle_percent: i128,
//    //) -> Result<bool, ContractError> {
//    //    log!(&env, "Starting settle function");
//    //
//    //    let settings_contract: Address = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::SettingsContract)
//    //        .unwrap();
//    //
//    //    let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//    //    let aggregator: Address = settings_client.get_aggregator_address();
//    //    aggregator.require_auth();
//    //
//    //    log!(&env, "Auth passed for aggregator:", aggregator);
//    //
//    //    if settle_percent <= 0 || settle_percent > 100_000 {
//    //        log!(&env, "Invalid settle percent:", settle_percent);
//    //        return Err(ContractError::InvalidSettlePercent);
//    //    }
//    //
//    //    let order_option: Option<Order> = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::Order(order_id.clone()));
//    //
//    //    if order_option.is_none() {
//    //        log!(&env, "Order not found for ID:", order_id);
//    //        return Err(ContractError::OrderNotFound);
//    //    }
//    //
//    //    let mut order: Order = order_option.unwrap();
//    //    log!(
//    //        &env,
//    //        "Order retrieved - amount:",
//    //        order.amount,
//    //        "current_bps:",
//    //        order.current_bps
//    //    );
//    //
//    //    if order.is_fulfilled {
//    //        log!(&env, "Order already fulfilled");
//    //        return Err(ContractError::OrderFulfilled);
//    //    }
//    //
//    //    if order.is_refunded {
//    //        log!(&env, "Order already refunded");
//    //        return Err(ContractError::OrderRefunded);
//    //    }
//    //
//    //    let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//    //    let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
//    //    let treasury: Address = settings_client.get_treasury_address();
//    //    let token_client = token::Client::new(&env, &usdc_asset);
//    //
//    //    // Store the current BPS before modifying it
//    //    let current_order_bps = order.current_bps;
//    //    log!(
//    //        &env,
//    //        "Current BPS:",
//    //        current_order_bps,
//    //        "Settle percent:",
//    //        settle_percent
//    //    );
//    //
//    //    // Reduce the remaining BPS by the settle percentage
//    //    order.current_bps -= settle_percent;
//    //
//    //    // Calculate liquidity provider amount
//    //    let liquidity_provider_amount = (order.amount * settle_percent) / current_order_bps;
//    //    log!(
//    //        &env,
//    //        "Liquidity provider amount:",
//    //        liquidity_provider_amount
//    //    );
//    //
//    //    // Reduce the remaining order amount
//    //    order.amount -= liquidity_provider_amount;
//    //
//    //    // Calculate protocol fee
//    //    let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//    //    log!(
//    //        &env,
//    //        "Fee details - percent:",
//    //        protocol_fee_percent,
//    //        "max_bps:",
//    //        max_bps
//    //    );
//    //
//    //    let protocol_fee =
//    //        (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);
//    //    let transfer_amount = liquidity_provider_amount - protocol_fee;
//    //
//    //    log!(
//    //        &env,
//    //        "Protocol fee:",
//    //        protocol_fee,
//    //        "Transfer amount:",
//    //        transfer_amount
//    //    );
//    //
//    //    // Check if the order is fully settled
//    //    if order.current_bps == 0 {
//    //        order.is_fulfilled = true;
//    //        log!(&env, "Order fully fulfilled");
//    //
//    //        // Transfer sender fee only when the order is completely fulfilled
//    //        if order.sender_fee > 0 {
//    //            log!(&env, "Transferring sender fee:", order.sender_fee);
//    //            token_client.transfer(
//    //                &wallet_contract,
//    //                &order.sender_fee_recipient,
//    //                &order.sender_fee,
//    //            );
//    //        }
//    //    }
//    //
//    //    // Transfer protocol fee to treasury
//    //    if protocol_fee > 0 {
//    //        log!(&env, "Transferring protocol fee to treasury:", protocol_fee);
//    //        token_client.transfer(&wallet_contract, &treasury, &protocol_fee);
//    //    }
//    //
//    //    // Transfer remaining amount to liquidity provider
//    //    if transfer_amount > 0 {
//    //        log!(&env, "Transferring to liquidity provider:", transfer_amount);
//    //        token_client.transfer(&wallet_contract, &liquidity_provider, &transfer_amount);
//    //    }
//    //
//    //    // Save the updated order
//    //    env.storage()
//    //        .persistent()
//    //        .set(&DataKey::Order(order_id.clone()), &order);
//    //
//    //    log!(&env, "Order updated and saved");
//    //
//    //    env.events().publish(
//    //        ("OrderSettled", split_order_id, order_id, liquidity_provider),
//    //        settle_percent,
//    //    );
//    //
//    //    log!(&env, "Settle function completed successfully");
//    //    Ok(true)
//    //}
//    //fn settle(
//    //    env: Env,
//    //    split_order_id: Bytes,
//    //    order_id: Bytes,
//    //    liquidity_provider: Address,
//    //    settle_percent: i128,
//    //) -> Result<bool, ContractError> {
//    //    let settings_contract: Address = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::SettingsContract)
//    //        .unwrap();
//    //
//    //    let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//    //    let aggregator: Address = settings_client.get_aggregator_address();
//    //    aggregator.require_auth();
//    //
//    //    if settle_percent <= 0 || settle_percent > 100_000 {
//    //        return Err(ContractError::InvalidSettlePercent);
//    //    }
//    //
//    //    let order_option: Option<Order> = env
//    //        .storage()
//    //        .persistent()
//    //        .get(&DataKey::Order(order_id.clone()));
//    //
//    //    log!(&env, "Order found:", order_option.is_some());
//    //
//    //    if order_option.is_none() {
//    //        log!(&env, "Order not found for ID:", order_id);
//    //        return Err(ContractError::OrderNotFound);
//    //    }
//    //
//    //    let mut order: Order = order_option.unwrap();
//    //    log!(&env, "Order retrieved successfully");
//    //    //let order_option: Option<Order> = env
//    //    //    .storage()
//    //    //    .persistent()
//    //    //    .get(&DataKey::Order(order_id.clone()));
//    //    //
//    //    //if order_option.is_none() {
//    //    //    log!(&env, "Order not found for ID:", order_id);
//    //    //    return Err(ContractError::OrderNotFound);
//    //    //}
//    //    //
//    //    //let mut order: Order = order_option.unwrap();
//    //
//    //    if order.is_fulfilled {
//    //        return Err(ContractError::OrderFulfilled);
//    //    }
//    //
//    //    if order.is_refunded {
//    //        return Err(ContractError::OrderRefunded);
//    //    }
//    //
//    //    let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//    //
//    //    let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
//    //
//    //    let treasury: Address = settings_client.get_treasury_address();
//    //
//    //    let token_client = token::Client::new(&env, &usdc_asset);
//    //
//    //    // Store the current BPS before modifying it (this is the total remaining to be settled)
//    //    let current_order_bps = order.current_bps;
//    //
//    //    // Reduce the remaining BPS by the settle percentage
//    //    order.current_bps -= settle_percent;
//    //
//    //    // Calculate liquidity provider amount: this is the portion of the current order amount
//    //    // that corresponds to the settle_percent relative to the current_order_bps
//    //    let liquidity_provider_amount = (order.amount * settle_percent) / (current_order_bps);
//    //
//    //    // Reduce the remaining order amount
//    //    order.amount -= liquidity_provider_amount;
//    //
//    //    // Calculate protocol fee based on the liquidity provider amount
//    //    let (protocol_fee_percent, max_bps) = settings_client.get_fee_details();
//    //
//    //    let protocol_fee =
//    //        (liquidity_provider_amount * protocol_fee_percent as i128) / (max_bps as i128);
//    //
//    //    let transfer_amount = liquidity_provider_amount - protocol_fee;
//    //
//    //    // Check if the order is fully settled
//    //    if order.current_bps == 0 {
//    //        order.is_fulfilled = true;
//    //
//    //        // Transfer sender fee only when the order is completely fulfilled
//    //        if order.sender_fee > 0 {
//    //            token_client.transfer(
//    //                &wallet_contract,
//    //                &order.sender_fee_recipient,
//    //                &order.sender_fee,
//    //            );
//    //
//    //            env.events().publish(
//    //                ("SenderFeeTransferred", order.sender_fee_recipient.clone()),
//    //                order.sender_fee,
//    //            );
//    //        }
//    //    }
//    //
//    //    // Transfer protocol fee to treasury
//    //    if protocol_fee > 0 {
//    //        token_client.transfer(&wallet_contract, &treasury, &protocol_fee);
//    //    }
//    //    // Transfer remaining amount to liquidity provider
//    //    if transfer_amount > 0 {
//    //        token_client.transfer(&wallet_contract, &liquidity_provider, &transfer_amount);
//    //    }
//    //    // Save the updated order
//    //    env.storage()
//    //        .persistent()
//    //        .set(&DataKey::Order(order_id.clone()), &order);
//    //
//    //    env.events().publish(
//    //        ("OrderSettled", split_order_id, order_id, liquidity_provider),
//    //        settle_percent,
//    //    );
//    //
//    //    Ok(true)
//    //}
//
//    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError> {
//        let settings_contract: Address = env
//            .storage()
//            .persistent()
//            .get(&DataKey::SettingsContract)
//            .unwrap();
//        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//        let aggregator: Address = settings_client.get_aggregator_address();
//        aggregator.require_auth();
//
//        let mut order: Order = env
//            .storage()
//            .persistent()
//            .get(&DataKey::Order(order_id.clone()))
//            .ok_or(ContractError::OrderNotFound)?;
//        if order.is_fulfilled {
//            return Err(ContractError::OrderFulfilled);
//        }
//        if order.is_refunded {
//            return Err(ContractError::OrderRefunded);
//        }
//        if fee > order.protocol_fee {
//            return Err(ContractError::FeeExceedsProtocolFee);
//        }
//
//        let usdc_asset: Address = env.storage().persistent().get(&DataKey::Usdc).unwrap();
//        let wallet_contract: Address = env.storage().persistent().get(&DataKey::Wallet).unwrap();
//        let treasury: Address = settings_client.get_treasury_address();
//        let token_client = token::Client::new(&env, &usdc_asset);
//
//        if fee > 0 {
//            token_client.transfer(&wallet_contract, &treasury, &fee);
//        }
//        let refund_amount = order.amount + order.sender_fee - fee;
//        token_client.transfer(&wallet_contract, &order.refund_address, &refund_amount);
//
//        order.is_refunded = true;
//        order.current_bps = 0;
//        env.storage()
//            .persistent()
//            .set(&DataKey::Order(order_id.clone()), &order);
//
//        env.events().publish(("OrderRefunded", order_id), fee);
//        Ok(())
//    }
//
//    fn get_order_id(env: Env, order_id: Bytes) -> Result<Bytes, ContractError> {
//        let order: Order = env
//            .storage()
//            .persistent()
//            .get(&DataKey::Order(order_id))
//            .ok_or(ContractError::OrderNotFound)?;
//
//        Ok(order.order_id)
//    }
//
//    fn get_order_info(env: Env, order_id: Bytes) -> Result<Order, ContractError> {
//        env.storage()
//            .persistent()
//            .get(&DataKey::Order(order_id))
//            .ok_or(ContractError::OrderNotFound)
//    }
//
//    fn get_fee_details(env: Env) -> (i64, i64) {
//        let settings_contract: Address = env
//            .storage()
//            .persistent()
//            .get(&DataKey::SettingsContract)
//            .unwrap();
//        let settings_client = LPSettingManagerContractClient::new(&env, &settings_contract);
//        settings_client.get_fee_details()
//    }
//}
//
//#[contractimpl]
//impl LPContract {
//    pub fn initialize(env: Env, admin: Address, usdc_asset: Address, settings_contract: Address) {
//        admin.require_auth();
//
//        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
//        env.storage()
//            .persistent()
//            .set(&DataKey::SettingsContract, &settings_contract);
//
//        // AUTHORIZE THE GATEWAY CONTRACT TO RECEIVE AND HOLD TOKENS
//        let token_client = token::Client::new(&env, &usdc_asset);
//
//        // Use a reasonable expiration that doesn't exceed the maximum
//        // The max is around 6,312,000 ledgers (about 100 days)
//        let current_ledger = env.ledger().sequence();
//        let expiration = current_ledger + 1000000; // ~16 days expiration
//
//        // Authorize this contract to receive tokens from itself (for holding)
//        token_client.approve(
//            &env.current_contract_address(), // from: this contract
//            &env.current_contract_address(), // to: this contract
//            &i128::MAX,                      // unlimited allowance
//            &expiration,                     // reasonable expiration
//        );
//
//        log!(
//            &env,
//            "Gateway contract initialized and authorized for token operations"
//        );
//    }
//    //pub fn initialize(env: Env, admin: Address, usdc_asset: Address, settings_contract: Address) {
//    //    admin.require_auth();
//    //    env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
//    //    env.storage()
//    //        .persistent()
//    //        .set(&DataKey::SettingsContract, &settings_contract);
//    //}
//    pub fn register_lp_node(
//        env: Env,
//        lp_node_id: Bytes,
//        capacity: i128,
//    ) -> Result<(), ContractError> {
//        if capacity <= 0 {
//            return Err(ContractError::InvalidLpNodeParameters);
//        }
//
//        let lp_node = LpNode { capacity };
//        let mut node_ids: Map<Bytes, bool> = env
//            .storage()
//            .persistent()
//            .get(&DataKey::NodeIDs)
//            .unwrap_or(Map::new(&env));
//
//        if node_ids.contains_key(lp_node_id.clone()) {
//            return Err(ContractError::LpNodeIdAlreadyExists);
//        }
//
//        env.storage().persistent().set(&lp_node_id, &lp_node);
//        node_ids.set(lp_node_id.clone(), true);
//        env.storage().persistent().set(&DataKey::NodeIDs, &node_ids);
//
//        env.events()
//            .publish(("LpNodeRegistered", lp_node_id), capacity);
//
//        Ok(())
//    }
//}
