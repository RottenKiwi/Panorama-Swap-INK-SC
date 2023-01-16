#![cfg_attr(not(feature = "std"), no_std)]





pub use self::trading_pair_azero::{
	TradingPairAzero,
	TradingPairAzeroRef,
};


#[ink::contract]
pub mod trading_pair_azero {
    
    
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    use ink::storage::Mapping;



    
    
    #[ink(storage)]
    pub struct TradingPairAzero {

        //Number of overall transactions (Not including LP provision)
        transasction_number: i64,
        //PSP22 contract address
        psp22_token: AccountId,
        //LP fee
        fee: Balance,
        //Total LP token supply
        total_supply: Balance,
        //LP token balances of LP providers
        balances: Mapping<AccountId, Balance>,
        //PANX contract address
        panx_contract: AccountId,
        //Store accounts LP tokens allowances
        lp_tokens_allowances: Mapping<(AccountId,AccountId), Balance>,
        //Account id to transfer trader's fee to
        vault: AccountId,
        //Trader's fee
        traders_fee:Balance


    }

    #[ink(event)]
    pub struct LiquidityPoolProvision {
        from:AccountId,
        a0_deposited_amount:Balance,
        psp22_deposited_amount: Balance,
        shares_given:Balance
    }

    #[ink(event)]
    pub struct LiquidityPoolWithdrawal {
        caller:AccountId,
        shares_given:Balance,
        a0_given_amount:Balance,
        psp22_given_amount: Balance,
        new_shares_balance:Balance
    }

    #[ink(event)]
    pub struct A0Swap {
        caller:AccountId,
        a0_deposited_amount:Balance,
        psp22_given_amount:Balance
    }

    #[ink(event)]
    pub struct PSP22Swap {
        caller:AccountId,
        psp22_deposited_amount:Balance,
        a0_given_amount:Balance,
  
    }


    impl TradingPairAzero {
        /// Creates a new instance of trading pair azero contract.
        #[ink(constructor)]
        pub fn new(psp22_contract:AccountId, fee: Balance,panx_contract:AccountId,vault:AccountId) -> Self {


            let transasction_number:i64 = 0;
            let balances = Mapping::default();
            let lp_tokens_allowances = Mapping::default();
            let psp22_token = psp22_contract;
            let total_supply = 0;
            let traders_fee:Balance = 25000000000 / 10u128.pow(12);

            Self {
                transasction_number,
                psp22_token,
                fee,
                total_supply,
                balances,
                panx_contract,
                lp_tokens_allowances,
                vault,
                traders_fee
            }

            
        }

       ///function to provide liquidity to a PSP22/A0 trading pair contract.
       #[ink(message,payable)]
       pub fn provide_to_pool(&mut self,psp22_deposit_amount:Balance,excpeted_lp_tokens:Balance,slippage:Balance)  {

           //fetching caller current psp22 balance  
           let caller_current_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, self.env().caller());

           //making sure that caller current PSP22 balance is greater than the deposit amount.
           if caller_current_balance < psp22_deposit_amount {
            panic!(
                 "Caller does not have enough PSP22 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22 tokens."
            )
            } 

           let contract_allowance:Balance = PSP22Ref::allowance(&self.psp22_token, self.env().caller(),Self::env().account_id());


           //making sure that trading pair contract has enough allowance.
           if contract_allowance < psp22_deposit_amount {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22 tokens."
            )
            }


           let mut shares:Balance = 0;
           
           //if its the pool first deposit
           if self.total_supply == 0 {

            //calculating the amount of shares to give to the provider if its the first LP deposit overall
            shares = 1000u128 * 10u128.pow(12);


           }

           //if its NOT the first LP deposit
           if self.total_supply > 0{


               //we need to sub the incoming amount of A0 by the current A0 reserve (current reserve includes incoming A0)
               let reserve_before_transaction = self.get_a0_balance() - self.env().transferred_value();

               //calculating the amount of shares to give to the provider if its not the first LP deposit
               match (self.env().transferred_value() * self.total_supply).checked_div(reserve_before_transaction) {
                    Some(result) => {
                        shares = result;
                    }
                    None => {
                        panic!("overflow!");
                    }
                };
             
           }

           //validating that shares is greater than 0
           if shares <= 0 {
            panic!(
                 "Expected given liquidity pool SHARES are equal to 0,
                 cannot proceed with liquidity pool provision."
            )
            }

           //function to return the percentage diff between the expected lp token that was shown in the front-end and the final shares amount.
           let percentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares);

           //validating slippage    
           if percentage_diff > slippage.try_into().unwrap() {
            panic!(
                "The percentage difference is bigger than the given slippage,
                kindly re-adjust the slippage settings."
            )
            }


           let current_shares:Balance = self.get_lp_token_of(self.env().caller());

           let new_caller_shares:Balance;

           //calculating the current caller shares with the new provided shares.
            match current_shares.checked_add(shares) {
                Some(result) => {
                    new_caller_shares = result;
                }
                None => {
                    panic!("overflow!");
                }
            };


           //cross contract call to psp22 contract to transfer psp22 token to the pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), psp22_deposit_amount, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22 transferFrom cross contract call function, kindly re-adjust your deposited PSP22 tokens."
           )
           }
           
                    
           //increasing LP balance of caller (mint)
           self.balances.insert(self.env().caller(), &(new_caller_shares));
           //adding to over LP tokens (mint)
           self.total_supply += shares;
           Self::env().emit_event(LiquidityPoolProvision{from:self.env().caller(),a0_deposited_amount:self.env().transferred_value(),psp22_deposited_amount:psp22_deposit_amount,shares_given:shares})



       }

       ///function to withdraw specific amount of LP share tokens.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, shares: Balance)  {
          
           //caller address 
           let caller = self.env().caller();

           //caller total LP shares
           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           //validating that the caller has more than the given number of shares.
           if caller_shares < shares {
            panic!(
                 "Caller does not have enough liquidity pool SHARES to withdraw,
                  kindly lower the liquidity pool SHARES withdraw amount."
            )
            }

           //amount of PSP22 to give to the caller
           let psp22_amount_to_give:Balance = self.get_psp22_withdraw_tokens_amount(shares);

           //amount of A0 to give to the caller
           let a0_amount_to_give:Balance = self.get_a0_withdraw_tokens_amount(shares);

           let new_caller_lp_shares:Balance;

           //calculating the current caller shares with the new provided shares.
           match caller_shares.checked_sub(shares) {
            Some(result) => {
                new_caller_lp_shares = result;
            }
            None => {
                panic!("overflow!");
            }
        };
           
           //cross contract call to PSP22 contract to transfer PSP2 to the caller.
           PSP22Ref::transfer(&self.psp22_token, caller, psp22_amount_to_give, ink::prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 tokens to caller : {:?}",
                error
            )
            });

           //function to transfer A0 to the caller
           if self.env().transfer(self.env().caller(), a0_amount_to_give).is_err() {
               panic!(
                   "requested transfer failed. this can be the case if the contract does not\
                    have sufficient free funds or if the transfer would have brought the\
                    contract's balance below minimum balance."
               )
           }

           //reducing caller LP token balance
           self.balances.insert(caller, &(new_caller_lp_shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;
           Self::env().emit_event(LiquidityPoolWithdrawal{caller:caller,shares_given:shares,a0_given_amount:a0_amount_to_give,psp22_given_amount:psp22_amount_to_give,new_shares_balance:new_caller_lp_shares});



       }


        ///funtion to get the amount of withdrawable PSP22 and A0 by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, shares_amount: Balance) -> (Balance,Balance) {

            let amount_of_a0_to_give:Balance;

            //calculating the amount of A0 to give to the caller.
            match (shares_amount * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let amount_of_psp22_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
        

            (amount_of_a0_to_give,amount_of_psp22_to_give)
        
        }


        ///funtion to get the amount of withdrawable PSP22 by given number of LP shares.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(&self, shares_amount: Balance) -> Balance {

            let amount_of_psp22_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            amount_of_psp22_to_give
        
        }

        ///funtion to get the amount of withdrawable A0 by given number of LP shares.
        #[ink(message)]
        pub fn get_a0_withdraw_tokens_amount(&self, shares_amount: Balance) -> Balance {


            let amount_of_a0_to_give:Balance;

            //calculating the amount of A0 to give to the caller.
            match (shares_amount * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            amount_of_a0_to_give
        
        }

        
        ///function to get the callers pooled PSP22 and A0.
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let caller = account_id;
            //get account LP tokens 
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let mut amount_of_a0_to_give:Balance = 0;

            let mut amount_of_psp22_to_give:Balance = 0;


            if caller_shares <= 0 {

                return (amount_of_psp22_to_give,amount_of_a0_to_give)
                 
            }

            
            //calculating the amount of A0 to give to the caller.
            match (caller_shares * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //calculating the amount of PSP22 to give to the caller.
            match (caller_shares * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
           
            (amount_of_psp22_to_give,amount_of_a0_to_give)

            
        }

        //function to get the expected amount of LP shares by given A0 amount.
        #[ink(message)]
        pub fn get_expected_lp_token_amount(&self,a0_deposit_amount:Balance) -> Balance {


           let mut shares:Balance = 0;
           
           //if its the trading pair first deposit 
           if self.total_supply == 0 {

            //calculating the amount of shares to give to the provider if its the first LP deposit overall
            shares = 1000u128 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

               //calculating the amount of shares to give to the provider if its not the first LP deposit
               match (a0_deposit_amount * self.total_supply).checked_div(self.get_a0_balance()) {
                Some(result) => {
                    shares = result;
                }
                None => {
                    panic!("overflow!");
                }
                };

           }

            shares
            
        }
 

        ///function to get the amount of A0 the caller will get for 1 PSP22 token.
        #[ink(message)]
        pub fn get_price_for_one_psp22(&self)-> Balance {

            let amount_out = self.get_est_price_psp22_to_a0(1u128 * (10u128.pow(12)));

            amount_out
        }

        ///function to get the amount of A0 the caller will get for given PSP22 amount.
        #[ink(message)]
        pub fn get_est_price_psp22_to_a0(&self, psp22_amount_in:Balance)-> Balance {

            //fetching caller current PSP22 balance
            let caller_current_balance:Balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the PSP22 amount in
            match psp22_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let tokens_to_validate:Balance = 3500u128 * 10u128.pow(12);

           //validating if caller has more than 3500 PANX to verify if the caller is eligible for the incentive program 
           if caller_current_balance >= tokens_to_validate{

                if self.fee  <= 1400000000000u128 {

                    //reducting HALF of the LP fee from PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match psp22_amount_in.checked_mul(100u128 - (actual_fee / 2u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };   
                }
 
                if self.fee  > 1400000000000u128 {

                    //reducting (LP fee - 1) of the LP fee from PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match psp22_amount_in.checked_mul(100u128 - (actual_fee - 1u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };  
                }
             }

             
            let a0_amount_out:Balance;
            
            //calculating the final A0 amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_a0_balance()).checked_div((self.get_psp22_balance() * 100u128) + amount_in_with_lp_fees) {
                Some(result) => {
                    a0_amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            return a0_amount_out  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (swap use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22_for_swap(&self,a0_amout_in:Balance) -> Balance { 


            let a0_reserve_before:Balance;

            //calculating the A0 contract reserve before the transaction
            match self.get_a0_balance().checked_sub(a0_amout_in) {
                Some(result) => {
                    a0_reserve_before = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let caller_current_balance:Balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };


            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amout_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let tokens_to_validate = 3500u128 * 10u128.pow(12);

            //validating if the caller has more than 3500 PANX
            if caller_current_balance >= tokens_to_validate{

                if self.fee  <= 1400000000000 {

                    //reducting HALF of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match a0_amout_in.checked_mul(100u128 - (actual_fee / 2u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };
                }
 
                if self.fee  > 1400000000000 {

                    //reducting (LP fee - 1) of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match a0_amout_in.checked_mul(100u128 - (actual_fee - 1u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };
                }
             }


            let amount_out:Balance;

            //calculating the final PSP22 amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_psp22_balance()).checked_div((a0_reserve_before * 100u128) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            return amount_out  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (front-end use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22(&self,a0_amout_in:Balance) -> Balance { 

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let amount_in_with_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amout_in.checked_mul(100u128 - actual_fee){
                Some(result) => {
                    amount_in_with_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let amount_out:Balance;

            //calculating the final PSP22 amount to transfer to the caller
            match (amount_in_with_fees * self.get_psp22_balance()).checked_div((self.get_a0_balance() * 100u128) + amount_in_with_fees){
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            return amount_out  

        }

        ///function to get the estimated price impact for given psp22 token amount
        #[ink(message)]
        pub fn get_price_impact_psp22_to_a0(&self,psp22_amount_in:Balance) -> Balance {

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //fetching the amount of A0 the caller WOULD get if he would swap
            let current_amount_out = self.get_est_price_psp22_to_a0(psp22_amount_in);

            let amount_in_with_fees:Balance;

            //reducting the LP fee from the PSP22 amount in
            match psp22_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
    
            let future_ao_amount_out:Balance;

            //calculating the final future A0 amount to transfer to the caller.
            match (amount_in_with_fees * (self.get_a0_balance() - current_amount_out)).checked_div(((self.get_psp22_balance() + psp22_amount_in) * 100) + amount_in_with_fees) {
                Some(result) => {
                    future_ao_amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            future_ao_amount_out
    
        }
        
        ///function to get the estimated price impact for given A0 amount
        #[ink(message)]
        pub fn get_price_impact_a0_to_psp22(&mut self,a0_amount_in:Balance) -> Balance {
            
            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let current_amount_out = self.get_est_price_a0_to_psp22(a0_amount_in);

            let amount_in_with_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let future_psp22_amount_out:Balance;

            //calculating the final future PSP22 amount to transfer to the caller.
            match (amount_in_with_fees * (self.get_psp22_balance() - current_amount_out)).checked_div(((self.get_a0_balance() + a0_amount_in)* 100) + amount_in_with_fees) {
                Some(result) => {
                    future_psp22_amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            future_psp22_amount_out


        }

        
        ///function to swap PSP22 to A0
        #[ink(message)]
        pub fn swap_psp22(&mut self,psp22_amount_to_transfer: Balance, a0_amount_to_validate: Balance,slippage: Balance) {

            let caller_current_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, self.env().caller());

            //making sure that the caller has more or equal the amount he wishes to transfers.
            if caller_current_balance < psp22_amount_to_transfer {
                panic!(
                    "Caller balance is lower than the amount of PSP22 token he wishes to trasnfer,
                    kindly lower your deposited PSP22 tokens amount."
                )
            }
            
            let contract_allowance:Balance = PSP22Ref::allowance(&self.psp22_token, self.env().caller(),Self::env().account_id());
            
            //making sure that the trading pair contract has enough allowance.
            if contract_allowance < psp22_amount_to_transfer {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22 tokens before swapping."
                )
            }
            
            //the amount of A0 to give to the caller before traders fee.
            let a0_amount_out_for_caller_before_traders_fee:Balance = self.get_est_price_psp22_to_a0(psp22_amount_to_transfer);

            //percentage dif between given A0 amount (from front-end) and acutal final AO amount
            let percentage_diff:Balance = self.check_diffrenece(a0_amount_to_validate,a0_amount_out_for_caller_before_traders_fee);

            //validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            let actual_a0_amount_out_for_caller:Balance;

            //calculating the final amount of A0 coins to give to the caller after reducing traders fee
            match  a0_amount_out_for_caller_before_traders_fee.checked_sub(a0_amount_out_for_caller_before_traders_fee * self.traders_fee) {
                Some(result) => {
                    actual_a0_amount_out_for_caller = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let a0_amount_out_for_vault:Balance;

            //calculating the amount of A0 coins to allocate to the vault account
            match  a0_amount_out_for_caller_before_traders_fee.checked_sub(actual_a0_amount_out_for_caller) {
                Some(result) => {
                    a0_amount_out_for_vault = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           //cross contract call to psp22 contract to transfer psp22 token to the Pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), psp22_amount_to_transfer, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22 transferFrom cross contract call function, kindly re-adjust your deposited PSP22 tokens."
           )
           }


            //function to transfer A0 to the caller.
            if self.env().transfer(self.env().caller(), actual_a0_amount_out_for_caller).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }

            //function to transfer A0 to the vault.
            if self.env().transfer(self.vault, a0_amount_out_for_vault).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            Self::env().emit_event(PSP22Swap{caller:self.env().caller(),psp22_deposited_amount:psp22_amount_to_transfer,a0_given_amount:actual_a0_amount_out_for_caller});


        }


        ///function to swap A0 to PSP22
        #[ink(message,payable)]
        pub fn swap_a0(&mut self,psp22_amount_to_validate: Balance,slippage: Balance) {
            
            //amount of PSP22 tokens to give to caller before traders fee.
            let psp22_amount_out_for_caller_before_traders_fee:Balance = self.get_est_price_a0_to_psp22_for_swap(self.env().transferred_value());

            //percentage dif between given PSP22 amount (from front-end) and the acutal final PSP22 amount.
            let percentage_diff:Balance = self.check_diffrenece(psp22_amount_to_validate,psp22_amount_out_for_caller_before_traders_fee);

            //validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            let actual_psp22_amount_out_for_caller:Balance;

            //calculating the final amount of PSP22 tokens to give to the caller after reducing traders fee
            match psp22_amount_out_for_caller_before_traders_fee.checked_sub(psp22_amount_out_for_caller_before_traders_fee * self.traders_fee) {
                Some(result) => {
                    actual_psp22_amount_out_for_caller = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let psp22_amount_out_for_vault:Balance;

            //calculating the amount of PSP22 tokens to allocate to the vault account
            match psp22_amount_out_for_caller_before_traders_fee.checked_sub(actual_psp22_amount_out_for_caller) {
                Some(result) => {
                    psp22_amount_out_for_vault = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //cross contract call to PSP22 contract to transfer PSP22 to the caller
            PSP22Ref::transfer(&self.psp22_token, self.env().caller(), actual_psp22_amount_out_for_caller, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 tokens to caller : {:?}",
                    error
                )
            });

            //cross contract call to PSP22 contract to transfer PSP22 to the vault
            PSP22Ref::transfer(&self.psp22_token, self.vault, psp22_amount_out_for_vault, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 tokens to vault : {:?}",
                    error
                )
            });


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            Self::env().emit_event(A0Swap{caller:self.env().caller(),a0_deposited_amount:self.env().transferred_value(),psp22_given_amount:actual_psp22_amount_out_for_caller});

            
            
        }


        ///function used to transfer LP share tokens from caller to recipient.
        #[ink(message)]
        pub fn transfer_lp_tokens(&mut self, recipient:AccountId,shares_to_transfer: Balance) {

            let caller = self.env().caller();

            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let recipient_shares:Balance = self.balances.get(&recipient).unwrap_or(0);
        
            if caller_shares < shares_to_transfer {
                panic!(
                    "Cannot transfer LP shares to recipient, caller balance is lower than the requested transfer amount."
                )
            }

            let new_caller_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match caller_shares.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_caller_lp_balance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let new_recipient_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match recipient_shares.checked_add(shares_to_transfer) {
                Some(result) => {
                    new_recipient_lp_balance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            self.balances.insert(caller, &(new_caller_lp_balance));
 
            self.balances.insert(recipient, &(new_recipient_lp_balance));


        }

        ///function used to approve the amount of LP token shares for the spender to spend from owner.
        #[ink(message)]
        pub fn approve_lp_tokens(&mut self, spender:AccountId,shares_to_approve: Balance)  {

           let caller = self.env().caller();

           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           if caller_shares < shares_to_approve {
            panic!(
                "Cannot approve LP tokens, owner LP token balance is lower than the given shares to approve."
            )
           }

           if shares_to_approve >= u128::MAX {
            panic!(
                "overflow!"
            )
           }


           self.lp_tokens_allowances.insert((caller,spender), &(shares_to_approve));

        }

        //function to transfer LP share tokens FROM owner TO receipent
        #[ink(message)]
        pub fn transfer_lp_tokens_from_to(&mut self,owner:AccountId,to:AccountId,shares_to_transfer: Balance)  {

           let spender = self.env().caller();

           let owner_shares:Balance = self.balances.get(&owner).unwrap_or(0);

           let to_shares:Balance = self.balances.get(&to).unwrap_or(0);

           let allowance:Balance = self.get_lp_tokens_allowance(owner,spender);

           if allowance < shares_to_transfer {
            panic!(
                "Cannot transfer LP shares to spender, allowance is lower than the requested transfer amount."
            )
           }

           if owner_shares < shares_to_transfer {
            panic!(
                "Cannot transfer LP shares to spender, caller balance is lower than the requested transfer amount."
            )
           }

           let new_owner_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match owner_shares.checked_sub(shares_to_transfer) {
               Some(result) => {
                   new_owner_lp_balance = result;
               }
               None => {
                   panic!("overflow!");
               }
           };

           let new_to_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match to_shares.checked_add(shares_to_transfer) {
               Some(result) => {
                new_to_lp_balance = result;
               }
               None => {
                   panic!("overflow!");
               }
           };

           let new_allowance:Balance;

           //calculating spender new allowance amount
           match allowance.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_allowance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           self.balances.insert(owner, &(new_owner_lp_balance));

           self.lp_tokens_allowances.insert((owner,spender), &(new_allowance));

           self.balances.insert(to, &(new_to_lp_balance));
    
         
        }
         
        //function to get the allowance of spender from the owner
        #[ink(message)]
        pub fn get_lp_tokens_allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.lp_tokens_allowances.get(&(owner,spender)).unwrap_or(0)
        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
            self.get_est_price_psp22_to_a0(1u128 * 10u128.pow(12))    
        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        ///function to get contract A0 balance
        #[ink(message)]
        pub fn get_a0_balance(&self) -> Balance {
            let a0_balance = self.env().balance();
            a0_balance
        }

        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(&self,account: AccountId) -> Balance {
            let account_balance:Balance = self.balances.get(&account).unwrap_or(0);
            account_balance
        }

        //function to get contract PSP22 reserve (self)
        #[ink(message)]
        pub fn get_psp22_balance(&self) -> Balance {
            let psp22_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, Self::env().account_id());
            psp22_balance
        }
        ///function to get current fee 
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            let fee:Balance = self.fee;
            fee
        }


    	#[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number

        }
        
        ///function to calculate the percentage between values.
        #[ink(message,payable)]
        pub fn check_diffrenece(&mut self,value1: Balance,value2: Balance) -> Balance {

            let absolute_difference = value1.abs_diff(value2);


            let absolute_difference_nominated = absolute_difference * (10u128.pow(12));


            let percentage_difference:Balance;

            match 100u128.checked_mul(absolute_difference_nominated / ((value1+value2) / 2)) {
                Some(result) => {
                    percentage_difference = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            percentage_difference

      
            
        }

 
    }
}