#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod vesting_contract {

    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

   

    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct VestingContract {
        
        //Deployer address 
        manager: AccountId,
        //PANX psp22 contract address
        panx_psp22: AccountId,
        //Vesting contract deploy date in tsp 
        started_date_in_timestamp:u64,
        //Vesting contract PANX reserve
        panx_reserve: Balance,
        //Locked PANX amount for users
        balances: ink_storage::Mapping<AccountId, Balance>,
        // 0 didnt collect, 1 did collect.
        collected_tge: ink_storage::Mapping<AccountId, i64>,
        //panx reward for a day, for each account
        panx_to_give_in_a_day: ink_storage::Mapping<AccountId, Balance>,
        //last time account redeemed
        last_redeemed:ink_storage::Mapping<AccountId, u64>,


    }

    impl VestingContract {
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


        ///adding seed events participants to vesting contract and their PANX vesting allocation
        ///Only manager can use this function
        #[ink(message)]
        pub fn add_to_vesting(&mut self,account:AccountId,panx_to_give_overall:Balance)  {

           //Making sure caller is the manager (Only manager can add)
           assert!(self.env().caller() == self.manager);
           //get current account balance (If any)
           let account_balance = self.balances.get(&account).unwrap_or(0);
           //add PANX allocation to account
           self.balances.insert(account, &(account_balance + panx_to_give_overall));
           //calc how many tokens to give in a day
           let amount_to_give_each_day = panx_to_give_overall / 365  ;
           //insert the daily amount to account
           self.panx_to_give_in_a_day.insert(account,&amount_to_give_each_day);
           //Allow account to collect TGE
           self.collected_tge.insert(account,&0);
           //Insert the current date as last redeem date for account.
           self.last_redeemed.insert(account, &self.get_current_timestamp());
              
        }


        ///function to collect TGE (10%) for account
        #[ink(message)]
        pub fn collect_tge_tokens(&mut self)  {

           //caller address
           let account = self.env().caller();
           //current account locked tokens
           let account_balance = self.balances.get(&account).unwrap_or(0);
           //making sure account didnt redeem tge yet
           assert!(self.collected_tge.get(&account).unwrap_or(0) == 0);
           //making sure account has more then 0 tokens
           assert!(account_balance > 0);

           //account balance without TGE token amounts
           let account_balance_with_tge_amount =  (account_balance * 900) / 1000;

           //calc the amount of TGE tokens to give 
           let amount_to_give = account_balance - account_balance_with_tge_amount;

           //transfers the TGE tokens to caller
           let _response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), amount_to_give, ink_prelude::vec![]);
           
           //deducts from overall vesting amount to give
           self.balances.insert(account, &(account_balance - amount_to_give));

           //make sure to change his collected tge status to 1 to prevent the user to call it again
           self.collected_tge.insert(account,&1);



        }

        ///function to get account redeemable amount of tokens
        #[ink(message)]
        pub fn get_redeemable_amount(&mut self) -> Balance {

            
            //call address 
            let account = self.env().caller();
            //current timestamp
            let current_tsp = self.get_current_timestamp();
            //account total locked PANX amount
            let account_total_vesting_amount = self.get_account_total_vesting_amount(account);

        
            //last time account (caller) redeemed tokens
            let last_redeemed = self.last_redeemed.get(account).unwrap_or(0);
            //How many PANX tokens to give to account each day
            let panx_to_give_each_day = self.panx_to_give_in_a_day.get(account).unwrap_or(0);
            //days since last redeem
            let days_diff = (current_tsp - last_redeemed) / 86400;
            //making sure that 24 hours has passed since last redeem
            assert!(days_diff > 0);
            //making sure that account has more then 0 PANX to redeem
            assert!(account_total_vesting_amount >= 0);
            //amount to give to caller
            let mut amount_redeemable_amount = panx_to_give_each_day * days_diff as u128;


            //if account has less tokens from the daily amount, give him the rest of tokens
            if amount_redeemable_amount > account_total_vesting_amount{

                amount_redeemable_amount = account_total_vesting_amount
            }

            amount_redeemable_amount

            

        }

        ///function for account to redeem his redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(&mut self) {

            
            //caller address
            let account = self.env().caller();
            //current timestamp
            let current_tsp = self.get_current_timestamp();
            //caller total locked PANX 
            let account_total_vesting_amount = self.get_account_total_vesting_amount(account);

        
            //timestamp for the last redeem data of caller
            let last_redeemed = self.last_redeemed.get(account).unwrap_or(0);
            //PANX amount to give caller in each passing day
            let panx_to_give_each_day = self.panx_to_give_in_a_day.get(account).unwrap_or(0);
            //difference between last redeem tsp with the current tsp
            let days_diff = (current_tsp - last_redeemed) / 86400;
            //making sure more then 24 hours has passed
            assert!(days_diff > 0);
            //making sure caller has more then 0 locked PANX
            assert!(account_total_vesting_amount >= 0);
            //amount to give to caller 
            let mut amount_redeemable_amount = panx_to_give_each_day * days_diff as u128;

            //if account has less tokens from the daily amount, give him the rest of tokens
            if amount_redeemable_amount > account_total_vesting_amount{

                amount_redeemable_amount = account_total_vesting_amount;
            }

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(account,&current_tsp);

            //make sure to deducte from overall amount
            self.balances.insert(account, &(account_total_vesting_amount -  amount_redeemable_amount));

            //cross contract call to PANX contract to transfer PANX to caller
            let _response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), amount_redeemable_amount, ink_prelude::vec![]);

        }


        ///function to get account total locked tokens
        #[ink(message)]
        pub fn get_account_total_vesting_amount(&mut self,account:AccountId)->Balance  {
        
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
        ///funtion to get vesting contract PANX reserve
        #[ink(message)]
        pub fn get_vesting_contract_panx_reserve(&self)->Balance  {
        
            let balance1 = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());
            balance1


        }
        ///function to get account TGE collection status
        #[ink(message)]
        pub fn user_tge_collection_status(&mut self,account:AccountId)->i64  {

            let tge_status = self.collected_tge.get(account).unwrap_or(0);
            tge_status


        }
        ///funtion to get the started date since issuing the vesting contract in timpstamp and str
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

        

        //funtion to change account tge status
        #[ink(message)]
        pub fn change_tge_status_to_0(&mut self,of: AccountId)  {

           //Making sure caller is the manager (Only manager can add)
           assert!(self.env().caller() == self.manager);

            self.collected_tge.insert(of, &(0));

            
        }

        ///funtion to change balance amount for account
        #[ink(message)]
        pub fn change_balance_amount(&mut self,of: AccountId,value:Balance)  {


            //Making sure caller is the manager (Only manager can add)
            assert!(self.env().caller() == self.manager);

            self.balances.insert(of, &(value));

            
        }

        ///function to change balance amount for account
        #[ink(message)]
        pub fn change_daily_claim(&mut self,of: AccountId,value:Balance)  {

            //Making sure caller is the manager (Only manager can add)
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