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

# upload: build
# 	@echo "📦 Uploading contract..."
# 	@stellar contract upload \
# 	  --network testnet \
# 	  --source-account josh \
# 	  --wasm ../../target/wasm32v1-none/release/wallet_contract.wasm > upload.log
# 	@tail -n 1 upload.log > wasm_hash.txt
# 	@echo "✅ Wasm hash saved to wasm_hash.txt"

# deploy: upload
# 	@echo "🚀 Deploying contract..."
# 	@stellar contract deploy \
# 	  --network testnet \
# 	  --source-account josh \
# 	  --wasm-hash `cat wasm_hash.txt` > deploy.log
# 	@grep "Contract ID" deploy.log | awk '{print $$3}' > deployed_contracts.txt
# 	@echo "✅ Contract ID saved to deployed_contracts.txt"
# 	@cat deployed_contracts.txt


WASM_HASH = 8b1f593e560a384b0c304411e0b2baece6a35dffc8881fa7ba124d413ef86d5a
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
	  --central_account $(CENTRAL)

# Initialize the deployed contract (call constructor after deployment)
initialize:
	stellar contract invoke \
	  --id $(CONTRACT_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  -- \
	  __constructor \
	  --admin $(ADMIN) \
	  --usdc_asset $(USDC_ASSET) \
	  --central_account $(CENTRAL)

# Deploy and initialize in one command
deploy-and-init: deploy
	@echo "Contract deployed. Now initializing..."
	$(MAKE) initialize CONTRACT_ID=$(stellar contract deploy --wasm-hash $(WASM_HASH) --network $(NETWORK) --source-account $(SOURCE) --dry-run | grep "Contract deployed" | cut -d' ' -f3)


