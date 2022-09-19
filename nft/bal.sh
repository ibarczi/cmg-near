#!/bin/bash
near state $CID | grep -E 'Account|formattedAmount:'
near state setalosas.testnet | grep -E 'Account|formattedAmount:'
near state kremilek.testnet | grep -E 'Account|formattedAmount:'
# near state vochomurka.testnet | grep -E 'Account|formattedAmount:'
near state helmut.testnet | grep -E 'Account|formattedAmount:'
near state gertrude.testnet | grep -E 'Account|formattedAmount:'