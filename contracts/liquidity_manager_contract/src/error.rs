use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvalidFeePercent = 1,
    ZeroAddress = 2,
    AddressAlreadySet = 3,
    InvalidParameter = 4,
    Unauthorized = 5,
}
