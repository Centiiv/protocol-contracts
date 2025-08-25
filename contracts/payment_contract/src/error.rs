use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum PaymentContractError {
    PaymentIDAlreadyExists = 1,
    AmountMustBePositive = 2,
}
