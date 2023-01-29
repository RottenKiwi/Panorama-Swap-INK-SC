#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod airdrop_contract {


    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct AirdropContract {
        
        //Deployer address 
        manager: AccountId,
        //PANX psp22 contract address
        panx_psp22: AccountId,
        // 0 didnt collect, 1 did collect.
        collected_airdrop: Mapping<AccountId, i64>,


    }

    #[ink(event)]
    pub struct PanxClaim50 {
        caller:AccountId
    }

    #[ink(event)]
    pub struct PanxClaim500 {
        caller:AccountId
    }


    impl AirdropContract {
        /// Creates a new airdrop contract.
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId) -> Self {
            
            let panx_psp22 = panx_contract;  
                let manager = Self::env().caller();
            let collected_airdrop = Mapping::default();
            
            Self{

                manager,
                panx_psp22,
                collected_airdrop

            }
            
        }

        ///function to collect 50 PANX
        #[ink(message)]
        pub fn collect_50_tokens(&mut self)  {

           //making sure account didnt redeem airdrop yet
           if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
            panic!(
                "Caller already redeemed the airdrop, cannot redeem again."
            )
            }

            let tokens_to_transfer:Balance;

            match 50u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           //transfers the airdrop tokens to caller
           PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), tokens_to_transfer, ink::prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 tokens to caller : {:?}",
                error
            )
            });

           //make sure to change his collected airdrop status to 1 to prevent the user to call it again
           self.collected_airdrop.insert(self.env().caller(),&1);
           Self::env().emit_event(PanxClaim50{caller:self.env().caller()});

        
        }

        ///function to collect 500 PANX
        #[ink(message)]
        pub fn collect_500_tokens(&mut self)  {

           //making sure account didnt redeem airdrop yet
           if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
            panic!(
                "Caller already redeemed the airdrop, cannot redeem again."
            )
            }

            let tokens_to_transfer:Balance;

            match 500u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           //transfers the airdrop tokens to caller
            PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), tokens_to_transfer, ink::prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 tokens to caller : {:?}",
                error
            )
            });
           //make sure to change his collected airdrop status to 1 to prevent the user to call it again
           self.collected_airdrop.insert(self.env().caller(),&1);
           Self::env().emit_event(PanxClaim500{caller:self.env().caller()});

        }


        ///funtion to get airdrop contract PANX reserve
        #[ink(message)]
        pub fn get_airdrop_contract_panx_reserve(&self)-> Balance  {
        
            let balance:Balance = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());
            balance

        }


        ///function to get account airdrop collection status
        #[ink(message)]
        pub fn user_airdrop_collection_status(&mut self,account:AccountId)->i64  {

            let airdrop_status = self.collected_airdrop.get(account).unwrap_or(0);
            airdrop_status


        }



 
    }
}