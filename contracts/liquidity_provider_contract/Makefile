all: test

test: build
	cargo test

debug: build
	RUST_BACKTRACE=full cargo test
t: test
	RUST_BACKTRACE=1 cargo test  -- --nocapture

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

LM_CONTRACT_ADDRESS = CBNOMMO5JQDNBQKMK4EVNWZUGXIFTNKI34KUI7OB6QY6XJQ7DO2AHTVU

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_provider_contract.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = a48aa0ffa53d0124e48835e47a936a22519353898f203e21f8500a8bc16330cf


NETWORK      = testnet
SOURCE       = josh
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

liquidity_provider_contract = CDBQHRHENGQTLYVODMJ23MNJSCATJ5PMP6ZB63FTBDDM6UWIBJJ5GBKG

deploy:
	stellar contract deploy \
	  --wasm-hash $(WASM_HASH) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE) \
	  -- \
	  --admin $(ADMIN) \
	  --usdc_asset $(USDC_ASSET) \
		--settings_contract $(LM_CONTRACT_ADDRESS)

interact: 
	stellar contract invoke \
  --id $(liquidity_provider_contract) \
  --source-account josh \
  --network testnet \
  -- \
  init \
  --admin $(ADMIN) \
  --usdc_asset $(USDC_ASSET) \
  --settings_contract $(LM_CONTRACT_ADDRESS) 


create:
	stellar contract invoke \
		--id CDBQHRHENGQTLYVODMJ23MNJSCATJ5PMP6ZB63FTBDDM6UWIBJJ5GBKG \
		--network testnet \
		--source-account josh \
		-- \
		create_order \
		--params '{"order_id":"1234123412341234123412341234123412341234123412341234123412342328","token":"GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5","sender":"GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN","amount":"10000000","rate":100,"temporary_wallet_address":"GDRXE2BQUC3AZNPV2N6YQ5A6L7Z2U4C7VJ3ZK5XKZ2K5N4LQ5X3A6PZ","refund_address":"GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN","message_hash":"abcd1234"}'

