use crate::liquidity_provider::{LPContract, LPContractClient};
use crate::storage_types::OrderParams;
use liquidity_manager::{
    liquidity_manager::{LPSettingManagerContract, LPSettingManagerContractClient},
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

#[derive(Debug)]
struct TestAddresses {
    sender: Address,
    lp_node: Address,
    refund_address: Address,
}

struct SetupResult<'a> {
    env: Env,
    lp_client: LPContractClient<'a>,
    usdc_asset: Address,
    token_client: token::StellarAssetClient<'a>,
    addresses: TestAddresses,
}

fn setup<'a>() -> SetupResult<'a> {
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
    lp_settings_client.initialize(&admin, &treasury, &aggregator);

    let lp_contract_id = env.register(LPContract, ());
    let lp_client = LPContractClient::new(&env, &lp_contract_id);

    lp_client.init(&admin, &usdc_asset, &lp_settings_contract_id);

    SetupResult {
        env,
        lp_client,
        usdc_asset,
        token_client,
        addresses: TestAddresses {
            sender,
            lp_node,
            refund_address,
        },
    }
}

#[test]
fn test_register_lp_node() {
    let setup_result = setup();
    let lp_id = Bytes::from_array(&setup_result.env, &[1u8; 32]);
    let result = setup_result.lp_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());
}

#[test]
fn test_create_order() {
    let setup_result = setup();
    let order_id = Bytes::from_array(&setup_result.env, &[2u8; 32]);
    let sender_fee_recipient = Address::generate(&setup_result.env);
    let amount = 100_0000000_i128;
    let sender_fee = 10_0000000_i128;
    let rate = 9500_i64;
    let message_hash = String::from_str(&setup_result.env, "hash123");

    let lp_id = Bytes::from_array(&setup_result.env, &[1u8; 32]);
    let result = setup_result.lp_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: setup_result.usdc_asset,
        sender: setup_result.addresses.sender.clone(),
        amount,
        rate,
        sender_fee_recipient,
        sender_fee,
        refund_address: setup_result.addresses.refund_address,
        message_hash,
    };
    setup_result
        .token_client
        .mint(&setup_result.addresses.sender, &(amount + sender_fee));
    let result = setup_result.lp_client.try_create_order(&order_params);
    assert!(result.is_ok());

    let order = setup_result.lp_client.get_order_info(&order_id);
    assert_eq!(order.sender, setup_result.addresses.sender);
    assert_eq!(order.amount, amount);
    assert_eq!(order_params.order_id, order_id);
    assert_eq!(order.protocol_fee, amount / 100);
}

#[test]
fn test_settle_full_order() {
    let setup_result = setup();
    let lp_id = Bytes::from_array(&setup_result.env, &[1u8; 32]);
    let result = setup_result.lp_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let amount = 100_i128;
    let sender_fee = 10_i128;

    setup_result
        .token_client
        .mint(&setup_result.addresses.sender, &(amount + sender_fee));

    let order_id = Bytes::from_array(&setup_result.env, &[2u8; 32]);

    let sender_fee_recipient = Address::generate(&setup_result.env);
    let rate = 9500_i64;
    let message_hash = String::from_str(&setup_result.env, "hash123");
    let settle_percent = 100_000_i128;

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: setup_result.usdc_asset,
        sender: setup_result.addresses.sender.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: Address::generate(&setup_result.env),
        message_hash,
    };

    setup_result.lp_client.create_order(&order_params);

    let created_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    setup_result.env.mock_all_auths();

    let result = setup_result.lp_client.try_settle(
        &order_id,
        &setup_result.addresses.lp_node,
        &settle_percent,
    );

    assert!(result.is_ok());

    let settled_order = setup_result.lp_client.get_order_info(&order_id);

    assert!(settled_order.is_fulfilled, "Order should be fulfilled");

    assert_eq!(settled_order.current_bps, 0, "Current BPS should be 0");

    assert_eq!(settled_order.amount, 0, "Remaining amount should be 0");
}

#[test]
fn test_refund_order() {
    let setup_result = setup();

    let lp_id = Bytes::from_array(&setup_result.env, &[1u8; 32]);
    let result = setup_result.lp_client.try_register_lp_node(&lp_id, &1000);
    assert!(result.is_ok());

    let amount = 100_i128;
    let sender_fee = 10_i128;

    setup_result
        .token_client
        .mint(&setup_result.addresses.sender, &(amount + sender_fee));

    let order_id = Bytes::from_array(&setup_result.env, &[4u8; 32]);
    let sender_fee_recipient = Address::generate(&setup_result.env);
    let rate = 9500_i64;
    let message_hash = String::from_str(&setup_result.env, "hash123");

    let order_params = OrderParams {
        order_id: order_id.clone(),
        token: setup_result.usdc_asset,
        sender: setup_result.addresses.sender.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: setup_result.addresses.refund_address.clone(),
        message_hash,
    };

    setup_result.lp_client.create_order(&order_params);
    let created_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    let refund_fee = 1_i128;
    setup_result.env.mock_all_auths();
    let result = setup_result.lp_client.try_refund(&order_id, &refund_fee);
    assert!(result.is_ok(), "Refund should succeed");

    let refunded_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(refunded_order.is_refunded, "Order should be refunded");
    let balance = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.refund_address);
    assert_eq!(balance, 109);

    assert_eq!(refunded_order.current_bps, 0, "Current BPS should be 0");
}
