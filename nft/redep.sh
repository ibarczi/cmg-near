#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
set -e
echo -e "🟣"$HI"_____Build contract."
exe eval "RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release"
set +e
echo -e "🟤"$HI"_____Copy contract."
exe cp target/wasm32-unknown-unknown/release/*.wasm ./res/
echo -e "🔴"$HI"_____Delete old contract / account, create new."
exe eval "echo y | near delete $CID $AID"
echo -e "🟠"$HI"_____create new contract account."
exe near create-account $CID --masterAccount $AID --initialBalance 30
set -e
echo -e "🟡"$HI"_____Deploy contract."
exe near deploy $CID --wasmFile res/nft_z2h.wasm
set +e
echo -e "🔵"$HI"_____Init contract."
exe near call $CID new_default_meta '{"owner_id": "'$AID'"}' --accountId $AID
# exe near call $CID test_content_init '{}' --accountId $AID
# . bal.sh
# echo "🔵_____Run test tx."
#near call $CID nft_mint_test '{"token_id": "11111", "receiver_id": "'$AID'", "token_metadata": { "title": "Brownies", "description": "My NFT brownie", "media": "https://assets.change.org/photos/2/lt/qf/TNLTqfJKJcJUwqj-800x450-noPad.jpg?1571334802", "copies": 1}}' --accountId $AID --deposit 0.1
#exe near call $CID modnfts '{}' --accountId $AID
###near call $CID add_bid '{"contentId": "9d724602-caa5-4aec-9fe3-6f1f5c08231b", "timestamp": 998877, "scoutId": 6, "value": 5.9, "maxPercent": 10}' --accountId $AID
# echo "🔵_____View tokens."
# exe near view $CID test_show_nfts '{}'
# echo "🔵_____View content."
# near view $CID dash_get_contents
# echo "🟢_____State after."
# ./bal.sh
#


#near view nftz2h.setalosas.testnet get_contract_id
#near view nftz2h.setalosas.testnet get_contract_cnt
#near call nftz2h.setalosas.testnet 'add_cnt {"cnt": "10"}' --account-id setalosas.testnet
#near view nftz2h.setalosas.testnet get_contract_cnt
#near call nftz2h.setalosas.testnet add_bids '{}' --account-id setalosas.testnet
#near call nftz2h.setalosas.testnet nft_mint '{"token_id": "0", "receiver_id": "setalosas.testnet", "token_metadata": { "title": "Some Art", "description": "My NFT media", "media": "https://assets.change.org/photos/2/lt/qf/TNLTqfJKJcJUwqj-800x450-noPad.jpg?1571334802", "copies": 1}}' --accountId setalosas.testnet --deposit 0.1
