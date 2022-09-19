/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use near_contract_standards::non_fungible_token::metadata::{
  NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::json_types::Base64VecU8;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::utils::assert_one_yocto;
use near_sdk::{
  log, env, require, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
  Gas, ext_contract,
  serde_json::{json}
};
use near_sdk::json_types::U128; //.todo: temp, remove later

use std::collections::HashMap;
use serde::ser::{Serialize, SerializeStruct, Serializer};

// pub mod external;
// pub use crate::external::*;

pub const COTO_GAS: Gas = Gas(2_000_000_000_000);
pub const REMAIN_GAS: Gas = Gas(10_000_000_000_000);
#[ext_contract(ext_contract)]
trait ExtContract {
  fn cross_call_test(&self);
  //fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
  fn callback();
  //fn on_buy_service(service_id: u64) -> Service;
}

const BIDPT: usize = 10;
const E24: u128 = 1_000_000_000_000_000_000_000_000;
const TREASURY_ID: &str = &"botticelli.testnet";
const defaultTokenMetadata: TokenMetadata = TokenMetadata {
  title: None,
  description: None,
  extra: None,
  media: None,
  copies: Some(1 as u64),
  media_hash: None,
  issued_at: None,
  expires_at: None,
  starts_at: None,
  updated_at: None,
  reference: None,
  reference_hash: None
};
const testContentData: [(&str, &str, u64); 5] = [
  ("85d491b3-18f8-40f6-be33-b83dd749a8a4", "kremilek.testnet", 125000000),
  ("9d724602-caa5-4aec-9fe3-6f1f5c08231b", "donatello.testnet", 126000000),
  ("770d7dd3-8c33-4834-9ba9-b5499a4b2e85", "kremilek.testnet", 127000000),
  ("5e7668b5-015b-4392-970d-aaff3d0091e2", "vochomurka.testnet", 133000000),
  ("b82501f5-edb7-4e2f-a055-7c31b036f433", "setalosas.testnet", 157000000)
];

#[derive(Debug)]
#[derive(BorshDeserialize, BorshSerialize)]
#[derive(serde::Serialize)]
pub struct ContentKey { //.rem: ContentKey___________________________
  key: String,
  contentId: String,
  creatorId: String,
  timestamp: u64
}

#[derive(Debug)]
#[derive(BorshDeserialize, BorshSerialize)]
#[derive(serde::Serialize)]
pub struct ContentRec {  //.rem: ContentRec___________________________
  creatorId: String,
  contentId: String,
  timestamp: u64,
  bidvalArr: [f32; BIDPT],
  tokensArr: [usize; BIDPT],
  tokenId: usize
}

#[derive(Debug)] //.fix: cleanup these
#[derive(BorshDeserialize, BorshSerialize)]
#[derive(serde::Serialize)]
pub struct AggRec {
  ownerId: String,
  pt: i32
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract { //.rem: Contract________________________________
  tokens: NonFungibleToken,
  licenceTokens: NonFungibleToken,
  lastTokenId: usize,
  lastContentTokenId: usize, //.todo  max range?
  metadata: LazyOption<NFTContractMetadata>,
  licenceMetadata: LazyOption<NFTContractMetadata>,
  cnt: u32,
  contents: HashMap<String, ContentRec>, //.fix  HashMap vs UnorderedMap
  emptyContentRec: ContentRec,
  lock: u32           //.fix  this should be in contentRec
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
  NonFungibleToken,
  Metadata,
  TokenMetadata,
  Enumeration,
  Approval,
}

#[near_bindgen]
impl Contract {
  //.dev  test methods

  pub fn get_contract_id(&self) -> String { format!("env::current_account_id() {}", env::current_account_id()) }
  pub fn get_contract_cnt(&self) -> u32 { self.cnt }
  pub fn inc_cnt(&mut self) { self.cnt = self.cnt + 1; }
  pub fn add_cnt(&mut self, cnt: u32) { self.cnt = self.cnt + cnt; }

  //.rem  original nft placeholder stuff modified for 2x nfts

  /// Initializes the contract owned by `owner_id` with default metadata (for example purposes only).
  #[init]
  pub fn new_default_meta(owner_id: AccountId) -> Self {
    Self::new(
      owner_id,
      NFTContractMetadata {
        spec: NFT_METADATA_SPEC.to_string(),
        name: format!("COW CMG content owner NFT {}", env::current_account_id()),
        symbol: "COW".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        base_uri: None,
        reference: None,
        reference_hash: None,
      },
      NFTContractMetadata {
        spec: NFT_METADATA_SPEC.to_string(),
        name: format!("CLI CMG licence NFT {}", env::current_account_id()),
        symbol: "CLI".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        base_uri: None,
        reference: None,
        reference_hash: None,
      },
    )
  } // calls -> new

  #[init]
  pub fn new(owner_id: AccountId, metadata: NFTContractMetadata, licenceMetadata: NFTContractMetadata)
    -> Self {
    log!("{}****Classic new is called for {}********{}", FgOrange, env::current_account_id(), R);
    
    assert!(!env::state_exists(), "Already initialized");
    metadata.assert_valid();
    Self {
      tokens: NonFungibleToken::new(
        StorageKey::NonFungibleToken,
        owner_id.clone(),
        Some(StorageKey::TokenMetadata),
        Some(StorageKey::Enumeration),
        Some(StorageKey::Approval),
      ),
      licenceTokens: NonFungibleToken::new(
        StorageKey::NonFungibleToken,
        owner_id.clone(),
        Some(StorageKey::TokenMetadata),
        Some(StorageKey::Enumeration),
        Some(StorageKey::Approval),
      ),
      lastTokenId: 1_000,
      lastContentTokenId: 1_000_000,
      metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
      licenceMetadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
      cnt: 0, 
      contents: HashMap::new(),
      emptyContentRec: Self::create_empty_content_rec(),
      lock: 0
    }
  }

  //.pub  dash accessors

  pub fn dash_get_contents(&self) -> Vec<&ContentRec> {
    let mut ret: Vec<&ContentRec> = Default::default();
    for (contentKeyKey, contentRec) in &self.contents {
      ret.push(contentRec);
    }
    ret
  }

  //.dev  debug_______________________________

  pub fn spectrum() {
    log!("{FgRed}: FgRed ======= testX{}", R);
    log!("{FgGreen}: FgGreen ======= testX{}", R);
    log!("{FgBrightGreen}: FgBrightGreen ======= testX{}", R);
    log!("{FgYellow}: FgYellow ======= testX{}", R);
    log!("{H}: H ======= testX{}", R);
    log!("{FgBlue}: FgBlue ======= testX{}", R);
    log!("{FgLime}: FgLime ======= testX{}", R);
    log!("{FgCyan}: FgCyan ======= testX{}", R);
    log!("{FgWhite}: FgWhite ======= testX{}", R);
    log!("{R}: R ======= testX{}", R);
    log!("{FgOrange}: FgOrange ======= testX{}", R);
    log!("{FgPurple}: FgPurple ======= testX{}", R);
    log!("{FgA}: FgA ======= testX{}", R);
    log!("{FgZ}: FgZ ======= testX{}", R);
    log!("{FgT}: FgT ======= testX{}", R);
    log!("{FgX}: FgX ======= testX{}", R);
  }

  fn print_env(&self, pre: &str) {
    log!("{pre}env::current_account_id(): {}{R}", env::current_account_id());
    log!("{pre}env::account_balance: {} NEAR{R}", (env::account_balance() as f64) / 1E24);
    log!("{pre}env::signer_account_id: {}{R}", env::signer_account_id());
    log!("{pre}env::predecessor_account_id: {}{R}", env::predecessor_account_id());
    log!("{pre}env::attached_deposit: {} NEAR{R}", (env::attached_deposit() as f32) / 1E24);
    log!("{pre}{}{R}", ugas());
    log!("{pre}env::prepaid_gas: {} G{R}", env::prepaid_gas().0 / 1_000_000_000);
    log!("{pre}env::storage_usage: {}{R}", env::storage_usage());
  }
    
  fn showContentInfo(&self, msg: &str, contentId: &str, contentRec: &ContentRec) {
    log!("{FgX}{msg}: {}{R}", contentId); 
  }

  fn _showContentList(&self, withBidding: bool) {
    log!("{FgYellow}Current content list:{}", R);
    for (contentKeyKey, contentRec) in &self.contents {
      self.showContentInfo("Content", &contentKeyKey, &contentRec);
      if withBidding {
        if contentRec.tokenId > 0 {
          let scoutList = self.get_nft_owners_for(contentRec.tokensArr.clone());
          for i in 0..BIDPT {
            log!("{FgT}Slot[{i}]: ${} #{} for {}{R}", 
              contentRec.bidvalArr[i], tokenId2Str(contentRec.tokensArr[i]), scoutList[i]);
          }
        } else {
          log!("No bids for {}", contentKeyKey);
        }
      }
    }  
  }
  pub fn showContentList(&self) { self._showContentList(false) }
  pub fn showContentListWithBidding(&self) { self._showContentList(true) }

  pub fn show_content_nfts(&self) {
    let tokens = self.tokens.nft_tokens(None, Some(1000));
    for token in tokens {
      let meta = token.metadata.unwrap();
      log!("{FgOrange}#{} {FgCyan}{} {R}{} {FgGreen}{}{R}", 
        token.token_id, token.owner_id.to_string(), meta.title.unwrap(), meta.media.unwrap());
    }
  }

  pub fn show_licence_nfts(&self) {
    let tokens = self.licenceTokens.nft_tokens(None, Some(1000));
    for token in tokens {
      let meta = token.metadata.unwrap();
      log!("{FgOrange}#{} {FgCyan}{} {R}{} {FgGreen}{}{R}", 
        token.token_id, token.owner_id.to_string(), meta.title.unwrap(), meta.media.unwrap());
    }
  }

  pub fn show_nfts(&self) {
    self.show_content_nfts();
    self.show_licence_nfts();
  }

  // rem subgraph serializers

  fn emit_content_bid(&self, contentKey: &ContentKey) {
    let contentRef: &ContentRec = &self.contents[&contentKey.key];
    let mut json = String::from(format!("EVENT_JSON:{{\"event\": \"content_bid\", \"data\": {{\"content_id\": \"{}\", \"bids\": [", contentKey.key));

    let aggMap: HashMap<String, (i32, f32)> = self.get_content_owners_pts_vals(&contentKey);
    let date = 1270000000;
    let mut comma = "";
    for (owner, ptval) in &aggMap {
      log!("owner: {} pt: {} val: {}", owner, ptval.0, ptval.1);
      json.push_str(&format!("{}{{\"owner\":\"{}\", \"value\": {}, \"percentage\": {}, \"date\": {} }}", 
         comma, owner, ptval.1, ptval.0, date));
      comma = ",";   
    }
    json.push_str(&"]}}");
    log!("{}", json);
  }
  
  fn emit_content_licensing(&self, contentKey: &ContentKey, scoutId: &str, val: f32) {
    let contentRef: &ContentRec = &self.contents[&contentKey.key];
    log!(format!("EVENT_JSON:{{\"event\": \"content_licensing\", \"data\": {{\"content_id\": \"{}\", \"licence\": {{\"buyer\": \"{}\", \"price\": {} }} }} }}", contentKey.key, scoutId, val));
  }

  fn emit_transfer_funds(&self, msg: &str, from: &str, to: &str, val: f32) {
    // let fmt = String::from(r#"EVENT_JSON:{{"event": "transfer_funds", "data": {{"from": "{}", "to": "{}" "value": "{}"}}"#);
    // log!(format!(fmt, from, to, val));
    //.rem  Rust is hell
    log!(format!("EVENT_JSON:{{\"event\": \"transfer_funds\", \"data\": {{\"from\": \"{}\", \"to\": \"{}\" \"value\": \"{}\"}} }}", env::signer_account_id(), to, val));
    //.fix: no from param needed, it's always the signer
  }
  
  // int basic content ops

  fn create_empty_content_rec() -> ContentRec {
    ContentRec {
      contentId: String::from(""),
      creatorId: String::from(""), 
      timestamp: 0,
      bidvalArr: [0.0; BIDPT],
      tokensArr: [0; BIDPT],
      tokenId: 0
    }
  }

  fn create_content_key(contentId: &str, creatorId: &str, timestamp: u64) -> ContentKey { 
    ContentKey { 
      key: format!("{contentId}:{creatorId}:{timestamp}"), 
      contentId: String::from(contentId),
      creatorId: String::from(creatorId),
      timestamp
  } }

  fn create_new_content(&mut self, contentKey: &ContentKey) {
    let content: ContentRec = ContentRec { 
      creatorId: contentKey.creatorId.clone(), 
      contentId: contentKey.contentId.clone(),
      timestamp: contentKey.timestamp,
      ..Self::create_empty_content_rec()
    };
    self.contents.insert(contentKey.key.clone(), content);
    self.create_content_nfts(&contentKey);
  }

  fn get_content_or_none(&self, contentKey: &ContentKey) -> Option<&ContentRec> {
    match self.contents.get(&contentKey.key) {
      Some(content) => Some(content),
      None => { 
        log!("{FgPurple}contentKey not found. {}{R}", contentKey.key);
        None
      }
    }
  }
  
  // rem NFT methods

  fn get_next_tokenid(&mut self) -> usize {
    self.lastTokenId = self.lastTokenId + 100;
    self.lastTokenId
  }

  fn get_next_licence_tokenid(&mut self) -> usize {
    self.lastContentTokenId = self.lastContentTokenId + 1;
    self.lastContentTokenId
  }

  fn create_content_nft(&mut self, contentKey: &ContentKey, tokenId: usize, pt: usize) {
    let receiverId = AccountId::new_unchecked(contentKey.creatorId.to_string());
    let tokenIdStr = tokenId2Str(tokenId);
    let extra: String = contentKey.key.clone();
    let url = tokenId2URL(tokenId);
    let title = format!("{}% of {} #{}", pt, contentKey.key, tokenIdStr);

    log!("create_content_nft: balance={H}{}{R}mNEAR ugas={H}{:#?}{R}G token: #{} for {} ({}%)", 
      1000 * env::account_balance() / E24, 
      env::used_gas().0 / 1_000_000_000,
      tokenIdStr, receiverId, pt
    );

    let token_metadata = Some(TokenMetadata {
      title: Some(title),
      description: None,
      extra: Some(extra),
      media: Some(url),
      ..defaultTokenMetadata
    });
    //self.tokens.internal_mint(tokenIdStr, receiverId, token_metadata);
    //.rem  calling non-standard minting method:
    self.tokens.internal_mint_with_refund(tokenIdStr, receiverId, token_metadata, None);
  }

  fn create_licence_nft(&mut self, contentKey: &ContentKey, receiverId: &AccountId, price: f32) {
    let tokenId = self.get_next_licence_tokenid();
    let tokenIdStr = tokenId2Str(tokenId);
    let extra: String = contentKey.key.clone();
    let url = tokenId2URL(tokenId);
    let title = format!("${} for {} #{}", price, contentKey.key, tokenIdStr);

    log!("create_licence_nft: balance={H}{}{R}mNEAR ugas={H}{:#?}{R}G token: #{} for {} (${})", 
      1000 * env::account_balance() / E24, 
      env::used_gas().0 / 1_000_000_000,
      tokenIdStr, receiverId, price
    );

    let token_metadata = Some(TokenMetadata {
      title: Some(title),
      description: None,
      extra: Some(extra),
      media: Some(url),
      ..defaultTokenMetadata
    });
    //self.tokens.internal_mint(tokenIdStr, receiverId, token_metadata);
    //.rem  calling non-standard minting method:
    self.licenceTokens.internal_mint_with_refund(tokenIdStr, receiverId.clone(), token_metadata, None);
  }

  // rem create 1 + BIDPT content nfts

  fn create_content_nfts(&mut self, contentKey: &ContentKey) -> usize { // no need to return tokenId
    let tokenIdRef = self.get_next_tokenid();
    self.create_content_nft(contentKey, tokenIdRef, 100 - BIDPT);

    for i in 1..(BIDPT + 1) {
      self.create_content_nft(contentKey, tokenIdRef + i, 1);
    }
    let mut contentRec: &mut ContentRec = self.get_content_by_key_unguarded(&contentKey);
    contentRec.tokenId = tokenIdRef;
    let mut ret = [0; BIDPT];
    for i in 0..BIDPT {
      ret[i] = tokenIdRef + i + 1;
    }
    contentRec.tokensArr = ret;

    tokenIdRef
  }

  // rem create licence nft

  pub fn get_nft_owners_for(&self, tokensArr: [usize; BIDPT]) -> [String; BIDPT] {
    let mut ret: [String; BIDPT] = Default::default();
    let mut i = 0;
    for tokenId in tokensArr {
      let token_id = tokenId2Str(tokenId);
      let nft = match self.tokens.nft_token(token_id.clone()) { //.fix: should fail here
        Some(val) => val,
        None => Token { //.fix: must fail here
          token_id: "0".to_string(),
          approved_account_ids: None,
          metadata: None,
          owner_id: AccountId::new_unchecked("setalosas.testnet".to_string())
        }, // some error logic
      };
      let owner_id = nft.owner_id.to_string();
      //.rem: temp  log!("get_nft_owners_for #{}: {FgCyan}{} {R}", nft.token_id, &owner_id);
  
      ret[i] = owner_id;
      i = i + 1;
    }
    ret
  }

  pub fn modcont(&mut self) { //.rem: test only
    let mut keys: Vec<String> = Vec::new();
    for (contentKeyKey, contentRec) in &self.contents {
      keys.push(String::from(contentKeyKey));
    };
    for key in keys {  
      //let mut contentRec = self.contents.get_mut(&key).unwrap();
      let mut contentRec = self.get_content_by_keykey_unguarded(&key);
      contentRec.tokenId = 2000;
    };
  }

  fn confirm_content_by_key(&mut self, contentKey: &ContentKey) -> bool {
    if let Some(content) = self.get_content_or_none(&contentKey) {
      log!("{FgLime}confirm_content: INFO contentKey found on 1st try! {}{R}", contentKey.key);
      return true
    } else {
      log!("{FgA}confirm_content: WARNING contentKey not found! Creating... {}{R}", contentKey.key);
      self.create_new_content(&contentKey);
      if let Some(content2) = self.get_content_or_none(&contentKey) {
        log!("{FgLime}confirm_content: INFO contentKey found on 2nd try! {}{R}", contentKey.key);
      } else {
        log!("{FgRed}confirm_content: ERROR contentKey not found after 2nd try! {}{R}", contentKey.key);
      }
    }
    false
  }
 
  fn get_content_by_keykey_unguarded(&mut self, key: &str) 
    -> &mut ContentRec { self.contents.get_mut(key).unwrap() }

  // fix contents[str]  

  fn get_content_by_key_unguarded(&mut self, contentKey: &ContentKey) 
    -> &mut ContentRec { self.contents.get_mut(&contentKey.key).unwrap() }

  fn get_content_by_key(&self, contentKey: &ContentKey) -> &ContentRec {
    let someContent = self.get_content_or_none(&contentKey);
    assert!(someContent.is_some(), "get_content ERROR: not found {}", contentKey.key);
    someContent.unwrap()
  }

  pub fn get_bidding_state(&self, contentId: String, timestamp: u64, creatorId: String) -> &ContentRec {
    let contentKey = Self::create_content_key(&contentId, &creatorId, timestamp);
    let contentRef: Option<&ContentRec> = self.contents.get(&contentKey.key);
    return if contentRef.is_some() {
      contentRef.unwrap()
    } else {
      &self.emptyContentRec
    }
  }

  //.pub  COTO transfer listener
  //.rem  possible msg formats: (closing : is optional)
  //.rem  - bid:creator.testnet:85d491b3-18f8-40f6-be33-b83dd749a8a4:123367777:10:22.5:
  //.rem  - buy:creator.testnet:85d491b3-18f8-40f6-be33-b83dd749a8a4:123367777:60.5:

  pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
    let vec: Vec<&str> = msg.split(":").collect();
    
    //.chk  len require
    //.chk  amount vs transferred coto
    
    let op = vec[0];
    let creatorId = String::from(vec[1]);
    let contentId = String::from(vec[2]);
    let timestamp = vec[3].parse::<u64>().unwrap();
    let contentKey = Self::create_content_key(&contentId, &creatorId, timestamp);
    log!("op: {} len: {} content: {}", op, vec.len(), contentKey.key);

    let transferBack: PromiseOrValue<U128> = PromiseOrValue::Value(U128(0));

    if op.eq("bid") {
      require!(vec.len() > 5, "too few parts in bid parameter string");
      let pt = vec[4].parse::<i32>().unwrap();
      let cotoValue = vec[5].parse::<f32>().unwrap();

      log!("val: {} pt: {}", cotoValue, pt);

      self.add_bid_coto(
        contentId, creatorId, timestamp,
        String::from(env::signer_account_id()), cotoValue, pt
      );
      self.showContentListWithBidding();

      // set transferBack
    } else if op.eq("buy") {
      require!(vec.len() > 4, "too few parts in buy parameter string");
      let cotoValue = vec[4].parse::<f32>().unwrap();

      log!("val: {}", cotoValue);

      self.buy_licence(
        contentId, creatorId, timestamp,
        String::from(env::signer_account_id()), cotoValue
      );
      self.showContentListWithBidding();
    }
    
    transferBack
  }

  #[payable]
  pub fn test_bid2(&mut self, ix: usize, value: f32, pt: i32) {
    let content = testContentData[ix];
    self.add_bid_coto(
      String::from(content.0), String::from(content.1), content.2,
      String::from(env::signer_account_id()), value, pt
    );
    self.showContentListWithBidding();
  }

  // id: internal lock & log stuff

  fn lock_start(&mut self, pre: &str) -> bool { //.rem: start locking mechanism (the caller will fail)
    // assert!(self.lock == 0, format!("{FgRed} contract is already locked (in TX) ({}){R}", self.lock));
    if self.lock > 0 {
      log!("{pre} {FgRed} add_bid is already running ({}), will abort.{R}", self.lock);
      false
    } else {
      self.lock = self.lock + 1;                  //.rem: MUTEX start
      log!("{pre}{FgRed} LOCKED ({}).{R}", self.lock);
      true
    }
  }

  fn lock_end(&mut self, pre: &str) { //.rem: end locking mechanism
    self.lock = self.lock - 1;        //.rem: MUTEX end
    log!("{pre}{FgRed} UNLOCKED ({}).{R}", self.lock);
  }
  
  //#f04: add_bid event handler core
  //.rem: on_ft_transfer event listener interface: add_bid_coto

  fn add_bid_coto(&mut self, 
    contentId: String, creatorId: String, timestamp: u64, 
    scoutId: String, cotoValue: f32, maxPercent: i32
  ) {
    let pre = format!("{} {FgOrange}add_bid:", self.lock);
    require!(self.lock_start(&pre), "Aborted on locked call");
    self.print_env(&pre);

    //.rem: checking deposit

    let deposit = env::attached_deposit();
    let depositNEAR = (deposit as f32) / 1E24;
    ////log!("{pre} scout: {} w/ attached deposit: {} NEAR{R}", &scoutId, depositNEAR);
    ////require!(depositNEAR >= value, format!("Requires attached deposit of at least {} NEAR", value));

    //.rem: getting the record

    let contentKey = Self::create_content_key(&contentId, &creatorId, timestamp);
    let contentExisted = self.confirm_content_by_key(&contentKey);
    //let contentRef: &ContentRec = self.get_content_by_key_unguarded(&contentKey);
    let contentRef: &ContentRec = &self.contents[&contentKey.key];

    //.rem: destructuring original content

    let bidvalArrOrig = contentRef.bidvalArr;
    let mut bidvalArrNew = bidvalArrOrig.clone();
    let mut tokensArr = contentRef.tokensArr.clone();
    let creatorId = contentRef.creatorId.clone();
    let contentId = contentRef.contentId.clone();

    //.rem: start bidding

    let bidLimit = cotoValue / maxPercent as f32;
    let mut remainingBids = maxPercent;
    
    log!("{pre} for {}: {FgX}{}% for ${}, bidLimit={}{R}", contentId, maxPercent, cotoValue, bidLimit);
    
    let biddingScoutId: AccountId = scoutId.parse().unwrap();
    let mut usedUpValue: f32 = 0.0;
    
    for bix in 0..BIDPT {
      if remainingBids < 1 {
        break
      }
      if bidvalArrOrig[bix] < bidLimit {
        bidvalArrNew[bix] = bidLimit;
        let token_id = tokenId2Str(tokensArr[bix]);

        let oldValue = bidvalArrOrig[bix];
        let creatorRefund = bidLimit - oldValue;
        let scoutRefund = oldValue;
        
        // retlog.push(format!("Will call transfer_mod with {token_id} {}", &biddingScoutId));
        // log!("token_id={token_id} ->rebid");
        self.rebid(&token_id, &biddingScoutId, scoutRefund);    //.todo: check self-payment (self-outbid)
        // log!("->pay_creator");
        self.pay_creator(&creatorId, creatorRefund); // never zero, no need for if

        remainingBids = remainingBids - 1;
        usedUpValue = usedUpValue + bidLimit;
        log!("{pre} {FgZ}->Percent slot won: %[{bix}] (NFT: {} -> {}) remaining: {} usedUpVal: {} gasUsed: {} {R}",
          "scoutIdOld", scoutId, remainingBids, usedUpValue, env::used_gas().0 / 1_000_000_000);
      }
    }
    //.todo  refund value - usedUpValue;
    let selfRefund = cotoValue - usedUpValue;
    if selfRefund > 0.001 {
      let refund = (selfRefund * 1E24) as u128;
      log!("{FgCyan}Partial or unsuccessful bid, will refund {} NEAR ->{}{R}", selfRefund, scoutId);
      //Promise::new(String::from(&scoutId).parse().unwrap()).transfer(refund);
      self.transfer_funds(&"selfRefund", &scoutId, refund);
      self.emit_transfer_funds("self_refund", &scoutId, &scoutId, selfRefund);
    } else {
      log!("{FgCyan}Successful bid, all funds used up.{}", R);
    }

    fn sortBidding(arr1: &mut [f32; BIDPT], arr2: &mut [usize; BIDPT]) {
      for i in 0..arr1.len() {
        for j in 0..arr1.len() - 1 - i {
          if arr1[j] > arr1[j + 1] {
            arr1.swap(j, j + 1);
            arr2.swap(j, j + 1);
          }
        }
      }
    }
    // for i in 0..BIDPT { log!("before: [{i}] val: {} token: {}", bidvalArrNew[i], tokensArrNew[i]); }
    sortBidding(&mut bidvalArrNew, &mut tokensArr);
    // for i in 0..BIDPT { log!("after: [{i}] val: {} token: {}", bidvalArrNew[i], tokensArrNew[i]); }

    let mut contentRec: &mut ContentRec = self.get_content_by_key_unguarded(&contentKey) ;
    contentRec.bidvalArr = bidvalArrNew;
    contentRec.tokensArr = tokensArr;

    // rem lock unlock

    self.lock_end(&pre);
    
    self.emit_content_bid(&contentKey);
    //.fix: check balance - balance at start, if diff > .1 -> warn
  }

  //#f04: all coto transfers call this one
  
  fn transfer_funds(&mut self, msg: &str, to: &str, amount: u128) -> Promise {
    log!("{LightBlue}transfer_funds called {}->{}->{} {}_COTO {}{R}", env::predecessor_account_id(), env::current_account_id(), to, amount, msg);
    // gas_log("pre::coto_transfer ");
    let coto = U128(amount / 1_000_000_000_000_000_000_000);

    Promise::new(String::from("ft1.setalosas.testnet").parse().unwrap()).function_call(
      String::from("coto_transfer"),
      json!({ "receiver_id": String::from(to), "amount": coto }).to_string().as_bytes().to_vec(),
      1,       // one yocto
      COTO_GAS // 2 Tgas
    )
  }

  //#f90: slot aggregation

  fn addToAgg(&self, ownerId: String, pt: i32, aggMap: &mut HashMap<String, i32>) {
    let rust1 = ownerId.clone(); 
    let rust2 = ownerId.clone();
    let oldpt = aggMap.get(&rust1).unwrap_or(&0);
    let newpt = oldpt + pt;
    aggMap.remove(&rust2);
    aggMap.insert(rust2.clone(), newpt);
  }

  fn get_content_owners_from_arr_internal(&self, tokensArr: [usize; BIDPT], creatorId: String)
    -> HashMap<String, i32> {
    // log!("{FgLime}buy_licence: content found: {}{R}", contentKey.key);
    let mut aggMap: HashMap<String, i32> = HashMap::new();

    let bidderArr: [String; BIDPT] = self.get_nft_owners_for(tokensArr);
    self.addToAgg(creatorId, 100 - BIDPT as i32, &mut aggMap);
    for i in 0..BIDPT {
      self.addToAgg(bidderArr[i].clone(), 1, &mut aggMap);
    }
    aggMap
  }

  pub fn get_content_owners(&self, contentId: String, creatorId: String, timestamp: u64)
    -> HashMap<String, i32> {
    let contentKey = Self::create_content_key(&contentId, &creatorId, timestamp);
    let someContent = self.get_content_or_none(&contentKey);
    require!(someContent.is_some(), format!("No content found with key {}", contentKey.key));

    let content = someContent.unwrap();
    self.get_content_owners_from_arr_internal(content.tokensArr, creatorId)
  }

  // rem dup, modified for values
    
  fn addToAggPtVal(&self, ownerId: String, pt: i32, val: f32, aggMap: &mut HashMap<String, (i32, f32)>) {
    let rust1 = ownerId.clone(); 
    let rust2 = ownerId.clone();
    let oldptval = aggMap.get(&rust1).unwrap_or(&(0, 0.0));
    let newptval = (oldptval.0 + pt, oldptval.1 + val);
    aggMap.remove(&rust2);
    aggMap.insert(rust2.clone(), newptval);
  }

  fn get_content_owners_internal(&self, content: &ContentRec) -> HashMap<String, (i32, f32)> {
    let tokensArr = content.tokensArr.clone();
    let bidvalArr = &content.bidvalArr;
    let bidderArr: [String; BIDPT] = self.get_nft_owners_for(tokensArr);

    let mut aggMap: HashMap<String, (i32, f32)> = HashMap::new();
    self.addToAggPtVal(content.creatorId.clone(), 100 - BIDPT as i32, 0.0, &mut aggMap);

    for i in 0..BIDPT {
      self.addToAggPtVal(bidderArr[i].clone(), 1, bidvalArr[i], &mut aggMap);
    }
    aggMap
  }

  //.fix  make it pub -> contentKey cannot be ref

  fn get_content_owners_pts_vals(&self, contentKey: &ContentKey) -> HashMap<String, (i32, f32)> {
    let someContent = self.get_content_or_none(&contentKey);
    require!(someContent.is_some(), format!("No content found with key {}", contentKey.key));
    let content = someContent.unwrap();

    self.get_content_owners_internal(content)
  }

  // public interface buy_licence

  #[payable]
  pub fn buy_licence(&mut self, 
    contentId: String, creatorId: String, timestamp: u64, scoutId: String, price: f32
  ) {
    //.todo  must create new content item if no bids yet

    let contentKey = Self::create_content_key(&contentId, &creatorId, timestamp);
    let contentExisted = self.confirm_content_by_key(&contentKey);
    let content: &ContentRec = self.get_content_by_key_unguarded(&contentKey);
    log!("{FgLime}buy_licence: content found: {}{R}", contentKey.key);
    
    //.todo: check? no
    //.todo  check for minimal deposit (contentExisted!)

    let deposit = env::attached_deposit();
    let depositNEAR = (deposit as f32) / 1E24;
    log!("{FgLime}buy_licence: attached deposit: {} near by {}{R}", depositNEAR, scoutId);
    require!(depositNEAR >= price, format!("Requires attached deposit of at least {} NEAR", depositNEAR));

    let sum = if contentExisted {
      let mut sum = 0.0;
      let bidvalArr = content.bidvalArr;
      for i in 0..BIDPT {
        sum = sum + bidvalArr[i];
      }
      //3.0 * sum
      1.0 * sum
    } else {
      0.05 // from config
    };
    require!(price >= sum, format!("Price ({price}) must be >= minimum price {sum}"));
    
    let tokensArr = content.tokensArr.clone();
    // log!("{:#?}", tokensArr);    
    let aggMap: HashMap<String, i32> = self.get_content_owners_from_arr_internal(tokensArr, creatorId);
    // log!("{:#?}", aggMap);

    let treasuryShare = price * 0.1;
    log!("{FgLime}buy_licence: payment 10% = {H}{}{FgLime} NEAR ->platform {R}", treasuryShare);
    self.general_pay_near(TREASURY_ID, treasuryShare);

    let remainingPrice = price * 0.9;
    for (owner, pt) in aggMap {
      let ownerPayment = remainingPrice * (pt as f32) / 100.;
      log!("{FgLime}buy_licence: payment {}% = {H}{}{FgLime} NEAR ->{}{R}", pt, ownerPayment, owner);
      self.general_pay_near(&owner, ownerPayment); 
    }
    self.create_licence_nft(&contentKey, &env::predecessor_account_id(), price);
    self.emit_content_licensing(&contentKey, &scoutId, price);
    // fix tokenid ^
  }

  // int system hacks and overrides

  /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
  ///
  /// Since this example implements metadata, it also requires per-token metadata to be provided
  /// in this call. `self.tokens.mint` will also require it to be Some, since
  /// `StorageKey::TokenMetadata` was provided at initialization.
  ///
  /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
  /// initialization call to `new`.
  #[payable]
  pub fn nft_mint(&mut self, token_id: TokenId, receiver_id: AccountId, token_metadata: TokenMetadata)
    -> Token { self.tokens.internal_mint(token_id, receiver_id, Some(token_metadata)) }

  // rem scout reinbursement method
  //
  fn payback_scout(&mut self, token_id: &TokenId, scoutRefund: f32) { //.todo  f32 is not good enough
    let account_id = self.tokens.owner_by_id
      .get(&token_id)
      .unwrap_or_else(|| env::panic_str(&format!("Token {} not found", token_id)));
      
    let refund = (scoutRefund * 1E24) as u128;
    log!("--payback_scout: from current owner ->{} amount: {H}{}{R} NEAR", account_id, scoutRefund);
    let accountId = &account_id.to_string(); // &String::from(&account_id);
    Promise::new(account_id).transfer(refund);
    //self.emit_transfer_funds("scout_payback", accountId, &accountId, scoutRefund);
    self.emit_transfer_funds("scout_payback", accountId, &accountId, scoutRefund);
  }
  // let amount: u128 = 1_000_000_000_000_000_000_000_000; // 1 $NEAR as yoctoNEAR

  // rem creator payment method
  //
  fn pay_creator(&mut self, creatorId: &str, creatorRefund: f32) {
    let refund = (creatorRefund * 1E24) as u128;
    let creator_id: AccountId = String::from(creatorId).parse().unwrap();
    log!("--pay_creator: from bidder to creator ->{} amount: {H}{}{R} NEAR", creatorId, creatorRefund);

    //Promise::new(creator_id).transfer(refund * 9 / 10);
    self.transfer_funds(&"payCreator", creatorId, refund * 9 / 10);

    self.emit_transfer_funds("bid_creator", &creatorId, &creatorId, creatorRefund * 0.9);
    
    //Promise::new(String::from(TREASURY_ID).parse().unwrap()).transfer(refund * 1 / 10);
    self.transfer_funds(&"payCreator", TREASURY_ID, refund * 1 / 10);

    self.emit_transfer_funds("bid_share", &creatorId, &creatorId, creatorRefund * 0.1);
  }

  // rem general_pay_near
  //
  fn general_pay_near(&self, accountId: &str, near: f32) {
    let yoctoNear = (near * 1E24) as u128;
    let account_id: AccountId = String::from(accountId).parse().unwrap();
    // log!("--general_pay_near: ->{} amount: {H}{}{R} NEAR", accountId, near);
    Promise::new(account_id).transfer(yoctoNear);
    self.emit_transfer_funds("general", &accountId, &accountId, near); //.fix: not here
  }
  
  // rem modded version of internal_transfer (no approvals, no event log)
  // Transfer from current owner to receiver_id, return previous owner and approvals.
  pub fn internal_transfer_mod(&mut self,
    sender_id: &AccountId, receiver_id: &AccountId,
    #[allow(clippy::ptr_arg)] token_id: &TokenId
  ) {
    self.tokens.internal_transfer_unguarded(token_id, &sender_id, receiver_id);
    // NonFungibleToken::emit_transfer(&owner_id, receiver_id, token_id, sender_id, memo);
  }

  // rem modified version of the nft_transfer method of near-contract-standards
  // We skip the check (sender === predecessor_account_id) and the (assert_one_yocto check (tmp)).
  // Otherwise it's the same code.
  //
  fn nft_transfer_mod(&mut self, token_id: &TokenId, receiver_id: &AccountId) {
    // assert_one_yocto(); // This needs to be put back at the end!
    let sender_id = self.tokens.owner_by_id
      .get(token_id)
      .unwrap_or_else(|| env::panic_str(&format!("Token {} not found", token_id)));

    log!("--nft_transfer_mod: {H}#{}{R} ->{}", token_id, receiver_id);
    // self.tokens.internal_transfer(&sender_id, &receiver_id, token_id, None, None);
    // self.internal_transfer_mod(&sender_id, &receiver_id, token_id);
    self.tokens.internal_transfer_unguarded(token_id, &sender_id, &receiver_id);
  }

  // rem one method for calling both transfer & reimbursement steps  
  // NFT (of tokenId) will be transferred to biddingScout (from creator or other scout)
  // if refund > 0, the prev scout + creator should be reimbursed (split)
  // if refund = 0, only the creator
  fn rebid(&mut self, token_id: &TokenId, biddingScoutId: &AccountId, scoutRefund: f32) {
    if scoutRefund > 0.0 {                         // only for scout, not for OG creator
      log!("rebid: #{} ->{} scoutRefund: {}", token_id, biddingScoutId, scoutRefund);
      self.payback_scout(token_id, scoutRefund); // this needs to be called first, before the transfer
    } else {
      log!("rebid: 1st bid, no reimbursement.");
    }
    self.nft_transfer_mod(token_id, &biddingScoutId);
  }
  
  // debug / test methods

  #[payable]
  pub fn test_buy(&mut self, ix: usize, price: f32) {
    let cont = testContentData[ix];
    self.buy_licence(String::from(cont.0), String::from(cont.1), cont.2,
      String::from(env::predecessor_account_id()), price
    );
    // self.showContentListWithBidding();
  } 
  #[payable]
  pub fn test_bid(&mut self, ix: usize, value: f32, pt: i32) {
    let content = testContentData[ix];
    self.add_bid_coto(String::from(content.0), String::from(content.1), content.2,
      String::from(env::predecessor_account_id()), value, pt
    );
    self.showContentListWithBidding();
  }
}

fn ugas() -> String { format!("gasUsed: {} G ", env::used_gas().0 / 1_000_000_000) }
fn gas_log(msg: &str) { log!("{Pink}{msg}{}{R}", ugas()); }

fn tokenId2Str(tokenId: usize) -> String { format!("00{:04}", tokenId) }
fn tokenId2URL(tokenId: usize) 
  -> String { format!("https://img.mork.work/lj/{:04}.jpg", (tokenId % 1000) + 819) }

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
  fn nft_metadata(&self) -> NFTContractMetadata {
      self.metadata.get().unwrap()
  }
}
const Pink: &str = "\x1b[38;2;255;100;125m";
const Bright: &str = "\x1b[1m";
const FgRed: &str = "\x1b[31m";
const FgGreen: &str = "\x1b[32m";
const FgBrightGreen: &str = "\x1b[1m\x1b[32m";
const FgYellow: &str = "\x1b[33m";
const H: &str = "\x1b[33m";
const FgBlue: &str = "\x1b[38;5;027m";//'\x1b[34m";
const FgLime: &str = "\x1b[38;5;148m"; // '\x1b[35m";
const FgCyan: &str = "\x1b[38;5;086m"; // '\x1b[36m";
const FgWhite: &str = "\x1b[37m";
const R: &str = "\x1b[0m";
const FgOrange: &str = "\x1b[38;5;202m";
const FgPurple: &str = "\x1b[38;5;127m";
const FgA: &str = "\x1b[38;5;163m"; // magentish
const FgZ: &str = "\x1b[38;5;039m";
const FgT: &str = "\x1b[38;5;035m"; // GGB
const FgX: &str = "\x1b[38;5;042m"; // GGB brighter
const LightBlue: &str = "\x1b[38;2;150;150;255m";

// --------------------NOT VALID FROM HERE (orig nft placeholder tests):--------------------------

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
  use near_sdk::test_utils::{accounts, VMContextBuilder};
  use near_sdk::testing_env;
  use std::collections::HashMap;

  use super::*;

  const MINT_STORAGE_COST: u128 = 5870000000000000000000;

  fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
      let mut builder = VMContextBuilder::new();
      builder
          .current_account_id(accounts(0))
          .signer_account_id(predecessor_account_id.clone())
          .predecessor_account_id(predecessor_account_id);
      builder
  }

  fn sample_token_metadata() -> TokenMetadata {
      TokenMetadata {
          title: Some("Olympus Mons".into()),
          description: Some("The tallest mountain in the charted solar system".into()),
          media: None,
          media_hash: None,
          copies: Some(1u64),
          issued_at: None,
          expires_at: None,
          starts_at: None,
          updated_at: None,
          extra: None,
          reference: None,
          reference_hash: None,
      }
  }

  #[test]
  fn test_new() {
      let mut context = get_context(accounts(1));
      testing_env!(context.build());
      let contract = Contract::new_default_meta(accounts(1).into());
      testing_env!(context.is_view(true).build());
      assert_eq!(contract.nft_token("1".to_string()), None);
  }

  #[test]
  #[should_panic(expected = "The contract is not initialized")]
  fn test_default() {
      let context = get_context(accounts(1));
      testing_env!(context.build());
      let _contract = Contract::default();
  }

  #[test]
  fn test_mint() {
      let mut context = get_context(accounts(0));
      testing_env!(context.build());
      let mut contract = Contract::new_default_meta(accounts(0).into());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(MINT_STORAGE_COST)
          .predecessor_account_id(accounts(0))
          .build());

      let token_id = "0".to_string();
      let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
      assert_eq!(token.token_id, token_id);
      assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
      assert_eq!(token.metadata.unwrap(), sample_token_metadata());
      assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
  }

  #[test]
  fn test_transfer() {
      let mut context = get_context(accounts(0));
      testing_env!(context.build());
      let mut contract = Contract::new_default_meta(accounts(0).into());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(MINT_STORAGE_COST)
          .predecessor_account_id(accounts(0))
          .build());
      let token_id = "0".to_string();
      contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(1)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_transfer(accounts(1), token_id.clone(), None, None);

      testing_env!(context
          .storage_usage(env::storage_usage())
          .account_balance(env::account_balance())
          .is_view(true)
          .attached_deposit(0)
          .build());
      if let Some(token) = contract.nft_token(token_id.clone()) {
          assert_eq!(token.token_id, token_id);
          assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
          assert_eq!(token.metadata.unwrap(), sample_token_metadata());
          assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
      } else {
          panic!("token not correctly created, or not found by nft_token");
      }
  }

  #[test]
  fn test_approve() {
      let mut context = get_context(accounts(0));
      testing_env!(context.build());
      let mut contract = Contract::new_default_meta(accounts(0).into());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(MINT_STORAGE_COST)
          .predecessor_account_id(accounts(0))
          .build());
      let token_id = "0".to_string();
      contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

      // alice approves bob
      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(150000000000000000000)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_approve(token_id.clone(), accounts(1), None);

      testing_env!(context
          .storage_usage(env::storage_usage())
          .account_balance(env::account_balance())
          .is_view(true)
          .attached_deposit(0)
          .build());
      assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
  }

  #[test]
  fn test_revoke() {
      let mut context = get_context(accounts(0));
      testing_env!(context.build());
      let mut contract = Contract::new_default_meta(accounts(0).into());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(MINT_STORAGE_COST)
          .predecessor_account_id(accounts(0))
          .build());
      let token_id = "0".to_string();
      contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

      // alice approves bob
      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(150000000000000000000)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_approve(token_id.clone(), accounts(1), None);

      // alice revokes bob
      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(1)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_revoke(token_id.clone(), accounts(1));
      testing_env!(context
          .storage_usage(env::storage_usage())
          .account_balance(env::account_balance())
          .is_view(true)
          .attached_deposit(0)
          .build());
      assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
  }

  #[test]
  fn test_revoke_all() {
      let mut context = get_context(accounts(0));
      testing_env!(context.build());
      let mut contract = Contract::new_default_meta(accounts(0).into());

      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(MINT_STORAGE_COST)
          .predecessor_account_id(accounts(0))
          .build());
      let token_id = "0".to_string();
      contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

      // alice approves bob
      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(150000000000000000000)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_approve(token_id.clone(), accounts(1), None);

      // alice revokes bob
      testing_env!(context
          .storage_usage(env::storage_usage())
          .attached_deposit(1)
          .predecessor_account_id(accounts(0))
          .build());
      contract.nft_revoke_all(token_id.clone());
      testing_env!(context
          .storage_usage(env::storage_usage())
          .account_balance(env::account_balance())
          .is_view(true)
          .attached_deposit(0)
          .build());
      assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
  }
}