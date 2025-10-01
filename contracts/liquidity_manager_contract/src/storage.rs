use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolAddressType {
    Treasury,
    Aggregator,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Treasury,
    Relayer,
    //Aggregator,
    ProtocolFeePercent,
    MaxBps,
    Paused,
    TokenSupported(Address),
}
