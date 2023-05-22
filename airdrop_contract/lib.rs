#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]



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
}