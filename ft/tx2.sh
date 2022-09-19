#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
echo -e "🟢"$HI"_____Call NFT contract with COTO transfer."
exe near call $FTID ft_transfer_call '{"receiver_id": "'$CID'", "amount": "11500", "msg": "bid:1:22.5:creator.testnet:85d491b3-18f8-40f6-be33-b83dd749a8a4:123367777"}' --accountId krtek.testnet --depositYocto 1 --gas 300000000000000 
