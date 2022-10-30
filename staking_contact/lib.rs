#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod staking_contract {

    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

    use ink_env::CallFlags;


    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct StakingContract {
        
        //Deployer address 
        manager: AccountId,
        //PANX psp22 contract address
        panx_psp22: AccountId,
        //Staking contract deploy date in tsp 
        started_date_in_timestamp:u64,
        //Locked PANX amount for users
        balances: ink_storage::Mapping<AccountId, Balance>,
        //panx reward for a day, for each account
        panx_to_give_in_a_day: ink_storage::Mapping<AccountId, Balance>,
        //last time account redeemed
        last_redeemed:ink_storage::Mapping<AccountId, u64>,


    }

    impl StakingContract {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.panx_psp22 = panx_contract;  
                
                contract.started_date_in_timestamp = contract.get_current_timestamp();
                contract.manager = Self::env().caller();
               
            });
            
            me
           
        }


        ///Function to add account into the staking program
        #[ink(message)]
        pub fn add_to_staking(&mut self,account:AccountId,panx_to_lock:Balance) {

           //fetching user current PSP22 balance
           let account_current_panx_balance = PSP22Ref::balance_of(&self.panx_psp22, self.env().caller());

           //get current account balance (If any)
           let account_locked_balance = self.balances.get(&account).unwrap_or(0);

           if account_current_panx_balance >= 1000*10u128.pow(12) && account_locked_balance == 0  {

               //validates if the the allowance is equal or greatee than the deposit PANX amount
               let contract_allowance = PSP22Ref::allowance(&self.panx_psp22, self.env().caller(),Self::env().account_id());
               
               //validates if the contract has sufficent allowance.
               assert!(contract_allowance >= panx_to_lock);
               
               //validates if the account has enought PANX to lock
               assert!(account_current_panx_balance >= panx_to_lock);

               //transfers PANX from account to staking contract
               PSP22Ref::transfer_from_builder(&self.panx_psp22, self.env().caller(), Self::env().account_id(), panx_to_lock, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
               
               //variable to hold current amount of locked PANX
               let new_balance = account_locked_balance + panx_to_lock;

               //add PANX allocation to account
               self.balances.insert(account, &new_balance);

               //calc how many tokens to give in a day
               let amount_to_give_each_day = new_balance + (new_balance * (70000000000 / 10u128.pow(12)))  ;

               //insert the daily amount to account
               self.panx_to_give_in_a_day.insert(account,&amount_to_give_each_day);

               //get the last redeem date by timestamp, if account didnt redeem yet, retunr 0.
               let account_last_redeem = self.last_redeemed.get(&account).unwrap_or(0);

               if account_last_redeem > 0 {

               //Insert the current date as last redeem date for account.
               self.last_redeemed.insert(account, &self.get_current_timestamp());
                   
               }



           }

           if account_locked_balance > 0{

                //validates if the the allowance is equal or greatee than the deposit PANX amount
                let contract_allowance = PSP22Ref::allowance(&self.panx_psp22, self.env().caller(),Self::env().account_id());

                //validates if the contract has sufficent allowance.
                assert!(contract_allowance >= panx_to_lock);

                //validates if the account has enought PANX to lock
                assert!(account_current_panx_balance >= panx_to_lock);

                //transfers PANX from account to staking contract
                PSP22Ref::transfer_from_builder(&self.panx_psp22, self.env().caller(), Self::env().account_id(), panx_to_lock, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
                
               //variable to hold current amount of locked PANX
                let new_balance = account_locked_balance + panx_to_lock;

                //add PANX allocation to account
                self.balances.insert(account, &new_balance);

                //calc how many tokens to give in a day
                let amount_to_give_each_day = new_balance + (new_balance * (70000000000 / 10u128.pow(12)))  ;

                //insert the daily amount to account
                self.panx_to_give_in_a_day.insert(account,&amount_to_give_each_day);

                //get the last redeem date by timestamp, if account didnt redeem yet, retunr 0.
                let account_last_redeem = self.last_redeemed.get(&account).unwrap_or(0);

                if account_last_redeem > 0 {

                //Insert the current date as last redeem date for account.
                self.last_redeemed.insert(account, &self.get_current_timestamp());
                    
                }

           }

           if account_current_panx_balance <= 1000*10u128.pow(12) && account_locked_balance == 0 {

            panic!(
                "insufficient PANX balance"
            )

           }
        }


        ///function to get account redeemable amount of tokens
        #[ink(message)]
        pub fn get_redeemable_amount(&mut self) -> Balance {

            
            //call address 
            let account = self.env().caller();
            //current timestamp
            let current_tsp = self.get_current_timestamp();
            //account total locked PANX amount
            let account_total_locked_amount = self.get_account_total_locked_amount(account);

        
            //last time account redeemed tokens
            let last_redeemed = self.last_redeemed.get(account).unwrap_or(0);
            //How many PANX tokens to give to account each day
            let panx_to_give_each_day = self.panx_to_give_in_a_day.get(account).unwrap_or(0);
            //days since last redeem
            let days_diff = (current_tsp - last_redeemed) / 86400;
            //making sure that 24 hours has passed since last redeem
            assert!(days_diff > 0);
            //making sure that account has more then 0 PANX to redeem
            assert!(account_total_locked_amount >= 0);
            //amount to give to account
            let mut panx_redeemable_amount = panx_to_give_each_day * days_diff as u128;


            //if account has less tokens from the daily amount, give him the rest of tokens
            if panx_redeemable_amount > account_total_locked_amount{

                panx_redeemable_amount = account_total_locked_amount
            }

            panx_redeemable_amount

            

        }

        ///function for account to redeem his redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(&mut self) {

            
            //account address
            let account = self.env().caller();
            //account timestamp
            let current_tsp = self.get_current_timestamp();

            //variable to hold redeemable panx amount
            let panx_redeemable_amount = self.get_redeemable_amount();


            //cross contract call to PANX contract to transfer PANX to account
            let _response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), panx_redeemable_amount, ink_prelude::vec![]);

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(account,&current_tsp);

        }

        ///function for account to auto stack redeemable locked tokens.
        #[ink(message)]
        pub fn auto_stack(&mut self) {

            
            //account address
            let account = self.env().caller();
            //account timestamp
            let current_tsp = self.get_current_timestamp();
            //account total locked PANX 
            let account_total_locked_amount = self.get_account_total_locked_amount(account);

            let amount_redeemable_amount = self.get_redeemable_amount();

            //variable to hold current amount of locked PANX
            let new_balance = account_total_locked_amount + amount_redeemable_amount;

            //calc how many tokens to give in a day
            let new_amount_to_give_each_day = new_balance + (new_balance * (70000000000 / 10u128.pow(12)))  ;

            //insert the daily amount to account
            self.panx_to_give_in_a_day.insert(account,&new_amount_to_give_each_day);

            //make sure to increase overall amount
            self.balances.insert(account, &(account_total_locked_amount +  amount_redeemable_amount));

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(account,&current_tsp);

        }

       ///Function to withdraw specific amount of locked PANX given from the front-end.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, amount_of_tokens: u128)  {
          
           //account address
           let account_address = self.env().caller();
           //account total LP shares
           let account_locked_tokens = self.balances.get(&account_address).unwrap_or(0);

           //variable to hold redeemable panx amount
           let panx_redeemable_amount = self.get_redeemable_amount();

           //Validating that the account has the given number of locked tokens.
           assert!((account_locked_tokens + panx_redeemable_amount) >= amount_of_tokens);

           //Amount of panx to give to the account
           let amount_of_panx_to_give = amount_of_tokens;

           //variable to hold new locked PANX balance
           let new_locked_balance = account_locked_tokens - amount_of_panx_to_give;

           //reducing account locked balance
           self.balances.insert(account_address, &new_locked_balance);

           //calc how many tokens to give in a day
           let new_amount_to_give_each_day = new_locked_balance + (new_locked_balance * (70000000000 / 10u128.pow(12))) ;

           //insert the daily amount to account
           self.panx_to_give_in_a_day.insert(account_address,&new_amount_to_give_each_day);

           //cross contract call to PANX contract to approve PANX to give to caller
           //let _response_1 = PSP22Ref::approve(&self.psp22_token, caller,psp22_amount_to_give);
           //cross contract call to PANX contract to transfer PANX to the account
           let _response_2 = PSP22Ref::transfer(&self.panx_psp22, account_address, amount_of_panx_to_give, ink_prelude::vec![]);

           let current_tsp = self.get_current_timestamp();

           //Making sure to set his last redeem to current timestamp
           self.last_redeemed.insert(account_address,&current_tsp);
           

       }
        ///function to get account total locked tokens
        #[ink(message)]
        pub fn get_account_total_locked_amount(&mut self,account:AccountId)->Balance  {
        
           let account_balance = self.balances.get(&account).unwrap_or(0);
           account_balance

        }
        ///funtion to get account last redeem in timestamp
        #[ink(message)]
        pub fn get_account_last_redeem(&mut self,account:AccountId)->u64  {
        
           let time_stamp = self.last_redeemed.get(&account).unwrap_or(0);
           time_stamp

        }
        ///function to get the amount of tokens to give to account each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_account(&mut self,account:AccountId)->Balance  {
        
           let account_balance = self.panx_to_give_in_a_day.get(&account).unwrap_or(0);
           account_balance

        }
        ///funtion to get staking contract PANX reserve
        #[ink(message)]
        pub fn get_staking_contract_panx_reserve(&self)->Balance  {
        
            let balance1 = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());
            balance1


        }

        ///funtion to get the started date since issuing the staking contract in timpstamp and str
        #[ink(message)]
        pub fn get_started_date(&self) -> u64 {
            let timestamp = self.started_date_in_timestamp;
            
            timestamp
        }

        
        ///function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(&self) -> u64 {
            let bts = self.env().block_timestamp() / 1000;
            bts
        }

        
        ///funtion to change balance amount for account
        #[ink(message)]
        pub fn change_balance_amount(&mut self,of: AccountId,value:Balance)  {


            //Making sure account is the manager (Only manager can add)
            assert!(self.env().caller() == self.manager);

            self.balances.insert(of, &(value));

            
        }

        ///function to change balance amount for account
        #[ink(message)]
        pub fn change_daily_claim(&mut self,of: AccountId,value:Balance)  {

            //Making sure account is the manager (Only manager can add)
            assert!(self.env().caller() == self.manager);

            self.panx_to_give_in_a_day.insert(of,&value);

            
        }
        //get the days pass since deployment
        #[ink(message)]
        pub fn get_days_passed_since_issue(&self) -> u64 {
            let current_tsp = self.env().block_timestamp() / 1000;

            let days_diff = (current_tsp - self.started_date_in_timestamp) / 86400;

            days_diff
        }
 
    }
}