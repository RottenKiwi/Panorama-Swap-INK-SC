#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod airdrop_contract {

    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct AirdropContract {
        
        //Deployer address 
        manager: AccountId,
        //PANX psp22 contract address
        panx_psp22: AccountId,
        //airdrop contract PANX reserve
        panx_reserve: Balance,
        // 0 didnt collect, 1 did collect.
        collected_airdrop: ink_storage::Mapping<AccountId, i64>,


    }

    impl AirdropContract {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {

                contract.panx_psp22 = panx_contract;  
                
                contract.manager = Self::env().caller();
               
            });
            
            me
            
        }

        ///function to collect 50 PANX
        #[ink(message)]
        pub fn collect_50_tokens(&mut self)  {

           //making sure account didnt redeem airdrop yet
           assert!(self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) == 0);

           //transfers the airdrop tokens to caller
           let _response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), 50 *10u128.pow(12), ink_prelude::vec![]);
           
           //make sure to change his collected airdrop status to 1 to prevent the user to call it again
           self.collected_airdrop.insert(self.env().caller(),&1);
        
        }

        ///function to collect 500 PANX
        #[ink(message)]
        pub fn collect_500_tokens(&mut self)  {

           //making sure account didnt redeem airdrop yet
           assert!(self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) == 0);

           //transfers the airdrop tokens to caller
           let _response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), 500 *10u128.pow(12), ink_prelude::vec![]);

           //make sure to change his collected airdrop status to 1 to prevent the user to call it again
           self.collected_airdrop.insert(self.env().caller(),&1);

        }


        ///funtion to get airdrop contract PANX reserve
        #[ink(message)]
        pub fn get_airdrop_contract_panx_reserve(&self)->Balance  {
        
            let balance = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());
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