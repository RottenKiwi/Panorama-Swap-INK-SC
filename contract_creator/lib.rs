#![cfg_attr(not(feature = "std"), no_std)]


#[ink::contract]
pub mod pair_creator {

    use ink::LangError;
    use trading_pair_azero::trading_pair_azero::TradingPairAzeroRef;

    #[ink(storage)]
    pub struct PairCreator {}

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum PairCreatorErrors {
        InstantiatingFailed,
    }

    impl From<ink::env::Error> for PairCreatorErrors {
        fn from(cause: ink::env::Error) -> Self {
            PairCreatorErrors::InstantiatingFailed
        }
    }

    impl From<LangError> for PairCreatorErrors {
        fn from(cause: LangError) -> Self {
            PairCreatorErrors::InstantiatingFailed
        }
    }

    #[ink(event)]
    pub struct NewTPA {
        caller: AccountId,
        psp22_address: AccountId,
        lp_fee: Balance,
    }

    #[ink(event)]
    pub struct NewTPP {
        caller: AccountId,
        psp22_1_address: AccountId,
        psp22_2_address: AccountId,
        lp_fee: Balance,
    }

    impl PairCreator {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message, payable)]
        pub fn create_azero_trading_pair(
            &mut self,
            azero_trading_pair_hash: Hash,
            version: u32,
            psp22_addrr: AccountId,
            fee: Balance,
            panx_contract: AccountId,
            vault_address: AccountId,
            lp_lock_timestamp: u64, // Lp lock timestamp
        ) -> Result<AccountId, PairCreatorErrors> {
            let salt = version.to_le_bytes();

            let deployer = self.env().caller();

            let trading_pair = TradingPairAzeroRef::new(
                psp22_addrr,
                fee,
                panx_contract,
                vault_address,
                lp_lock_timestamp,
                deployer
            )
            .endowment(0)
            .code_hash(azero_trading_pair_hash)
            .salt_bytes(salt)
            .try_instantiate()??;

            let new_pair_address = trading_pair.get_account_id();

            Ok(new_pair_address)
        }





    }
}
