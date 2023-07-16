#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]


#[ink::contract]
pub mod pair_creator {



    use ink::LangError;
    use trading_pair_psp22::TradingPairPsp22Ref;
    use trading_pair_azero::trading_pair_azero::TradingPairAzeroRef;
    use multi_sig::MultiSigRef;


    #[ink(storage)]
    pub struct PairCreator {
    

    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum PairCreatorErrors {
        InstantiatingFailed

    }

    impl From<ink::env::Error > for PairCreatorErrors {
        fn from(cause: ink::env::Error) -> Self {
            PairCreatorErrors::InstantiatingFailed
        }
    }

    impl From<LangError> for PairCreatorErrors {
        fn from(cause:LangError) -> Self {
            PairCreatorErrors::InstantiatingFailed
        }
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
        #[ink(constructor)]
        pub fn new() -> Self {
            
            Self{

            }
            
        }

        #[ink(message,payable)]
        pub fn create_azero_trading_pair(
            &mut self,
            azero_trading_pair_hash: Hash,
            version:u32,
            psp22_addrr:AccountId,
            fee:Balance,
            panx_contract:AccountId,
            vault_address:AccountId
        )   -> Result<AccountId, PairCreatorErrors> {

            
            let salt = version.to_le_bytes();

            let trading_pair = TradingPairAzeroRef::new(
                    psp22_addrr,
                    fee,
                    panx_contract,
                    vault_address)
                        .endowment(0)
                        .code_hash(azero_trading_pair_hash)
                        .salt_bytes(salt)
                        .try_instantiate()??;

            let new_pair_address = trading_pair.get_account_id();

            Ok(new_pair_address)
        
 
        }

        #[ink(message,payable)]
        pub fn create_psp22_trading_pair(
            &mut self,
            psp22_trading_pair_hash: Hash,
            version:u32,
            psp22_token1_addrr:AccountId,
            psp22_token2_addrr:AccountId,
            fee:Balance,
            panx_contract:AccountId,
            vault_address:AccountId
        )   -> Result<AccountId, PairCreatorErrors> {

            
            let salt = version.to_le_bytes();

            let trading_pair = TradingPairPsp22Ref::new(
                psp22_token1_addrr,
                psp22_token2_addrr,
                fee,
                panx_contract,
                vault_address)
                        .endowment(0)
                        .code_hash(psp22_trading_pair_hash)
                        .salt_bytes(salt)
                        .try_instantiate()??;

            let new_pair_address = trading_pair.get_account_id();

            Ok(new_pair_address)
        
 
        }

        #[ink(message,payable)]
        pub fn create_multi_sig_wallet(
            &mut self,
            multi_sig_hash: Hash,
            version:u32,
            vault_address:AccountId
        )   -> Result<AccountId, PairCreatorErrors> {

            
            let salt = version.to_le_bytes();

            let multi_sig_wallet = MultiSigRef::new(vault_address)
                        .endowment(0)
                        .code_hash(multi_sig_hash)
                        .salt_bytes(salt)
                        .try_instantiate()??;

            let new_multi_sig_wallet_address = multi_sig_wallet.get_account_id();

            Ok(new_multi_sig_wallet_address)
        
 
        }



    }

}