use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum ContractError {
    InvlidLpNodeParameters = 1,
    LpNodeIdAlreadyExists = 2,
    AmountMustBePositive = 3,
    UnauthorizedLpNode = 4,
    RequestNotPending = 5,
    NoSuitableLPNode = 6,
    InvalidLPNode = 7,
    UnsupportedAlgorithm = 8,
}
