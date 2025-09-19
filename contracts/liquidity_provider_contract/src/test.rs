use crate::liquidity_provider::{LPContract, LPContractClient};
#[cfg(test)]
use crate::storage_types::{Order, OrderParams};
use liquidity_manager::{
    liquidity_manager::{LPSettingManagerContract, LPSettingManagerContractClient},
    storage::ProtocolAddressType,
};
use soroban_sdk::{log, testutils::Address as _, token, Address, Bytes, Env, IntoVal, String};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, token::StellarAssetClient<'a>) {
    let contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    (
        contract_id.address(),
        token::StellarAssetClient::new(env, &contract_id.address()),
    )
}

fn setup<'a>() -> (
    Env,
    LPContractClient<'a>,
    LPSettingManagerContractClient<'a>,
    Address,
    token::StellarAssetClient<'a>,
    (Address, Address, Address, Address, Address),
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (usdc_asset, token_client) = create_token_contract(&env, &admin);
    let treasury = Address::generate(&env);
    let aggregator = Address::generate(&env);
    let sender = Address::generate(&env);
    let lp_node = Address::generate(&env);
    let refund_address = Address::generate(&env);

    let lp_settings_contract_id = env.register(LPSettingManagerContract, ());
    let lp_settings_client = LPSettingManagerContractClient::new(&env, &lp_settings_contract_id);
    lp_settings_client.initialize(&admin);
    lp_settings_client.update_protocol_address(&ProtocolAddressType::Treasury, &treasury);
    lp_settings_client.update_protocol_address(&ProtocolAddressType::Aggregator, &aggregator);

    let lp_contract_id = env.register(LPContract, ());
    let lp_client = LPContractClient::new(&env, &lp_contract_id);

    lp_client.initialize(&admin, &usdc_asset, &lp_settings_contract_id);

    (
        env,
        lp_client,
        lp_settings_client,
        usdc_asset,
        token_client,
        (admin, sender, lp_node, refund_address, aggregator),
    )
}

#[test]
fn test_register_lp_node() {
    let (
        env,
        gateway_client,
        _settings_client,
        _usdc_asset,
        _token_client,
        (_admin, _sender, _lp_node, _refund_address, _aggregator),
    ) = setup();
    let lp_id = Bytes::from_array(&env, &[1u8; 32]);
    let result = gateway_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());
}

#[test]
fn test_create_order() {
    let (
        env,
        gateway_client,
        _settings_client,
        usdc_asset,
        token_client,
        (_admin, sender, _lp_node, refund_address, _aggregator),
    ) = setup();
    let order_id = Bytes::from_array(&env, &[2u8; 32]);
    let sender_fee_recipient = Address::generate(&env);
    let amount = 100_0000000_i128;
    let sender_fee = 10_0000000_i128;
    let rate = 9500_i64;
    let message_hash = String::from_str(&env, "hash123");

    let lp_id = Bytes::from_array(&env, &[1u8; 32]);
    let result = gateway_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: usdc_asset,
        sender: sender.clone(),
        amount,
        rate,
        sender_fee_recipient,
        sender_fee,
        refund_address,
        message_hash,
    };
    token_client.mint(&sender, &(amount + sender_fee));
    let result = gateway_client.try_create_order(&order_params);
    assert!(result.is_ok());

    let order = gateway_client.get_order_info(&order_id);
    assert_eq!(order.sender, sender);
    assert_eq!(order.amount, amount);
    assert_eq!(order_params.order_id, order_id);
    assert_eq!(order.protocol_fee, amount / 100);
}

#[test]
fn test_settle_full_order() {
    let (
        env,
        gateway_client,
        _settings_client,
        usdc_asset,
        token_client,
        (_admin, sender, lp_node, _refund_address, _aggregator),
    ) = setup();

    let lp_id = Bytes::from_array(&env, &[1u8; 32]);
    let result = gateway_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let amount = 100_i128;
    let sender_fee = 10_i128;

    token_client.mint(&sender, &(amount + sender_fee));

    let order_id = Bytes::from_array(&env, &[2u8; 32]);
    let split_order_id = Bytes::from_array(&env, &[3u8; 32]);
    let sender_fee_recipient = Address::generate(&env);
    let rate = 9500_i64;
    let message_hash = String::from_str(&env, "hash123");
    let settle_percent = 100_000_i128;

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: usdc_asset,
        sender: sender.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: Address::generate(&env),
        message_hash,
    };

    gateway_client.create_order(&order_params);

    let created_order = gateway_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    env.mock_all_auths();

    let result = gateway_client.try_settle(&split_order_id, &order_id, &lp_node, &settle_percent);

    assert!(result.is_ok());

    let settled_order = gateway_client.get_order_info(&order_id);

    assert!(settled_order.is_fulfilled, "Order should be fulfilled");

    assert_eq!(settled_order.current_bps, 0, "Current BPS should be 0");

    assert_eq!(settled_order.amount, 0, "Remaining amount should be 0");
}

#[test]
fn test_refund_order() {
    let (
        env,
        gateway_client,
        _settings_client,
        usdc_asset,
        token_client,
        (_admin, sender, _lp_node, refund_address, _aggregator),
    ) = setup();

    let lp_id = Bytes::from_array(&env, &[1u8; 32]);
    let result = gateway_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let amount = 100_i128;
    let sender_fee = 10_i128;

    token_client.mint(&sender, &(amount + sender_fee));

    let order_id = Bytes::from_array(&env, &[4u8; 32]);
    let sender_fee_recipient = Address::generate(&env);
    let rate = 9500_i64;
    let message_hash = String::from_str(&env, "hash123");

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: usdc_asset,
        sender: sender.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: refund_address.clone(),
        message_hash,
    };

    gateway_client.create_order(&order_params);
    let created_order = gateway_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    let refund_fee = 1_i128;
    env.mock_all_auths();
    let result = gateway_client.try_refund(&order_id, &refund_fee);
    assert!(result.is_ok(), "Refund should succeed");

    let refunded_order = gateway_client.get_order_info(&order_id);
    assert!(refunded_order.is_refunded, "Order should be refunded");
    let balance = gateway_client.get_token_balance(&refund_address);
    assert_eq!(balance, 109);

    assert_eq!(refunded_order.current_bps, 0, "Current BPS should be 0");
}
