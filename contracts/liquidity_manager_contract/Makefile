all: test

test: build
	cargo test

debug: build
	RUST_BACKTRACE=full cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

LM =0fb140c6b1d574f77cf65762b7495232060acb0e1e2e0dfcd935a76162bbc95a

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_manager.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = 0fb140c6b1d574f77cf65762b7495232060acb0e1e2e0dfcd935a76162bbc95a
NETWORK      = testnet
SOURCE       = josh
CONTRACT_ID  = 
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

deploy:
	stellar contract deploy \
	  --wasm-hash $(WASM_HASH) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE) \
	  -- \
	  --admin $(ADMIN) \

interact: 
	stellar contract invoke \
  --id CDRV7LZKX7BODJS7JBEFTTTNRZMOZ54FLLNMCCWAVW4DV6NL3PVPZMLY \
  --source-account josh \
  --network testnet \
  -- \
  initialize \
  --admin $(ADMIN) \
