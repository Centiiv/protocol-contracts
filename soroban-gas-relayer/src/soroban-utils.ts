import {
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  Account,
  Contract,
  Address,
  xdr,
  BASE_FEE,
  scValToNative,
  nativeToScVal,
} from "@stellar/stellar-sdk";
import { NetworkConfig, OrderParams, TransactionResult } from "./types";
import fetch from "node-fetch";

(global as any).fetch = fetch;
export class SorobanUtils {
  static NETWORKS: Record<string, NetworkConfig> = {
    TESTNET: {
      name: "testnet",
      rpcUrl:
        process.env.SOROBAN_RPC_URL || "https://soroban-testnet.stellar.org",
      networkPassphrase: Networks.TESTNET,
      horizonUrl: "https://horizon-testnet.stellar.org",
    },
    FUTURENET: {
      name: "futurenet",
      rpcUrl:
        process.env.SOROBAN_FUTURENET_RPC_URL ||
        "https://rpc-futurenet.stellar.org",
      networkPassphrase: Networks.FUTURENET,
      horizonUrl: "https://horizon-futurenet.stellar.org",
    },
    MAINNET: {
      name: "mainnet",
      rpcUrl:
        process.env.SOROBAN_MAINNET_RPC_URL ||
        "https://soroban-mainnet.stellar.org",
      networkPassphrase: Networks.PUBLIC,
      horizonUrl: "https://horizon.stellar.org",
    },
  };
  static getNetworkConfig(networkName: string = "TESTNET"): NetworkConfig {
    const network = this.NETWORKS[networkName.toUpperCase()];
    if (!network) {
      throw new Error(`Unsupported network: ${networkName}`);
    }
    return network;
  }

  static getSponsorKeypair(): Keypair {
    const sponsorSecret = process.env.SPONSOR_SECRET_KEY;
    if (!sponsorSecret) {
      throw new Error("SPONSOR_SECRET_KEY not found in environment variables");
    }
    return Keypair.fromSecret(sponsorSecret);
  }

  static getRpcServer(networkName: string = "TESTNET"): SorobanRpc.Server {
    const config = this.getNetworkConfig(networkName);
    return new SorobanRpc.Server(config.rpcUrl);
  }

  static convertToScVal(value: any, type: any) {
    try {
      switch (type) {
        case "address":
          if (typeof value !== "string" || value.length === 0) {
            throw new Error("Invalid address value provided");
          }
          return nativeToScVal(new Address(value), { type: "address" });

        case "bytes":
          if (typeof value !== "string") {
            throw new Error(
              `Invalid bytes value: must be a string, got ${value}`,
            );
          }
          const hexString = value.startsWith("0x") ? value.slice(2) : value;
          if (!/^[0-9a-fA-F]{64}$/.test(hexString)) {
            throw new Error(
              `Invalid bytes value: must be a 32-byte hex string (64 chars), got ${hexString}`,
            );
          }
          return xdr.ScVal.scvBytes(Buffer.from(hexString, "hex"));

        case "i128":
          const bigIntValue = typeof value === "bigint" ? value : BigInt(value);
          return nativeToScVal(bigIntValue, { type: "i128" });

        case "i64":
          const bigIntI64Value =
            typeof value === "bigint" ? value : BigInt(value);

          const MIN_I64 = BigInt("-9223372036854775808");
          const MAX_I64 = BigInt("9223372036854775807");

          if (bigIntI64Value < MIN_I64 || bigIntI64Value > MAX_I64) {
            throw new Error(`i64 value out of range: ${bigIntI64Value}`);
          }

          if (
            bigIntI64Value >= BigInt(Number.MIN_SAFE_INTEGER) &&
            bigIntI64Value <= BigInt(Number.MAX_SAFE_INTEGER)
          ) {
            return nativeToScVal(Number(bigIntI64Value), { type: "i64" });
          } else {
            return nativeToScVal(bigIntI64Value, { type: "i64" });
          }

        case "string":
          if (typeof value !== "string") {
            throw new Error(
              `Invalid string value: must be a string, got ${value}`,
            );
          }
          return xdr.ScVal.scvString(value);

        default:
          throw new Error(`Unsupported type: ${type}`);
      }
    } catch (error: any) {
      throw new Error(
        `Failed to convert value to ScVal (type: ${type}, value: ${value}): ${error.message}`,
      );
    }
  }

  static parseOrderParams(params: OrderParams): xdr.ScVal {
    return xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("amount"),
        val: this.convertToScVal(params.amount, "i128"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("message_hash"),
        val: this.convertToScVal(params.message_hash, "string"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("order_id"),
        val: this.convertToScVal(params.order_id, "bytes"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("rate"),
        val: this.convertToScVal(params.rate, "i64"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("refund_address"),
        val: this.convertToScVal(params.refund_address, "address"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("sender"),
        val: this.convertToScVal(params.sender, "address"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("sender_fee"),
        val: this.convertToScVal(params.sender_fee, "i128"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("sender_fee_recipient"),
        val: this.convertToScVal(params.sender_fee_recipient, "address"),
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("token"),
        val: this.convertToScVal(params.token, "address"),
      }),
    ]);
  }

  static async buildAndSubmitSponsoredTransaction(
    contractAddress: string,
    functionName: string,
    args: xdr.ScVal[],
    sourceAccount: string,
    networkName: string = "TESTNET",
  ): Promise<TransactionResult> {
    try {
      const networkConfig = this.getNetworkConfig(networkName);
      const server = this.getRpcServer(networkName);
      const sponsorKeypair = this.getSponsorKeypair();

      console.log(`🔗 Using RPC: ${networkConfig.rpcUrl}`);
      console.log(`🔐 Sponsor: ${sponsorKeypair.publicKey()}`);

      const sponsorAccount = await server.getAccount(
        sponsorKeypair.publicKey(),
      );

      const contract = new Contract(contractAddress);

      const transaction = new TransactionBuilder(sponsorAccount, {
        fee: BASE_FEE,
        networkPassphrase: networkConfig.networkPassphrase,
      })
        .addOperation(contract.call(functionName, ...args))
        .setTimeout(300)
        .build();

      console.log("🔄 Simulating transaction...");
      const simulateResponse = await server.simulateTransaction(transaction);

      if (SorobanRpc.Api.isSimulationError(simulateResponse)) {
        const errorMsg = `Simulation failed: ${JSON.stringify(simulateResponse.error)}`;
        console.error("❌", errorMsg);
        throw new Error(errorMsg);
      }

      if (!simulateResponse.result) {
        throw new Error("Simulation returned no result");
      }

      console.log("✅ Simulation successful");

      const preparedTransaction = await server.prepareTransaction(transaction);
      preparedTransaction.sign(sponsorKeypair);

      console.log("🚀 Submitting transaction...");
      const sendResponse = await server.sendTransaction(preparedTransaction);

      if (sendResponse.status === "ERROR") {
        const errorMsg = `Submission failed: ${JSON.stringify(sendResponse.errorResult)}`;
        console.error("❌", errorMsg);
        throw new Error(errorMsg);
      }

      console.log("✅ Transaction submitted, hash:", sendResponse.hash);

      const result = await this.reliableTransactionCheck(
        sendResponse.hash,
        networkName,
        server,
        networkConfig,
      );

      return result;
    } catch (error: any) {
      console.error("💥 Transaction error:", error);
      return {
        success: false,
        message: error.message || "Transaction failed",
        network: networkName.toLowerCase(),
      };
    }
  }

  static async reliableTransactionCheck(
    txHash: string,
    networkName: string,
    server: SorobanRpc.Server,
    networkConfig: NetworkConfig,
  ): Promise<TransactionResult> {
    const maxAttempts = 15;
    const explorerUrl = `https://stellar.expert/explorer/testnet/tx/${txHash}`;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      console.log(`⏳ Checking status (${attempt}/${maxAttempts})...`);

      try {
        const horizonResult = await this.safeHorizonCheck(
          txHash,
          networkConfig.horizonUrl!,
        );
        if (horizonResult.success !== undefined) {
          return {
            ...horizonResult,
            explorerUrl: explorerUrl,
          };
        }
      } catch (horizonError: any) {
        console.log(`⚠️  Horizon check failed: ${horizonError.message}`);
      }

      if (attempt < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, 3000));
      }
    }

    return {
      success: true,
      txHash: txHash,
      message: "Transaction submitted - check explorer for confirmation",
      network: networkName.toLowerCase(),
      explorerUrl: explorerUrl,
    };
  }

  static async safeHorizonCheck(
    txHash: string,
    horizonUrl: string,
  ): Promise<any> {
    try {
      const response = await fetch(`${horizonUrl}/transactions/${txHash}`);

      if (response.status === 404) {
        return { success: undefined };
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const txData = await response.json();
      console.log(
        `🌐 Horizon Status: ${txData.successful ? "SUCCESS" : "FAILED"}`,
      );

      return {
        success: txData.successful,
        txHash: txHash,
        message: txData.successful
          ? "Transaction successful (Horizon)"
          : `Transaction failed: ${txData.result_codes?.transaction || "Unknown error"}`,
      };
    } catch (error: any) {
      throw new Error(`Horizon API error: ${error.message}`);
    }
  }

  static async callViewFunction(
    contractAddress: string,
    functionName: string,
    args: xdr.ScVal[],
    networkName: string = "TESTNET",
  ): Promise<any> {
    const server = this.getRpcServer(networkName);
    const networkConfig = this.getNetworkConfig(networkName);

    const contract = new Contract(contractAddress);
    const dummyKeypair = Keypair.random();

    const transaction = new TransactionBuilder(
      new Account(dummyKeypair.publicKey(), "0"),
      {
        fee: BASE_FEE,
        networkPassphrase: networkConfig.networkPassphrase,
      },
    )
      .addOperation(contract.call(functionName, ...args))
      .setTimeout(30)
      .build();

    const simulateResponse = await server.simulateTransaction(transaction);

    if (SorobanRpc.Api.isSimulationError(simulateResponse)) {
      throw new Error(`Simulation failed: ${simulateResponse.error}`);
    }

    if (!simulateResponse.result) {
      throw new Error("Simulation returned no result");
    }

    const result = scValToNative(simulateResponse.result.retval);

    return this.convertBigIntToString(result);
  }

  static convertBigIntToString(obj: any): any {
    if (obj === null || obj === undefined) {
      return obj;
    }

    if (typeof obj === "bigint") {
      return obj.toString();
    }

    if (Array.isArray(obj)) {
      return obj.map((item) => this.convertBigIntToString(item));
    }

    if (typeof obj === "object") {
      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        result[key] = this.convertBigIntToString(value);
      }
      return result;
    }

    return obj;
  }
}
