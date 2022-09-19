#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
eko() { echo -e "$1"$HI"$2" ; }
set -e
eko "🟣" "_____Build contract."
exe eval "RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release"
set +e
eko "🟤" "_____Copy contract."
exe cp target/wasm32-unknown-unknown/release/*.wasm ./res/
eko "🔴" "_____Delete old contract / account, create new."
exe eval "echo y | near delete $FTID $AID"
eko "🟠" "_____create new contract account."
exe near create-account $FTID --masterAccount $AID --initialBalance 30
set -e
eko "🟡" "_____Deploy contract."
exe near deploy $FTID --wasmFile res/coto.wasm
set +e
eko "🔵" "_____Init FT contract."
exe near call $FTID new_default_meta '{"owner_id": "'$AID'", "total_supply": "1000000000"}' --accountId $AID
eko "🟢" "_____Register user accounts."
near call $FTID storage_deposit '{"account_id": "krtek.testnet"}' --accountId $AID --amount 0.00125 &
near call $FTID storage_deposit '{"account_id": "kremilek.testnet"}' --accountId $AID --amount 0.00125 &
near call $FTID storage_deposit '{"account_id": "botticelli.testnet"}' --accountId $AID --amount 0.00125
near call $FTID storage_deposit '{"account_id": "creator.testnet"}' --accountId $AID --amount 0.00125
eko "🟢" "_____Register contract account <"$CID">."
exe near call $FTID storage_deposit '{"account_id": "'$CID'"}' --accountId $AID --amount 0.00125

# . bal.sh
# echo "🟢_____Transfer FT to user."
# exe near call $FTID ft_transfer '{"receiver_id": "krtek.testnet", "amount": "70000"}' --accountId $AID --depositYocto 1
# . bal.sh
# exe near call $FTID ft_transfer_call '{"receiver_id": "'$CID'", "amount": "11500", "msg": "turbo-rozi-szolarium"}' --accountId krtek.testnet --depositYocto 1 --gas 300000000000000 