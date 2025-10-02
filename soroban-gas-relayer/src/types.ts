export interface NetworkConfig {
  name: string;
  rpcUrl: string;
  networkPassphrase: string;
  friendbotUrl?: string;
  horizonUrl: string;
}

export interface OrderParams {
  order_id: string;
  token: string;
  sender: string;
  amount: bigint;
  rate: bigint;
  sender_fee_recipient: string;
  sender_fee: bigint;
  refund_address: string;
  message_hash: string;
}

export interface TransactionResult {
  success: boolean;
  txHash?: string;
  message: string;
  network: string;
  explorerUrl?: string;
  error?: string;
}

export interface OrderInfo {
  order_id: string;
  sender: string;
  token: string;
  amount: bigint;
  sender_fee_recipient: string;
  sender_fee: bigint;
  protocol_fee: bigint;
  is_fulfilled: boolean;
  is_refunded: boolean;
  refund_address: string;
  current_bps: bigint;
  rate: bigint;
  message_hash: string;
}

export interface FeeDetails {
  protocol_fee_percent: number;
  max_bps: number;
}
