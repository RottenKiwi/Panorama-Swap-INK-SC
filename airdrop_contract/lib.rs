#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]


#[ink::contract]
pub mod airdrop_contract {


    // Import the required dependencies from the `openbrush` crate.
    use openbrush::contracts::{
        traits::psp22::PSP22Ref,
    };
    // Import the `Mapping` struct from the `ink` crate, used for storage.
    use ink::storage::Mapping;
    // Import the `vec` function from the `ink` prelude, used for creating dynamic arrays.
    use ink::prelude::vec;



    #[ink(storage)]
    /// Represents an Airdrop contract.
    pub struct AirdropContract {

        // Account ID of the PANX PSP22 contract.
        panx_psp22: AccountId,
        // Mapping to track the collected airdrop status for accounts.
        collected_airdrop: Mapping<AccountId, i64>,


    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    /// Enum representing possible errors that can occur during the Airdrop process.
    pub enum AirDropErrors {

        // The caller has already redeemed their airdrop.
        CallerRedeemedAirdrop,
        // Overflow occurred during a calculation.
        Overflow,
        // Failed to transfer tokens from the PSP22 contract.
        PSP22TransferFailed,

    }

    /// Event emitted when a PANX claim of 50 tokens is made.
    #[ink(event)]
    pub struct PanxClaim50 {
        /// Account making the PANX claim.
        caller: AccountId,
    }
    
    /// Event emitted when a PANX claim of 500 tokens is made.
    #[ink(event)]
    pub struct PanxClaim500 {
        /// Account making the PANX claim.
        caller: AccountId,
    }

    impl AirdropContract {
        /// Constructor for the AirdropContract.
        #[ink(constructor)]
        pub fn new(
            panx_contract: AccountId,
        ) -> Self {

            // Assign the PANX contract to the panx_psp22 field.
            let panx_psp22 = panx_contract;
            
            // Initialize the collected_airdrop mapping.
            let collected_airdrop = Mapping::default();
            
            // Return a new instance of AirdropContract with the initialized fields.
            Self {
                panx_psp22,
                collected_airdrop,
            }
        }

        /// Function to collect 50 tokens from the airdrop.
        #[ink(message)]
        pub fn collect_50_tokens(
            &mut self,
        ) -> Result<(), AirDropErrors> {

            // Making sure the account didn't redeem the airdrop yet.
            if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
                return Err(AirDropErrors::CallerRedeemedAirdrop);
            }

            let tokens_to_transfer: Balance;

            // Calculate the amount of tokens to transfer (50 tokens).
            match 50u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    return Err(AirDropErrors::Overflow);
                }
            };

            // Transfer the airdrop tokens to the caller.
            if PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), tokens_to_transfer, vec![]).is_err() {
                return Err(AirDropErrors::PSP22TransferFailed)
            }

            // Change the collected airdrop status to 1 to prevent the user from calling it again.
            self.collected_airdrop.insert(self.env().caller(), &1);

            // Emit the PanxClaim50 event.
            Self::env().emit_event(PanxClaim50 {
                caller: self.env().caller(),
            });

            Ok(())

        }

        /// Function to collect 500 tokens from the airdrop.
        #[ink(message)]
        pub fn collect_500_tokens(
            &mut self,
        ) -> Result<(), AirDropErrors> {
            let caller = self.env().caller();

            // Making sure the account didn't redeem the airdrop yet.
            if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
                return Err(AirDropErrors::CallerRedeemedAirdrop);
            }

            let tokens_to_transfer: Balance;

            // Calculate the amount of tokens to transfer (500 tokens).
            match 500u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    return Err(AirDropErrors::Overflow);
                }
            };

            // Transfer the airdrop tokens to the caller.
            if PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), tokens_to_transfer, vec![]).is_err() {
                return Err(AirDropErrors::PSP22TransferFailed)
            }

            // Change the collected airdrop status to 1 to prevent the user from calling it again.
            self.collected_airdrop.insert(self.env().caller(), &1);
            
            // Emit the PanxClaim500 event.
            Self::env().emit_event(PanxClaim500 {
                caller: caller,
            });

            Ok(())
        }

        /// Function to get the PANX reserve of the airdrop contract.
        #[ink(message)]
        pub fn get_airdrop_contract_panx_reserve(&self) -> Balance {

            let balance: Balance = PSP22Ref::balance_of(
                &self.panx_psp22,
                Self::env().account_id(),
            );

            balance

        }

        /// Function to get the airdrop collection status of an account.
        #[ink(message)]
        pub fn user_airdrop_collection_status(&mut self, account: AccountId) -> i64 {

            let airdrop_status = self.collected_airdrop.get(account).unwrap_or(0);

            airdrop_status
        }

    }

    /// ink! end-to-end (E2E) tests
    ///
    /// cargo test --features e2e-tests -- --nocapture
    ///
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        use openbrush::contracts::psp22::psp22_external::PSP22;
        use my_psp22::my_psp22::MyPsp22Ref;
        use openbrush::{
            contracts::psp22::extensions::metadata::*,
            traits::{
                Storage,
                
            },
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        fn get_bob_account_id() -> AccountId {
            let bob = ink_e2e::bob::<ink_e2e::PolkadotConfig>();
            let bob_account_id_32 = bob.account_id();
            let bob_account_id = AccountId::try_from(bob_account_id_32.as_ref()).unwrap();

            bob_account_id
        }

        
        ///Tests included in "provide_to_pool_works":
        /// 1. new
        /// 2. get_airdrop_contract_panx_reserve
        /// 3. user_airdrop_collection_status
        /// 4. collect_50_tokens
        #[ink_e2e::test( additional_contracts = "../my_psp22/Cargo.toml" )]
        async fn collect_50_tokens(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        
            // Create a new instance of MyPsp22Ref contract
            let psp22_constructor = MyPsp22Ref::new(10000000000000000, Some(String::from("TOKEN").into()), Some(String::from("TKN").into()), 12);
        
            // Instantiate MyPsp22Ref contract and get the account ID
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
            // Create a new instance of AirdropContractRef contract
            let airdrop_constructor = AirdropContractRef::new(psp22_acc_id);
        
            // Instantiate AirdropContractRef contract and get the account ID
            let airdrop_acc_id = client
                .instantiate("airdrop_contract", &ink_e2e::alice(), airdrop_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
        
            // Build a transfer message to transfer tokens to the airdrop account
            let transfer_to_airdrop = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.transfer(airdrop_acc_id, 1000000000000000, vec![]));
        
            // Call the transfer message
            client
                .call(&ink_e2e::alice(), transfer_to_airdrop, 0, None)
                .await
                .expect("calling `transfer_to_airdrop` failed");
        
            // Build a message to get the PANX reserve of the airdrop contract
            let get_airdrop_contract_panx_reserve = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.get_airdrop_contract_panx_reserve());
        
            // Call the message to get the PANX reserve
            let get_airdrop_contract_panx_reserve_res = client
                .call(&ink_e2e::alice(), get_airdrop_contract_panx_reserve, 0, None)
                .await
                .expect("get_airdrop_contract_panx_reserve failed");
        
            // Assert that the PANX reserve is equal to the expected value
            assert_eq!(get_airdrop_contract_panx_reserve_res.return_value(), 1000000000000000);
        
            // Build a message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the balance value from the response
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the balance is equal to 0
            assert_eq!(psp22_balance, 0);
        
            // Build a message to collect 50 tokens from the airdrop contract
            let collect_50_tokens = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.collect_50_tokens());
        
            // Call the message to collect 50 tokens
            client
                .call(&ink_e2e::bob(), collect_50_tokens, 0, None)
                .await
                .expect("calling `collect_50_tokens` failed");
        
            // Build a message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the balance value from the response
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the balance is equal to 50000000000000 (50 * 10^12)
            assert_eq!(psp22_balance, 50000000000000);
        
            // Build a message to get the airdrop collection status for Bob's account
            let user_airdrop_collection_status = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.user_airdrop_collection_status(get_bob_account_id()));
        
            // Call the message to get the airdrop collection status
            let user_airdrop_collection_status_res = client
                .call(&ink_e2e::alice(), user_airdrop_collection_status, 0, None)
                .await
                .expect("user_airdrop_collection_status failed");
        
            // Get the status value from the response
            let status = user_airdrop_collection_status_res.return_value();
        
            // Assert that the status is equal to 1
            assert_eq!(status, 1);
        
            Ok(())
        }

        ///Tests included in "provide_to_pool_works":
        /// 1. new
        /// 2. get_airdrop_contract_panx_reserve
        /// 3. user_airdrop_collection_status
        /// 4. collect_500_tokens
        #[ink_e2e::test( additional_contracts = "../my_psp22/Cargo.toml" )]
        async fn collect_500_tokens(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        
            // Create a new instance of MyPsp22Ref contract
            let psp22_constructor = MyPsp22Ref::new(10000000000000000, Some(String::from("TOKEN").into()), Some(String::from("TKN").into()), 12);
        
            // Instantiate MyPsp22Ref contract and get the account ID
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
            // Create a new instance of AirdropContractRef contract
            let airdrop_constructor = AirdropContractRef::new(psp22_acc_id);
        
            // Instantiate AirdropContractRef contract and get the account ID
            let airdrop_acc_id = client
                .instantiate("airdrop_contract", &ink_e2e::alice(), airdrop_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
        
            // Build a transfer message to transfer tokens to the airdrop account
            let transfer_to_airdrop = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.transfer(airdrop_acc_id, 1000000000000000, vec![]));
        
            // Call the transfer message
            client
                .call(&ink_e2e::alice(), transfer_to_airdrop, 0, None)
                .await
                .expect("calling `transfer_to_airdrop` failed");
        
            // Build a message to get the PANX reserve of the airdrop contract
            let get_airdrop_contract_panx_reserve = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.get_airdrop_contract_panx_reserve());
        
            // Call the message to get the PANX reserve
            let get_airdrop_contract_panx_reserve_res = client
                .call(&ink_e2e::alice(), get_airdrop_contract_panx_reserve, 0, None)
                .await
                .expect("get_airdrop_contract_panx_reserve failed");
        
            // Assert that the PANX reserve is equal to the expected value
            assert_eq!(get_airdrop_contract_panx_reserve_res.return_value(), 1000000000000000);
        
            // Build a message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the balance value from the response
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the balance is equal to 0
            assert_eq!(psp22_balance, 0);
        
            // Build a message to collect 50 tokens from the airdrop contract
            let collect_50_tokens = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.collect_500_tokens());
        
            // Call the message to collect 50 tokens
            client
                .call(&ink_e2e::bob(), collect_50_tokens, 0, None)
                .await
                .expect("calling `collect_50_tokens` failed");
        
            // Build a message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the message to get the balance of the MyPsp22Ref contract for Bob's account
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the balance value from the response
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the balance is equal to 500000000000000 (500 * 10^12)
            assert_eq!(psp22_balance, 500000000000000);
        
            // Build a message to get the airdrop collection status for Bob's account
            let user_airdrop_collection_status = build_message::<AirdropContractRef>(airdrop_acc_id.clone())
                .call(|airdrop_contract| airdrop_contract.user_airdrop_collection_status(get_bob_account_id()));
        
            // Call the message to get the airdrop collection status
            let user_airdrop_collection_status_res = client
                .call(&ink_e2e::alice(), user_airdrop_collection_status, 0, None)
                .await
                .expect("user_airdrop_collection_status failed");
        
            // Get the status value from the response
            let status = user_airdrop_collection_status_res.return_value();
        
            // Assert that the status is equal to 1
            assert_eq!(status, 1);
        
            Ok(())
        }
 
    }

}
