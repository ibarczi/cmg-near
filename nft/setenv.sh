#!/bin/bash
exe() { echo -e "\\x1b[33m+ $@ \x1b[0m" ; "$@" ; }
ee() { echo -e "\\x1b[33m+ $@" ; }
read sub < sub.txt
exe export CID=$sub".setalosas.testnet"
exe export AID=setalosas.testnet
export HI="\\x1b[38;5;208m"

./redep.sh
. bal.sh
