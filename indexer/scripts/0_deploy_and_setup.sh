# !bin/sh
cd `dirname $0`
dfx canister call indexer_mainnet setup '(variant{Mainnet},variant{DAI},17425968)' --network mainnet
dfx canister call indexer_optimism setup '(variant{Optimism},variant{DAI},105256208)' --network mainnet
