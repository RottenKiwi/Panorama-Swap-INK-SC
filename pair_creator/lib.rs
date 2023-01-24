#![cfg_attr(not(feature = "std"), no_std)]


#[ink::contract]
pub mod pair_creator {


    use trading_pair_psp22::TradingPairPsp22Ref;
    use trading_pair_azero::TradingPairAzeroRef;
    use openbrush::traits::{
        Storage,
        AccountIdExt
    };


    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct PairCreator {
       azero_pairs:ink::storage::Mapping<AccountId, AccountId>,
       psp22_pairs:ink::storage::Mapping<(AccountId, AccountId), AccountId>,
       zero_address:AccountId,
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
        
        #[inline]
        pub fn azero_pair_exists(&self, psp22_token:AccountId) -> bool {
            let mut pool_exists = false;
            let exists = self.azero_pairs.get(psp22_token).unwrap_or(self.zero_address);
            if exists.is_zero() {
                pool_exists = false;
            }
            else {
                pool_exists = true;
            }
            pool_exists
        }
        
        #[inline]
        pub fn psp22_pair_exists(&self, psp22_token_a:AccountId, psp2_token_b:AccountId) -> bool {
            let mut pool_exists = false;
            let exists = self.psp22_pairs.get(&(psp22_token_a, psp22_token_b)).unwrap_or(self.zero_address);
            if exists.is_zero() {
                pool_exists = false;
            }
            else {
                pool_exists = true;
            }
            pool_exists
        }

        #[ink(message,payable)]
        pub fn create_azero_trading_pair(&mut self,azero_trading_pair_hash: Hash,version:u32,psp22_addrr:AccountId,fee:Balance,panx_contract:AccountId,vault_address:AccountId) -> AccountId {

            let pool_found = self.azero_pair_exists(psp22_addr);
            
            if pool_found == true {
                panic!(
                    "Liquidity pool already exists for Azero"
                )
            }
            
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
            
            self.azero_pairs.insert(psp22_addr, &add);
            
            add
        
 
        }

        #[ink(message,payable)]
        pub fn create_psp22_trading_pair(&mut self,psp22_trading_pair_hash: Hash,version:u32,psp22_token1_addrr:AccountId,psp22_token2_addrr:AccountId,fee:Balance,panx_contract:AccountId,vault_address:AccountId) -> AccountId {
            
            let pool_found = self.psp22_pair_exists(psp22_token1_addr, psp22_token2_addr);
            
            if pool_found == true {
                panic!(
                    "Liquidity pool already exists for PSP22"
                )
            }
            
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
            
            self.psp22_pairs.insert(&(psp22_token1_addr, psp22_token2_addr), &add);

            self.psp22_pairs.insert(&(psp22_token2_addr, psp22_token1_addr), &add);
            
            add
        
 
        }



    }

}
