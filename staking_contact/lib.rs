#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod staking_contract {

    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,
        },
    };

    use ink::storage::Mapping;
    use ink::env::CallFlags;
    use ink::prelude::vec;


    
    
    #[ink(storage)]
    pub struct StakingContract {
        
        manager: AccountId,
        panx_psp22: AccountId,
        started_date_in_timestamp:Balance,
        balances: Mapping<AccountId, Balance>,
        panx_to_give_in_a_day: Mapping<AccountId, Balance>,
        last_redeemed:Mapping<AccountId, u64>,


    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId) -> Self {
            

            let manager:AccountId = Self::env().caller();
            let panx_psp22:AccountId = panx_contract;
            let started_date_in_timestamp:Balance = Self::env().block_timestamp().into();
            let balances = Mapping::default();
            let panx_to_give_in_a_day = Mapping::default();
            let last_redeemed = Mapping::default();

            Self{
                manager,
                panx_psp22,
                started_date_in_timestamp,
                balances,
                panx_to_give_in_a_day,
                last_redeemed
            }
           
        }


        ///Function to add caller into the staking program
        #[ink(message)]
        pub fn add_to_staking(
            &mut self,
            panx_to_lock:Balance
        ) {

            let caller = self.env().caller();

            //fetching user current PSP22 balance
            let caller_current_panx_balance = PSP22Ref::balance_of(
                &self.panx_psp22,
                caller
            );

            //get current caller balance (If any)
            let caller_locked_balance:Balance = self.balances.get(&caller).unwrap_or(0);

            let tokens_to_validate:Balance = 1000*10u128.pow(12);

            if caller_current_panx_balance >= tokens_to_validate && caller_locked_balance == 0  {

                //validates if the the allowance is equal or greater than the deposit PANX amount
                let contract_allowance = PSP22Ref::allowance(
                    &self.panx_psp22,
                    caller,
                    Self::env().account_id()
                );
               
                //validates if the contract has sufficent allowance.
                if contract_allowance < panx_to_lock {
                    panic!(
                        "Not enough allowance, please make sure you approved the correct amount
                        before adding to staking program."
                    )
                }
               
                //validates if the caller has enought PANX to lock
                if caller_current_panx_balance < panx_to_lock {
                    panic!(
                        "Caller does not have enough PANX tokens to lock,
                        kindly re-adjust deposit PANX amount."
                    )
                }

                //transfers PANX from caller to staking contract
                PSP22Ref::transfer_from_builder(
                    &self.panx_psp22,
                    caller,
                    Self::env().account_id(),
                    panx_to_lock,
                    vec![])
                        .call_flags(CallFlags::default()
                        .set_allow_reentry(true))
                        .try_invoke()
                        .expect("Transfer failed")
                        .unwrap_or_else(|error| {
                            panic!(
                                "Failed to transfer PSP22 tokens to staking contract : {:?}",
                                error
                            )
                });

                //variable to hold current amount of locked PANX
                let new_balance:Balance = caller_locked_balance + panx_to_lock;

                //add PANX allocation to caller
                self.balances.insert(caller, &new_balance);

                let actual_staking_percentage:Balance = 70000000000 / 10u128.pow(12);

                //calc how many tokens to give in a day
                let amount_to_give_each_day:Balance = (new_balance + (new_balance * actual_staking_percentage )) / 365  ;

                //insert the daily amount to caller
                self.panx_to_give_in_a_day.insert(caller,&amount_to_give_each_day);

                //get the last redeem date by timestamp, if caller didnt redeem yet, return 0.
                let caller_last_redeem = self.last_redeemed.get(&caller).unwrap_or(0);

                if caller_last_redeem > 0 {

                    //Insert the current date as last redeem date for callerdddddccccccccc.
                    self.last_redeemed.insert(caller, &self.get_current_timestamp());
                   
                }



           }

            if caller_locked_balance > 0{

                //validates if the the allowance is equal or greatee than the deposit PANX amount
                let contract_allowance = PSP22Ref::allowance(
                    &self.panx_psp22,
                    caller,
                    Self::env().account_id()
                );

                //validates if the contract has sufficent allowance.
                if contract_allowance < panx_to_lock {
                    panic!(
                        "Not enough allowance, please make sure you approved the correct amount of PANX
                        tokens before adding to staking program."
                    )
                }


                //validates if the caller has enought PANX to lock
                if caller_current_panx_balance < panx_to_lock {
                    panic!(
                        "Caller does not have enough PANX tokens to lock,
                        kindly re-adjust deposit PANX amount."
                    )
                }

                //transfers PANX from caller to staking contract
                PSP22Ref::transfer_from_builder(
                    &self.panx_psp22,
                    caller,
                    Self::env().account_id(),
                    panx_to_lock, 
                    vec![])
                        .call_flags(CallFlags::default()
                        .set_allow_reentry(true))
                        .try_invoke()
                        .expect("Transfer failed")
                        .unwrap_or_else(|error| {
                            panic!(
                                "Failed to transfer PSP22 tokens to staking contract : {:?}",
                                error
                            )
                });
                
               //variable to hold current amount of locked PANX
                let new_balance:Balance = caller_locked_balance + panx_to_lock;

                //add PANX allocation to caller
                self.balances.insert(caller, &new_balance);

                let actual_staking_percentage:Balance = 70000000000 / 10u128.pow(12);

                //calc how many tokens to give in a day
                let amount_to_give_each_day:Balance = (new_balance + (new_balance * actual_staking_percentage)) / 365  ;

                //insert the daily amount to caller
                self.panx_to_give_in_a_day.insert(caller,&amount_to_give_each_day);


           }

           let tokens_to_validate:Balance = 1000*10u128.pow(12);

           if caller_current_panx_balance < tokens_to_validate && caller_locked_balance == 0 {

            panic!(
                "Caller has less than 1,000 PANX,
                cannot add to staking program."
            )

           }
        }


        ///function to get caller redeemable amount of tokens
        #[ink(message)]
        pub fn get_redeemable_amount(
            &mut self
        ) -> Balance {

            
            //call address 
            let caller = self.env().caller();
            //current timestamp
            let current_tsp = self.get_current_timestamp();
            //caller total locked PANX amount
            let caller_total_locked_amount:Balance = self.get_caller_total_locked_amount(caller);

        
            //last time caller redeemed tokens
            let last_redeemed = self.last_redeemed.get(caller).unwrap_or(0);
            //How many PANX tokens to give to caller each day
            let panx_to_give_each_day:Balance = self.panx_to_give_in_a_day.get(caller).unwrap_or(0);
            //days since last redeem
            let days_diff = (current_tsp - last_redeemed) / 86400;
            //making sure that 24 hours has passed since last redeem
            if days_diff <= 0 {
                panic!(
                     "0 Days passed since the last redeem,
                     kindly wait 24 hours after redeem."
                )
                }
            //making sure that caller has more then 0 PANX to redeem
            if caller_total_locked_amount <= 0 {
                panic!(
                     "Caller has balance of 0 locked tokens. "
                )
                }
            //amount to give to caller
            let mut panx_redeemable_amount:Balance = panx_to_give_each_day * days_diff as u128;


            //if caller has less tokens from the daily amount, give him the rest of tokens
            if panx_redeemable_amount > caller_total_locked_amount{

                panx_redeemable_amount = caller_total_locked_amount
            }

            panx_redeemable_amount

            

        }

        ///function for caller to redeem his redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(
            &mut self
        ) {

            
            //caller address
            let caller = self.env().caller();
            //caller timestamp
            let current_tsp = self.get_current_timestamp();

            //variable to hold redeemable panx amount
            let panx_redeemable_amount:Balance = self.get_redeemable_amount();

            //cross contract call to PANX contract to transfer PANX to caller
            PSP22Ref::transfer(
                &self.panx_psp22,
                caller,
                panx_redeemable_amount,
                vec![])
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to transfer PSP22 tokens to caller : {:?}",
                        error
                    )
            });

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(caller,&current_tsp);

        }

        ///function for caller to auto stack redeemable locked tokens.
        #[ink(message)]
        pub fn auto_stack(
            &mut self
        ) {

            
            //caller address
            let caller = self.env().caller();

            //caller timestamp
            let current_tsp = self.get_current_timestamp();

            //caller total locked PANX 
            let caller_total_locked_amount:Balance = self.get_caller_total_locked_amount(caller);

            let amount_redeemable_amount:Balance = self.get_redeemable_amount();

            //variable to hold current amount of locked PANX
            let new_balance:Balance = caller_total_locked_amount + amount_redeemable_amount;

            let actual_staking_percentage:Balance = 70000000000 / 10u128.pow(12);

            //calc how many tokens to give in a day
            let new_amount_to_give_each_day:Balance = (new_balance + (new_balance * actual_staking_percentage)) / 365;

            //insert the daily amount to caller
            self.panx_to_give_in_a_day.insert(caller,&new_amount_to_give_each_day);

            //make sure to increase overall amount
            self.balances.insert(caller, &(caller_total_locked_amount +  amount_redeemable_amount));

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(caller,&current_tsp);

        }

       ///Function to withdraw specific amount of locked PANX given from the front-end.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(
        &mut self,
        amount_of_tokens: u128
    )  {
          
           //caller address
           let caller = self.env().caller();

           //caller total LP shares
           let caller_locked_tokens = self.balances.get(&caller).unwrap_or(0);

           //variable to hold redeemable panx amount
           let panx_redeemable_amount = self.get_redeemable_amount();

           //Validating that the caller has the given number of locked tokens.
           if (caller_locked_tokens + panx_redeemable_amount) < amount_of_tokens {
            panic!(
                 "Caller does not have the amount of requested PANX to withdraw. "
            )
            }

           //Amount of panx to give to the caller
           let amount_of_panx_to_give = amount_of_tokens;

           //variable to hold new locked PANX balance
           let new_locked_balance = caller_locked_tokens - amount_of_panx_to_give;

           //reducing caller locked balance
           self.balances.insert(caller, &new_locked_balance);

           let actual_staking_percentage:Balance = 70000000000 / 10u128.pow(12);

           //calc how many tokens to give in a day
           let new_amount_to_give_each_day = (new_locked_balance + (new_locked_balance * actual_staking_percentage)) / 365 ;

           //insert the daily amount to caller
           self.panx_to_give_in_a_day.insert(caller,&new_amount_to_give_each_day);

           //cross contract call to PANX contract to transfer PANX to the caller
           PSP22Ref::transfer(
                &self.panx_psp22,
                caller,
                amount_of_panx_to_give,
                vec![])
                    .unwrap_or_else(|error| {
                        panic!(
                        "Failed to transfer PSP22 tokens to caller : {:?}",
                        error
                        )
            });
           let current_tsp = self.get_current_timestamp();

           //Making sure to set his last redeem to current timestamp
           self.last_redeemed.insert(caller,&current_tsp);
           

       }
        ///function to get caller total locked tokens
        #[ink(message)]
        pub fn get_caller_total_locked_amount(
            &mut self,
            caller:AccountId
        )-> Balance  {
        
           let caller_balance:Balance = self.balances.get(&caller).unwrap_or(0);
           caller_balance

        }
        ///funtion to get caller last redeem in timestamp
        #[ink(message)]
        pub fn get_caller_last_redeem(
            &mut self,
            caller:AccountId
        )->u64  {
        
           let time_stamp = self.last_redeemed.get(&caller).unwrap_or(0);
           time_stamp

        }
        ///function to get the amount of tokens to give to caller each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_caller(
            &mut self,
            caller:AccountId
        )-> Balance  {
        
           let caller_balance:Balance = self.panx_to_give_in_a_day.get(&caller).unwrap_or(0);
           caller_balance

        }
        ///funtion to get staking contract PANX reserve
        #[ink(message)]
        pub fn get_staking_contract_panx_reserve(
            &self
        )-> Balance  {
        
            let staking_contract_reserve:Balance = PSP22Ref::balance_of(
                &self.panx_psp22,
                Self::env().account_id()
            );
            staking_contract_reserve


        }

        ///funtion to get the started date since issuing the staking contract in timpstamp and str
        #[ink(message)]
        pub fn get_started_date(
            &self
        ) -> Balance {

            let timestamp = self.started_date_in_timestamp;
            timestamp.into()

        }

        
        ///function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(
            &self
        ) -> u64 {

            let time_stamp_in_seconds = self.env().block_timestamp() / 1000;
            time_stamp_in_seconds

        }


        //get the days pass since deployment
        #[ink(message)]
        pub fn get_days_passed_since_issue(
            &self
        ) -> Balance {

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