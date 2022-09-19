import { useEffect, useState } from 'react'
import { useNearContext } from './NearContext'
import { OwnerNfts } from './OwnerNfts'
import './App.css'

const bidPt = 10  // 10 or 20 percent buckets / slots (must be changed together w contract)
const bidData = [ // bidding values for predefined buttons, can be anything
  {value: bidPt * 1, maxPercent: bidPt / 2},
  {value: bidPt * 2, maxPercent: bidPt / 3},
  {value: 16.0, maxPercent: 2},
  {value: 12.0, maxPercent: 3},
  {value: 10.0, maxPercent: bidPt},
  {value: 9.0, maxPercent: 4}
]

const contentData = [ // sample content data
  ['85d491b3-18f8-40f6-be33-b83dd749a8a4', 'kremilek.testnet', 125000000],
  ['9d724602-caa5-4aec-9fe3-6f1f5c08231b', 'donatello.testnet', 126000000],
  ['770d7dd3-8c33-4834-9ba9-b5499a4b2e85', 'kremilek.testnet', 127000000],
  ['5e7668b5-015b-4392-970d-aaff3d0091e2', 'vochomurka.testnet', 133000000],
  ['b82501f5-edb7-4e2f-a055-7c31b036f433', 'setalosas.testnet', 175000000],
  ['85d491b3-18f8-40f6-be33-b83dd749a8a4', 'creator.testnet', 123367777]
]

const ContentBiddingState = ({contentId, creatorId, timestamp}) => {
  const [content, setContent] = useState({})
  const {loggedIn, accountId, normalizeContentRec, biddingContract, ftContract} = useNearContext()
  const {bidvalArr, tokensArr, normArr} = content

  useEffect(() => {
    let mounted = true
    if (loggedIn) {
      biddingContract.get_bidding_state({contentId, timestamp, creatorId})
        .then(async rec => {
          await normalizeContentRec(rec, biddingContract) 
          mounted && setContent(rec)
        })
    }
    return () => mounted = false
  }, [biddingContract, loggedIn, normalizeContentRec, contentId, creatorId, timestamp])
  
  const doBid = (value, maxPercent) => { // handler for bid buttons
    const msg = `bid:${maxPercent}:${value}:${creatorId}:${contentId}:${timestamp}`
    const bidPars = {
      receiver_id: biddingContract.contractId,
      msg,
      amount: '' + value
    }
    console.log('Will call ft_transfer_call:', bidPars)
    ftContract?.ft_transfer_call(
      bidPars,
      "300000000000000",         // attached GAS (optional)
      1                          // attached deposit in yoctoNEAR
    )
  }

  return (
    <div className='appContentRec shBox'>
      <div className='appContentRecLeft'>
        <div>{contentId}</div>
        <div>Creator: {creatorId}</div>
        <div>Timestamp: {timestamp}</div>
      </div>
      <div className={`appContentRecMid ${tokensArr?.[0] ? '' : 'nobids'}`}> 
        <div key='999' className='bidPt shBox sum'>
          {normArr?.map(({owner, pt, sum}) => 
            <div key={owner} className={owner === creatorId ? 'own' : ''}>
              {owner === creatorId
                ? `${owner.split('.')[0]}/creator: ${pt}%`
                : `${owner.split('.')[0]}: $${sum.toFixed(1)} for ${pt}%`
              }
            </div>
          )}
        </div>
        {Array(bidPt).fill(0).map((_, ix) => 
          <div key={ix} className='bidPt shBox'>
            <div className='pt'>1%</div>
            <div className={`val ${bidvalArr?.[ix] ? '' : 'zero'}`}>{(bidvalArr?.[ix] || 0).toFixed(2)}$</div>
            <div className='token'>{tokensArr?.[ix] || 0}</div>
          </div>
        )}
      </div>
      <div className='appContentRecRight'>
        {bidData.map(({value, maxPercent}, ix) => 
          <button className='bidButton' key={ix} onClick={() => doBid(value, ~~maxPercent)}>
            {`${~~maxPercent}% for ${~~value}$`}
          </button>
        )}
      </div>
    </div>
  )
}

const AppHeader = ({children}) => {
  const {loggedIn, accountId, login, logout} = useNearContext()

  return (
    <div className='appHeader shBox'>
      {loggedIn
        ? <>
            {`${accountId} logged in.`}
            <button onClick={logout}>Sign out</button>
          </>
        : <button onClick={login}>Sign in</button>
      } 
      {children}
    </div>
  )
}

const AppContent = ({children}) => <div className='appContent shBox'>{children}</div>

export const App = () => {
  const [contractStatus, setContractStatus] = useState('')
  const {loggedIn, biddingContract} = useNearContext()

  useEffect(() => {
    let mounted = true
    if (loggedIn && biddingContract) {
      biddingContract.get_contract_id().then(id => 
        mounted && setContractStatus(`${biddingContract.contractId || '-'} / ${id}`))
    }
    return () => mounted = false  
  }, [loggedIn, biddingContract])

  return (
    <div className='App'>
      <AppHeader>
        Contract: {contractStatus}
      </AppHeader>
      {loggedIn
        ? <AppContent>
            {contentData.map(([contentId, creatorId, timestamp], ix) =>
              <ContentBiddingState key={ix} {...{contentId, creatorId, timestamp}} />
            )}
          </AppContent>
        : <>
            <h1>Welcome to CMG!</h1>
            <p>To make use of the NEAR blockchain, you need to sign in.</p>
          </>}
      <OwnerNfts />
    </div>
  )
}
