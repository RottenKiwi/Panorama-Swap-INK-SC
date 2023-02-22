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
    use ink::env::CallFlags;
    use ink::prelude::vec;

    #[ink(storage)]
    pub struct AirdropContract {
        
        panx_psp22: AccountId,
        collected_airdrop: Mapping<AccountId, i64>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum AirDropErrors {
        CallerRedeemedAirdrop,
        Overflow,
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
        #[ink(constructor)]
        pub fn new(
            panx_contract:AccountId
        ) -> Self {
            
            let panx_psp22 = panx_contract;  
            let collected_airdrop = Mapping::default();
            
            Self{

                panx_psp22,
                collected_airdrop

            }
            
        }

        ///function to collect 50 PANX
        #[ink(message)]
        pub fn collect_50_tokens(
            &mut self
        )   -> Result<(), AirDropErrors>  {

            //making sure account didnt redeem airdrop yet
            if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
                return Err(AirDropErrors::CallerRedeemedAirdrop);
            }

            let tokens_to_transfer:Balance;

            match 50u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    return Err(AirDropErrors::Overflow);
                }
            };

            //transfers the airdrop tokens to caller
            PSP22Ref::transfer(
                &self.panx_psp22,
                self.env().caller(),
                tokens_to_transfer,
                vec![])
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to transfer PSP22 tokens to caller : {:?}",
                        error
                    )
            });

            //make sure to change his collected airdrop status to 1 to prevent the user to call it again
            self.collected_airdrop.insert(self.env().caller(),&1);

            Self::env().emit_event(PanxClaim50{
                caller:self.env().caller()
            });

            Ok(())

        
        }

        ///function to collect 500 PANX
        #[ink(message)]
        pub fn collect_500_tokens(
            &mut self
        )   -> Result<(), AirDropErrors>  {

            let caller = self.env().caller();

            //making sure account didnt redeem airdrop yet
            if self.collected_airdrop.get(&self.env().caller()).unwrap_or(0) != 0 {
                return Err(AirDropErrors::CallerRedeemedAirdrop);
            }

            let tokens_to_transfer:Balance;

            match 500u128.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    tokens_to_transfer = result;
                }
                None => {
                    return Err(AirDropErrors::Overflow);
                }
            };

            //transfers the airdrop tokens to caller
            PSP22Ref::transfer(
                    &self.panx_psp22,
                    caller,
                    tokens_to_transfer,
                    vec![])
                    .unwrap_or_else(|error| {
                        panic!(
                            "Failed to transfer PSP22 tokens to caller : {:?}",
                            error
                        )
            });
           //make sure to change his collected airdrop status to 1 to prevent the user to call it again
           self.collected_airdrop.insert(self.env().caller(),&1);
           
            Self::env().emit_event(PanxClaim500{
                caller:caller
            });

            Ok(())

        }


        ///funtion to get airdrop contract PANX reserve
        #[ink(message)]
        pub fn get_airdrop_contract_panx_reserve(
            &self
        )   -> Balance  {
        
            let balance:Balance = PSP22Ref::balance_of(
                &self.panx_psp22,
                Self::env().account_id());
            balance

        }


        ///function to get account airdrop collection status
        #[ink(message)]
        pub fn user_airdrop_collection_status(
            &mut self,
            account:AccountId
        )   ->i64  {

            let airdrop_status = self.collected_airdrop.get(account).unwrap_or(0);

            airdrop_status

        }



 
    }
}