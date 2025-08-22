use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum ContractError {
    InvalidEscrowState = 1,
    TimeoutNotReached = 2,
}
