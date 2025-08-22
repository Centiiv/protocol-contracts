#![cfg(test)]

use crate::liquidity_provider::{LiquidityProviderContract, LiquidityProviderContractClient};

use super::*;
use soroban_sdk::{vec, Env, String};

fn setup<'a>() -> (Env, LiquidityProviderContractClient<'a>) {
    let env = Env::default();
    let contract_id = env.register(LiquidityProviderContract, ());
    let client = LiquidityProviderContractClient::new(&env, &contract_id);
    (env, client)
}
