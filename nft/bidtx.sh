#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
HI="\\x1b[38;5;208m"
echo -e "🟢"$HI"_____3x bid TXs."
near call $CID test_bid '{"ix":0, "value":6, "pt": 2}' --accountId helmut.testnet --gas 300000000000000 --deposit 6.001             & near call $CID test_bid '{"ix":0, "value":6, "pt":3}' --accountId gertrude.testnet --gas 300000000000000 --deposit 6.001            & near call $CID test_bid '{"ix":0, "value":6, "pt":1}' --accountId setalosas.testnet --gas 300000000000000 --deposit 6.001
