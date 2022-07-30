#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
extern crate chrono;


#[ink::contract]
pub mod pair_creator {


    use trading_pair_azero::TradingPairAzeroRef;
    use chrono::prelude::*;
    use ink_storage::traits::SpreadAllocate;
    
   
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    use ink_env::call::FromAccountId;
    
    use ink_env::CallFlags;
    use ink_prelude::vec::Vec;
    use ink_prelude::string::ToString;
    use ink_prelude::string::String;
    use ink_prelude::borrow::ToOwned;

    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct PairCreator {
        

    }

    impl PairCreator {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId) -> Self {
            
            let new_contract = ink_lang::utils::initialize_contract(|contract: &mut Self| {});
            
            new_contract
            
        }

        #[ink(message,payable)]
        pub fn create_azero_trading_pair(&mut self,azero_trading_pair_hash: Hash,version:u32,psp22_addrr:AccountId,fee:u128) -> AccountId {

            
            let salt = version.to_le_bytes();
            let trading_pair = TradingPairAzeroRef::new(psp22_addrr,fee)
                .endowment(0)
                .code_hash(azero_trading_pair_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the Accumulator contract: {:?}",
                        error
                    )
            });
            let add = trading_pair.get_account_id();

            

            add
        
 
        }


    }

}