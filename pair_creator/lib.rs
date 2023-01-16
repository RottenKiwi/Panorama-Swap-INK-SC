#![cfg_attr(not(feature = "std"), no_std)]


#[ink::contract]
pub mod pair_creator {


    use trading_pair_psp22::TradingPairPsp22Ref;
    use trading_pair_azero::TradingPairAzeroRef;


    #[ink(storage)]
    pub struct PairCreator {
    

    }

    #[ink(event)]
    pub struct NewTPA {
        caller:AccountId,
        psp22_address:AccountId,
        lp_fee: Balance,
    }

    #[ink(event)]
    pub struct NewTPP {
        caller:AccountId,
        psp22_1_address:AccountId,
        psp22_2_address:AccountId,
        lp_fee: Balance,
    }

    impl PairCreator {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            
            Self{

            }
            
        }

        #[ink(message,payable)]
        pub fn create_azero_trading_pair(&mut self,azero_trading_pair_hash: Hash,version:u32,psp22_addrr:AccountId,fee:Balance,panx_contract:AccountId,vault_address:AccountId) -> AccountId {

            
            let salt = version.to_le_bytes();
            let trading_pair = TradingPairAzeroRef::new(psp22_addrr,fee,panx_contract,vault_address)
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
        pub fn create_psp22_trading_pair(&mut self,psp22_trading_pair_hash: Hash,version:u32,psp22_token1_addrr:AccountId,psp22_token2_addrr:AccountId,fee:Balance,panx_contract:AccountId,vault_address:AccountId) -> AccountId {

            
            let salt = version.to_le_bytes();
            let trading_pair = TradingPairPsp22Ref::new(psp22_token1_addrr,psp22_token2_addrr,fee,panx_contract,vault_address)
                .endowment(0)
                .code_hash(psp22_trading_pair_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the PSP22 trading pair contract: {:?}",
                        error
                    )
            });
            let add = trading_pair.get_account_id();
            add
        
 
        }



    }

}