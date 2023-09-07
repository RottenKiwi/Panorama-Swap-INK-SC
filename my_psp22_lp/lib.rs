#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::my_psp22_lp::{
    MyPsp22Lp,
    MyPsp22LpRef,
};

#[openbrush::implementation(PSP22, PSP22Metadata)]
#[openbrush::contract]
pub mod my_psp22_lp {
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct MyPsp22Lp {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl MyPsp22Lp {
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            tpa_address: AccountId
        ) -> Self {
            let mut instance = Self::default();

            instance.metadata.name.set(&name);
            instance.metadata.symbol.set(&symbol);
            instance.metadata.decimals.set(&decimal);

            psp22::Internal::_mint_to(&mut instance, tpa_address, 1000000000000000000000)
                .expect("Should mint total_supply");

            instance
        }

        /// function to get lp psp22 contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }

    }
}