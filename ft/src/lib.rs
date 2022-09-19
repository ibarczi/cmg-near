/*!
Fungible Token implementation with JSON serialization.
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

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
  token: FungibleToken,
  metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

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
const FgOrange: &str = "\x1b[38;5;202m";
const FgPurple: &str = "\x1b[38;5;127m";
const FgA: &str = "\x1b[38;5;163m"; // magentish
const FgZ: &str = "\x1b[38;5;039m";
const FgT: &str = "\x1b[38;5;035m"; // GGB
const FgX: &str = "\x1b[38;5;042m"; // GGB brighter

//.rem: only these are used
const R: &str = "\x1b[0m";
const LightBlue: &str = "\x1b[38;2;150;150;255m";

//.todo: decimals and max supply
//.todo: NEAR / COTO ratio -> dynamic
//.todo: users need to register -> auto

#[near_bindgen]
impl Contract {
  /// Initializes the contract with the given total supply owned by the given `owner_id` with
  /// default metadata (for example purposes only).
  #[init]
  pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
    Self::new(
      owner_id,
      total_supply,
      FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "COTO NEAR fungible token".to_string(),
        symbol: "COTO".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        reference: None,
        reference_hash: None,
        decimals: 3,
      },
    )
  }

  /// Initializes the contract with the given total supply owned by the given `owner_id` with
  /// the given fungible token metadata.
  #[init]
  pub fn new(
    owner_id: AccountId,
    total_supply: U128,
    metadata: FungibleTokenMetadata,
  ) -> Self {
    assert!(!env::state_exists(), "Already initialized");
    log!("trace 1");
    metadata.assert_valid();
    log!("trace 2");
    let mut this = Self {
      token: FungibleToken::new(b"a".to_vec()),
      metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
    };
    this.token.internal_register_account(&owner_id);
    this.token.internal_deposit(&owner_id, total_supply.into());
    near_contract_standards::fungible_token::events::FtMint {
      owner_id: &owner_id,
      amount: &total_supply,
      memo: Some("Initial tokens supply is minted"),
    }
    .emit();
    this
  }

  fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
    log!("Closed @{} with {}", account_id, balance);
  }

  fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
    log!("Account @{} burned {}", account_id, amount);
  }

  //.pub  coto-specific parts

  #[payable]
  pub fn coto_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
    let pregas = env::used_gas().0 / 1_000_000_000;
    let sender_id = env::predecessor_account_id();
    let paramLog = format!("FT::coto_transfer: {} -> {} {}_COTO", sender_id, receiver_id, amount.0);

    self.token.ft_transfer(receiver_id, amount, memo);

    let postgas = env::used_gas().0 / 1_000_000_000;
    log!("{LightBlue}{} // gas: {} + {} = {}{R}", paramLog, pregas, postgas - pregas, postgas);

    // orig:
    // assert_one_yocto();
    // let sender_id = env::predecessor_account_id();
    // let amount: Balance = amount.into();
    // self.internal_transfer(&sender_id, &receiver_id, amount, memo);
  }

  //.dev:  for faucet (not working)

  fn ft_free_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
    // assert_one_yocto();
    // let amount: Balance = amount.into();
    // let sender_id = env::predecessor_account_id();
    let sender_id = AccountId::new_unchecked("setalosas.testnet".to_string());
    self.token.ft_resolve_transfer(&sender_id, receiver_id, amount);
  }

  pub fn cross_call_test(&mut self, msg: String) {
    log!("FT cross_call_test called. {}", msg);
  }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
  fn ft_metadata(&self) -> FungibleTokenMetadata {
    self.metadata.get().unwrap()
  }
}

// tests are from the original placeholder/sample, not modified

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
