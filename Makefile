escrow:
	make -C contracts/escrow_contract debug
ed: 
	make -C contracts/escrow_contract deploy

lp:
	make -C contracts/liquidity_provider_contract debug

lu:
	make -C contracts/liquidity_provider_contract upload 

ld:
	make -C contracts/liquidity_provider_contract deploy 

payment:
	make -C contracts/payment_contract debug

pu:
	make -C contracts/payment_contract upload 

pd:
	make -C contracts/payment_contract deploy 


wallet: 
	make -C contracts/wallet_contract debug

wd: 
	make -C contracts/wallet_contract deploy

wu:
	make -C contracts/wallet_contract upload 

wf:
	make -C contracts/wallet_contract full-deploy 

wi:
	make -C contracts/wallet_contract init 
