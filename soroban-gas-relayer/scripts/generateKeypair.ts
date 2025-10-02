import { Keypair } from "@stellar/stellar-sdk";
import fs from "fs-extra";
import dotenv from "dotenv";

dotenv.config();

/**
 * Generates a new Stellar keypair for sponsoring transactions
 */
async function generateSponsorKeypair() {
  try {
    // Generate a new random keypair
    const keypair = Keypair.random();

    console.log("Generated new Stellar keypair:");
    console.log("Public Key:", keypair.publicKey());
    console.log("Secret Key:", keypair.secret());

    // Save to .env file format
    const envContent = `
# Soroban Gas Relayer Configuration
SPONSOR_SECRET_KEY=${keypair.secret()}
SPONSOR_PUBLIC_KEY=${keypair.publicKey()}

# Network Configuration (choose one)
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_FUTURENET_RPC_URL=https://rpc-futurenet.stellar.org
SOROBAN_MAINNET_RPC_URL=https://soroban-rpc.mainnet.stellar.org

# Server Configuration
PORT=5000
CM_DOMAIN=https://your-frontend-domain.com
DOMAIN_2=https://your-other-domain.com
`;

    // Write to .env.example file
    fs.writeFileSync("./.env.example", envContent.trim());

    console.log("\n✅ Keypair generated successfully!");
    console.log("📝 Configuration saved to .env.example");
    console.log("\n⚠️  IMPORTANT:");
    console.log("1. Copy .env.example to .env");
    console.log("2. Fund the sponsor account with XLM for transaction fees");
    console.log("3. Never share your secret key!");

    // For testnet, provide friendbot funding instructions
    console.log("\n🤖 To fund on testnet, visit:");
    console.log(`https://friendbot.stellar.org?addr=${keypair.publicKey()}`);
  } catch (error) {
    console.error("❌ Error generating keypair:", error);
    process.exit(1);
  }
}

// Run the function
generateSponsorKeypair()
  .then(() => {
    console.log("\n🎉 Setup complete!");
    process.exit(0);
  })
  .catch((error) => {
    console.error("Fatal error:", error);
    process.exit(1);
  });
