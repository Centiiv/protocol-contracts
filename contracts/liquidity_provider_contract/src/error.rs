use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvalidAmount = 1,
    ZeroAddress = 2,
    InvalidSenderFeeRecipient = 3,
    InvalidMessageHash = 4,
    InvalidSettlePercent = 5,
    OrderAlreadyExists = 6,
    OrderNotFound = 7,
    OrderFulfilled = 8,
    OrderRefunded = 9,
    FeeExceedsProtocolFee = 10,
    Paused = 11,
    Unauthorized = 12,
    TransferFailed = 13,
    AddressAlreadySet = 14,
    InvalidParameter = 15,
    InvalidLpNodeParameters = 16,
    LpNodeIdAlreadyExists = 17,
    SettingsContractNotSet = 18,
    UsdcNotSet = 19,
}
