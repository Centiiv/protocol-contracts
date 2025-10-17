use soroban_sdk::{contracttype, String};
use soroban_sdk::{Address, Bytes};

/// # Storage Data Keys
///
/// ## Overview:
/// Enum defining all storage keys used in the LP contract.
///
/// ## Note:
/// - Keys are namespaced to prevent collisions
/// - Order-specific keys include order_id for isolation
/// - Persistent storage used for all data
#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    /// Contract administrator address
    Admin,
    /// Address of the settings manager contract
    SettingsContract,
    /// Map of registered LP node IDs to their existence flags
    NodeIDs,
    /// Nonces for each sender to prevent replay attacks
    Nonces,
    /// Order data storage, keyed by order_id
    Order(Bytes),
    /// USDC token contract address
    Usdc,
    /// Pending settlement data, keyed by order_id
    PendingSettlement(Bytes),
    /// Pending refund data, keyed by order_id
    PendingRefund(Bytes),
}

/// # Liquidity Provider Node
///
/// ## Description:
/// Represents a registered liquidity provider node in the system.
///
/// ## Fields:
/// - `capacity`: Maximum order amount this node can handle
///
/// ## Usage:
/// Used to track and manage LP node capabilities and limits.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LpNode {
    pub capacity: i128,
}

/// # Pending Settlement Data
///
/// ## Description:
/// Stores calculated settlement amounts between state update and transfer execution.
///
/// ## Note:
/// - Temporary storage cleared after transfer execution
/// - Prevents double-spending of settlement amounts
/// - Ensures atomic transfer execution
///
/// ## Fields:
/// - `order_id`: Associated order identifier
/// - `protocol_fee`: Amount to send to treasury
/// - `transfer_amount`: Amount to send to liquidity provider  
/// - `liquidity_provider`: Recipient of settled funds
/// - `settle_percent`: Percentage of order that was settled
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingSettlement {
    pub order_id: Bytes,
    pub protocol_fee: i128,
    pub transfer_amount: i128,
    pub liquidity_provider: Address,
    pub settle_percent: i128,
}

/// # Pending Refund Data
///
/// ## Description:
/// Stores calculated refund amounts between state update and transfer execution.
///
/// ## Note:
/// - Temporary storage cleared after transfer execution
/// - Ensures refund amounts match state changes
/// - Prevents partial refund execution
///
/// ## Fields:
/// - `order_id`: Associated order identifier
/// - `fee`: Protocol fee deducted from refund
/// - `refund_amount`: Net amount to refund to sender
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRefund {
    pub order_id: Bytes,
    pub fee: i128,
    pub refund_amount: i128,
}

/// # Order Creation Parameters
///
/// ## Description:
/// Input parameters for creating a new liquidity order.
///
/// ## Validation:
/// - `order_id` must be unique
/// - `amount` must be positive
/// - `message_hash` must not be empty
/// - All addresses must be valid
///
/// ## Fields:
/// - `order_id`: Unique identifier for the order (32 bytes recommended)
/// - `token`: Token contract address (currently only USDC supported)
/// - `sender`: Order creator who provides funds
/// - `amount`: Order amount in token units
/// - `rate`: Exchange rate for the order (implementation specific)
/// - `temporary_wallet_address`: Non-custodial wallet holding order funds
/// - `refund_address`: Recipient for refunds if order fails
/// - `message_hash`: Cross-chain message identifier or order metadata
#[contracttype]
#[derive(Clone, Debug)]
pub struct OrderParams {
    pub order_id: Bytes,
    pub token: Address,
    pub sender: Address,
    pub amount: i128,
    pub rate: i64,
    pub temporary_wallet_address: Address,
    pub refund_address: Address,
    pub message_hash: String,
}

/// # Order State
///
/// ## Description:
/// Complete order state tracking settlement progress and status.
///
/// ## State Machine:
/// - Created → [Partially Settled] → Fully Settled
/// - Created → Refunded
/// - States are mutually exclusive and one-way
///
/// ## Note:
/// - `is_fulfilled` and `is_refunded` are mutually exclusive
/// - `current_bps` ensures settlement percentage calculations are accurate
/// - `protocol_fee` is calculated once at order creation for predictability
///
/// ## Fields:
/// - `order_id`: Unique order identifier
/// - `sender`: Order creator
/// - `token`: Token contract address
/// - `temporary_wallet_address`: Wallet holding order funds
/// - `protocol_fee`: Calculated protocol fee for entire order
/// - `is_fulfilled`: True when order is completely settled
/// - `is_refunded`: True when order has been refunded
/// - `refund_address`: Fallback recipient for refunds
/// - `current_bps`: Remaining basis points (100,000 = 100% remaining)
/// - `amount`: Remaining order amount to be settled
/// - `rate`: Order exchange rate
/// - `message_hash`: Cross-chain or order metadata
#[contracttype]
#[derive(Clone, Debug)]
pub struct Order {
    pub order_id: Bytes,
    pub sender: Address,
    pub token: Address,
    pub temporary_wallet_address: Address,
    pub protocol_fee: i128,
    pub is_fulfilled: bool,
    pub is_refunded: bool,
    pub refund_address: Address,
    pub current_bps: i128,
    pub amount: i128,
    pub rate: i64,
    pub message_hash: String,
}
