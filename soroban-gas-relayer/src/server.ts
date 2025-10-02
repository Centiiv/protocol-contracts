import express from "express";
import cors from "cors";
import morgan from "morgan";
import dotenv from "dotenv";
import rateLimit from "express-rate-limit";
import { SorobanUtils } from "./soroban-utils";
import { Address } from "@stellar/stellar-sdk";

dotenv.config();

const app = express();

const corsOptions: cors.CorsOptions = {
  origin: [
    "http://localhost:3000",
    process.env.CM_DOMAIN,
    process.env.DOMAIN_2,
  ].filter((d): d is string => Boolean(d)),
  methods: ["GET", "POST", "OPTIONS"],
  allowedHeaders: ["Content-Type", "Authorization", "X-API-Key"],
  credentials: true,
};

app.use(cors(corsOptions));
app.options("*", cors(corsOptions));
app.use(express.json({ limit: "50mb" }));
app.use(express.urlencoded({ extended: true, limit: "50mb" }));
app.use(morgan("combined"));

const limiter = rateLimit({
  windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || "900000", 10),
  max: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || "100", 10),
  message: {
    success: false,
    message: "Too many requests, please try again later.",
  },
});
app.use(limiter);

const apiKeyMiddleware = (
  req: express.Request,
  res: express.Response,
  next: express.NextFunction,
): void => {
  const apiKey = req.headers["x-api-key"] as string | undefined;

  if (process.env.API_KEY && apiKey !== process.env.API_KEY) {
    void res.status(401).json({
      success: false,
      message: "Invalid API key",
    });
    return;
  }

  next();
};

app.use(apiKeyMiddleware);

app.get("/", (_req, res) => {
  res.json({
    success: true,
    message: "🚀 Soroban Gas Relayer is running",
    sponsor: process.env.SPONSOR_PUBLIC_KEY,
    networks: ["TESTNET", "FUTURENET", "MAINNET"],
  });
});

app.post("/createOrder", async (req, res) => {
  try {
    const { contractAddress, orderParams, networkName = "TESTNET" } = req.body;

    console.log("Request body:", JSON.stringify(req.body, null, 2));

    const {
      sender,
      order_id,
      token,
      amount,
      sender_fee_recipient,
      sender_fee,
      refund_address,
      rate,
      message_hash,
    } = orderParams;

    if (!sender || !order_id || !token || amount === undefined) {
      return res.status(400).json({
        success: false,
        message:
          "Missing required order parameters (sender, order_id, token, amount)",
      });
    }

    const validateAddress = (addr: any, fieldName: any) => {
      if (
        typeof addr !== "string" ||
        (!addr.startsWith("G") && !addr.startsWith("C")) ||
        addr.length !== 56
      ) {
        throw new Error(`Invalid ${fieldName} address: ${addr}`);
      }
      try {
        new Address(addr);
      } catch (error: any) {
        throw new Error(
          `Invalid ${fieldName} address format: ${error.message}`,
        );
      }
    };
    validateAddress(sender, "sender");
    validateAddress(token, "token");
    validateAddress(sender_fee_recipient || sender, "sender_fee_recipient");
    validateAddress(refund_address || sender, "refund_address");

    const hexRegex = /^(0x)?[0-9a-fA-F]{64}$/;
    if (typeof order_id !== "string" || !hexRegex.test(order_id)) {
      throw new Error(
        `Invalid order_id: must be a 32-byte hex string (64 chars), got ${order_id}`,
      );
    }

    const validateBigInt = (
      value: any,
      fieldName: any,
      minValue = -Infinity,
    ) => {
      try {
        const bigIntValue = BigInt(value);
        if (bigIntValue < minValue) {
          throw new Error(`${fieldName} must be >= ${minValue}`);
        }
        return bigIntValue;
      } catch {
        throw new Error(
          `Invalid ${fieldName}: must be a valid number, got ${value}`,
        );
      }
    };

    const amountBigInt = validateBigInt(amount, "amount", 1);

    const senderFeeBigInt = validateBigInt(sender_fee || 0, "sender_fee", 0);

    const rateBigInt = validateBigInt(rate || 0, "rate");

    if (typeof message_hash !== "string" || message_hash.length === 0) {
      throw new Error("Invalid message_hash: must be a non-empty string");
    }

    const params = {
      sender,
      order_id,
      token,
      amount: amountBigInt,
      sender_fee_recipient: sender_fee_recipient || sender,
      sender_fee: senderFeeBigInt,
      refund_address: refund_address || sender,
      rate: rateBigInt,
      message_hash,
    };

    const orderParamsScVal = SorobanUtils.parseOrderParams(params);

    const result = await SorobanUtils.buildAndSubmitSponsoredTransaction(
      contractAddress,
      "create_order",
      [orderParamsScVal],
      sender,
      networkName,
    );

    return res.status(result.success ? 200 : 500).json(result);
  } catch (error: any) {
    console.error("Create order error:", error);
    return res.status(500).json({
      success: false,
      message: error.message || "Internal server error",
    });
  }
});

app.post("/settle", async (req, res): Promise<void> => {
  try {
    const {
      contractAddress,
      orderId,
      liquidityProvider,
      settlePercent,
      caller,
      networkName = "TESTNET",
    } = req.body;

    if (
      !contractAddress ||
      !orderId ||
      !liquidityProvider ||
      settlePercent === undefined ||
      !caller
    ) {
      return void res.status(400).json({
        success: false,
        message: "Missing required parameters",
      });
    }

    const args = [
      SorobanUtils.convertToScVal(orderId, "bytes"),
      SorobanUtils.convertToScVal(liquidityProvider, "address"),
      SorobanUtils.convertToScVal(BigInt(settlePercent), "i128"),
    ];

    const result = await SorobanUtils.buildAndSubmitSponsoredTransaction(
      contractAddress,
      "settle",
      args,
      caller,
      networkName,
    );

    return void res.status(result.success ? 200 : 500).json(result);
  } catch (error: any) {
    console.error("Settle error:", error);
    return void res.status(500).json({
      success: false,
      message: error?.message || "Internal server error",
    });
  }
});

app.post("/refund", async (req, res): Promise<void> => {
  try {
    const {
      contractAddress,
      orderId,
      fee,
      caller,
      networkName = "TESTNET",
    } = req.body;

    if (!contractAddress || !orderId || fee === undefined || !caller) {
      return void res.status(400).json({
        success: false,
        message: "Missing required parameters",
      });
    }

    const args = [
      SorobanUtils.convertToScVal(orderId, "bytes"),
      SorobanUtils.convertToScVal(BigInt(fee), "i128"),
    ];

    const result = await SorobanUtils.buildAndSubmitSponsoredTransaction(
      contractAddress,
      "refund",
      args,
      caller,
      networkName,
    );

    return void res.status(result.success ? 200 : 500).json(result);
  } catch (error: any) {
    console.error("Refund error:", error);
    return void res.status(500).json({
      success: false,
      message: error?.message || "Internal server error",
    });
  }
});

app.post("/registerLpNode", async (req, res): Promise<void> => {
  try {
    const {
      contractAddress,
      lpNodeId,
      capacity,
      caller,
      networkName = "TESTNET",
    } = req.body;

    if (!contractAddress || !lpNodeId || capacity === undefined || !caller) {
      return void res.status(400).json({
        success: false,
        message: "Missing required parameters",
      });
    }

    const args = [
      SorobanUtils.convertToScVal(lpNodeId, "bytes"),
      SorobanUtils.convertToScVal(BigInt(capacity), "i128"),
    ];

    const result = await SorobanUtils.buildAndSubmitSponsoredTransaction(
      contractAddress,
      "register_lp_node",
      args,
      caller,
      networkName,
    );

    return void res.status(result.success ? 200 : 500).json(result);
  } catch (error: any) {
    console.error("Register LP node error:", error);
    return void res.status(500).json({
      success: false,
      message: error?.message || "Internal server error",
    });
  }
});

app.get(
  "/getOrderById/:contractAddress/:orderId",
  async (req, res): Promise<void> => {
    try {
      const { contractAddress, orderId } = req.params;
      const { networkName = "TESTNET" } = req.query as { networkName?: string };

      if (!contractAddress || !orderId) {
        return void res.status(400).json({
          success: false,
          message: "Missing contractAddress or orderId",
        });
      }

      const args = [SorobanUtils.convertToScVal(orderId, "bytes")];

      const orderInfo = await SorobanUtils.callViewFunction(
        contractAddress as string,
        "get_order_id",
        args,
        networkName as string,
      );

      return void res.status(200).json({
        success: true,
        data: orderInfo,
        network: networkName,
      });
    } catch (error: any) {
      console.error("Get order info error:", error);

      return void res.status(200).json({
        success: false,
        message: error?.message || "Failed to get order info",
        network: req.query.networkName || "TESTNET",
      });
    }
  },
);

app.get(
  "/getOrderInfo/:contractAddress/:orderId",
  async (req, res): Promise<void> => {
    try {
      const { contractAddress, orderId } = req.params;
      const { networkName = "TESTNET" } = req.query as { networkName?: string };

      if (!contractAddress || !orderId) {
        return void res.status(400).json({
          success: false,
          message: "Missing contractAddress or orderId",
        });
      }

      const args = [SorobanUtils.convertToScVal(orderId, "bytes")];

      const orderInfo = await SorobanUtils.callViewFunction(
        contractAddress as string,
        "get_order_info",
        args,
        networkName as string,
      );

      return void res.status(200).json({
        success: true,
        data: orderInfo,
        network: networkName,
      });
    } catch (error: any) {
      console.error("Get order info error:", error);

      return void res.status(200).json({
        success: false,
        message: error?.message || "failed to get order info",
        network: req.query.networkname || "testnet",
      });
    }
  },
);

app.get(
  "/getTokenBalance/:contractAddress/:userAddress",
  async (req, res): Promise<void> => {
    try {
      const { contractAddress, userAddress } = req.params;
      const { networkName = "TESTNET" } = req.query as { networkName?: string };

      if (!contractAddress || !userAddress) {
        return void res.status(400).json({
          success: false,
          message: "Missing contractAddress or userAddress",
        });
      }

      const args = [SorobanUtils.convertToScVal(userAddress, "address")];

      const balance = await SorobanUtils.callViewFunction(
        contractAddress as string,
        "get_token_balance",
        args,
        networkName as string,
      );

      return void res.status(200).json({
        success: true,
        balance: balance?.toString?.() ?? String(balance),
        network: networkName,
      });
    } catch (error: any) {
      console.error("Get token balance error:", error);

      return void res.status(200).json({
        success: false,
        message: error?.message || "failed to get order info",
        network: req.query.networkname || "testnet",
      });
    }
  },
);

app.get("/getLpFeeDetails/:contractAddress", async (req, res) => {
  try {
    const { contractAddress } = req.params;
    const { networkName = "TESTNET" } = req.query;

    const rawVal = await SorobanUtils.callViewFunction(
      contractAddress,
      "get_lp_fee_details",
      [],
      networkName as string,
    );

    if (!Array.isArray(rawVal)) {
      throw new Error(`Unexpected response type: ${JSON.stringify(rawVal)}`);
    }

    const [protocolFeePercent, maxBps] = rawVal;

    const protocolFeePercentNum = Number(protocolFeePercent);
    const maxBpsNum = Number(maxBps);

    if (
      protocolFeePercent > Number.MAX_SAFE_INTEGER ||
      maxBps > Number.MAX_SAFE_INTEGER
    ) {
      throw new Error("BigInt value exceeds safe integer range for JSON");
    }

    return res.status(200).json({
      success: true,
      feeDetails: {
        protocol_fee_percent: protocolFeePercentNum,
        max_bps: maxBpsNum,
      },
      network: networkName,
    });
  } catch (error: any) {
    console.error("Get LP fee details error:", error);
    return res.status(500).json({
      success: false,
      message: error.message || "Internal server error",
    });
  }
});

app.use(
  (
    error: Error,
    _req: express.Request,
    res: express.Response,
    _next: express.NextFunction,
  ) => {
    console.error("Unhandled error:", error);
    return void res.status(500).json({
      success: false,
      message: "Internal server error",
    });
  },
);

app.use("*", (_req, res) => {
  return res.status(404).json({
    success: false,
    message: "Endpoint not found",
  });
});

const PORT = process.env.PORT || 5000;
app.listen(PORT, () => {
  console.log(`🚀 Soroban Gas Relayer running on port ${PORT}`);
  console.log(
    `💰 Sponsor account: ${SorobanUtils.getSponsorKeypair().publicKey()}`,
  );
  console.log(
    `🌐 Supported networks: ${Object.keys(SorobanUtils.NETWORKS).join(", ")}`,
  );
});

export default app;
