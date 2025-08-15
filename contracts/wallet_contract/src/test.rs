#![cfg(test)]

use crate::wallet::{WalletContract, WalletContractClient};

use super::*;
use soroban_sdk::{vec, Env, String};

fn setup<'a>() -> (Env, WalletContractClient<'a>) {
    let env = Env::default();
    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);
    (env, client)
}
