#![cfg(test)]

use crate::{
    liquidity_provider::{LiquidityProviderContract, LiquidityProviderContractClient},
    storage_types::{Algorithm, LpNodeStatus, RegistrationStatus},
};

use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env};

use crate::storage_types::DataKey;

use wallet_contract::wallet::{WalletContract, WalletContractClient};

fn generate_addresses(env: &Env) -> Address {
    Address::generate(env)
}

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (Address, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract_v2(admin.clone());
    (
        contract_address.address(),
        token::StellarAssetClient::new(e, &contract_address.address()),
    )
}

fn create_user(
    env: &Env,
    token: &token::StellarAssetClient,
    admin: &Address,
    amount: i128,
) -> (Address, Address) {
    let user = Address::generate(env);
    token.mint(admin, &amount);
    token.mint(&user, &amount);
    (user, admin.clone())
}

fn setup<'a>() -> (
    Env,
    LiquidityProviderContractClient<'a>,
    WalletContractClient<'a>,
    Address,
    token::StellarAssetClient<'a>,
    (Address, Address),
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = generate_addresses(&env);
    let (token_address, token_client) = create_token_contract(&env, &admin);
    let (buyer, admin) = create_user(&env, &token_client, &admin, 10000);

    let central_account = generate_addresses(&env);

    let wallet_id = env.register(
        WalletContract,
        (
            admin.clone(),
            token_address.clone(),
            central_account.clone(),
        ),
    );
    let wallet_client = WalletContractClient::new(&env, &wallet_id);

    let contract_id = env.register(
        LiquidityProviderContract,
        (admin.clone(), token_address.clone(), wallet_id.clone()),
    );
    let client = LiquidityProviderContractClient::new(&env, &contract_id);

    (
        env,
        client,
        wallet_client,
        token_address,
        token_client,
        (buyer, admin),
    )
}

#[test]
fn test_initialize() {
    let (env, escrow_client, wallet_client, token_address, _token_client, _) = setup();

    let stored_usdc: Address = env.as_contract(&escrow_client.address, || {
        env.storage().persistent().get(&DataKey::Usdc).unwrap()
    });
    let stored_wallet: Address = env.as_contract(&escrow_client.address, || {
        env.storage().persistent().get(&DataKey::Wallet).unwrap()
    });

    assert_eq!(stored_usdc, token_address);
    assert_eq!(stored_wallet, wallet_client.address);
}

#[test]
fn test_register_lp_node_success() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (_buyer, _admin)) = setup();

    let lp_id = generate_addresses(&env);

    let result = lp_client.try_register_lp_node(&lp_id, &1000, &120, &95, &30);
    assert!(result.is_ok());

    let second_reg_result = lp_client.try_register_lp_node(&lp_id, &500, &100, &90, &25);
    assert!(second_reg_result.is_err());
}

#[test]
fn test_register_lp_node_with_invalid_values() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, _) = setup();

    let lp_id = generate_addresses(&env);

    let result = lp_client.try_register_lp_node(&lp_id, &0, &100, &90, &30);
    assert!(result.is_err());

    let result = lp_client.try_register_lp_node(&lp_id, &1000, &0, &120, &30);
    assert!(result.is_err());
}

#[test]
fn test_approve_unregistered_lp_node_fails() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, _) = setup();

    let lp_id = generate_addresses(&env);

    let status = lp_client.get_lp_registration_status(&lp_id);
    assert_eq!(status, RegistrationStatus::Unregistered);
}

#[test]
fn test_duplicate_disbursal_request_id_fails() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (buyer, _admin)) = setup();

    let req_id = Bytes::from_array(&env, &[7, 7, 7, 7]);

    let _ = lp_client
        .try_create_disbursal_request(&req_id, &buyer, &500)
        .unwrap();
    let result = lp_client.try_create_disbursal_request(&req_id, &buyer, &600);

    assert!(result.is_err());
}

#[test]
fn test_select_lp_node_invalid_algorithm() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (buyer, _admin)) = setup();

    let req_id = Bytes::from_array(&env, &[3, 3, 3, 3]);
    let _ = lp_client
        .try_create_disbursal_request(&req_id, &buyer, &500)
        .unwrap();

    let algo = Algorithm::Wrr;
    let result = lp_client.try_select_lp_node(&req_id, &algo, &None::<Bytes>);
    assert!(result.is_err(), "Should fail if no LP nodes are available");
}

#[test]
fn test_approve_lp_node_success() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (_buyer, _admin)) = setup();

    let lp_id = generate_addresses(&env);

    let result = lp_client.try_register_lp_node(&lp_id, &1000, &120, &95, &30);
    assert!(result.is_ok());

    let second_reg_result = lp_client.try_register_lp_node(&lp_id, &500, &100, &90, &25);
    assert!(second_reg_result.is_err());

    let status = lp_client.get_lp_node_status(&lp_id);
    assert_eq!(status, LpNodeStatus::AwaitingApproval);
}

#[test]
fn test_create_disbursal_request_success() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (buyer, _admin)) = setup();

    let req_id = Bytes::from_array(&env, &[9, 9, 9, 9]);

    let result = lp_client.try_create_disbursal_request(&req_id, &buyer, &500);
    assert!(result.is_ok());

    let bad_req = Bytes::from_array(&env, &[8, 8, 8, 8]);
    let result2 = lp_client.try_create_disbursal_request(&bad_req, &buyer, &0);
    assert!(result2.is_err());
}

#[test]
fn test_select_lp_node_with_wrr() {
    let (env, lp_client, _wallet_client, _token_address, _token_client, (buyer, _admin)) = setup();

    let lp_add = generate_addresses(&env);

    let _ = lp_client
        .try_register_lp_node(&lp_add, &1000, &150, &95, &30)
        .unwrap();

    let _ = lp_client.try_register_lp_node(&lp_add, &2000, &120, &90, &25);

    let req_id = Bytes::from_array(&env, &[5, 5, 5, 5]);
    let _ = lp_client
        .try_create_disbursal_request(&req_id, &buyer, &500)
        .unwrap();

    let algo = Algorithm::Wrr;

    let chosen = lp_client.try_select_lp_node(&req_id, &algo, &None::<Bytes>);

    assert!(chosen.is_err());
}
