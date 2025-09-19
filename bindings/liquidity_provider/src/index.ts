import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const ContractError = {
  1: {message:"InvalidAmount"},
  2: {message:"ZeroAddress"},
  3: {message:"InvalidSenderFeeRecipient"},
  4: {message:"InvalidMessageHash"},
  5: {message:"InvalidSettlePercent"},
  6: {message:"OrderAlreadyExists"},
  7: {message:"OrderNotFound"},
  8: {message:"OrderFulfilled"},
  9: {message:"OrderRefunded"},
  10: {message:"FeeExceedsProtocolFee"},
  11: {message:"Paused"},
  12: {message:"Unauthorized"},
  13: {message:"TransferFailed"},
  14: {message:"AddressAlreadySet"},
  15: {message:"InvalidParameter"},
  16: {message:"InvalidLpNodeParameters"},
  17: {message:"LpNodeIdAlreadyExists"},
  18: {message:"SettingsContractNotSet"},
  19: {message:"UsdcNotSet"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "SettingsContract", values: void} | {tag: "NodeIDs", values: void} | {tag: "Nonces", values: void} | {tag: "Order", values: readonly [Buffer]} | {tag: "Usdc", values: void};


export interface LpNode {
  capacity: i128;
}


export interface OrderParams {
  amount: i128;
  message_hash: string;
  order_id: Buffer;
  rate: i64;
  refund_address: string;
  sender: string;
  sender_fee: i128;
  sender_fee_recipient: string;
  token: string;
}


export interface Order {
  amount: i128;
  current_bps: i128;
  is_fulfilled: boolean;
  is_refunded: boolean;
  message_hash: string;
  order_id: Buffer;
  protocol_fee: i128;
  rate: i64;
  refund_address: string;
  sender: string;
  sender_fee: i128;
  sender_fee_recipient: string;
  token: string;
}

export const ContractError = {
  1: {message:"InvalidFeePercent"},
  2: {message:"ZeroAddress"},
  3: {message:"AddressAlreadySet"},
  4: {message:"InvalidParameter"},
  5: {message:"Unauthorized"}
}

export type PaymentStatus = {tag: "Pending", values: void} | {tag: "Completed", values: void} | {tag: "Failed", values: void};

export type ProtocolAddressType = {tag: "Treasury", values: void} | {tag: "Aggregator", values: void};

export type DataKey = {tag: "Admin", values: void} | {tag: "Treasury", values: void} | {tag: "Aggregator", values: void} | {tag: "ProtocolFeePercent", values: void} | {tag: "MaxBps", values: void} | {tag: "Paused", values: void} | {tag: "TokenSupported", values: readonly [string]};

export interface Client {
  /**
   * Construct and simulate a create_order transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  create_order: ({params}: {params: OrderParams}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a settle transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  settle: ({split_order_id, order_id, liquidity_provider, settle_percent}: {split_order_id: Buffer, order_id: Buffer, liquidity_provider: string, settle_percent: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<boolean>>>

  /**
   * Construct and simulate a refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refund: ({order_id, fee}: {order_id: Buffer, fee: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_token_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_token_balance: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_order_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_order_id: ({order_id}: {order_id: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<Buffer>>>

  /**
   * Construct and simulate a get_order_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_order_info: ({order_id}: {order_id: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<Order>>>

  /**
   * Construct and simulate a get_lp_fee_details transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_lp_fee_details: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<readonly [i64, i64]>>

  /**
   * Construct and simulate a init transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init: ({admin, usdc_asset, settings_contract}: {admin: string, usdc_asset: string, settings_contract: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a register_lp_node transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  register_lp_node: ({lp_node_id, capacity}: {lp_node_id: Buffer, capacity: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a upgrade_lp transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade_lp: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_protocol_fee transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_protocol_fee: ({protocol_fee_percent}: {protocol_fee_percent: i64}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_protocol_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_protocol_address: ({what, value}: {what: ProtocolAddressType, value: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pause: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_fee_details transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_fee_details: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<readonly [i64, i64]>>

  /**
   * Construct and simulate a get_treasury_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_treasury_address: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_aggregator_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_aggregator_address: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a is_paused transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_paused: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a is_token_supported transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_token_supported: ({token}: {token: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a upgrade_lp_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade_lp_manager: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABAAAAAAAAAAAAAAADUNvbnRyYWN0RXJyb3IAAAAAAAATAAAAAAAAAA1JbnZhbGlkQW1vdW50AAAAAAAAAQAAAAAAAAALWmVyb0FkZHJlc3MAAAAAAgAAAAAAAAAZSW52YWxpZFNlbmRlckZlZVJlY2lwaWVudAAAAAAAAAMAAAAAAAAAEkludmFsaWRNZXNzYWdlSGFzaAAAAAAABAAAAAAAAAAUSW52YWxpZFNldHRsZVBlcmNlbnQAAAAFAAAAAAAAABJPcmRlckFscmVhZHlFeGlzdHMAAAAAAAYAAAAAAAAADU9yZGVyTm90Rm91bmQAAAAAAAAHAAAAAAAAAA5PcmRlckZ1bGZpbGxlZAAAAAAACAAAAAAAAAANT3JkZXJSZWZ1bmRlZAAAAAAAAAkAAAAAAAAAFUZlZUV4Y2VlZHNQcm90b2NvbEZlZQAAAAAAAAoAAAAAAAAABlBhdXNlZAAAAAAACwAAAAAAAAAMVW5hdXRob3JpemVkAAAADAAAAAAAAAAOVHJhbnNmZXJGYWlsZWQAAAAAAA0AAAAAAAAAEUFkZHJlc3NBbHJlYWR5U2V0AAAAAAAADgAAAAAAAAAQSW52YWxpZFBhcmFtZXRlcgAAAA8AAAAAAAAAF0ludmFsaWRMcE5vZGVQYXJhbWV0ZXJzAAAAABAAAAAAAAAAFUxwTm9kZUlkQWxyZWFkeUV4aXN0cwAAAAAAABEAAAAAAAAAFlNldHRpbmdzQ29udHJhY3ROb3RTZXQAAAAAABIAAAAAAAAAClVzZGNOb3RTZXQAAAAAABM=",
        "AAAAAAAAAAAAAAAMY3JlYXRlX29yZGVyAAAAAQAAAAAAAAAGcGFyYW1zAAAAAAfQAAAAC09yZGVyUGFyYW1zAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAAAAAAAGc2V0dGxlAAAAAAAEAAAAAAAAAA5zcGxpdF9vcmRlcl9pZAAAAAAADgAAAAAAAAAIb3JkZXJfaWQAAAAOAAAAAAAAABJsaXF1aWRpdHlfcHJvdmlkZXIAAAAAABMAAAAAAAAADnNldHRsZV9wZXJjZW50AAAAAAALAAAAAQAAA+kAAAABAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAAAAAAAGcmVmdW5kAAAAAAACAAAAAAAAAAhvcmRlcl9pZAAAAA4AAAAAAAAAA2ZlZQAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANQ29udHJhY3RFcnJvcgAAAA==",
        "AAAAAAAAAAAAAAARZ2V0X3Rva2VuX2JhbGFuY2UAAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAMZ2V0X29yZGVyX2lkAAAAAQAAAAAAAAAIb3JkZXJfaWQAAAAOAAAAAQAAA+kAAAAOAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAAAAAAAOZ2V0X29yZGVyX2luZm8AAAAAAAEAAAAAAAAACG9yZGVyX2lkAAAADgAAAAEAAAPpAAAH0AAAAAVPcmRlcgAAAAAAB9AAAAANQ29udHJhY3RFcnJvcgAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X2xwX2ZlZV9kZXRhaWxzAAAAAAAAAAAAAQAAA+0AAAACAAAABwAAAAc=",
        "AAAAAAAAAAAAAAAEaW5pdAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKdXNkY19hc3NldAAAAAAAEwAAAAAAAAARc2V0dGluZ3NfY29udHJhY3QAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAQcmVnaXN0ZXJfbHBfbm9kZQAAAAIAAAAAAAAACmxwX25vZGVfaWQAAAAAAA4AAAAAAAAACGNhcGFjaXR5AAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAAAAAAAKdXBncmFkZV9scAAAAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABgAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAQU2V0dGluZ3NDb250cmFjdAAAAAAAAAAAAAAAB05vZGVJRHMAAAAAAAAAAAAAAAAGTm9uY2VzAAAAAAABAAAAAAAAAAVPcmRlcgAAAAAAAAEAAAAOAAAAAAAAAAAAAAAEVXNkYw==",
        "AAAAAQAAAAAAAAAAAAAABkxwTm9kZQAAAAAAAQAAAAAAAAAIY2FwYWNpdHkAAAAL",
        "AAAAAQAAAAAAAAAAAAAAC09yZGVyUGFyYW1zAAAAAAkAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAMbWVzc2FnZV9oYXNoAAAAEAAAAAAAAAAIb3JkZXJfaWQAAAAOAAAAAAAAAARyYXRlAAAABwAAAAAAAAAOcmVmdW5kX2FkZHJlc3MAAAAAABMAAAAAAAAABnNlbmRlcgAAAAAAEwAAAAAAAAAKc2VuZGVyX2ZlZQAAAAAACwAAAAAAAAAUc2VuZGVyX2ZlZV9yZWNpcGllbnQAAAATAAAAAAAAAAV0b2tlbgAAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAABU9yZGVyAAAAAAAADQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAtjdXJyZW50X2JwcwAAAAALAAAAAAAAAAxpc19mdWxmaWxsZWQAAAABAAAAAAAAAAtpc19yZWZ1bmRlZAAAAAABAAAAAAAAAAxtZXNzYWdlX2hhc2gAAAAQAAAAAAAAAAhvcmRlcl9pZAAAAA4AAAAAAAAADHByb3RvY29sX2ZlZQAAAAsAAAAAAAAABHJhdGUAAAAHAAAAAAAAAA5yZWZ1bmRfYWRkcmVzcwAAAAAAEwAAAAAAAAAGc2VuZGVyAAAAAAATAAAAAAAAAApzZW5kZXJfZmVlAAAAAAALAAAAAAAAABRzZW5kZXJfZmVlX3JlY2lwaWVudAAAABMAAAAAAAAABXRva2VuAAAAAAAAEw==",
        "AAAABAAAAAAAAAAAAAAADUNvbnRyYWN0RXJyb3IAAAAAAAAFAAAAAAAAABFJbnZhbGlkRmVlUGVyY2VudAAAAAAAAAEAAAAAAAAAC1plcm9BZGRyZXNzAAAAAAIAAAAAAAAAEUFkZHJlc3NBbHJlYWR5U2V0AAAAAAAAAwAAAAAAAAAQSW52YWxpZFBhcmFtZXRlcgAAAAQAAAAAAAAADFVuYXV0aG9yaXplZAAAAAU=",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAATdXBkYXRlX3Byb3RvY29sX2ZlZQAAAAABAAAAAAAAABRwcm90b2NvbF9mZWVfcGVyY2VudAAAAAcAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAAAAAAAXdXBkYXRlX3Byb3RvY29sX2FkZHJlc3MAAAAAAgAAAAAAAAAEd2hhdAAAB9AAAAATUHJvdG9jb2xBZGRyZXNzVHlwZQAAAAAAAAAABXZhbHVlAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAAAAAAAFcGF1c2UAAAAAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANQ29udHJhY3RFcnJvcgAAAA==",
        "AAAAAAAAAAAAAAAHdW5wYXVzZQAAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAANQ29udHJhY3RFcnJvcgAAAA==",
        "AAAAAAAAAAAAAAAPZ2V0X2ZlZV9kZXRhaWxzAAAAAAAAAAABAAAD7QAAAAIAAAAHAAAABw==",
        "AAAAAAAAAAAAAAAUZ2V0X3RyZWFzdXJ5X2FkZHJlc3MAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAWZ2V0X2FnZ3JlZ2F0b3JfYWRkcmVzcwAAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAJaXNfcGF1c2VkAAAAAAAAAAAAAAEAAAAB",
        "AAAAAAAAAAAAAAASaXNfdG9rZW5fc3VwcG9ydGVkAAAAAAABAAAAAAAAAAV0b2tlbgAAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAASdXBncmFkZV9scF9tYW5hZ2VyAAAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAgAAAAAAAAAAAAAADVBheW1lbnRTdGF0dXMAAAAAAAADAAAAAAAAAAAAAAAHUGVuZGluZwAAAAAAAAAAAAAAAAlDb21wbGV0ZWQAAAAAAAAAAAAAAAAAAAZGYWlsZWQAAA==",
        "AAAAAgAAAAAAAAAAAAAAE1Byb3RvY29sQWRkcmVzc1R5cGUAAAAAAgAAAAAAAAAAAAAACFRyZWFzdXJ5AAAAAAAAAAAAAAAKQWdncmVnYXRvcgAA",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABwAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAIVHJlYXN1cnkAAAAAAAAAAAAAAApBZ2dyZWdhdG9yAAAAAAAAAAAAAAAAABJQcm90b2NvbEZlZVBlcmNlbnQAAAAAAAAAAAAAAAAABk1heEJwcwAAAAAAAAAAAAAAAAAGUGF1c2VkAAAAAAABAAAAAAAAAA5Ub2tlblN1cHBvcnRlZAAAAAAAAQAAABM=" ]),
      options
    )
  }
  public readonly fromJSON = {
    create_order: this.txFromJSON<Result<void>>,
        settle: this.txFromJSON<Result<boolean>>,
        refund: this.txFromJSON<Result<void>>,
        get_token_balance: this.txFromJSON<i128>,
        get_order_id: this.txFromJSON<Result<Buffer>>,
        get_order_info: this.txFromJSON<Result<Order>>,
        get_lp_fee_details: this.txFromJSON<readonly [i64, i64]>,
        init: this.txFromJSON<null>,
        register_lp_node: this.txFromJSON<Result<void>>,
        upgrade_lp: this.txFromJSON<null>,
        initialize: this.txFromJSON<null>,
        update_protocol_fee: this.txFromJSON<Result<void>>,
        update_protocol_address: this.txFromJSON<Result<void>>,
        pause: this.txFromJSON<Result<void>>,
        unpause: this.txFromJSON<Result<void>>,
        get_fee_details: this.txFromJSON<readonly [i64, i64]>,
        get_treasury_address: this.txFromJSON<string>,
        get_aggregator_address: this.txFromJSON<string>,
        is_paused: this.txFromJSON<boolean>,
        is_token_supported: this.txFromJSON<boolean>,
        upgrade_lp_manager: this.txFromJSON<null>
  }
}