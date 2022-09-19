#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
HI="\\x1b[38;5;208m"
echo -e "🟢"$HI"_____licence_buy TX (amount too low)."
exe near call $CID test_buy '{"ix":0, "price":3}' --accountId setalosas.testnet --gas 300000000000000 --deposit 3.001
echo -e "🟢"$HI"_____licence_buy TX."
exe near call $CID test_buy '{"ix":0, "price":30}' --accountId setalosas.testnet --gas 300000000000000 --deposit 30.001
exe near call $CID showContentListWithBidding --accountId $AID
exe near call $CID show_nfts --accountId $AID