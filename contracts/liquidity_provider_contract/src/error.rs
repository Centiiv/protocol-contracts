use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum ContractError {
    InvlidLpNodeParameters = 1,
    LpNodeIdAlreadyExists = 2,
    AmountMustBePositive = 3,
    UnauthorizedLpNode = 4,
    RequestNotPending = 5,
}
