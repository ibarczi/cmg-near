#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
read ftsub < ftsub.txt
exe export FTID=$ftsub".setalosas.testnet"
exe export AID=setalosas.testnet
exe export CID=nft64.setalosas.testnet
export HI="\\x1b[38;5;208m"
./redep.sh
. bal.sh
# ./tx1.sh
# . bal.sh
# ./tx2.sh
# . bal.sh
