use crate::{
    error::ContractError,
    storage::{DataKey, ProtocolAddressType},
};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

/// # Liquidity Provider Settings Manager Contract
///
/// ## Overview
/// Manages protocol-wide settings, fees, and administrative functions for the LP system.
///
/// ## Key Responsibilities:
/// - Protocol fee configuration and management
/// - Treasury and relayer address management  
/// - Contract pausing/unpausing
/// - Administrative access control
///
/// ## Security Model:
/// - **Admin**: Full control over all settings
/// - **Relayer**: Authorized to execute settlements/refunds
/// - **Treasury**: Receives protocol fees
#[contract]
pub struct LPSettingManagerContract;

#[contractimpl]
impl LPSettingManagerContract {
    /// # Initialize Settings Contract
    ///
    /// ## Description:
    /// Sets up initial protocol configuration with admin, treasury, and relayer addresses.
    ///
    /// ## Default Configuration:
    /// - Protocol fee: 1% (1000 basis points)
    /// - Max BPS: 100,000 (100%)
    /// - Paused: false
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize initialization
    ///
    /// ## Parameters:
    /// - `treasury`: Address to receive protocol fees
    /// - `relayer_address`: Authorized address for settlement operations
    pub fn initialize(env: Env, admin: Address, treasury: Address, relayer_address: Address) {
        let storage = env.storage().persistent();

        if storage.get::<DataKey, Address>(&DataKey::Admin).is_some() {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage().persistent().set(&DataKey::Admin, &admin);

        env.storage()
            .persistent()
            .set(&DataKey::ProtocolFeePercent, &1000_i64); // 1% fee

        env.storage()
            .persistent()
            .set(&DataKey::MaxBps, &100_000_i64); // 100,000 = 100%

        env.storage().persistent().set(&DataKey::Paused, &false);

        env.storage()
            .persistent()
            .set(&DataKey::Treasury, &treasury);

        env.storage()
            .persistent()
            .set(&DataKey::Relayer, &relayer_address);
    }

    /// # Update Protocol Fee Percentage
    ///
    /// ## Description:
    /// Changes the protocol fee percentage applied to all orders.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the change
    ///
    /// ## Validation:
    /// - Fee must be between 0 and 100,000 basis points (0% to 100%)
    ///
    /// ## Events:
    /// - `("ProtocolFeeUpdated",)` with new fee percentage
    ///
    /// ## Parameters:
    /// - `protocol_fee_percent`: New fee in basis points (e.g., 1000 = 1%)
    pub fn update_protocol_fee(env: Env, protocol_fee_percent: i64) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // Validate fee is within reasonable bounds (0% to 100%)
        if !(0..=100_000).contains(&protocol_fee_percent) {
            return Err(ContractError::InvalidFeePercent);
        }

        env.storage()
            .persistent()
            .set(&DataKey::ProtocolFeePercent, &protocol_fee_percent);
        env.events()
            .publish(("ProtocolFeeUpdated",), protocol_fee_percent);
        Ok(())
    }

    /// # Internal: Update Protocol Address
    ///
    /// ## Description:
    /// Helper function to update protocol addresses with validation and event emission.
    ///
    /// ## Validation:
    /// - New address must be different from current address
    ///
    /// ## Events:
    /// - `("ProtocolAddressUpdated", address_type)` with new address
    fn update_address(
        env: &Env,
        key: DataKey,
        what: ProtocolAddressType,
        new_value: Address,
    ) -> Result<(), ContractError> {
        // Prevent setting the same address
        if let Some(current) = env.storage().persistent().get::<_, Address>(&key) {
            if current == new_value {
                return Err(ContractError::AddressAlreadySet);
            }
        }

        env.storage().persistent().set(&key, &new_value);
        env.events()
            .publish(("ProtocolAddressUpdated", what), new_value);
        Ok(())
    }

    /// # Update Protocol Addresses
    ///
    /// ## Description:
    /// Updates treasury or relayer addresses with proper validation.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the change
    ///
    /// ## Parameters:
    /// - `what`: Type of address to update (Treasury or Aggregator/Relayer)
    /// - `value`: New address value
    pub fn update_protocol_address(
        env: Env,
        what: ProtocolAddressType,
        value: Address,
    ) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        match what {
            ProtocolAddressType::Treasury => {
                Self::update_address(&env, DataKey::Treasury, what, value)
            }
            ProtocolAddressType::Aggregator => {
                Self::update_address(&env, DataKey::Relayer, what, value)
            }
        }
    }

    /// # Pause Contract Operations
    ///
    /// ## Description:
    /// Emergency function to pause all order creation and settlements.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the pause
    ///
    /// ## Events:
    /// - `("Paused",)` when contract is paused
    ///
    /// ## Note:
    /// - Prevents new order creation
    /// - Existing orders can still be settled/refunded
    pub fn pause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused, &true);
        env.events().publish(("Paused",), ());
        Ok(())
    }

    /// # Unpause Contract Operations
    ///
    /// ## Description:
    /// Resumes normal contract operations after a pause.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the unpause
    ///
    /// ## Events:
    /// - `("Unpaused",)` when contract is unpaused
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.events().publish(("Unpaused",), ());
        Ok(())
    }

    // ========== VIEW FUNCTIONS ==========

    /// # Get Current Fee Details
    ///
    /// ## Returns:
    /// - Tuple of (protocol_fee_percent, max_bps)
    /// - Default: (1000, 100000) = 1% fee
    pub fn get_fee_details(env: Env) -> (i64, i64) {
        let protocol_fee_percent: i64 = env
            .storage()
            .persistent()
            .get(&DataKey::ProtocolFeePercent)
            .unwrap_or(0);
        let max_bps: i64 = env
            .storage()
            .persistent()
            .get(&DataKey::MaxBps)
            .unwrap_or(100_000);
        (protocol_fee_percent, max_bps)
    }

    /// # Get Treasury Address
    ///
    /// ## Returns:
    /// - Current treasury address for fee collection
    pub fn get_treasury_address(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Treasury).unwrap()
    }

    /// # Get Relayer Address
    ///
    /// ## Returns:
    /// - Current relayer address authorized for settlements
    pub fn get_relayer_address(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Relayer).unwrap()
    }

    /// # Check if Contract is Paused
    ///
    /// ## Returns:
    /// - `true` if contract operations are paused
    /// - `false` if contract is operational
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// # Check if Token is Supported
    ///
    /// ## Description:
    /// Currently hardcoded to only support USDC. Can be extended for multi-token support.
    ///
    /// ## Returns:
    /// - `true` if token is supported for orders
    /// - `false` if token is not supported
    pub fn is_token_supported(env: Env, token: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::TokenSupported(token))
            .unwrap_or(false)
    }

    /// # Upgrade Settings Manager WASM
    ///
    /// ## Description:
    /// Updates the settings manager contract's WASM code.
    ///
    /// ## Authorization:
    /// - `admin`: Must authorize the upgrade
    ///
    /// ## Note:
    /// - Maintains all existing settings and state
    /// - Only admin can perform upgrades
    pub fn upgrade_lp_manager(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
