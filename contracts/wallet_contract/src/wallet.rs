use crate::{error::WalletError, storage::DataKey};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env};

#[contract]
pub struct WalletContract;

#[contractimpl]
impl WalletContract {
    pub fn init(env: Env, admin: Address, usdc_asset: Address, central_account: Address) {
        if env.storage().persistent().has(&DataKey::Usdc) {
            panic!("Already initialized");
        }
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

        if amount <= 0 {
            return Err(WalletError::InvalidAmount);
        }

        let usdc_asset: Address = Self::get_usdc_address(&env);
        let central_account: Address = Self::get_central_account(&env);

        let token_client = token::Client::new(&env, &usdc_asset);
        let actual_token_balance = token_client.balance(&stellar_address);

        if amount > actual_token_balance {
            return Err(WalletError::InsufficientFunds);
        }

        token_client.transfer(&stellar_address, &central_account, &amount);

        let current_wallet_balance: i128 = Self::get_balance(&env, stellar_address.clone());
        env.storage()
            .persistent()
            .set(&stellar_address, &(current_wallet_balance + amount));

        env.events()
            .publish((("Deposit confirmed"), user_id, stellar_address), amount);

        Ok(())
    }

    fn transfer_internal(
        env: &Env,
        from_address: Address,
        to_address: Address,
        amount: i128,
    ) -> Result<(), WalletError> {
        if amount <= 0 {
            return Err(WalletError::InvalidAmount);
        }

        let from_balance: i128 = Self::get_balance(env, from_address.clone());

        if from_balance < amount {
            return Err(WalletError::InsufficientFunds);
        }

        env.storage()
            .persistent()
            .set(&from_address, &(from_balance - amount));

        let to_balance: i128 = Self::get_balance(env, to_address.clone());

        env.storage()
            .persistent()
            .set(&to_address, &(to_balance + amount));

        Ok(())
    }

    pub fn withdraw(
        env: Env,
        user_id: Bytes,
        stellar_address: Address,
        amount: i128,
    ) -> Result<(), WalletError> {
        stellar_address.require_auth();

        let usdc_asset = Self::get_usdc_address(&env);
        let central_account = Self::get_central_account(&env);

        Self::transfer_internal(
            &env,
            stellar_address.clone(),
            central_account.clone(),
            amount,
        )?;

        central_account.require_auth();
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.transfer(&central_account, &stellar_address, &amount);

        env.events()
            .publish((("Withdrawal confirmed"), user_id, stellar_address), amount);

        Ok(())
    }

    pub fn transfer(
        env: Env,
        from_address: Address,
        to_address: Address,
        amount: i128,
    ) -> Result<(), WalletError> {
        from_address.require_auth();

        Self::transfer_internal(&env, from_address.clone(), to_address.clone(), amount)?;

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

    pub fn get_token_balance(env: Env, stellar_address: Address) -> i128 {
        let usdc_asset: Address = Self::get_usdc_address(&env);
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.balance(&stellar_address)
    }

    pub fn get_central_balance(env: Env) -> i128 {
        let usdc_asset: Address = Self::get_usdc_address(&env);
        let central_account: Address = Self::get_central_account(&env);
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.balance(&central_account)
    }

    fn get_usdc_address(env: &Env) -> Address {
        env.storage().persistent().get(&DataKey::Usdc).unwrap()
    }

    fn get_central_account(env: &Env) -> Address {
        env.storage().persistent().get(&DataKey::Central).unwrap()
    }
}
