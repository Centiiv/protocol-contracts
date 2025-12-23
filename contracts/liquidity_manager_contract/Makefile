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

LM =CBNOMMO5JQDNBQKMK4EVNWZUGXIFTNKI34KUI7OB6QY6XJQ7DO2AHTVU

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_manager.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = c18249ff62d376d535189173a43382039a2a58abea4bcabf40ac69066d0b94d5

NETWORK      = testnet
SOURCE       = josh
CONTRACT_ID  = 
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA 
RELAYER_ADDRESS = GD2NWO6EETYCLZJZ4QSTFU2YKQQ3WMFT4WUPGVZT45S6YJOZSZOSFLGD
RS = SBZNINR4IQPMMHPGZ2BUTPIBQEY6UNFK5GYZUELPJPD7NYUL27PINDBH
TREASURY = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN 
deploy:
	stellar contract deploy \
	  --wasm-hash $(WASM_HASH) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE) \
	  -- \
	  --admin $(ADMIN) \

interact: 
	stellar contract invoke \
  --id $(LM) \
  --source-account josh \
  --network testnet \
  -- \
  initialize \
  --admin $(ADMIN) \
	--treasury $(TREASURY) \
	--relayer_address $(RELAYER_ADDRESS) \
