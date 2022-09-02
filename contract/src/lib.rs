use std::str::FromStr;

use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use near_sdk::collections::LazyOption;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, env, PanicOnDefault, AccountId, Balance, log, PromiseError, Promise, Gas};
use near_sdk::json_types::U128;

pub mod external;
pub use crate::external::*;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: near_sdk::AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
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

    // fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
    //     log!("Account @{} burned {}", account_id, amount);
    // }

    #[payable]
    pub fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    pub fn ft_transfer_call(&mut self, receiver_id: String, amount: String, memo: Option<String>, msg: String,
    ) -> Promise {
        self.token.ft_transfer(receiver_id.parse().unwrap(), near_sdk::json_types::U128(u128::from_str(&amount).unwrap()), Option::Some("".to_string()));
        log!("Signer acc {}", env::signer_account_id());
        let promise = cross_comunication::ext(receiver_id.parse().unwrap())
            .with_static_gas(Gas(30*TGAS))
            .ft_on_transfer(
                env::current_account_id().to_string(),
                amount.clone(),
                memo,
                msg
            );

        return promise.then(Self::ext(env::current_account_id().clone())
            .with_static_gas(Gas(30*TGAS))
            .ft_resolve_transfer(
                env::signer_account_id().clone().to_string(),
                receiver_id.clone(),
                amount
            ))
    }

    pub fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }

    #[private]
    pub fn ft_resolve_transfer(&self, _sender_id: String, _receiver_id: String, _amount: String, #[callback_result] call_result: Result<U128, PromiseError>) -> U128 {
        if call_result.is_err() {
            panic!("There was an error contacting My FT contract");
        }
    
        // let (used_amount, burned_amount) =
        //             self.token.internal_ft_resolve_transfer(
        //                 &sender_id.parse().unwrap(), 
        //                 receiver_id.parse().unwrap(), 
        //                 amount
        //             );
        // // if burned_amount > 0 {
        // //     (self.on_tokens_burned_fn(sender_id, burned_amount))
        // // }
        // U128(used_amount)
        let metadata: U128 = call_result.unwrap();
        metadata
    }
}


// The core methods for a basic fungible token. Extension standards may be added in addition to this macro.
// near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
// Ensures that when fungible token storage grows by collections adding entries,
// the storage is be paid by the caller. This ensures that storage cannot grow to a point that the FT contract runs out of near
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env,Balance, AccountId};
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
        let contract = Contract::new(accounts(1).into(), TOTAL_SUPPLY.into(), FungibleTokenMetadata {
            spec: "".to_string(),
            name: "Example NEAR fungible token".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some("".to_string()),
            reference: None,
            reference_hash: None,
            decimals: 24,
        });
        testing_env!(context.is_view(true).build());
        //assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        //assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }
}
