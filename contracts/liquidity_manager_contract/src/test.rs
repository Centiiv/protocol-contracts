#![cfg(test)]
use crate::{
    payment::{PaymentModule, PaymentModuleClient},
    storage::DataKey,
};
use soroban_sdk::{
    log,
    testutils::{Address as _, Events},
    token, Address, Bytes, Env, Val, Vec,
};

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

fn get_contract_events(
    env: &Env,
    _client: &PaymentModuleClient,
) -> Vec<(soroban_sdk::Address, Vec<Val>, Val)> {
    let events = env.events().all();
    log!(env, "events inside get_contract_events: {:?}", events);
    events
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

fn setup<'a>() -> (Env, PaymentModuleClient<'a>, Address, (Address, Address)) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = generate_addresses(&env);
    let (token_address, token_client) = create_token_contract(&env, &admin);

    let (buyer, admin) = create_user(&env, &token_client, &admin, 10000);

    token_client.mint(&buyer, &10000);

    let contract_id = env.register(PaymentModule, (admin.clone(), token_address.clone()));
    let client = PaymentModuleClient::new(&env, &contract_id);

    (env, client, token_address, (buyer, admin))
}

#[test]
fn test_initialize() {
    let (env, _client, token_address, _) = setup();

    let stored_usdc: Address = env.as_contract(&_client.address, || {
        env.storage().persistent().get(&DataKey::Usdc).unwrap()
    });

    assert_eq!(stored_usdc, token_address);
}

#[test]
fn test_make_payment() {
    let (env, client, _token_address, (buyer, _admin)) = setup();

    let seller = Address::generate(&env);

    let payment_id = Bytes::from_array(&env, &[1; 32]);

    let invoice_id = Bytes::from_array(&env, &[2; 32]);

    let amount = 500;

    env.mock_all_auths();

    let result = client.make_payment(&payment_id, &invoice_id, &buyer, &seller, &amount);

    assert_eq!(result, payment_id);

    let seller_balance = client.get_balance(&seller);

    assert_eq!(seller_balance, amount);

    log!(&env, "balance", seller_balance);

    let stored: Option<crate::storage::Payment> = env.as_contract(&client.address, || {
        env.storage().persistent().get(&payment_id)
    });

    assert!(stored.is_some());

    let events = get_contract_events(&env, &client);

    log!(&env, "Events in test_event_emission", events);
}
