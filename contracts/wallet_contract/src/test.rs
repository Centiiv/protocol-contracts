#![cfg(test)]
use crate::{
    storage::DataKey,
    wallet::{WalletContract, WalletContractClient},
};
use soroban_sdk::{log, testutils::Address as _, token, Address, Bytes, Env};

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
    WalletContractClient<'a>,
    Address,
    Address,
    token::StellarAssetClient<'a>,
    (Address, Address),
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = generate_addresses(&env);
    let (token_address, token_client) = create_token_contract(&env, &admin);
    let (buyer, admin) = create_user(&env, &token_client, &admin, 10000);
    token_client.mint(&buyer, &10000);

    let central_account = generate_addresses(&env);

    let contract_id = env.register(
        WalletContract,
        (
            admin.clone(),
            token_address.clone(),
            central_account.clone(),
        ),
    );
    let client = WalletContractClient::new(&env, &contract_id);

    (
        env,
        client,
        token_address,
        central_account,
        token_client,
        (buyer, admin),
    )
}

#[test]
fn test_initialize() {
    let (env, client, token_address, central_account, _token_client, _) = setup();

    let stored_usdc: Address = env.as_contract(&client.address, || {
        env.storage().persistent().get(&DataKey::Usdc).unwrap()
    });
    let central_address: Address = env.as_contract(&client.address, || {
        env.storage().persistent().get(&DataKey::Central).unwrap()
    });

    assert_eq!(stored_usdc, token_address);
    assert_eq!(central_address, central_account);
}

#[test]
fn test_deposit() {
    let (env, client, _token_address, _central_account, _token_client, (buyer, _admin)) = setup();
    let user_id = Bytes::from_array(&env, &[1; 32]);
    let amount = 500;

    env.mock_all_auths();

    let initial_token_balance = client.get_token_balance(&buyer);
    let initial_wallet_balance = client.get_balance(&buyer);
    let initial_central_balance = client.get_central_balance();

    log!(&env, "Initial token balance:", initial_token_balance);
    log!(&env, "Initial wallet balance:", initial_wallet_balance);
    log!(&env, "Initial central balance:", initial_central_balance);

    client.deposit(&user_id, &buyer, &amount);

    let final_token_balance = client.get_token_balance(&buyer);
    let final_wallet_balance = client.get_balance(&buyer);
    let final_central_balance = client.get_central_balance();

    log!(&env, "Final token balance:", final_token_balance);
    log!(&env, "Final wallet balance:", final_wallet_balance);
    log!(&env, "Final central balance:", final_central_balance);

    assert_eq!(final_token_balance, initial_token_balance - amount);
    assert_eq!(final_wallet_balance, initial_wallet_balance + amount);
    assert_eq!(final_central_balance, initial_central_balance + amount);
}

#[test]
fn test_deposit_insufficient_funds() {
    let (env, client, _token_address, _central_account, _token_client, (buyer, _admin)) = setup();
    let user_id = Bytes::from_array(&env, &[1; 32]);

    env.mock_all_auths();

    let user_balance = client.get_token_balance(&buyer);
    let excessive_amount = user_balance + 1000;

    let result = client.try_deposit(&user_id, &buyer, &excessive_amount);
    assert!(result.is_err());
    log!(&env, "Deposit with insufficient funds correctly failed");
}

#[test]
fn test_deposit_invalid_amount() {
    let (env, client, _token_address, _central_account, _token_client, (buyer, _admin)) = setup();
    let user_id = Bytes::from_array(&env, &[1; 32]);

    env.mock_all_auths();

    let result_zero = client.try_deposit(&user_id, &buyer, &0);
    assert!(result_zero.is_err());

    let result_negative = client.try_deposit(&user_id, &buyer, &(-100));
    assert!(result_negative.is_err());

    log!(&env, "Invalid amount deposits correctly failed");
}

#[test]
fn test_internal_transfer() {
    let (env, client, _token_address, _central_account, _token_client, (buyer, admin)) = setup();
    let user_id = Bytes::from_array(&env, &[1; 32]);
    let deposit_amount = 1000;
    let transfer_amount = 300;

    env.mock_all_auths();

    client.deposit(&user_id, &buyer, &deposit_amount);
    client.deposit(&user_id, &admin, &deposit_amount);

    let buyer_wallet_before = client.get_balance(&buyer);
    let admin_wallet_before = client.get_balance(&admin);

    log!(&env, "Buyer wallet before transfer:", buyer_wallet_before);
    log!(&env, "Admin wallet before transfer:", admin_wallet_before);

    client.transfer(&buyer, &admin, &transfer_amount);

    let buyer_wallet_after = client.get_balance(&buyer);
    let admin_wallet_after = client.get_balance(&admin);

    log!(&env, "Buyer wallet after transfer:", buyer_wallet_after);
    log!(&env, "Admin wallet after transfer:", admin_wallet_after);

    assert_eq!(buyer_wallet_after, buyer_wallet_before - transfer_amount);
    assert_eq!(admin_wallet_after, admin_wallet_before + transfer_amount);
}

#[test]
fn test_withdraw() {
    let (env, client, _token_address, _central_account, _token_client, (buyer, _admin)) = setup();
    let user_id = Bytes::from_array(&env, &[1; 32]);
    let deposit_amount = 1000;
    let withdraw_amount = 400;

    env.mock_all_auths();

    client.deposit(&user_id, &buyer, &deposit_amount);

    let token_balance_before = client.get_token_balance(&buyer);
    let wallet_balance_before = client.get_balance(&buyer);
    let central_balance_before = client.get_central_balance();

    log!(&env, "Before withdrawal - Token:", token_balance_before);
    log!(&env, "Before withdrawal - Wallet:", wallet_balance_before);
    log!(&env, "Before withdrawal - Central:", central_balance_before);

    client.withdraw(&user_id, &buyer, &withdraw_amount);

    let token_balance_after = client.get_token_balance(&buyer);
    let wallet_balance_after = client.get_balance(&buyer);
    let central_balance_after = client.get_central_balance();

    log!(&env, "After withdrawal - Token:", token_balance_after);
    log!(&env, "After withdrawal - Wallet:", wallet_balance_after);
    log!(&env, "After withdrawal - Central:", central_balance_after);

    assert_eq!(token_balance_after, token_balance_before + withdraw_amount);
    assert_eq!(
        wallet_balance_after,
        wallet_balance_before - withdraw_amount
    );
    assert_eq!(
        central_balance_after,
        central_balance_before - withdraw_amount
    );
}
