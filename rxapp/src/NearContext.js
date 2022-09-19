/* eslint-disable spaced-comment, no-console, quotes, object-curly-spacing, no-void, 
   no-unused-expressions, prefer-template, react/prop-types
*/
import { useState, createContext, useEffect, useContext } from 'react'
import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'

// We connect to both the NFT (view) and the FT (COTO) (change) contracts here.

const FT_CONTRACT_NAME = 'ft1.setalosas.testnet'
const BID_CONTRACT_NAME = 'nft64.setalosas.testnet'

const nearFtConfig = {
  networkId: 'testnet',
  nodeUrl: 'https://rpc.testnet.near.org',
  contractName: FT_CONTRACT_NAME,
  walletUrl: 'https://wallet.testnet.near.org',
  helperUrl: 'https://helper.testnet.near.org',
  explorerUrl: 'https://explorer.testnet.near.org'
}
const nearBidConfig = {
  ...nearFtConfig,
  contractName: BID_CONTRACT_NAME
}

// Initialize contract & set global variables
const initContract = async () => {
  const nearFt = await connect({
    ...nearFtConfig,
    deps: {keyStore: new keyStores.BrowserLocalStorageKeyStore()}
  })
  const nearBid = await connect({
    ...nearBidConfig,
    deps: {keyStore: new keyStores.BrowserLocalStorageKeyStore()}
  })

  // Initializing Wallet based Account. It can work with NEAR testnet wallet that
  // is hosted at https://wallet.testnet.near.org
  window.walletConnection = new WalletConnection(nearFt) // nearBid is read only, no need for wallet
  console.log(`initContract / walletConnection:`, window.walletConnection)

  // Getting the Account ID. If still unauthorized, it's just empty string
  const accountId = window.accountId = window.walletConnection.getAccountId()
  console.log(`initContract / accountId:`, window.accountId)

  // Initializing our contract APIs by contract name and configuration
  window.bidContract = await new Contract(window.walletConnection.account(), nearBidConfig.contractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: [
      'get_contract_id', 
      'get_bidding_state', 
      'get_content_nfts_for',
      'get_nft_owners_for',
      'nft_tokens',
      'nft_tokens_for_owner',
      'dash_get_contents'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['add_bid'],
  })
  window.ftContract = await new Contract(window.walletConnection.account(), nearFtConfig.contractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: [],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['ft_transfer_call'],
  })
  console.log(`initContract / FT contract:`, window.ftContract)
  console.log(`initContract / bid contract:`, window.bidContract)

  return { loggedIn: window.walletConnection.isSignedIn(), accountId }
}

export const logout = () => {
  window.walletConnection.signOut()
  window.location.replace(window.location.origin + window.location.pathname) // reload page
}

export const login = () => {
  // Allow the current app to make calls to the specified contract on the user's behalf.
  // This works by creating a new access key for the user's account and storing
  // the private key in localStorage.
  window.walletConnection.requestSignIn(nearFtConfig.contractName)
}

// const clog = (...args) => console.log('🍄' + args.shift(), ...args)

const shallowObjectEq = (objA, objB) => {
  if( typeof objA !== 'object') {
    return objA === objB
  }
  for (const key in objA) {
    if (objA[key] !== objB[key]) {
      // console.log(`shallowDiff in key ${key}:`, objA[key], objB[key])
      return false
    }
  }
  return true
}

const normalizeContentRec = async (rec, biddingContract) => {
  console.log('contentRec', rec)
  const {creatorId, bidvalArr, tokensArr} = rec

  if (creatorId) { // we have bids here
    const nftOwnersArr = await biddingContract.get_nft_owners_for({tokensArr})
    console.log({nftOwnersArr})
    const maxSlots = bidvalArr.length
    const hash = {}
    for (let i = 0; i < maxSlots; i++) {
      const owner = nftOwnersArr[i]
      hash[owner] || (hash[owner] = {sum: 0, pt: 0})
      hash[owner].sum += bidvalArr[i]
      hash[owner].pt++
    }
    rec.normArr = Object.entries(hash).map(([key, {sum, pt}]) => ({owner: key, sum, pt}))
    console.log('contentRec mod', rec)
  }
}

const NearContext = createContext({})

export const useNearContext = () => useContext(NearContext)

export const NearProvider = ({children}) => {
  const [nearState, setNearState] = useState({loggedIn: false, accountId: ''})
  const nearContext = {
    ...nearState, login, logout, normalizeContentRec, 
    biddingContract: window.bidContract,
    ftContract: window.ftContract
  }

  console.log('NearContext', nearState)

  useEffect(() => {
    let isMounted = true
    
    initContract().then(state => {
      if (isMounted) {
        if (!shallowObjectEq(state, nearState)) {
          console.log('NearContext changed', {state, nearState})
          setNearState({...state}) // no need
        }
      }
    })

    return () => isMounted = false
  }, [nearState])  

  return <NearContext.Provider children={children} value={nearContext} />
}

// Example:
// const {loggedIn, accountId, login, logout, biddingContract} = useNearContext()