use crate::liquidity_provider::{LPContract, LPContractClient};
use crate::storage_types::OrderParams;
use liquidity_manager::liquidity_manager::{
    LPSettingManagerContract, LPSettingManagerContractClient,
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
    temporary_wallet: Address,
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
    let temporary_wallet = Address::generate(&env);

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
            temporary_wallet,
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
        temporary_wallet_address: setup_result.addresses.temporary_wallet.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: setup_result.addresses.refund_address.clone(),
        message_hash: message_hash.clone(),
    };

    setup_result
        .token_client
        .mint(&setup_result.addresses.sender, &(amount + sender_fee));

    setup_result.env.mock_all_auths();
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
        temporary_wallet_address: setup_result.addresses.temporary_wallet.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: Address::generate(&setup_result.env),
        message_hash: message_hash.clone(),
    };

    setup_result.env.mock_all_auths();
    setup_result.lp_client.create_order(&order_params);

    let created_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    let temp_wallet_balance_after_create = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.temporary_wallet);
    assert_eq!(
        temp_wallet_balance_after_create,
        amount + sender_fee,
        "Temporary wallet should have all tokens after order creation"
    );

    setup_result.env.mock_all_auths();
    let result = setup_result.lp_client.try_settle(
        &order_id,
        &setup_result.addresses.lp_node,
        &settle_percent,
    );
    assert!(result.is_ok(), "Settle failed: {:?}", result.err());

    let settled_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(settled_order.is_fulfilled, "Order should be fulfilled");
    assert_eq!(settled_order.current_bps, 0, "Current BPS should be 0");
    assert_eq!(settled_order.amount, 0, "Remaining amount should be 0");

    setup_result.env.mock_all_auths();
    let transfer_result = setup_result
        .lp_client
        .try_execute_settlement_transfer(&order_id);
    assert!(
        transfer_result.is_ok(),
        "Transfer execution failed: {:?}",
        transfer_result.err()
    );

    let lp_balance = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.lp_node);

    let treasury_balance = setup_result
        .lp_client
        .get_token_balance(&Address::generate(&setup_result.env));

    let sender_fee_balance = setup_result
        .lp_client
        .get_token_balance(&sender_fee_recipient);

    let temp_wallet_balance_after_settle = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.temporary_wallet);

    assert_eq!(lp_balance, 99, "LP should receive 99 tokens");

    assert_eq!(
        sender_fee_balance, 10,
        "Sender fee recipient should receive 10 tokens"
    );

    assert_eq!(
        temp_wallet_balance_after_settle, 0,
        "Temporary wallet should be empty after settlement"
    );
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
        temporary_wallet_address: setup_result.addresses.temporary_wallet.clone(),
        amount,
        rate,
        sender_fee_recipient: sender_fee_recipient.clone(),
        sender_fee,
        refund_address: setup_result.addresses.refund_address.clone(),
        message_hash: message_hash.clone(),
    };

    setup_result.env.mock_all_auths();
    setup_result.lp_client.create_order(&order_params);

    let created_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(!created_order.is_fulfilled);
    assert!(!created_order.is_refunded);
    assert_eq!(created_order.amount, amount);

    let temp_wallet_balance_after_create = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.temporary_wallet);
    assert_eq!(
        temp_wallet_balance_after_create,
        amount + sender_fee,
        "Temporary wallet should have all tokens after order creation"
    );

    let refund_fee = 1_i128;

    setup_result.env.mock_all_auths();
    let result = setup_result.lp_client.try_refund(&order_id, &refund_fee);
    assert!(
        result.is_ok(),
        "Refund initiation failed: {:?}",
        result.err()
    );

    let refunded_order = setup_result.lp_client.get_order_info(&order_id);
    assert!(
        refunded_order.is_refunded,
        "Order should be marked as refunded"
    );

    setup_result.env.mock_all_auths();
    let transfer_result = setup_result
        .lp_client
        .try_execute_refund_transfer(&order_id);
    assert!(
        transfer_result.is_ok(),
        "Refund transfer failed: {:?}",
        transfer_result.err()
    );

    let refund_address_balance = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.refund_address);
    let treasury_balance = setup_result
        .lp_client
        .get_token_balance(&Address::generate(&setup_result.env));
    let temp_wallet_balance_after_refund = setup_result
        .lp_client
        .get_token_balance(&setup_result.addresses.temporary_wallet);

    assert_eq!(
        refund_address_balance, 109,
        "Refund address should have 109 tokens"
    );
    assert_eq!(
        temp_wallet_balance_after_refund, 0,
        "Temporary wallet should be empty after refund"
    );

    let final_order = setup_result.lp_client.get_order_info(&order_id);
    assert_eq!(final_order.current_bps, 0, "Current BPS should be 0");
}
