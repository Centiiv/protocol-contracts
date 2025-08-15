#![cfg(test)]

use soroban_sdk::{vec, Env, String};

use crate::escrow::{EscrowContract, EscrowContractClient};

fn setup<'a>() -> (Env, EscrowContractClient<'a>) {
    let env = Env::default();
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    (env, client)
}
