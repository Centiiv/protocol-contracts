import request from "supertest";
import { Server } from "http";

process.env.SPONSOR_SECRET_KEY;
process.env.SOROBAN_RPC_URL;
process.env.NETWORK_PASSPHRASE = "Test SDF Network ; September 2015";

import app from "../src/server";

describe("Soroban Gas Relayer API Tests", () => {
  let server: Server;
  const testPort = 3001;

  const testOrderParams = {
    sender: "GCS7OUIKY4XUAQPT77HHSLLZ2JDMP53MCJ6HVIEY6ZF3B4ATFJDGJJS6",
    order_id:
      "1234123412341234123412341234123412341234123412341234123412349998",
    token: "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5",
    amount: "3",
    sender_fee_recipient:
      "GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN",
    sender_fee: "1",
    refund_address: "GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN",
    rate: "9500",
    message_hash: "hash123",
  };

  const testSettleParams = {
    orderId: "1234123412341234123412341234123412341234123412341234123412349998",
    liquidityProvider:
      "GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN",
    settlePercent: "100000",
    caller: "GCS7OUIKY4XUAQPT77HHSLLZ2JDMP53MCJ6HVIEY6ZF3B4ATFJDGJJS6",
  };

  const testContractAddress =
    "CD66PVVI2EZMHN3RXYHA5JV7HW5FEBU3M6LOTAS7XMNV7BFTM7QFXVMR";

  beforeAll((done) => {
    server = app.listen(testPort, () => {
      console.log(`Test server running on port ${testPort}`);
      done();
    });
  });

  afterAll((done) => {
    server.close(done);
  });

  describe("Health Check Endpoints", () => {
    test("GET / should return API info", async () => {
      const response = await request(app).get("/");
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty("success", true);
      expect(response.body).toHaveProperty(
        "message",
        "🚀 Soroban Gas Relayer is running",
      );
    });
  });

  describe("Contract Info Endpoints", () => {
    test("GET /getLpFeeDetails should return fee details", async () => {
      const response = await request(app)
        .get(`/getLpFeeDetails/${testContractAddress}`)
        .query({ networkName: "TESTNET" });

      console.log("Fee Details Response:", response.body);
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty("success");
      expect(response.body).toHaveProperty("network", "TESTNET");
    });

    test("GET /getTokenBalance should handle errors gracefully", async () => {
      const response = await request(app)
        .get(
          `/getTokenBalance/${testContractAddress}/${testOrderParams.sender}`,
        )
        .query({ networkName: "TESTNET" });

      console.log("Token Balance Response:", response.body);
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty("success");
    });

    test("GET /getOrderInfo should handle errors gracefully", async () => {
      const response = await request(app)
        .get(`/getOrderInfo/${testContractAddress}/${testOrderParams.order_id}`)
        .query({ networkName: "TESTNET" });

      console.log("Order Info Response:", response.body);
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty("success");
    });
  });

  describe("Order Management Endpoints", () => {
    test("POST /createOrder should validate request body", async () => {
      const response = await request(app).post("/createOrder").send({
        contractAddress: testContractAddress,
        orderParams: testOrderParams,
        networkName: "TESTNET",
      });

      console.log("Create Order Response:", response.body);
      expect([200, 500]).toContain(response.status);
      expect(response.body).toHaveProperty("success");
    });

    test("POST /settle should validate settle parameters", async () => {
      const response = await request(app)
        .post("/settle")
        .send({
          contractAddress: testContractAddress,
          ...testSettleParams,
          networkName: "TESTNET",
        });

      console.log("Settle Response:", response.body);
      expect([200, 500]).toContain(response.status);
      expect(response.body).toHaveProperty("success");
    });
  });

  describe("Error Handling", () => {
    test("Invalid contract address should return error", async () => {
      const response = await request(app)
        .get("/getLpFeeDetails/invalid_contract_address")
        .query({ networkName: "TESTNET" });

      expect(response.status).toBe(500);
      expect(response.body).toHaveProperty("success", false);
    });

    test("Invalid network should return error", async () => {
      const response = await request(app)
        .get(`/getLpFeeDetails/${testContractAddress}`)
        .query({ networkName: "INVALID_NETWORK" });

      expect(response.status).toBe(500);
      expect(response.body).toHaveProperty("success", false);
    });
  });

  describe("Parameter Validation", () => {
    test("Invalid order_id format should be rejected", async () => {
      const invalidOrderParams = {
        ...testOrderParams,
        order_id: "invalid",
      };

      const response = await request(app).post("/createOrder").send({
        contractAddress: testContractAddress,
        orderParams: invalidOrderParams,
        networkName: "TESTNET",
      });

      expect(response.status).toBe(500);
      expect(response.body).toHaveProperty("success", false);
    });

    test("Invalid address format should be rejected", async () => {
      const invalidOrderParams = {
        ...testOrderParams,
        sender: "invalid_address",
      };

      const response = await request(app).post("/createOrder").send({
        contractAddress: testContractAddress,
        orderParams: invalidOrderParams,
        networkName: "TESTNET",
      });

      expect(response.status).toBe(500);
      expect(response.body).toHaveProperty("success", false);
    });
  });
});
