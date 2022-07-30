#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
extern crate chrono;


pub use self::trading_pair_azero::{
	TradingPairAzero,
	TradingPairAzeroRef,
};


#[ink::contract]
pub mod trading_pair_azero {
    

    use chrono::prelude::*;
    use ink_storage::traits::SpreadAllocate;

    
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    
    use ink_env::CallFlags;
    use ink_prelude::vec::Vec;
    use num_integer::Roots; 
    use ink_prelude::string::ToString;
    use ink_prelude::string::String;
    use ink_prelude::borrow::ToOwned;

    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct TradingPairAzero {

        //Number of overall transactions (Not including LP provision)
        transasction_number: i64,
        //Deployer address
        manager: AccountId,
        //PSP22 contract address
        psp22_token: AccountId,
        //A0 coin reserve
        a0_reserve:Balance,
        //PSP22 token reserve
        psp22_reserve: Balance,
        //LP fee
        fee: u128,
        //Total LP token supply
        total_supply: Balance,
        //LP token balances of LP providers
        balances: ink_storage::Mapping<AccountId, Balance>,
        //Hashmap of LP providers
        lp_providers: ink_storage::Mapping<AccountId, Balance>,
    }


    impl TradingPairAzero {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(psp22_contract:AccountId, fee: u128) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.psp22_token = psp22_contract;  
                contract.manager = Self::env().caller();
                contract.fee = fee
               
            });
            
            me
            
        }

       ///function to add liquidity to PSP22/A0 pair.
       ///If its the first liquidity provision, the provider will always receive 1000 LP shares.
       ///We validate that the provider gets the correct amount of shares as displayed to him in front-end (add LP UI) and never gets 0 shares.
       #[ink(message,payable)]
       pub fn provide_to_pool(&mut self,psp22_deposit_amount:u128,excpeted_lp_tokens:u128,slippage:u128)  {

           //init LP shares variable (shares to give to provider)
           let mut shares:Balance = 0;
           
           

           //if its the pool first deposit
           if self.total_supply == 0 {

               
               shares = 1000 * 10u128.pow(12);

           }

           //if its not the first LP deposit
           if self.total_supply > 0{


               //We need to sub the incoming amount of A0 by the current A0 reserve (current reserve includes incoming A0)
               let reserve_before_transaction = self.get_a0_balance() - self.env().transferred_value();
               //Shares to give to provider
               shares = (self.env().transferred_value() * self.total_supply) / reserve_before_transaction;

             
           }

           //validating that shares is greater than 0
           assert!(shares > 0);

           //function to return the precenrage diff between the expected lp token that was shown in the front-end and the final shares amount.
           let precentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares);

           //Validating slippage
           assert!(precentage_diff < slippage.try_into().unwrap());

           //A0 deposited amount
           let a0_deposit_amount = self.env().transferred_value();
           //cross contract call to psp22 contract to transfer psp22 token to the Pair contract
           PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), psp22_deposit_amount, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
                       
          

           //caller current shares (if any)
           let current_shares = self.get_lp_token_of(self.env().caller());
           //adding caller to LP providers
           self.lp_providers.insert(self.env().caller(), &(current_shares + shares));
           //increasing LP balance of caller (mint)
           self.balances.insert(self.env().caller(), &(current_shares + shares));
           //adding to over LP tokens (mint)
           self.total_supply += shares;



       }

       ///function to withdraw specific amount of LP tokens given from the front-end.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, shares: u128)  {
          
           //caller address
           let caller = self.env().caller();
           //caller total LP shares
           let caller_shares = self.balances.get(&caller).unwrap_or(0);

           ///Validating that the caller has the given number of shares.
           assert!(caller_shares >= shares);

           //Amount of psp22 to give to the caller
           let psp22_amount_to_give = self.get_psp22_withdraw_tokens_amount(shares);
           //Amount of psp22 to give to the caller
           let a0_amount_to_give = self.get_A0_withdraw_tokens_amount(shares);

           //reducing caller LP token balance
           self.balances.insert(caller, &(caller_shares - shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;

           
           //cross contract call to PSP22 contract to approve PSP22 to give to caller
           let response_1 = PSP22Ref::approve(&self.psp22_token, caller,psp22_amount_to_give);
           //cross contract call to PSP22 contract to transfer PSP22 to the caller
           let response_2 = PSP22Ref::transfer(&self.psp22_token, caller, psp22_amount_to_give, ink_prelude::vec![]);
           
           //fun to transfer A0 to the caller
           if self.env().transfer(self.env().caller(), a0_amount_to_give).is_err() {
               panic!(
                   "requested transfer failed. this can be the case if the contract does not\
                    have sufficient free funds or if the transfer would have brought the\
                    contract's balance below minimum balance."
               )
           }


       }

        
        //funtion to get amount of withdrable PSP22/A0 tokens by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, share_amount: u128) -> (Balance,Balance) {

            //caller address
            let caller = self.env().caller();        
            //amount of shares given
            let caller_shares = share_amount;

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (share_amount * self.get_a0_balance()) / self.total_supply;
            //calc amount of PSP22 to give 
            let amount_of_psp22_to_give = (share_amount * self.get_psp22_balance()) / self.total_supply;
        

            (amount_of_a0_to_give,amount_of_psp22_to_give)
        
        }


        ///function to get the amount of withdrawable psp22 tokens by given shares.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(&self, share_amount: u128) -> Balance {

       
            //amount of shares given
            let caller_shares = share_amount;


            //calc amount of PSP22 to give 
            let amount_of_psp22_to_give = (share_amount * self.get_psp22_balance()) / self.total_supply;
        

            amount_of_psp22_to_give
        
        }

        ///function to get the amount of withdrawable A0 by given LP shares.
        #[ink(message)]
        pub fn get_A0_withdraw_tokens_amount(&self, share_amount: u128) -> Balance {

        
            //amount of shares given
            let caller_shares = share_amount;

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (share_amount * self.get_a0_balance()) / self.total_supply;

        

            amount_of_a0_to_give
        
        }

        
        ///function to get caller pooled PSP22 tokens and A0
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let user = account_id;
            //get account LP tokens 
            let user_shares = self.balances.get(&user).unwrap_or(0);

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (user_shares * self.get_a0_balance()) / self.total_supply;
            //calc amount of PSP22 to give 
            let amount_of_psp22_to_give = (user_shares * self.get_psp22_balance()) / self.total_supply;

           
            (amount_of_psp22_to_give,amount_of_a0_to_give)

            
        }

        //function to get expected amount of LP shares.
        #[ink(message,payable)]
        pub fn get_expected_lp_token_amount(&self,a0_deposit_amount:Balance) -> Balance {

           //init LP shares variable (shares to give to user)
           let mut shares:Balance = 0;
           
           //if its the caller first deposit 
           if self.total_supply == 0 {

               shares = 1000 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

               let reserve_after_tras_value = self.get_a0_balance() - a0_deposit_amount;
               
               shares = (a0_deposit_amount * self.total_supply) / reserve_after_tras_value;

             
           }

            shares
            
        }
 

        ///function to get the amount of A0 given for 1 PSP22 token
	    #[ink(message)]
        pub fn get_price_for_one_psp22(&self)-> Balance {
            

            //nominating 1 to base 12 (10^12)
            let value = 1 * 10u128.pow(12);
            //formula to calculate the price
            let amount_out = (self.env().balance() * value) / (self.get_psp22_balance() + value);

            return amount_out

        }

        ///function to get the amount of A0 the caller will get for given PSP22 amount
        #[ink(message)]
        pub fn get_est_price_psp22_to_a0(&self, amount_in: Balance)-> Balance {


            let amount_in_with_fees = amount_in * 99;
            let numerator = amount_in_with_fees * self.get_a0_balance();
            
            
            let deno = (self.get_psp22_balance() * 100) + amount_in_with_fees;
            let amount_out = numerator / deno;
            
            return amount_out                        

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (swap use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22_for_swap(&mut self,ao_amount_to_tranfer:Balance) -> Balance { 

            let a0_reserve_before = self.get_a0_balance() - ao_amount_to_tranfer;

            let amount_in_with_fees = ao_amount_to_tranfer * 99;
            let numertraor = amount_in_with_fees * self.get_psp22_balance();
            
            //uint256 numerator = inputAmountWithFee * outputReserve;
            let deno = (a0_reserve_before * 100) + amount_in_with_fees;
            let amount_out = numertraor / deno;
            //uint256 denominator = (inputReserve * 100) + inputAmountWithFee;
            return amount_out  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (front-end use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22(&mut self,ao_amount_to_tranfer:Balance) -> Balance { 

            let amount_in_with_fees = ao_amount_to_tranfer * 99;
            let numertraor = amount_in_with_fees * self.get_psp22_balance();
            
            //uint256 numerator = inputAmountWithFee * outputReserve;
            let deno = (self.get_a0_balance() * 100) + amount_in_with_fees;
            let amount_out = numertraor / deno;
            //uint256 denominator = (inputReserve * 100) + inputAmountWithFee;
            return amount_out  

        }

        ///function to get the estimated price impact for given psp22 token amount
        #[ink(message)]
        pub fn get_price_impact_psp22_to_a0(&self,value: u128) -> Balance {
            
            //Reduct LP fee from the amount in
            let amount_in_with_fees =  (value * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.env().balance()  * amount_in_with_fees) / (self.get_psp22_balance() + amount_in_with_fees);
            //calc the price impact using the amount_out tokens
            let amount_out_with_impact = ((self.env().balance() - amount_out ) * amount_in_with_fees) / ((self.get_psp22_balance() + value ) + amount_in_with_fees);
            
            amount_out_with_impact

        }
        ///function to get the estimated price impact for given A0 amount
        #[ink(message,payable)]
        pub fn get_price_impact_a0_to_psp22(&self,a0_to_trasfer:Balance) -> Balance {
            
         
            //amount of A0 given by user
            let amount_in = a0_to_trasfer;
            //Reduct LP fee from the amount in
            let amount_in_with_feed =  (amount_in * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.get_psp22_balance() * amount_in_with_feed) / (self.env().balance() + amount_in_with_feed);
            //calc the price impact using the amount_out tokens
            let amount_out_with_impact = ((self.get_psp22_balance() - amount_out ) * amount_in_with_feed) / ((self.env().balance() + amount_in ) + amount_in_with_feed);
            
            amount_out_with_impact


        }

        
        ///function to swap psp22 to a0
        #[ink(message)]
        pub fn swap_psp22(&mut self,amount_to_transfer: u128, amount_to_validate: u128,slippage: u128) {
            
            //the amount of A0 to give to the caller.
            let amount_out = self.get_est_price_psp22_to_a0(amount_to_transfer);


            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            let response = PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), amount_to_transfer, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire();

            //let amount_in_with_fees = (amount_to_transfer * 997) / 1000;

            //let amount_out = (self.get_a0_balance() * amount_in_with_fees) / (self.get_psp22_balance() + amount_in_with_fees);

            //let abs_value = amount_to_validate.abs_diff(amount_out);

            //let diff_in_percentage = (abs_value / amount_to_validate) * 100;

            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            assert!(precentage_diff < slippage.try_into().unwrap());
            //fun to transfer A0 to swapper
            if self.env().transfer(self.env().caller(), amount_out).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

        }


        ///function to swap a0 and psp22
        #[ink(message,payable)]
        pub fn swap_a0(&mut self,amount_to_validate: u128,slippage: u128) {
            
            //amount of PSP22 tokens to give to caller.
            let amount_out = self.get_est_price_a0_to_psp22_for_swap(self.env().transferred_value());
            //amount of transferred A0
            //let amount_in = self.env().transferred_value();
            //100000000
            //Reduct LP fee from the amount in
            //let amount_in_with_feed =  (amount_in * 997) / 1000;
            //calc how much tokens to give to swapper
            //let amount_out = (self.get_psp22_balance() * amount_in_with_feed) / (self.get_a0_balance() + amount_in_with_feed);

            //precentage dif between given PSP22 amount (from front-end) and acutal final PSP22 amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            assert!(precentage_diff < slippage.try_into().unwrap());
            //cross contract call to PSP22 contract to transfer PSP22 to the swapper
            let response = PSP22Ref::transfer(&self.psp22_token, self.env().caller(), amount_out, ink_prelude::vec![]);


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            
        }



        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
        
            self.get_est_price_psp22_to_a0(1 * 10u128.pow(12))    
        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        ///function to get contract A0 balance
        #[ink(message)]
        pub fn get_a0_balance(&self) -> Balance {
            
            let amount = self.env().balance();
            amount
        }
        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(&self,of: AccountId) -> Balance {
            let of_balance = self.balances.get(&of).unwrap_or(0);

            of_balance
        }
        //function to get contract PSP22 reserve (self)
        #[ink(message)]
        pub fn get_psp22_balance(&self) -> Balance {
            let balance1 = PSP22Ref::balance_of(&self.psp22_token, Self::env().account_id());
            balance1
        }
        ///function to get current fee 
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            let amount = self.fee;
            amount
        }


    	#[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number

        }
        
        #[ink(message,payable)]
        pub fn check_diffrenece(&mut self,value1: u128,value2: u128) -> u128 {

            let abs_dif = value1.abs_diff(value2);

            let abs_dif_nominated = abs_dif * 10u128.pow(12);

            let diff = (100 * (abs_dif_nominated / ((value1+value2) / 2))) ;

            diff
            
        }

        #[ink(message, payable, selector = 0xCAFEBABE)]
        pub fn was_it_ten(&self) {
            ink_env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );
            assert!(self.env().transferred_value() == 10, "payment was not ten");
        }
 
    }
}