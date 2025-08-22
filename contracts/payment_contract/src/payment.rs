use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Bytes, Env, String,
};

#[contracttype]
struct Payment {
    // UUID from off-chain
    payment_id: Bytes,
    // UUID from Invoice table
    invoice_id: Bytes,
    // Customer Stellar address
    sender: Address,
    // Merchant/aggregator Stellar address
    receiver: Address,
    // USDC amount (in smallest unit)
    amount: i64,
    // pending, completed, failed
    status: String,
    // Stellar transaction hash
    stellar_tx_id: Bytes,
}

#[contract]
pub struct PaymentModule;

#[contractimpl]
impl PaymentModule {
    // Initialize contract with USDC asset
    pub fn initialize(env: Env, admin: Address, usdc_asset: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("usdc"), &usdc_asset);
    }

    // Process a USDC payment
    pub fn make_payment(
        env: Env,
        payment_id: Bytes,
        invoice_id: Bytes,
        sender: Address,
        receiver: Address,
        amount: i128,
    ) -> Bytes {
        // Verify sender's signature
        sender.require_auth();
        let usdc_asset: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("usdc"))
            .unwrap();

        // Transfer USDC using token client
        let token_client = token::Client::new(&env, &usdc_asset);
        token_client.transfer(&sender, &receiver, &amount);

        // Record payment
        let payment = (
            invoice_id.clone(),
            sender.clone(),
            receiver.clone(),
            amount,
            String::from_str(&env, "completed"),
        );
        env.storage().persistent().set(&payment_id, &payment);

        // Emit event
        env.events().publish(
            (symbol_short!("Processed"), payment_id.clone(), invoice_id),
            (amount, receiver),
        );

        payment_id
    }

    pub fn get_payment_status(
        env: Env,
        payment_id: Bytes,
    ) -> Option<(Bytes, Address, Address, i128, String)> {
        env.storage().persistent().get(&payment_id)
    }
}
