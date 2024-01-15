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

    #[overrider(psp22::Internal)]
    fn _approve_from_to(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        amount: Balance,
    ) -> Result<(), PSP22Error> {
        self.psp22.allowances.insert(&(&owner, &spender), &amount);
        self._emit_approval_event(owner, spender, amount);
        Ok(())
    }

    #[overrider(PSP22)]
    fn _mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
        let mut new_balance = self._balance_of(&account);
        new_balance += amount;
        self.psp22.balances.insert(&account, &new_balance);
        self.psp22.supply += amount;
        self._emit_transfer_event(None, Some(account), amount);
        Ok(())
    }




    #[overrider(PSP22)]
    fn _transfer_from_to(
        &mut self,
        from: AccountId,
        to: AccountId,
        amount: Balance,
        _data: Vec<u8>,
    ) -> Result<(), PSP22Error> {
        let from_balance = self._balance_of(&from);

        if from_balance < amount {
            return Err(PSP22Error::InsufficientBalance)
        }

        self.psp22.balances.insert(&from, &(from_balance - amount));
        let to_balance = self._balance_of(&to);
        self.psp22.balances.insert(&to, &(to_balance + amount));

        self._emit_transfer_event(Some(from), Some(to), amount);
        Ok(())
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



    }
}