.PHONY: _help
_help:
	@echo "Used to generate stubs for contract(s) defined in native (via evm-coder). See Makefile for details."

NATIVE_FUNGIBLE_EVM_STUBS=./pallets/balances-adapter/src/stubs
EVM_ASSETS_STUBS=./pallets/evm-assets/src/stubs



NativeFungible.sol:
	PACKAGE=pallet-balances-adapter NAME=eth::gen_impl OUTPUT=$(NATIVE_FUNGIBLE_EVM_STUBS)/$@ ./.maintain/scripts/generate_sol.sh

NativeFungible: NativeFungible.sol
	INPUT=$(NATIVE_FUNGIBLE_EVM_STUBS)/$< OUTPUT=$(NATIVE_FUNGIBLE_EVM_STUBS)/NativeFungible.raw ./.maintain/scripts/compile_stub.sh

NativeFungibleAssets.sol:
	PACKAGE=pallet-evm-assets NAME=eth::gen_impl OUTPUT=$(EVM_ASSETS_STUBS)/$@ ./.maintain/scripts/generate_sol.sh

NativeFungibleAssets: NativeFungibleAssets.sol
	INPUT=$(EVM_ASSETS_STUBS)/$< OUTPUT=$(EVM_ASSETS_STUBS)/NativeFungibleAssets.raw ./.maintain/scripts/compile_stub.sh