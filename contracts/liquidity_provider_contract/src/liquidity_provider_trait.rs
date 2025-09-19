use soroban_sdk::{token, Address, Bytes, Env};

use crate::{
    error::ContractError,
    storage_types::{Order, OrderParams},
};

pub trait IGateway {
    //fn initialize(env: Env, admin: Address, usdc_asset: Address, settings_contract: Address);
    //
    //fn register_lp_node(env: Env, lp_node_id: Bytes, capacity: i128) -> Result<(), ContractError>;
    fn get_token_balance(env: Env, user: Address) -> i128;
    fn create_order(env: Env, order_params: OrderParams) -> Result<(), ContractError>;

    fn settle(
        env: Env,
        split_order_id: Bytes,
        order_id: Bytes,
        liquidity_provider: Address,
        settle_percent: i128,
    ) -> Result<bool, ContractError>;

    fn get_order_id(env: Env, order_id: Bytes) -> Result<Bytes, ContractError>;

    fn refund(env: Env, order_id: Bytes, fee: i128) -> Result<(), ContractError>;

    fn get_order_info(env: Env, order_id: Bytes) -> Result<Order, ContractError>;

    fn get_fee_details(env: Env) -> (i64, i64);
}
