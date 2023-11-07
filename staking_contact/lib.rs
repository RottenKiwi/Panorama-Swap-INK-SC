#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "ink-as-dependency"))]
#![feature(default_alloc_error_handler)]



#[ink::contract]
pub mod staking_contract {

    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,
        }, traits::Balance,
    };

    use ink::storage::Mapping;
    use ink::env::CallFlags;
    use ink::prelude::vec;


    
    
    #[ink(storage)]
    pub struct StakingContract {
        
        manager: AccountId,
        psp22_contract: AccountId,
        started_date_in_timestamp:Balance,
        balances: Mapping<AccountId, Balance>,
        psp22_to_give_in_a_day: Mapping<AccountId, Balance>,
        last_redeemed:Mapping<AccountId, u64>,
        staking_percentage:Balance,
        account_overall_taken_rewards:Mapping<AccountId, Balance>,


    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum StakingErrors {
        CallerInsufficientPSP22Balance,
        CallerInsufficientPSP22LockedBalance,
        NoRedeemableAmount,
        ZeroDaysPassed,
        NotEnoghtPSP22ToLock,
        NotEnoughAllowance,
        Overflow,
        PSP22TransferFromFailed,
        PSP22TransferFailed,
        LessThanMinimumRequired

    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(psp22_contract:AccountId,staking_percentage:Balance) -> Self {
            

            let manager:AccountId = Self::env().caller();
            let psp22_contract:AccountId = psp22_contract;
            let started_date_in_timestamp:Balance = Self::env().block_timestamp().into();
            let balances = Mapping::default();
            let psp22_to_give_in_a_day = Mapping::default();
            let last_redeemed = Mapping::default();
            let staking_percentage = 5;
            let account_overall_taken_rewards = Mapping::default(); 

            Self{
                manager,
                psp22_contract,
                started_date_in_timestamp,
                balances,
                psp22_to_give_in_a_day,
                last_redeemed,
                staking_percentage,
                account_overall_taken_rewards
            }
           
        }

        //function to add the caller to the staking program or to update his allocations
        #[ink(message)]
        pub fn add_to_staking(
            &mut self,
            psp22_to_lock:Balance
        )   -> Result<(), StakingErrors> {

            // Get the caller's account ID
            let caller = self.env().caller();

            // Get the current PSP22 token balance of the caller from the PSP22 contract
            let caller_current_psp22_balance = PSP22Ref::balance_of(
                &self.psp22_contract,
                caller
            );

            // Get the locked PSP22 token balance of the caller from the local balances storage
            let caller_locked_psp22_balance:Balance = self.balances.get(&caller).unwrap_or(0);

            // Set the minimum PSP22 tokens required to enter staking
            let minimum_psp22_tokens_to_enter:Balance = 1000*10u128.pow(12);

            // Check if the caller's current PSP22 balance is greater than or equal to the minimum tokens required to enter
            // staking, and if the caller's locked PSP22 balance is zero
            if caller_current_psp22_balance >= minimum_psp22_tokens_to_enter && caller_locked_psp22_balance == 0  {

                if psp22_to_lock < minimum_psp22_tokens_to_enter {
                    return Err(StakingErrors::NotEnoghtPSP22ToLock);
                }

                // Get the allowance given by the caller to this contract in the PSP22 contract
                let contract_allowance = PSP22Ref::allowance(
                    &self.psp22_contract,
                    caller,
                    Self::env().account_id()
                );

                // Check if the allowance is less than the PSP22 tokens to be locked
                if contract_allowance < psp22_to_lock {
                    return Err(StakingErrors::NotEnoughAllowance);
                }

                // Check if the caller's current PSP22 balance is less than the PSP22 tokens to be locked
                if caller_current_psp22_balance < psp22_to_lock {
                    return Err(StakingErrors::CallerInsufficientPSP22Balance);
                }

                // Transfer the PSP22 tokens from the caller to this contract using the transfer_from_builder method
                if PSP22Ref::transfer_from_builder(&self.psp22_contract,caller,Self::env().account_id(),psp22_to_lock,vec![])
                        .call_flags(CallFlags::default()
                        .set_allow_reentry(true))
                        .try_invoke()
                        .is_err(){
                            return Err(StakingErrors::PSP22TransferFromFailed);
                }

                // Get the caller's PSP22 balance after the transfer
                let caller_psp22_balance_after_transfer = PSP22Ref::balance_of(
                    &self.psp22_contract,
                    caller
                );

                // Check if the caller's PSP22 balance is the same after the transfer, indicating a failed transfer
                if caller_psp22_balance_after_transfer == caller_current_psp22_balance {
                    return Err(StakingErrors::PSP22TransferFromFailed);
                }

                // Calculate the new locked balance of the caller by adding the PSP22 tokens to be locked
                let new_locked_balance:Balance;

                // Check for potential overflow during the calculation
                match caller_locked_psp22_balance.checked_add(psp22_to_lock) {
                    Some(result) => {
                        new_locked_balance = result;
                    }
                    None => {
                        return Err(StakingErrors::Overflow);
                    }
                };

                // Update the caller's locked PSP22 balance in the local balances storage
                self.balances.insert(caller, &new_locked_balance);

                // Calculate the amount of PSP22 tokens to be given to the caller each day
                let amount_of_psp22_tokens_to_give_each_day:Balance;

                // Check for potential overflow during the calculation
                match ((new_locked_balance * self.staking_percentage) / 100u128 ).checked_div(365) {
                    Some(result) => {
                        amount_of_psp22_tokens_to_give_each_day = result;
                    }
                    None => {
                        return Err(StakingErrors::Overflow);
                    }
                };

                // Update the amount of PSP22 tokens to be given to the caller each day in the local storage
                self.psp22_to_give_in_a_day.insert(caller,&amount_of_psp22_tokens_to_give_each_day);

                self.last_redeemed.insert(caller, &self.get_current_timestamp());
                   

            }

            // Check if the caller's locked PSP22 balance is greater than 0
            if caller_locked_psp22_balance > 0  {

                // Get the contract allowance for the caller
                let contract_allowance = PSP22Ref::allowance(
                    &self.psp22_contract,
                    caller,
                    Self::env().account_id()
                );

                // Check if the contract allowance is less than the amount to lock
                if contract_allowance < psp22_to_lock {
                    return Err(StakingErrors::NotEnoughAllowance);
                }

                // Check if the caller's current PSP22 balance is less than the amount to lock
                if caller_current_psp22_balance < psp22_to_lock {
                    return Err(StakingErrors::CallerInsufficientPSP22Balance);
                }

                // Transfer the PSP22 tokens from the caller to this contract using the transfer_from_builder method
                if PSP22Ref::transfer_from_builder(&self.psp22_contract,caller,Self::env().account_id(),psp22_to_lock,vec![])
                        .call_flags(CallFlags::default()
                        .set_allow_reentry(true))
                        .try_invoke()
                        .is_err(){
                            return Err(StakingErrors::PSP22TransferFromFailed);
                }

                // Get the caller's PSP22 balance after the transfer
                let caller_psp22_balance_after_transfer = PSP22Ref::balance_of(
                    &self.psp22_contract,
                    caller
                );

                // Check if the caller's PSP22 balance after the transfer is same as the current balance
                if  caller_psp22_balance_after_transfer == caller_current_psp22_balance {
                    return Err(StakingErrors::PSP22TransferFromFailed);
                }

                // Calculate the new locked balance of the caller by adding the PSP22 tokens to be locked
                let new_locked_balance:Balance;

                // Check for potential overflow during the calculation
                match caller_locked_psp22_balance.checked_add(psp22_to_lock) {
                    Some(result) => {
                        new_locked_balance = result;
                    }
                    None => {
                        return Err(StakingErrors::Overflow);
                    }
                };

                // Update the caller's balances with the new locked balance
                self.balances.insert(caller, &new_locked_balance);

                // Define a variable to hold the amount to give each day
                let amount_of_psp22_tokens_to_give_each_day:Balance;

                // Calculate the amount to give each day based on the new locked balance and staking percentage
                match ((new_locked_balance * self.staking_percentage) / 100u128).checked_div(365) {
                    Some(result) => {
                        amount_of_psp22_tokens_to_give_each_day = result;
                    }
                    None => {
                        return Err(StakingErrors::Overflow);
                    }
                };

                // Update the PSP22 to give in a day for the caller with the calculated amount
                self.psp22_to_give_in_a_day.insert(caller,&amount_of_psp22_tokens_to_give_each_day);
            
            }

            if caller_current_psp22_balance < minimum_psp22_tokens_to_enter && caller_locked_psp22_balance == 0 {

                return Err(StakingErrors::LessThanMinimumRequired);

            }

           Ok(())
        }


        ///function to get caller redeemable amount of tokens
        #[ink(message)]
        pub fn get_redeemable_amount(
            &mut self
        )   -> Result<Balance, StakingErrors> {

            
            // Get the caller's address
            let caller = self.env().caller(); 

            // Get the current timestamp from the contract
            let current_tsp = self.get_current_timestamp(); 
        
            // Get the total locked PSP22 balance of the caller
            let caller_total_locked_psp22_balance:Balance = self.get_caller_total_locked_amount(caller); 
        
            // Get the last redeemed timestamp of the caller, or 0 if it doesn't exist
            let last_redeemed = self.last_redeemed.get(caller).unwrap_or(0); 
        
            // Get the amount of PSP22 to give each day to the caller, or 0 if it doesn't exist
            let psp22_to_give_each_day:Balance = self.psp22_to_give_in_a_day.get(caller).unwrap_or(0); 
        
            // Declare a variable to hold the difference in days between current timestamp and last redeemed timestamp
            let days_difference:u64; 
            
            // Calculate the difference in days by dividing the difference between current timestamp and last redeemed timestamp by 86400 (number of seconds in a day)
            match (current_tsp - last_redeemed).checked_div(86400) { 
                Some(result) => {
                    days_difference = result; 
                }
                None => {
                    // If the division results in overflow, return an error of StakingErrors::Overflow
                    return Err(StakingErrors::Overflow); 
                }
            };
            
            // Check if days_difference is less than or equal to 0
            // If so, return an error of StakingErrors::ZeroDaysPassed
            if days_difference <= 0 { 
                return Err(StakingErrors::ZeroDaysPassed); 
            }
        
            // Check if caller's total locked PSP22 balance is less than or equal to 0
            // If so, return an error of StakingErrors::CallerInsufficientPSP22LockedBalance
            if caller_total_locked_psp22_balance <= 0 { 
                return Err(StakingErrors::CallerInsufficientPSP22LockedBalance); 
            }
            
            // Calculate the redeemable amount of PSP22 by multiplying the PSP22 to give each day by the days difference, and convert it to u128 to avoid overflow
            let psp22_redeemable_amount:Balance = psp22_to_give_each_day * days_difference as u128; 
        
            // Return the redeemable amount as a Result::Ok
            Ok(psp22_redeemable_amount) 

        }

        ///function for caller to redeem his redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(
            &mut self
        )   -> Result<(), StakingErrors> {

             // Get the caller's address
            let caller = self.env().caller();

            // Get the current timestamp from the contract
            let current_tsp = self.get_current_timestamp(); 
            
            // Get the redeemable amount of PSP22 from the get_redeemable_amount function, or 0 if it fails
            let psp22_redeemable_amount:Balance = self.get_redeemable_amount().unwrap_or(0); 

            // Check if the redeemable amount is less than or equal to 0
            // If so, return an error of StakingErrors::Overflow
            if psp22_redeemable_amount <= 0 { 
                return Err(StakingErrors::NoRedeemableAmount); 
            }

            // Call the transfer function of PSP22Ref contract to transfer PSP22 tokens from the contract to the caller
            // If the transfer fails, return an error of StakingErrors::PSP22TransferFailed
            if PSP22Ref::transfer(&self.psp22_contract,caller,psp22_redeemable_amount,vec![]).is_err(){ 
                return Err(StakingErrors::PSP22TransferFailed); 
            }

            self.account_overall_taken_rewards.insert(&caller, &psp22_redeemable_amount);

            // Insert the current timestamp as the last redeemed timestamp for the caller in the last_redeemed storage mapping
            self.last_redeemed.insert(caller,&current_tsp); 

            Ok(())

        }

        ///function for caller to auto stack redeemable locked tokens.
        #[ink(message)]
        pub fn auto_stack(
            &mut self
        )   -> Result<(), StakingErrors> {

            // Get the caller's address
            let caller = self.env().caller();

            // Get the current timestamp from the contract
            let current_tsp = self.get_current_timestamp();
        
            // Get the total locked amount of PSP22 for the caller
            let caller_total_psp22_locked_amount:Balance = self.get_caller_total_locked_amount(caller); 
        
            // Get the redeemable amount of PSP22 from the get_redeemable_amount function, or 0 if it fails
            let psp22_redeemable_amount:Balance = self.get_redeemable_amount().unwrap_or(0); 
        
            // Check if the redeemable amount is less than or equal to 0
            // If so, return an error of StakingErrors::Overflow
            if psp22_redeemable_amount <= 0 { 
                return Err(StakingErrors::NoRedeemableAmount); 
            }

            // Insert the current timestamp as the last redeemed timestamp for the caller in the last_redeemed storage mapping
            self.last_redeemed.insert(caller,&current_tsp); 
        
            // Calculate the new total locked amount of PSP22 for the caller after redeeming
            let new_psp22_locked_balance:Balance = caller_total_psp22_locked_amount + psp22_redeemable_amount;
        
            // Declare a new variable to store the new amount of PSP22 to give each day
            let new_amount_of_psp22_to_give_each_day:Balance; 
        
            // Calculate the new amount of PSP22 to give each day based on the staking percentage and the new locked balance
            // If the calculation fails, return an error of StakingErrors::Overflow
            match ((new_psp22_locked_balance * self.staking_percentage) / 100u128 ).checked_div(365) { 
                Some(result) => {
                    new_amount_of_psp22_to_give_each_day = result;
                }
                None => {
                    return Err(StakingErrors::Overflow); 
                }
            };
        
            // Insert the new amount of PSP22 to give each day for the caller in the psp22_to_give_in_a_day storage mapping
            self.psp22_to_give_in_a_day.insert(caller,&new_amount_of_psp22_to_give_each_day); 
        
            // Update the caller's PSP22 locked balance in the balances storage mapping
            self.balances.insert(caller, &new_psp22_locked_balance); 

            self.account_overall_taken_rewards.insert(&caller, &psp22_redeemable_amount);
            
            // Return a success result
            Ok(()) 

        }

        ///Function to withdraw specific amount of locked PANX given from the front-end.
        #[ink(message,payable)]
        pub fn withdraw_specific_amount(
            &mut self,
            amount_of_tokens_to_withdraw: Balance
        )   -> Result<(), StakingErrors> {
          
            // Get the caller's address
            let caller = self.env().caller(); 

            // Get the total locked amount of PSP22 for the caller from the balances storage mapping, or 0 if it fails
            let caller_total_psp22_locked_amount = self.balances.get(&caller).unwrap_or(0); 
            
            // Check if the caller's locked PSP22 balance is less than the amount of tokens to withdraw
            // If so, return an error of StakingErrors::CallerInsufficientPSP22LockedBalance
            if caller_total_psp22_locked_amount < amount_of_tokens_to_withdraw { 
                return Err(StakingErrors::CallerInsufficientPSP22LockedBalance); 
            }

            // Calculate the new total locked amount of PSP22 for the caller after withdrawing
            let new_psp22_locked_balance = caller_total_psp22_locked_amount - amount_of_tokens_to_withdraw; 
            
            // Update the caller's PSP22 locked balance in the balances storage mapping
            self.balances.insert(caller, &new_psp22_locked_balance); 
            
            // Declare a new variable to store the new amount of PSP22 to give each day
            let new_amount_to_give_each_day:Balance; 
            
            // Calculate the new amount of PSP22 to give each day based on the staking percentage and the new locked balance
            // If the calculation fails, return an error of StakingErrors::Overflow
            match ((new_psp22_locked_balance * self.staking_percentage) / 100u128 ).checked_div(365) { 
                Some(result) => {
                    new_amount_to_give_each_day = result; 
                }
                None => {
                    return Err(StakingErrors::Overflow); 
                }
            };
            
            // Insert the new amount of PSP22 to give each day for the caller in the psp22_to_give_in_a_day storage mapping
            self.psp22_to_give_in_a_day.insert(caller,&new_amount_to_give_each_day); 
            
            // Attempt to transfer the withdrawn PSP22 tokens to the caller
            // If the transfer fails, return an error of StakingErrors::PSP22TransferFailed
            if PSP22Ref::transfer(&self.psp22_contract,caller,amount_of_tokens_to_withdraw,vec![]).is_err(){ 
                return Err(StakingErrors::PSP22TransferFailed); 
            }
            
            // Return a success result
            Ok(()) 

        }

        ///function to get caller total locked tokens
        #[ink(message)]
        pub fn get_caller_total_locked_amount(
            &mut self,
            caller:AccountId
        )   -> Balance  {
        
           let caller_balance:Balance = self.balances.get(&caller).unwrap_or(0);
           caller_balance

        }

        ///funtion to get caller last redeem in timestamp
        #[ink(message)]
        pub fn get_caller_last_redeem(
            &mut self,
            caller:AccountId
        )   ->u64  {
        
           let time_stamp = self.last_redeemed.get(&caller).unwrap_or(0);
           time_stamp

        }

        ///function to get the amount of tokens to give to caller each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_caller(
            &self,
            caller:AccountId
        )   -> Balance  {
        
           let caller_balance:Balance = self.psp22_to_give_in_a_day.get(&caller).unwrap_or(0);
           caller_balance

        }

        ///funtion to get staking contract PANX reserve
        #[ink(message)]
        pub fn get_staking_contract_panx_reserve(
            &self
        )   -> Balance  {
        
            let staking_contract_reserve:Balance = PSP22Ref::balance_of(
                &self.psp22_contract,
                Self::env().account_id()
            );
            staking_contract_reserve


        }

        ///funtion to get the started date since issuing the staking contract in timpstamp and str
        #[ink(message)]
        pub fn get_started_date(
            &self
        )   -> Balance {

            let timestamp = self.started_date_in_timestamp;
            timestamp.into()

        }

        
        ///function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(
            &self
        )   -> u64 {

            let time_stamp_in_seconds = self.env().block_timestamp() / 1000;
            time_stamp_in_seconds

        }


        //get the days pass since deployment
        #[ink(message)]
        pub fn get_days_passed_since_issue(
            &self
        )   -> Balance {

            let current_tsp:Balance = (self.env().block_timestamp() / 1000).into();

            let days_diff :Balance;

            match  (current_tsp - self.started_date_in_timestamp).checked_div(86400) {
                Some(result) => {
                    days_diff = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            days_diff
        }
 
    }
}
