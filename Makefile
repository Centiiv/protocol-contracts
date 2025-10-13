lpd:
	make -C contracts/liquidity_provider_contract debug

lu:
	make -C contracts/liquidity_provider_contract upload 

ld:
	make -C contracts/liquidity_provider_contract deploy 

lc:
	make -C contracts/liquidity_provider_contract create 

lp_abi:
	stellar contract bindings typescript --wasm build_output/liquidity_provider_contract.wasm --output-dir ./bindings/liquidity_provider

li:
	make -C contracts/liquidity_provider_contract interact

lmc:
	make -C contracts/liquidity_provider_contract create

lm:
	make -C contracts/liquidity_manager_contract debug

lmu:
	make -C contracts/liquidity_manager_contract upload 

lmd:
	make -C contracts/liquidity_manager_contract deploy 

lm_abi:
	stellar contract bindings typescript --wasm build_output/liquidity_manager.wasm --output-dir ./bindings/liquidity_manager


lmi:
	make -C contracts/liquidity_manager_contract interact
 
