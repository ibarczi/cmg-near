#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
echo -e "🟢"$HI"_____Transfer FT to user."
exe near call $FTID ft_transfer '{"receiver_id": "krtek.testnet", "amount": "70000"}' --accountId $AID --depositYocto 1
exe near call $FTID ft_transfer '{"receiver_id": "kremilek.testnet", "amount": "70000"}' --accountId $AID --depositYocto 1
exe near call $FTID ft_transfer '{"receiver_id": "botticelli.testnet", "amount": "70000"}' --accountId $AID --depositYocto 1
exe near call $FTID ft_transfer '{"receiver_id": "'$CID'", "amount": "70000"}' --accountId $AID --depositYocto 1
