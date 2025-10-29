# Liquidity Provider System Documentation

## Overview

This document describes the **Liquidity Provider (LP) System**, a decentralized, blockchain-based solution built using the Soroban SDK for managing cross-chain liquidity provisioning. The system consists of two smart contracts: the **LP Setting Manager Contract** and the **Liquidity Provider Contract**. Together, they enable secure, non-custodial liquidity provisioning with a two-step settlement process, protocol fee management, and administrative controls.

The system is designed to:

- Facilitate cross-chain liquidity orders with secure fund management.
- Support partial settlements and guaranteed refunds.
- Provide administrative controls for protocol settings, such as fees and addresses.
- Ensure security through role-based access control (Admin, Relayer, Treasury).

This documentation explains the contracts' structure, functionality, workflows, and key data structures, making it suitable for developers, auditors, and stakeholders.

---

## LP Setting Manager Contract

### Overview

The **LP Setting Manager Contract** is responsible for managing protocol-wide settings, fees, and administrative functions. It acts as the configuration hub for the Liquidity Provider system, controlling parameters like protocol fees, treasury and relayer addresses, and contract pause states.

### Key Responsibilities

- **Protocol Fee Management**: Configures and updates the protocol fee percentage applied to orders.
- **Address Management**: Manages the treasury (fee recipient) and relayer (settlement/refund authorizer) addresses.
- **Contract Pausing**: Allows pausing/unpausing of operations for emergency control.
- **Administrative Control**: Restricts sensitive operations to authorized admin addresses.

### Security Model

- **Admin**: Has full control over settings, including fee updates, address changes, pausing, and contract upgrades.
- **Relayer**: Authorized to perform settlement and refund operations in the LP Contract.
- **Treasury**: Receives protocol fees deducted from orders.

### Key Functions

#### 1. Initialize (`initialize`)

- **Purpose**: Sets up the contract with initial configuration.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `admin`: The address with administrative privileges.
  - `treasury`: The address receiving protocol fees.
  - `relayer_address`: The address authorized for settlements and refunds.
- **Default Settings**:
  - Protocol fee: 1% (1000 basis points).
  - Maximum basis points (BPS): 100,000 (100%).
  - Contract paused: False.
- **Flow**:
  1. Admin authenticates the transaction.
  2. Stores admin, treasury, and relayer addresses in persistent storage.
  3. Sets default protocol fee and max BPS.
  4. Marks contract as unpaused.

#### 2. Update Protocol Fee (`update_protocol_fee`)

- **Purpose**: Changes the protocol fee percentage.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `protocol_fee_percent`: New fee in basis points (e.g., 1000 = 1%).
- **Validation**:
  - Fee must be between 0 and 100,000 BPS (0% to 100%).
- **Events**: Emits `ProtocolFeeUpdated` with the new fee percentage.
- **Flow**:
  1. Admin authenticates.
  2. Validates the new fee percentage.
  3. Updates the fee in persistent storage.
  4. Publishes an event to log the change.

#### 3. Update Protocol Addresses (`update_protocol_address`)

- **Purpose**: Updates treasury or relayer addresses.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `what`: Enum specifying `Treasury` or `Aggregator` (relayer).
  - `value`: New address.
- **Validation**: Ensures the new address differs from the current one.
- **Events**: Emits `ProtocolAddressUpdated` with the address type and new address.
- **Flow**:
  1. Admin authenticates.
  2. Validates the new address.
  3. Updates the address in persistent storage.
  4. Publishes an event.

#### 4. Pause (`pause`)

- **Purpose**: Pauses all order creation and settlements for emergency control.
- **Authorization**: Requires admin authentication.
- **Events**: Emits `Paused` event.
- **Flow**:
  1. Admin authenticates.
  2. Sets `Paused` to true in storage.
  3. Publishes the pause event.
- **Note**: Existing orders can still be settled or refunded.

#### 5. Unpause (`unpause`)

- **Purpose**: Resumes normal contract operations.
- **Authorization**: Requires admin authentication.
- **Events**: Emits `Unpaused` event.
- **Flow**:
  1. Admin authenticates.
  2. Sets `Paused` to false in storage.
  3. Publishes the unpause event.

#### 6. View Functions

- **Get Fee Details (`get_fee_details`)**: Returns the current protocol fee and max BPS (e.g., (1000, 100000) for 1% fee).
- **Get Treasury Address (`get_treasury_address`)**: Returns the current treasury address.
- **Get Relayer Address (`get_relayer_address`)**: Returns the current relayer address.
- **Check Paused (`is_paused`)**: Returns true if the contract is paused.
- **Check Token Support (`is_token_supported`)**: Checks if a token (currently only USDC) is supported.

#### 7. Upgrade (`upgrade_lp_manager`)

- **Purpose**: Updates the contract's WASM code for upgrades or fixes.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `new_wasm_hash`: Hash of the new WASM code.
- **Flow**:
  1. Admin authenticates.
  2. Updates the contract's WASM code while preserving state.

---

## Liquidity Provider Contract

### Overview

The **Liquidity Provider Contract** implements a two-step settlement system for cross-chain liquidity provisioning. It manages the lifecycle of liquidity orders, from creation to settlement or refund, using a non-custodial approach where funds are held in temporary wallets rather than the contract itself.

### Key Features

- **Two-Step Settlement**: Separates state updates (settlement/refunded status) from token transfers for security.
- **Non-Custodial**: Funds are held in temporary wallets, not the contract, until settlement or refund.
- **Partial Settlements**: Orders can be settled in chunks (e.g., 20% at a time).
- **Secure Refunds**: Guarantees refunds for failed orders with proper fee handling.
- **Replay Protection**: Uses nonces to prevent duplicate order submissions.

### Security Model

- **Sender**: Authorizes order creation and fund transfers to temporary wallets.
- **Relayer**: Authorizes state changes for settlements and refunds.
- **Temporary Wallet**: Authorizes final token transfers during settlement or refund execution.

### State Transitions

- **Created**: Order is created and funds are transferred to a temporary wallet.
- **Partially Settled**: Partial settlement reduces the order’s remaining amount.
- **Fully Settled**: Order is completely settled (current BPS = 0).
- **Refunded**: Order is marked for refund, and funds are returned to the refund address.
- **Note**: States are one-way; settled or refunded orders cannot revert.

### Key Functions

#### 1. Initialize (`init`)

- **Purpose**: Sets up the contract with required addresses.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `admin`: Administrative address.
  - `usdc_asset`: USDC token contract address.
  - `settings_contract`: LP Setting Manager Contract address.
- **Flow**:
  1. Admin authenticates.
  2. Stores USDC and settings contract addresses in persistent storage.

#### 2. Create Order (`create_order`)

- **Purpose**: Creates a new liquidity order and transfers funds to a temporary wallet.
- **Authorization**: Requires sender authentication for token transfer.
- **Parameters**:
  - `params`: Struct containing order details (order ID, amount, sender, token, etc.).
- **Validation**:
  - Contract must not be paused.
  - Amount must be positive.
  - Message hash must not be empty.
  - Order ID must be unique.
- **Events**: Emits `OrderCreated` with order details.
- **Flow**:
  1. Checks if the contract is paused.
  2. Validates input parameters.
  3. Transfers funds from sender to temporary wallet using the USDC token contract.
  4. Calculates protocol fee (1% default).
  5. Stores order details in persistent storage.
  6. Increments sender nonce for replay protection.
  7. Publishes the creation event.

#### 3. Settle (`settle`)

- **Purpose**: Updates order state for settlement (Step 1 of two-step process).
- **Authorization**: Requires relayer authentication.
- **Parameters**:
  - `order_id`: Unique order identifier.
  - `liquidity_provider`: Address receiving settled funds.
  - `settle_percent`: Percentage to settle (in BPS, e.g., 10000 = 10%).
- **Validation**:
  - Contract must not be paused.
  - Order must exist and not be fulfilled or refunded.
  - Settle percent must be between 0 and 100,000 BPS.
- **Events**: Emits `OrderSettled` with order ID, liquidity provider, and settle percent.
- **Flow**:
  1. Validates inputs and relayer authentication.
  2. Retrieves and validates the order.
  3. Calculates settlement amounts (liquidity provider amount, protocol fee).
  4. Updates order’s remaining amount and BPS.
  5. Marks order as fulfilled if fully settled.
  6. Stores pending settlement details.
  7. Publishes the settlement event.

#### 4. Execute Settlement Transfers (`execute_settlement_transfer`)

- **Purpose**: Executes token transfers for a settled order (Step 2).
- **Authorization**: Requires temporary wallet authentication.
- **Parameters**:
  - `order_id`: Unique order identifier.
- **Transfers**:
  - Protocol fee to treasury.
  - Remaining amount to liquidity provider.
- **Events**: Emits `SettlementTransferred` with order ID and settle percent.
- **Flow**:
  1. Validates order and pending settlement existence.
  2. Temporary wallet authenticates.
  3. Transfers protocol fee to treasury (if any).
  4. Transfers remaining amount to liquidity provider.
  5. Clears pending settlement to prevent re-execution.
  6. Publishes the transfer event.

#### 5. Refund (`refund`)

- **Purpose**: Marks an order for refund and calculates amounts (Step 1).
- **Authorization**: Requires relayer authentication.
- **Parameters**:
  - `order_id`: Unique order identifier.
  - `fee`: Protocol fee to deduct from refund.
- **Validation**:
  - Contract must not be paused.
  - Order must exist and not be fulfilled or refunded.
  - Fee must not exceed the order’s protocol fee.
- **Events**: Emits `OrderRefunded` with order ID and fee.
- **Flow**:
  1. Validates inputs and relayer authentication.
  2. Retrieves and validates the order.
  3. Calculates refund amount (order amount minus fee).
  4. Marks order as refunded and zeros out amounts.
  5. Stores pending refund details.
  6. Publishes the refund event.

#### 6. Execute Refund Transfers (`execute_refund_transfer`)

- **Purpose**: Executes token transfers for a refunded order (Step 2).
- **Authorization**: Requires temporary wallet authentication.
- **Parameters**:
  - `order_id`: Unique order identifier.
- **Transfers**:
  - Protocol fee to treasury (if any).
  - Remaining amount to refund address.
- **Events**: Emits `RefundTransferred` with order ID and refund amount.
- **Flow**:
  1. Validates order and pending refund existence.
  2. Temporary wallet authenticates.
  3. Transfers protocol fee to treasury (if any).
  4. Transfers remaining amount to refund address.
  5. Clears pending refund to prevent re-execution.
  6. Publishes the transfer event.

#### 7. Register LP Node (`register_lp_node`)

- **Purpose**: Registers a liquidity provider node with a specified capacity.
- **Parameters**:
  - `lp_node_id`: Unique identifier for the LP node.
  - `capacity`: Maximum order amount the node can handle.
- **Validation**:
  - Capacity must be positive.
  - LP node ID must be unique.
- **Events**: Emits `LpNodeRegistered` with node ID and capacity.
- **Flow**:
  1. Validates inputs.
  2. Creates an LP node struct with the specified capacity.
  3. Stores the node and updates the node ID map.
  4. Publishes the registration event.

#### 8. Upgrade (`upgrade_lp`)

- **Purpose**: Updates the contract’s WASM code.
- **Authorization**: Requires admin authentication.
- **Parameters**:
  - `new_wasm_hash`: Hash of the new WASM code.
- **Flow**:
  1. Admin authenticates.
  2. Updates the contract’s WASM code while preserving state.

#### 9. View Functions

- **Get Token Balance (`get_token_balance`)**: Returns the USDC balance of a user.
- **Get Order ID (`get_order_id`)**: Validates and returns an order ID.
- **Get Order Info (`get_order_info`)**: Returns complete order details.
- **Get Fee Details (`get_lp_fee_details`)**: Retrieves current fee details from the settings contract.

---

## Data Structures

### DataKey

- **Purpose**: Enum defining storage keys for persistent data.
- **Keys**:
  - `Admin`: Stores the admin address.
  - `SettingsContract`: Stores the settings contract address.
  - `NodeIDs`: Map of registered LP node IDs.
  - `Nonces`: Map of sender nonces for replay protection.
  - `Order(Bytes)`: Order data, keyed by order ID.
  - `Usdc`: USDC token contract address.
  - `PendingSettlement(Bytes)`: Pending settlement data, keyed by order ID.
  - `PendingRefund(Bytes)`: Pending refund data, keyed by order ID.

### LpNode

- **Purpose**: Represents a liquidity provider node.
- **Fields**:
  - `capacity`: Maximum order amount the node can handle.

### PendingSettlement

- **Purpose**: Stores settlement details between state update and transfer.
- **Fields**:
  - `order_id`: Order identifier.
  - `protocol_fee`: Amount to send to treasury.
  - `transfer_amount`: Amount to send to liquidity provider.
  - `liquidity_provider`: Recipient address.
  - `settle_percent`: Settled percentage (in BPS).

### PendingRefund

- **Purpose**: Stores refund details between state update and transfer.
- **Fields**:
  - `order_id`: Order identifier.
  - `fee`: Protocol fee deducted from refund.
  - `refund_amount`: Amount to refund to sender.

### OrderParams

- **Purpose**: Input parameters for creating a liquidity order.
- **Fields**:
  - `order_id`: Unique order identifier.
  - `token`: Token contract address (currently USDC).
  - `sender`: Order creator.
  - `amount`: Order amount in token units.
  - `rate`: Exchange rate for the order.
  - `temporary_wallet_address`: Wallet holding funds.
  - `refund_address`: Recipient for refunds.
  - `message_hash`: Cross-chain message or metadata.

### Order

- **Purpose**: Tracks the complete state of a liquidity order.
- **Fields**:
  - `order_id`: Unique identifier.
  - `sender`: Order creator.
  - `token`: Token contract address.
  - `temporary_wallet_address`: Wallet holding funds.
  - `protocol_fee`: Calculated fee for the order.
  - `is_fulfilled`: True if fully settled.
  - `is_refunded`: True if refunded.
  - `refund_address`: Refund recipient.
  - `current_bps`: Remaining basis points (100,000 = 100%).
  - `amount`: Remaining order amount.
  - `rate`: Exchange rate.
  - `message_hash`: Cross-chain metadata.

---

## Operational Flow

### Order Creation

1. **User Action**: A sender submits an order with parameters (order ID, amount, temporary wallet, etc.).
2. **Validation**: The contract checks:
   - Contract is not paused.
   - Amount is positive, message hash is valid, order ID is unique.
3. **Fund Transfer**: USDC tokens are transferred from the sender to the temporary wallet.
4. **State Update**: Order details are stored, sender nonce is incremented, and an `OrderCreated` event is emitted.

### Settlement Process

1. **Step 1: State Update (`settle`)**:
   - Relayer authorizes the settlement.
   - Contract validates the order and settle percent.
   - Calculates amounts for the liquidity provider and protocol fee.
   - Updates order state (amount, BPS, fulfilled status).
   - Stores pending settlement and emits `OrderSettled` event.
2. **Step 2: Token Transfer (`execute_settlement_transfer`)**:
   - Temporary wallet authorizes transfers.
   - Transfers protocol fee to treasury and remaining amount to liquidity provider.
   - Clears pending settlement and emits `SettlementTransferred` event.

### Refund Process

1. **Step 1: State Update (`refund`)**:
   - Relayer authorizes the refund.
   - Contract validates the order and fee.
   - Marks order as refunded, zeros amounts, and stores pending refund.
   - Emits `OrderRefunded` event.
2. **Step 2: Token Transfer (`execute_refund_transfer`)**:
   - Temporary wallet authorizes transfers.
   - Transfers protocol fee to treasury and remaining amount to refund address.
   - Clears pending refund and emits `RefundTransferred` event.

### Administrative Actions

- **Fee Updates**: Admin updates protocol fee via `update_protocol_fee`.
- **Address Updates**: Admin updates treasury or relayer addresses via `update_protocol_address`.
- **Pause/Unpause**: Admin pauses or unpauses operations via `pause` or `unpause`.
- **Contract Upgrade**: Admin upgrades contract WASM code via `upgrade_lp` or `upgrade_lp_manager`.

---

## Security Considerations

- **Non-Custodial Design**: Funds are held in temporary wallets, reducing contract risk.
- **Two-Step Process**: Separates state updates from transfers, ensuring atomicity.
- **Role-Based Access**: Admin, relayer, and temporary wallet roles restrict actions.
- **Replay Protection**: Sender nonces prevent duplicate order submissions.
- **Event Logging**: All state changes emit events for transparency and auditing.
- **Pause Mechanism**: Allows emergency halting of operations without affecting existing orders.

---

