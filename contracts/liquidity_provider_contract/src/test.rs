use crate::{
    liquidity_provider::{GatewayContract, GatewayContractClient},
    storage_types::{Order, OrderParams},
};
use liquidity_manager::{
    liquidity_manager::{GatewaySettingManagerContract, GatewaySettingManagerContractClient},
    storage::ProtocolAddressType,
};
use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env, String};

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
    GatewayContractClient<'a>,
    GatewaySettingManagerContractClient<'a>,
    Address,
    Address,
    token::StellarAssetClient<'a>,
    (Address, Address, Address, Address, Address),
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (usdc_asset, token_client) = create_token_contract(&env, &admin);
    let wallet_contract = Address::generate(&env);
    let treasury = Address::generate(&env);
    let aggregator = Address::generate(&env);
    let sender = Address::generate(&env);
    let lp_node = Address::generate(&env);
    let refund_address = Address::generate(&env);

    let settings_contract_id = env.register(GatewaySettingManagerContract, ());
    let settings_client = GatewaySettingManagerContractClient::new(&env, &settings_contract_id);
    settings_client.initialize(&admin);
    settings_client.update_protocol_address(&ProtocolAddressType::Treasury, &treasury);
    settings_client.update_protocol_address(&ProtocolAddressType::Aggregator, &aggregator);

    let gateway_contract_id = env.register(GatewayContract, ());
    let gateway_client = GatewayContractClient::new(&env, &gateway_contract_id);
    gateway_client.initialize(&admin, &usdc_asset, &wallet_contract, &settings_contract_id);

    (
        env,
        gateway_client,
        settings_client,
        wallet_contract,
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
        _wallet_contract,
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
        _wallet_contract,
        _usdc_asset,
        token_client,
        (_admin, sender, _lp_node, refund_address, _aggregator),
    ) = setup();
    let order_id = Bytes::from_array(&env, &[2u8; 32]);
    let sender_fee_recipient = Address::generate(&env);
    let amount = 100_0000000_i128;
    let sender_fee = 10_0000000_i128;
    let rate = 9500_i64;
    let message_hash = String::from_str(&env, "hash123");

    let order_params = OrderParams {
        order_id: order_id.clone(),
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
    assert_eq!(order.protocol_fee, amount / 100);
}

#[test]
fn test_settle() {
    let (
        env,
        gateway_client,
        _settings_client,
        wallet_contract,
        _usdc_asset,
        token_client,
        (_admin, sender, lp_node, refund_address, _aggregator),
    ) = setup();
    let order_id = Bytes::from_array(&env, &[2u8; 32]);
    let split_order_id = Bytes::from_array(&env, &[3u8; 32]);
    let sender_fee_recipient = Address::generate(&env);
    let amount = 100_0000000_i128;
    let settle_percent = 100_000_i64;
    let sender_fee = 10_0000000_i128;
    let rate = 9500_i64;
    let message_hash = String::from_str(&env, "hash123");

    token_client.mint(&sender, &(amount + sender_fee));

    let order_params = OrderParams {
        order_id: order_id.clone(),
        sender: sender.clone(),
        amount,
        rate,
        sender_fee_recipient,
        sender_fee,
        refund_address,
        message_hash,
    };

    gateway_client.create_order(&order_params);

    token_client.mint(&wallet_contract, &(amount + sender_fee));
    let result = gateway_client.try_settle(&split_order_id, &order_id, &lp_node, &settle_percent);
    assert!(result.is_ok());

    let order: Order = gateway_client.get_order_info(&order_id);
    assert!(order.is_fulfilled);
    assert_eq!(order.current_bps, 0);
}

//#[test]
//fn test_refund() {
//    let (
//        env,
//        gateway_client,
//        _settings_client,
//        wallet_contract,
//        _usdc_asset,
//        token_client,
//        (_admin, sender, _lp_node, refund_address, _aggregator),
//    ) = setup();
//    let order_id = Bytes::from_array(&env, &[2u8; 32]);
//    let sender_fee_recipient = Address::generate(&env);
//    let amount = 100_0000000_i128;
//    let sender_fee = 5_0000000_i128;
//    let fee = 10_000000_i128;
//    let rate = 9500_i64;
//    let message_hash = String::from_str(&env, "hash123");
//
//    token_client.mint(&sender, &(amount + sender_fee));
//
//    let order_params = OrderParams {
//        order_id: order_id.clone(),
//        sender: sender.clone(),
//        amount,
//        rate,
//        sender_fee_recipient,
//        sender_fee,
//        refund_address,
//        message_hash,
//    };
//
//    gateway_client.create_order(&order_params);
//
//    token_client.mint(&wallet_contract, &(amount + sender_fee));
//    let result = gateway_client.try_refund(&order_id, &fee);
//    assert!(result.is_ok());
//
//    let order: Order = gateway_client.get_order_info(&order_id);
//    assert!(order.is_refunded);
//    assert_eq!(order.current_bps, 0);
//}
