use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum EscrowError {
    InvalidEscrowState = 1,
    TimeoutNotReached = 2,
}
