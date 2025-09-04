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

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/escrow_contract.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

deploy: upload
	@echo "🚀 Deploying contract..."
	@stellar contract deploy \
	  --network testnet \
	  --source-account josh \
	  --wasm-hash `cat wasm_hash.txt` > deploy.log
	@grep "Contract ID" deploy.log | awk '{print $$3}' > deployed_contracts.txt
	@echo "✅ Contract ID saved to deployed_contracts.txt"
	@cat deployed_contracts.txt


