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

LM_CONTRACT_ADDRESS = CA7KBDMJOZD2H4YT4RHCLIMQLXD67UNWUMP2ZGCOCJS5HQKPUN5CPRY5

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_provider_contract.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = f93bda98cb3b01c5093965b803763e6b2296d570d6e0de8342724ba9504e9993

NETWORK      = testnet
SOURCE       = josh
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

deploy:
	stellar contract deploy \
	  --wasm-hash $(WASM_HASH) \
	  --network $(NETWORK) \
	  --source-account $(SOURCE) \
	  -- \
	  --admin $(ADMIN) \
	  --usdc_asset $(USDC_ASSET) \
		--wallet_contract $(LM_CONTRACT_ADDRESS)
