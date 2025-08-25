use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env};

use crate::{error::WalletError, storage::DataKey};

#[contract]
pub struct WalletContract;

#[contractimpl]
impl WalletContract {
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address, central_account: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Usdc, &usdc_asset);
        env.storage()
            .persistent()
            .set(&DataKey::Central, &central_account);
    }

    pub fn deposit(
        env: Env,
        user_id: Bytes,
        stellar_address: Address,
        amount: i128,
    ) -> Result<(), WalletError> {
        stellar_address.require_auth();

        let current_balance: i128 = Self::get_balance(&env, stellar_address.clone());

        if amount > current_balance {
            return Err(WalletError::InsufficientFunds);
        }

        let usdc_asset: Address = Self::get_usdc_address(&env);

        let central_account: Address = Self::get_central_account(&env);

        // Transfer USDC to central account
        let token_client = token::Client::new(&env, &usdc_asset);

        token_client.transfer(&stellar_address, &central_account, &amount);

        // Update balance
        let current_balance: i128 = Self::get_balance(&env, stellar_address.clone());

        env.storage()
            .persistent()
            .set(&stellar_address, &(current_balance + amount));

        env.events()
            .publish((("Deposit confirmed"), user_id, stellar_address), amount);

        Ok(())
    }

    pub fn transfer(
        env: Env,
        from_address: Address,
        to_address: Address,
        amount: i128,
    ) -> Result<(), WalletError> {
        from_address.require_auth();
        let usdc_asset: Address = Self::get_usdc_address(&env);
        let central_account: Address = Self::get_central_account(&env);

        // Validate balance
        let from_balance: i128 = env.storage().persistent().get(&from_address).unwrap_or(0);

        if from_balance < amount {
            return Err(WalletError::InsufficientFunds);
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

        env.events()
            .publish((("Transfer"), from_address, to_address), amount);

        Ok(())
    }

    pub fn get_balance(env: &Env, stellar_address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&stellar_address)
            .unwrap_or(0)
    }

    fn get_usdc_address(env: &Env) -> Address {
        env.storage().persistent().get(&DataKey::Usdc).unwrap()
    }

    fn get_central_account(env: &Env) -> Address {
        env.storage().persistent().get(&DataKey::Central).unwrap()
    }
}
