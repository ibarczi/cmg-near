import { useEffect, useState } from 'react';
import { useNearContext } from './NearContext'

// This component displays all the content NFTs grouped by owner (for dev).

export const OwnerNfts = () => {
  const [nftOwners, setNftOwners] = useState({})
  const {loggedIn, biddingContract} = useNearContext()

  useEffect(() => {
    let mounted = true
    if (loggedIn && biddingContract) {
      console.log({biddingContract})
      biddingContract.nft_tokens({})
        .then(tokens => {
          if (mounted) {
            const ownerHash = {}
            for (const token of tokens) {
              const {token_id, owner_id, metadata: {title, media, extra: contentKey}} = token || {}
              const pt = ~~title.split('%')[0]
              // console.log({token_id, owner_id, contentKey, title, media, pt})
              ownerHash[owner_id] || (ownerHash[owner_id] = [])
              ownerHash[owner_id].push({token_id, contentKey, title, media, pt})
            }
            setNftOwners(ownerHash)
          }
        })
        .catch(err => console.warn('nft_tokens', err))
    }
    return () => mounted = false  
  }, [loggedIn, biddingContract])

  return (
    <div className="DashContents">
      {loggedIn
        ? Object.keys(nftOwners).map(key => 
          <div {...{key}} className='nftFrame'>
            <div className='nftHead'>{`NFTs for ${key}:`}</div>
            {nftOwners[key].map(({pt, media, title}, ix) => 
              <div key={ix}>
                <div>{title}</div>
                {/* <img alt='' className={pt > 50 ? 's90' : ''} src={media} /> */}
              </div>
            )}
          </div>)
        : <div>No NFTs</div>}
    </div>
  )
}
