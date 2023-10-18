#![cfg_attr(not(feature = "std"), no_std, no_main)]


pub use self::my_psp22::MyPsp22Ref;

#[openbrush::implementation(PSP22, PSP22Metadata)]
#[openbrush::contract]
pub mod my_psp22 {

    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String,
        },
    };
    use ink::{
        codegen::{EmitEvent, Env},
        reflect::ContractEventBase,
    };
    use ink::storage::Mapping;

    

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    #[derive(Debug)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<AccountId>,
        #[ink(topic)]
        pub to: Option<AccountId>,
        pub value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    #[derive(Debug)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct MyPsp22 {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        holders: Balance,
        // If holder held tokens before, but transfered all of his tokens (balance = 0), assign false.
        is_holder :Mapping<AccountId, bool>
    }

    pub type Event = <MyPsp22 as ContractEventBase>::Type;

    #[overrider(PSP22)]
    fn _emit_transfer_event(
        &self,
        _from: Option<AccountId>,
        _to: Option<AccountId>,
        _amount: Balance,
    ) {
        MyPsp22::emit_event(
            self.env(),
            Event::Transfer(Transfer {
                from: _from,
                to: _to,
                value: _amount,
            }),
        );

    }

    #[overrider(PSP22)]
    fn _emit_approval_event(&self, _owner: AccountId, _spender: AccountId, _amount: Balance) {
        MyPsp22::emit_event(
            self.env(),
            Event::Approval(Approval {
                owner: _owner,
                spender: _spender,
                value: _amount,
            }),
        )
    }
    

    impl MyPsp22 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance, name: Option<String>, symbol: Option<String>, decimal: u8) -> Self {
            let mut _instance = Self::default();
			psp22::Internal::_mint_to(&mut _instance, Self::env().caller(), initial_supply).expect("Should mint"); 
			_instance.metadata.name.set(&name);
			_instance.metadata.symbol.set(&symbol);
			_instance.metadata.decimals.set(&decimal);
			_instance

        }

        // Emit event abstraction. Otherwise ink! deserializes events incorrectly when there are events from more than one contract.
        pub fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }


    }
}