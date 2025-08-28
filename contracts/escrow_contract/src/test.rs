#![cfg(test)]
use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env};

use crate::{
    escrow::{EscrowContract, EscrowContractClient},
    storage_types::{DataKey, EscrowStatus},
};

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
    EscrowContractClient<'a>,
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
        EscrowContract,
        (admin.clone(), token_address.clone(), wallet_id.clone()),
    );
    let client = EscrowContractClient::new(&env, &contract_id);

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
fn test_lock_and_release() {
    let (env, escrow_client, wallet_client, _token_address, _token_client, (buyer, _admin)) =
        setup();

    let lp_node = Address::generate(&env);

    let req_id = Bytes::from_array(&env, &[1, 2, 3, 4]);

    let user_id = Bytes::from_array(&env, &[1; 32]);

    wallet_client.deposit(&user_id, &buyer, &10000);

    escrow_client.lock_funds(&req_id, &buyer, &lp_node, &1000, &60);

    let escrow = escrow_client.get_escrow_status(&req_id).unwrap();

    assert_eq!(escrow.amount, 1000);

    assert_eq!(escrow.status, EscrowStatus::Locked);

    escrow_client.release_funds(&req_id, &lp_node);

    let lp_balance = wallet_client.get_balance(&lp_node);

    let escrow_after = escrow_client.get_escrow_status(&req_id).unwrap();

    assert_eq!(escrow_after.status, EscrowStatus::Released);

    assert_eq!(lp_balance, 1000);
}

#[test]
fn test_unauthorized_release_attempt() {
    let (env, escrow_client, wallet_client, _token_address, _token_client, (buyer, _admin)) =
        setup();

    let lp_node = Address::generate(&env);
    let attacker = Address::generate(&env);

    let req_id = Bytes::from_array(&env, &[9, 9, 9, 9]);
    let user_id = Bytes::from_array(&env, &[1; 32]);

    wallet_client.deposit(&user_id, &buyer, &5000);

    escrow_client.lock_funds(&req_id, &buyer, &lp_node, &1000, &60);

    let result = escrow_client.try_release_funds(&req_id, &attacker);

    assert!(result.is_err());
}

#[test]
fn test_double_lock_same_request() {
    let (env, escrow_client, wallet_client, _token_address, _token_client, (buyer, _admin)) =
        setup();

    let lp_node = Address::generate(&env);
    let req_id = Bytes::from_array(&env, &[5, 5, 5, 5]);
    let user_id = Bytes::from_array(&env, &[1; 32]);

    wallet_client.deposit(&user_id, &buyer, &5000);

    escrow_client.lock_funds(&req_id, &buyer, &lp_node, &1000, &60);

    let result = escrow_client.try_lock_funds(&req_id, &buyer, &lp_node, &500, &60);
    assert!(result.is_err());
}

#[test]
fn test_release_without_lock() {
    let (env, escrow_client, _wallet_client, _token_address, _token_client, (_buyer, _admin)) =
        setup();

    let lp_node = Address::generate(&env);
    let fake_req_id = Bytes::from_array(&env, &[8, 8, 8, 8]);

    let tofail = escrow_client.try_release_funds(&fake_req_id, &lp_node);
    assert!(tofail.is_err());
}
