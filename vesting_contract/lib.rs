#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod vesting_contract {

    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    use ink::storage::Mapping;
    use ink::env::CallFlags;
    use ink::prelude::vec;

    #[ink(storage)]
    pub struct VestingContract {
        
        manager: AccountId,
        panx_psp22: AccountId,
        started_date_in_timestamp:Balance,
        balances: Mapping<AccountId, Balance>,
        collected_tge: Mapping<AccountId, Balance>,
        panx_to_give_in_a_day:Mapping<AccountId, Balance>,
        last_redeemed:Mapping<AccountId, Balance>,


    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum VestingProgramErrors {
        CallerIsNotManager,
        CallerCollectedTGEAlready,
        CallerInsufficientLockedTokens,
        ZeroDaysPassed,
        PSP22TransferFailed,
        Overflow,
    }

    #[ink(event)]
    pub struct AddToVestingProgram {
        added_account:AccountId,
        total_vesting_amount:Balance,
        panx_amount_to_give_each_day: Balance,
    }

    #[ink(event)]
    pub struct CollectedTGE {
        caller:AccountId,
        panx_given_amount:Balance,
        caller_new_balance:Balance
    }

    #[ink(event)]
    pub struct Redeem {
        caller:AccountId,
        panx_given_amount:Balance,
        caller_new_balance:Balance
    }

    impl VestingContract {
        #[ink(constructor)]
        pub fn new(
            panx_contract:AccountId
        ) -> Self {
            

                let panx_psp22 = panx_contract;  
                let started_date_in_timestamp:Balance = Self::env().block_timestamp().into();
                let manager = Self::env().caller();
                let balances = Mapping::default();
                let collected_tge = Mapping::default();
                let panx_to_give_in_a_day = Mapping::default();
                let last_redeemed =  Mapping::default();

            
            

            Self{

                manager,
                panx_psp22,
                started_date_in_timestamp,
                balances,
                collected_tge,
                panx_to_give_in_a_day,
                last_redeemed

            }
            
        }


        ///adding seed events participants to vesting contract and their PANX vesting allocation
        ///Only manager can use this function
        #[ink(message)]
        pub fn add_to_vesting(
            &mut self,
            account:AccountId,
            panx_to_give_overall:Balance
        )   -> Result<(), VestingProgramErrors>  {

            //Making sure caller is the manager (Only manager can add to the vesting program)
            if self.env().caller() != self.manager {
                return Err(VestingProgramErrors::CallerIsNotManager);
            }

            let account_balance = self.balances.get(&account).unwrap_or(0);

            let new_vesting_panx_amount:Balance;

            //calculating the new vesting amount of the caller.
            match account_balance.checked_add(panx_to_give_overall) {
            Some(result) => {
                new_vesting_panx_amount = result;
            }
            None => {
                return Err(VestingProgramErrors::Overflow);
            }
            };


            self.balances.insert(account, &(new_vesting_panx_amount));

            let panx_amount_to_give_each_day:Balance;

            //calculating how much PANX tokens the caller needs to get each day.
            match new_vesting_panx_amount.checked_div(365) {
            Some(result) => {
                panx_amount_to_give_each_day = result;
            }
            None => {
                return Err(VestingProgramErrors::Overflow);
            }
            };

            self.panx_to_give_in_a_day.insert(account,&panx_amount_to_give_each_day);
            //Allow account to collect TGE
            self.collected_tge.insert(account,&0);
            //Insert the current date as last redeem date for account.
            self.last_redeemed.insert(account, &self.get_current_timestamp());

            Self::env().emit_event(AddToVestingProgram{
                added_account:self.env().caller(),
                total_vesting_amount:panx_to_give_overall,
                panx_amount_to_give_each_day:panx_amount_to_give_each_day
            });

            Ok(())

              
        }


        ///function to collect TGE (10%) for caller
        #[ink(message)]
        pub fn collect_tge_tokens(
            &mut self
        )  -> Result<(), VestingProgramErrors>  {

            let caller = self.env().caller();

            let caller_current_balance:Balance = self.balances.get(&caller).unwrap_or(0);

            //making sure caller didnt redeem tge tokens yet
            if self.collected_tge.get(&caller).unwrap_or(0) != 0 {
                return Err(VestingProgramErrors::CallerCollectedTGEAlready);
            }

            //making sure caller has more then 0 locked tokens
            if caller_current_balance <= 0 {
                return Err(VestingProgramErrors::CallerInsufficientLockedTokens);
            }

            let caller_locked_panx_after_tge:Balance;

            //calculating how callers balance after reducing tge amount (10%)
            match (caller_current_balance * 900).checked_div(1000) {
            Some(result) => {
                caller_locked_panx_after_tge = result;
            }
            None => {
                return Err(VestingProgramErrors::Overflow);
            }
            };

            let amount_of_panx_to_give:Balance;

            //calculating the amount of PANX to give to the caller
            match caller_current_balance.checked_sub(caller_locked_panx_after_tge) {
            Some(result) => {
                amount_of_panx_to_give = result;
            }
            None => {
                return Err(VestingProgramErrors::Overflow);
            }
            };

            //transfers the TGE tokens to caller
            if PSP22Ref::transfer(&self.panx_psp22,caller,amount_of_panx_to_give,vec![]).is_err(){
                return Err(VestingProgramErrors::PSP22TransferFailed);

            }
            //deducts from overall vesting amount to give
            self.balances.insert(caller, &(caller_current_balance - amount_of_panx_to_give));

            //make sure to change his collected tge status to 1 to prevent the user to call it again
            self.collected_tge.insert(caller,&1);

            Self::env().emit_event(CollectedTGE{
                caller:caller,
                panx_given_amount:amount_of_panx_to_give,
                caller_new_balance:caller_current_balance - amount_of_panx_to_give
            });


            Ok(())

        }

        ///function to get caller redeemable amount of tokens
        #[ink(message)]
        pub fn get_redeemable_amount(
            &mut self
        )  -> Result<Balance, VestingProgramErrors> {

            
            let caller = self.env().caller();

            let current_date_in_tsp = self.get_current_timestamp();

            let caller_total_vesting_amount:Balance = self.get_account_total_vesting_amount(caller);

            let date_of_last_redeem_in_tsp:Balance = self.last_redeemed.get(caller).unwrap_or(0);

            let panx_to_give_each_day:Balance = self.panx_to_give_in_a_day.get(caller).unwrap_or(0);

            let days_difference:Balance;

            //calculating the days difference between the last redeem date and current date
            match (current_date_in_tsp - date_of_last_redeem_in_tsp).checked_div(86400) {
                Some(result) => {
                    days_difference = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            //making sure that 24 hours has passed since last redeem 
            if days_difference <= 0 {
                return Err(VestingProgramErrors::ZeroDaysPassed);
            }

            //making sure that caller has more then 0 PANX to redeem
            if caller_total_vesting_amount <= 0 {
                return Err(VestingProgramErrors::CallerInsufficientLockedTokens);
            }


            let mut redeemable_amount:Balance;

            //calculating the amount of PANX the caller needs to get.
            match panx_to_give_each_day.checked_mul(days_difference) {
                Some(result) => {
                    redeemable_amount = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };


            //if caller has less tokens from the daily amount, give him the rest of tokens
            if redeemable_amount > caller_total_vesting_amount{

                redeemable_amount = caller_total_vesting_amount
            }

            Ok(redeemable_amount)

            

        }

        ///function for caller to redeem his redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(
            &mut self
        )  -> Result<(), VestingProgramErrors> {

            
            let caller = self.env().caller();

            let current_date_in_tsp = self.get_current_timestamp();

            let caller_total_vesting_amount = self.get_account_total_vesting_amount(caller);

            let mut redeemable_amount = self.get_redeemable_amount().unwrap();

            //make sure to set new date of reedem for the caller.
            self.last_redeemed.insert(caller,&current_date_in_tsp);

            let caller_new_vesting_amount:Balance;

            //calculating the callers new total vesting amount
            match caller_total_vesting_amount.checked_sub(redeemable_amount) {
                Some(result) => {
                    caller_new_vesting_amount = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            //make sure to deducte from overall amount
            self.balances.insert(caller, &(caller_new_vesting_amount));

            //cross contract call to PANX contract to transfer PANX to caller
            if PSP22Ref::transfer(&self.panx_psp22,caller,redeemable_amount,vec![]).is_err(){
                return Err(VestingProgramErrors::PSP22TransferFailed);
            }

            Self::env().emit_event(Redeem{
                    caller:caller,
                    panx_given_amount:redeemable_amount,
                    caller_new_balance:caller_total_vesting_amount - redeemable_amount
            });

            Ok(())


        }


        ///function to get caller total locked tokens
        #[ink(message)]
        pub fn get_account_total_vesting_amount(
            &mut self,
            account:AccountId
        )   -> Balance  {
        
           let account_balance:Balance = self.balances.get(&account).unwrap_or(0);
           account_balance

        }

        ///funtion to get caller last redeem in timestamp
        #[ink(message)]
        pub fn get_account_last_redeem(
            &mut self,
            account:AccountId
        )   -> Balance  {
        
           let time_stamp = self.last_redeemed.get(&account).unwrap_or(0);
           time_stamp

        }
        ///function to get the amount of tokens to give to account each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_account(
            &mut self,
            account:AccountId
        )   -> Balance  {
        
           let account_balance:Balance = self.panx_to_give_in_a_day.get(&account).unwrap_or(0);
           account_balance

        }
        ///funtion to get vesting contract PANX reserve
        #[ink(message)]
        pub fn get_vesting_contract_panx_reserve(
            &self
        )   -> Balance  {
        
            let vesting_panx_reserve = PSP22Ref::balance_of(
                    &self.panx_psp22,
                    Self::env().account_id());

            vesting_panx_reserve


        }
        ///function to get account TGE collection status
        #[ink(message)]
        pub fn user_tge_collection_status(
            &mut self,
            account:AccountId
        )   ->Balance  {

            let tge_status = self.collected_tge.get(account).unwrap_or(0);
            tge_status


        }

        ///funtion to get the started date since issuing the vesting contract in timpstamp and str
        #[ink(message)]
        pub fn get_started_date(
            &self
        )   -> Balance {

            let timestamp = self.started_date_in_timestamp;

            timestamp
        }

        
        ///function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(
            &self
        )   -> Balance {

            let bts = self.env().block_timestamp() / 1000;
            bts.into()

        }

        

        //get the days pass since deployment
        #[ink(message)]
        pub fn get_days_passed_since_issue(
            &self
        )   -> Result<Balance, VestingProgramErrors>{
            
            let current_tsp:Balance = (self.env().block_timestamp() / 1000).into();

            let days_diff :Balance;

            match  (current_tsp - self.started_date_in_timestamp).checked_div(86400) {
                Some(result) => {
                    days_diff = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            Ok((days_diff))
        }
 
    }
}