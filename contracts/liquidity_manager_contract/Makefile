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

# CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA
WALLET =CB2TDMTYO5ECIS2BPTDYLIEGJCCBWIRKE7CLRGJNVKSKHJBPL7H6XFK2
PAYMENT =CCNUCFHP27U3LO73JVXVKG2OMG2K3NDXCPZA36XTI635L2VKQIYXKUYZ
upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/payment_contract.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = d11a4d3287304874b1123005c21ec658814fe85539f5a4e0a2761b5a6b6a2ba2
# Variables (change as needed)
NETWORK      = testnet
SOURCE       = josh
CONTRACT_ID  = CB2TDMTYO5ECIS2BPTDYLIEGJCCBWIRKE7CLRGJNVKSKHJBPL7H6XFK2
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA
CENTRAL      = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN

# Deploy the contract (just deployment, no initialization)
deploy:
	stellar contract deploy \
	  --wasm-hash $(WASM_HASH) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE) \
	  -- \
	  --admin $(ADMIN) \
	  --usdc_asset $(USDC_ASSET) \


