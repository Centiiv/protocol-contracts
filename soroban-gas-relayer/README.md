# Soroban Gas Relayer

A Node.js/TypeScript service for sponsoring gas fees on Soroban smart contracts. This relayer allows users to execute contract functions without paying transaction fees directly.

## Features

- **Gas Sponsorship**: Sponsor transaction fees for users
- **Multi-Network Support**: Works with Stellar Testnet, Futurenet, and Mainnet
- **Contract Functions**: Supports all LP contract functions except initialize
- **Type Safety**: Full TypeScript implementation
- **Express API**: RESTful endpoints for easy integration

## Supported Contract Functions

### Write Operations (Gas Sponsored)

- `create_order` - Create new liquidity orders
- `settle` - Settle orders with liquidity providers
- `refund` - Refund orders with fees
- `register_lp_node` - Register liquidity provider nodes

### Read Operations (No Gas Required)

- `get_order_info` - Get order details
- `get_token_balance` - Get user token balance
- `get_lp_fee_details` - Get fee configuration

## Setup

### 1. Install Dependencies

```bash
npm install
```

### 2. Generate Sponsor Keypair

```bash
npm run generate-keypair
```

This will create:

- `.env.example` with configuration template
- `keypair-backup.json` with your keypair (keep secure!)

### 3. Configure Environment

```bash
cp .env.example .env
# Edit .env with your settings
```

### 4. Fund Sponsor Account

**For Testnet:**

```bash
# Visit: https://friendbot.stellar.org?addr=YOUR_PUBLIC_KEY
# Or use curl:
curl "https://friendbot.stellar.org?addr=YOUR_PUBLIC_KEY"
```

**For Mainnet:**
Send XLM to your sponsor public key for transaction fees.

### 5. Build and Start

```bash
# Development
npm run dev

# Production
npm run build
npm start
```

## API Endpoints

### POST /createOrder

Sponsor a `create_order` transaction.

```json
{
  "contractAddress": "C...",
  "sender": "G...",
  "orderId": "0x...",
  "amount": "1000000",
  "senderFeeRecipient": "G...",
  "senderFee": "1000",
  "refundAddress": "G...",
  "rate": "100",
  "messageHash": "0x...",
  "networkName": "TESTNET"
}
```

### POST /settle

Sponsor a `settle` transaction.

```json
{
  "contractAddress": "C...",
  "splitOrderId": "0x...",
  "orderId": "0x...",
  "liquidityProvider": "G...",
  "settlePercent": "50000",
  "caller": "G...",
  "networkName": "TESTNET"
}
```

### POST /refund

Sponsor a `refund` transaction.

```json
{
  "contractAddress": "C...",
  "orderId": "0x...",
  "fee": "1000",
  "caller": "G...",
  "networkName": "TESTNET"
}
```

### POST /registerLpNode

Sponsor a `register_lp_node` transaction.

```json
{
  "contractAddress": "C...",
  "lpNodeId": "0x...",
  "capacity": "1000000",
  "caller": "G...",
  "networkName": "TESTNET"
}
```

### GET /getOrderInfo/:contractAddress/:orderId

Get order information (read-only).

```
GET /getOrderInfo/C.../0x123?networkName=TESTNET
```

### GET /getTokenBalance/:contractAddress/:userAddress

Get user's token balance (read-only).

```
GET /getTokenBalance/C.../G...?networkName=TESTNET
```

### GET /getLpFeeDetails/:contractAddress

Get LP fee configuration (read-only).

```
GET /getLpFeeDetails/C...?networkName=TESTNET
```

### GET /health

Health check endpoint.

```json
{
  "success": true,
  "message": "Soroban Gas Relayer is running",
  "timestamp": "2024-01-01T00:00:00.000Z"
}
```

## Response Format

All endpoints return JSON responses:

```json
{
  "success": boolean,
  "txHash": "string (optional)",
  "message": "string",
  "network": "string"
}
```

## Network Configuration

Supported networks:

- `TESTNET` - Stellar Testnet (default)
- `FUTURENET` - Stellar Futurenet
- `MAINNET` - Stellar Mainnet

## Security Considerations

1. **Private Key Security**: Keep your sponsor secret key secure
2. **Rate Limiting**: Consider implementing rate limiting for production
3. **Access Control**: Add authentication/authorization as needed
4. **Fund Management**: Monitor sponsor account balance
5. **Contract Validation**: Validate contract addresses before sponsoring

## Error Handling

The relayer includes comprehensive error handling:

- Input validation for all parameters
- Network connectivity checks
- Transaction simulation before submission
- Detailed error messages for debugging

## Development

### Project Structure

```
├── src/
│   └── index.ts          # Main relayer service
├── scripts/
│   └── generateKeypair.ts # Keypair generation utility
├── dist/                 # Compiled JavaScript
├── package.json
├── tsconfig.json
└── README.md
```

### Testing

```bash
# Run tests
npm test

# Watch mode
npm run test:watch
```

### Building

```bash
# Compile TypeScript
npm run build

# Watch for changes
npm run watch
```

## Migration from EVM

Key differences from your EVM relayer:

1. **Network**: Uses Stellar instead of Ethereum networks
2. **Keys**: Uses Stellar keypairs instead of encrypted JSON
3. **Transactions**: Uses Soroban contract calls instead of ethers.js
4. **Parameters**: Uses ScVal encoding for contract parameters
5. **Simulation**: Uses Soroban RPC simulation instead of gas estimation

## Troubleshooting

### Common Issues

**"Account not found"**

- Ensure sponsor account is funded with XLM

**"Contract not found"**

- Verify contract address and network
- Ensure contract is deployed on the specified network

**"Simulation failed"**

- Check contract function parameters
- Verify contract state allows the operation

**"Transaction failed"**

- Check sponsor account balance
- Verify transaction parameters
- Review contract error messages

### Debugging

Enable detailed logging:

```bash
DEBUG=soroban:* npm run dev
```

### Support

For issues related to:

- **Stellar SDK**: [Stellar Developers Discord](https://discord.gg/stellardev)
- **Soroban**: [Soroban Documentation](https://soroban.stellar.org/)
- **This Relayer**: Open an issue in the repository

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Deployment

### Docker (Recommended)

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 5000
CMD ["node", "dist/index.js"]
```

### Environment Variables for Production

```bash
# Production settings
NODE_ENV=production
SPONSOR_SECRET_KEY=your_secret_key
SOROBAN_RPC_URL=https://soroban-rpc.mainnet.stellar.org
PORT=5000
```

### Monitoring

Consider adding:

- Health checks
- Metrics collection (Prometheus)
- Log aggregation
- Error tracking (Sentry)
- Account balance monitoring

## Scaling

For high-volume usage:

- Implement connection pooling
- Add Redis for caching
- Use multiple sponsor accounts
- Implement request queuing
- Add load balancing
