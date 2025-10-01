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

LM =CARQXZOURGZIZNL42HPM6E26TOFDIYH5Y5OGANUVY67OI7OLK4XYP6IL

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_manager.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = 1be77b13ad5af1db1aa1f2de222993b539c119e72e88f4ebca9d0bbc2f977655
NETWORK      = testnet
SOURCE       = josh
CONTRACT_ID  = 
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA 
RELAYER_ADDRESS = GCS7OUIKY4XUAQPT77HHSLLZ2JDMP53MCJ6HVIEY6ZF3B4ATFJDGJJS6
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
  --id CARQXZOURGZIZNL42HPM6E26TOFDIYH5Y5OGANUVY67OI7OLK4XYP6IL \
  --source-account josh \
  --network testnet \
  -- \
  initialize \
  --admin $(ADMIN) \
	--treasury $(TREASURY) \
	--relayer_address $(RELAYER_ADDRESS) \
