use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    //<<<<<<< Updated upstream
    //    InvlidLpNodeParameters = 1,
    //    LpNodeIdAlreadyExists = 2,
    //    AmountMustBePositive = 3,
    //    UnauthorizedLpNode = 4,
    //    RequestNotPending = 5,
    //    NoSuitableLPNode = 6,
    //    InvalidLPNode = 7,
    //    UnsupportedAlgorithm = 8,
    //    RequestNotFound = 9,
    //    InvalidRequest = 10,
    //    NotFound = 11,
    //    NotAuthorized = 12,
    //    RequestIdAlreadyExists = 13,
    //    InvalidEarnings = 14,
    //    InvalidAmount = 15,
    //=======
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
    InvalidFeePercent = 13,
    AddressAlreadySet = 14,
    InvalidParameter = 15,
    InvalidLpNodeParameters = 16,
    LpNodeIdAlreadyExists = 17,
    NotFound = 18,
}
