#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;



#[ink::contract]
pub mod pair_creator {


    use trading_pair_azero::TradingPairAzeroRef;
    use trading_pair_psp22::TradingPairPsp22Ref;
    use ink_storage::traits::SpreadAllocate;
    
    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct PairCreator {
    

    }

    impl PairCreator {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            
            let me = ink_lang::utils::initialize_contract(|_contract: &mut Self| {});
            
            me
            
        }

        #[ink(message,payable)]
        pub fn create_azero_trading_pair(&mut self,azero_trading_pair_hash: Hash,version:u32,psp22_addrr:AccountId,fee:u128,panx_contract:AccountId) -> AccountId {

            
            let salt = version.to_le_bytes();
            let trading_pair = TradingPairAzeroRef::new(psp22_addrr,fee,panx_contract)
                .endowment(0)
                .code_hash(azero_trading_pair_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the Azero trading pair contract: {:?}",
                        error
                    )
            });
            let add = trading_pair.get_account_id();

            

            add
        
 
        }

        #[ink(message,payable)]
        pub fn create_psp22_trading_pair(&mut self,psp22_trading_pair_hash: Hash,version:u32,psp22_token1_addrr:AccountId,psp22_token2_addrr:AccountId,fee:u128,panx_contract:AccountId) -> AccountId {

            
            let salt = version.to_le_bytes();
            let trading_pair = TradingPairPsp22Ref::new(psp22_token1_addrr,psp22_token2_addrr,fee,panx_contract)
                .endowment(0)
                .code_hash(psp22_trading_pair_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the Azero trading pair contract: {:?}",
                        error
                    )
            });
            let add = trading_pair.get_account_id();

            

            add
        
 
        }



    }

}