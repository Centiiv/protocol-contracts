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

LM_CONTRACT_ADDRESS = CARQXZOURGZIZNL42HPM6E26TOFDIYH5Y5OGANUVY67OI7OLK4XYP6IL

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract upload \
	  --network testnet \
	  --source-account josh \
	  --wasm ../../target/wasm32v1-none/release/liquidity_provider_contract.wasm > upload.log
	@tail -n 1 upload.log > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"

WASM_HASH = 65330c3617d8216793758f252c5c395a112b8705943a935c529bc871039fa832

NETWORK      = testnet
SOURCE       = josh
ADMIN        = GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN
USDC_ASSET   = CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

liquidity_provider_contract = CAK62V4LSOCFGKLQLSRPGCL77HNZ3HYXUSAGB3F6WRPG3CYJC5FPQMBD

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
  --id $(liquidity_provider_contract) \
  --network $(NETWORK) \
  --source-account josh \
  -- \
   create_order \
	 --params '{"order_id":"1234123412341234123412341234123412341234123412341234123412342328","token":"GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5","sender":"GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN","amount":"10000000","rate":100,"sender_fee_recipient":"GCS7OUIKY4XUAQPT77HHSLLZ2JDMP53MCJ6HVIEY6ZF3B4ATFJDGJJS6","sender_fee":"5","refund_address":"GA3N5T6H7QL5RH66C3CVU6IRALPFCMKYX753OWJY6BOE3ESEXBOMV2KN","message_hash":"abcd1234"}'

 # stellar contract invoke \
 #  --id CDRV7LZKX7BODJS7JBEFTTTNRZMOZ54FLLNMCCWAVW4DV6NL3PVPZMLY \
 # --source-account josh \
 #  --network testnet \
 # -- \
 #  get_lp_fee_details \
 #
 # stellar contract invoke \
 #  --id CDRV7LZKX7BODJS7JBEFTTTNRZMOZ54FLLNMCCWAVW4DV6NL3PVPZMLY \
 #  --network testnet \
 #  --source-account josh \
 #  -- \
 #  create_order \
 #  --params "{\"sender\":\"GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF\",\"order_id\":\"1234123412341234123412341234123412341234123412341234123412341234\",\"token\":\"CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA\",\"amount\":\"10000000000\",\"sender_fee_recipient\":\"GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF\",\"sender_fee\":\"100000000\",\"refund_address\":\"GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF\",\"message_hash\":\"hash123\"}"

