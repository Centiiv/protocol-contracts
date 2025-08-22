use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Bytes, Env};

#[contract]
pub struct WalletContract;

#[contractimpl]
impl WalletContract {
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address, central_account: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("usdc"), &usdc_asset);
        env.storage()
            .persistent()
            .set(&symbol_short!("central"), &central_account);
    }

    pub fn deposit(env: Env, user_id: Bytes, stellar_address: Address, amount: i128) {
        stellar_address.require_auth();
        let usdc_asset: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("usdc"))
            .unwrap();
        let central_account: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("central"))
            .unwrap();

        // Transfer USDC to central account
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.transfer(&stellar_address, &central_account, &amount);

        // Update balance
        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&stellar_address)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&stellar_address, &(current_balance + amount));

        env.events()
            .publish((symbol_short!("Deposit"), user_id, stellar_address), amount);
    }

    pub fn transfer(env: Env, from_address: Address, to_address: Address, amount: i128) {
        from_address.require_auth();
        let usdc_asset: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("usdc"))
            .unwrap();
        let central_account: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("central"))
            .unwrap();

        // Validate balance
        let from_balance: i128 = env.storage().persistent().get(&from_address).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        // Transfer USDC
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.transfer(&central_account, &to_address, &amount);

        // Update balances
        env.storage()
            .persistent()
            .set(&from_address, &(from_balance - amount));
        let to_balance: i128 = env.storage().persistent().get(&to_address).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&to_address, &(to_balance + amount));

        env.events().publish(
            (symbol_short!("Transfer"), from_address, to_address),
            amount,
        );
    }

    pub fn get_balance(env: Env, stellar_address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&stellar_address)
            .unwrap_or(0)
    }
}
