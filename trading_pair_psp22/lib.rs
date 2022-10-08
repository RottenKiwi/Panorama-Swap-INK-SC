#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;



pub use self::trading_pair_psp22::{
	TradingPairPsp22,
	TradingPairPsp22Ref,
};


#[ink::contract]
pub mod trading_pair_psp22 {
    


    use ink_storage::traits::SpreadAllocate;

    
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    
    use ink_env::CallFlags;




    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct TradingPairPsp22 {

        //Number of overall transactions (Not including LP provision)
        transasction_number: i64,
        //PSP22 contract address
        psp22_token1_address: AccountId,
        //PSP22 contract address
        psp22_token2_address: AccountId,
        //LP fee
        fee: u128,
        //Total LP token supply
        total_supply: Balance,
        //LP token balances of LP providers
        balances: ink_storage::Mapping<AccountId, Balance>,
        //PANX contract address
        panx_contract: AccountId,
    }


    impl TradingPairPsp22 {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(psp22_token1_contract:AccountId,psp22_token2_contract:AccountId, fee: u128,panx_contract:AccountId) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.psp22_token1_address = psp22_token1_contract;  
                contract.psp22_token2_address = psp22_token2_contract; 
                contract.fee = fee;
                contract.panx_contract = panx_contract;
               
            });
            
            me
            
        }

       ///function to add liquidity to PSP22/PSP22 pair.
       ///If its the first liquidity provision, the provider will always receive 1000 LP shares.
       ///We validate that the provider gets the correct amount of shares as displayed to him in front-end (add LP UI) and never gets 0 shares.
       #[ink(message,payable)]
       pub fn provide_to_pool(&mut self,psp22_token1_deposit_amount:u128,psp22_token2_deposit_amount:u128,excpeted_lp_tokens:u128,slippage:u128)  {

           //init LP shares variable (shares to give to provider)
           let mut shares:Balance = 0;
          
           //if its the pool first deposit
           if self.total_supply == 0 {
            
            shares = 1000 * 10u128.pow(12);

           }

           //if its not the first LP deposit
           if self.total_supply > 0{

            //Shares to give to provider
            shares = (psp22_token1_deposit_amount * self.total_supply) / self.get_psp22_token1_reserve();

             
           }

           //validating that shares is greater than 0
           assert!(shares > 0);

           //function to return the precenrage diff between the expected lp token that was shown in the front-end and the final shares amount.
           let precentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares);

           //Validating slippage
           assert!(precentage_diff < slippage.try_into().unwrap());

           let user_current_balance_token1 = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());

           assert!(user_current_balance_token1 >= psp22_token1_deposit_amount);

           let user_current_balance_token2 = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());

           assert!(user_current_balance_token2 >= psp22_token2_deposit_amount);

           let contract_token1_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());
           //making sure trading pair contract has enough allowance.
           assert!(contract_token1_allowance >= psp22_token1_deposit_amount);

           let contract_token2_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());
           //making sure trading pair contract has enough allowance.
           assert!(contract_token2_allowance >= psp22_token2_deposit_amount);

           //cross contract call to psp22 token1 contract to transfer psp22 token1 to the Pair contract
           PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), psp22_token1_deposit_amount, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
           //cross contract call to psp22 token2 contract to transfer psp22 token2 to the Pair contract
           PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), psp22_token2_deposit_amount, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
                       
          

           //caller current shares (if any)
           let current_shares = self.get_lp_token_of(self.env().caller());
           //increasing LP balance of caller (mint)
           self.balances.insert(self.env().caller(), &(current_shares + shares));
           //adding to over LP tokens (mint)
           self.total_supply += shares;



       }

       ///Function to withdraw specific amount of LP tokens given from the front-end.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, shares: u128)  {
          
           //caller address
           let caller = self.env().caller();
           //caller total LP shares
           let _caller_shares = self.balances.get(&caller).unwrap_or(0);

           //Validating that the caller has the given number of shares.
           assert!(_caller_shares >= shares);

           //Amount of psp22 token1 to give to the caller
           let psp22_token1_amount_to_give = self.get_psp22_token1_withdraw_tokens_amount(shares);
           //Amount of psp22 token1 to give to the caller
           let psp22_token2_amount_to_give = self.get_psp22_token2_withdraw_tokens_amount(shares);
           
           //reducing caller LP token balance_caller_shares
           self.balances.insert(caller, &(_caller_shares - shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;

           
           //cross contract call to PSP22 token1 contract to approve PSP22 token1 to give to caller
          // PSP22Ref::approve(&self.psp22_token1_address, caller,psp22_token1_amount_to_give).fire().expect("approve failed").expect("approve failed");
           //cross contract call to PSP22 token1 contract to transfer PSP22 token1 to the caller
           let _response_1 = PSP22Ref::transfer(&self.psp22_token1_address, caller, psp22_token1_amount_to_give, ink_prelude::vec![]);
           
           //cross contract call to PSP22 token2 contract to approve PSP22 token2 to give to caller
           //PSP22Ref::approve(&self.psp22_token2_address, caller,psp22_token1_amount_to_give).fire().expect("approve failed").expect("approve failed");
           //cross contract call to PSP22 token2 contract to transfer PSP22 token2 to the caller
           let _response_2 = PSP22Ref::transfer(&self.psp22_token2_address, caller, psp22_token2_amount_to_give, ink_prelude::vec![]);



       }

        
        ///funtion to get amount of withdrable PSP22/PSP22 tokens by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, share_amount: u128) -> (Balance,Balance) {

            //calc amount of PSP22 token 1 to give
            let amount_of_psp22_token1_to_give = (share_amount * self.get_psp22_token1_reserve()) / self.total_supply;
            //calc amount of PSP22 token 2 to give
            let amount_of_psp22_token2_to_give = (share_amount * self.get_psp22_token2_reserve()) / self.total_supply;
        

            (amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give)
        
        }


        ///function to get the amount of withdrawable PSP22 token1 by given shares.
        #[ink(message)]
        pub fn get_psp22_token1_withdraw_tokens_amount(&self, share_amount: u128) -> Balance {

       
            //calc amount of PSP22 token1 to give 
            let amount_of_psp22_token1_to_give = (share_amount * self.get_psp22_token1_reserve()) / self.total_supply;
        

            amount_of_psp22_token1_to_give
        
        }

        ///function to get the amount of withdrawable PSP22 token2 by given LP shares.
        #[ink(message)]
        pub fn get_psp22_token2_withdraw_tokens_amount(&self, share_amount: u128) -> Balance {

        
            //calc amount of PSP22 token2 to give 
            let amount_of_psp22_token2_to_give = (share_amount * self.get_psp22_token2_reserve()) / self.total_supply;
        

            amount_of_psp22_token2_to_give
        
        
        }

        
        ///function to get caller pooled PSP22 token1 and PSP22 token2 amounts
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let user = account_id;
            //get account LP tokens 
            let user_shares = self.balances.get(&user).unwrap_or(0);

            //calc amount of PSP22 token 1 to give
            let amount_of_psp22_token1_to_give = (user_shares * self.get_psp22_token1_reserve()) / self.total_supply;
            //calc amount of PSP22 token 2 to give
            let amount_of_psp22_token2_to_give = (user_shares * self.get_psp22_token2_reserve()) / self.total_supply;
        

            (amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give)

            
        }

        //function to get expected amount of LP shares.
        #[ink(message,payable)]
        pub fn get_expected_lp_token_amount(&self,psp22_token1_deposit_amount:Balance) -> Balance {

           //init LP shares variable (shares to give to user)
           let mut shares:Balance = 0;
           
           //if its the caller first deposit 
           if self.total_supply == 0 {

               shares = 1000 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

               //Shares to give to provider
               shares = (psp22_token1_deposit_amount * self.total_supply) / self.get_psp22_token1_reserve();
             
           }

            shares
            
        }
 

        ///function to get the amount of PSP22 token2 given for 1 PSP22 token1
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token1(&self)-> Balance {
            


            //formula to calculate the price
            let amount_out = self.get_est_price_psp22_token1_to_psp22_token2(1u128 * 10u128.pow(12));

            return amount_out

        }

        ///function to get the amount of PSP22 token1 given for 1 PSP22 token2
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token2(&self)-> Balance {
            


            //formula to calculate the price
            let amount_out = self.get_est_price_psp22_token2_to_psp22_token1(1u128 * 10u128.pow(12));

            return amount_out

        }

        ///function to get the amount of PSP22 token2 the caller will get for given PSP22 token1 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token1_to_psp22_token2(&self, amount_in: Balance)-> Balance {


            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            //Init variable
            let mut amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

           //validating if user has more than 1000 PANX
           if user_current_balance >= 1000 * 10u128.pow(12){

            if self.fee  <= 1400000000000 {
                 amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) / 2));
            }

            if self.fee  > 1400000000000 {
                 amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) - 1));
            }
         }



            let numerator = amount_in_with_fees * self.get_psp22_token2_reserve();
            let deno = (self.get_psp22_token1_reserve() * 100) + amount_in_with_fees;
            let amount_out = numerator / deno;
            
            return amount_out                        

        }


        ///function to get the amount of PSP22 token1 the caller will get for given PSP22 token2 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token2_to_psp22_token1(&self, amount_in: Balance)-> Balance {

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            //Init variable
            let mut amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            //validating if user has more than 1000 PANX
            if user_current_balance >= 1000 * 10u128.pow(12){

               if self.fee <= 1400000000000 {
                    amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));
               }

               if self.fee > 1400000000000 {
                    amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));
               }
            }


            let numerator = amount_in_with_fees * self.get_psp22_token1_reserve();
            let deno = (self.get_psp22_token2_reserve() * 100) + amount_in_with_fees;
            let amount_out = numerator / deno;
            
            return amount_out                        

        }

        ///function to get the estimated price impact for given psp22 token1 amount
        #[ink(message)]
        pub fn get_price_impact_psp22_token1_to_psp22_token2(&self,amount_in: u128) -> Balance {
            

            //calc how much tokens to give to swapper
            let psp22_token2_amount_out = self.get_est_price_psp22_token1_to_psp22_token2(amount_in);

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            let numerator = amount_in_with_fees * (self.get_psp22_token2_reserve() - psp22_token2_amount_out );
            let deno = ((self.get_psp22_token1_reserve() + amount_in_with_fees ) * 100) + amount_in_with_fees;
            let amount_out = numerator / deno;
            
            return amount_out    

        }
        ///function to get the estimated price impact for given psp22 token2 amount
        #[ink(message,payable)]
        pub fn get_price_impact_psp22_token2_to_psp22_token1(&self,amount_in:Balance) -> Balance {
            
         
            //calc how much tokens to give to swapper
            let psp22_token1_amount_out = self.get_est_price_psp22_token2_to_psp22_token1(amount_in);

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            let numerator = amount_in_with_fees * (self.get_psp22_token1_reserve() - psp22_token1_amount_out );
            let deno = ((self.get_psp22_token2_reserve() + amount_in_with_fees ) * 100) + amount_in_with_fees;
            let amount_out = numerator / deno;
            
            return amount_out  


        }

        
        ///function to swap psp22 token1 to psp22 token2
        #[ink(message)]
        pub fn swap_psp22_token1(&mut self,psp22_token1_amount_to_swap: u128, amount_to_validate: u128,slippage: u128) {

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());
            //making sure user has more or equal to the amount he transfers.
            assert!(user_current_balance >= psp22_token1_amount_to_swap);

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            assert!(contract_allowance >= psp22_token1_amount_to_swap);
            
            //the amount of A0 to give to the caller.
            let amount_out = self.get_est_price_psp22_token1_to_psp22_token2(psp22_token1_amount_to_swap);


            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), psp22_token1_amount_to_swap, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");


            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            assert!(precentage_diff < slippage.try_into().unwrap());

            //fun to transfer PSP22 token2 to caller
            let _response_1 = PSP22Ref::transfer(&self.psp22_token2_address, self.env().caller(), amount_out, ink_prelude::vec![]);


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

        }


        ///function to swap psp22 token2 to psp22 token1
        #[ink(message,payable)]
        pub fn swap_psp22_token2(&mut self,psp22_token2_amount_to_swap: u128, amount_to_validate: u128,slippage: u128) {
            

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());
            //making sure user has more or equal to the amount he transfers.
            assert!(user_current_balance >= psp22_token2_amount_to_swap);

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            assert!(contract_allowance >= psp22_token2_amount_to_swap);


            //the amount of A0 to give to the caller.
            let amount_out = self.get_est_price_psp22_token2_to_psp22_token1(psp22_token2_amount_to_swap);


            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), psp22_token2_amount_to_swap, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");


            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            assert!(precentage_diff < slippage.try_into().unwrap());

            //fun to transfer PSP22 token2 to swapper
            let _response_1 = PSP22Ref::transfer(&self.psp22_token1_address, self.env().caller(), amount_out, ink_prelude::vec![]);


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            
        }



        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            self.fee
        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
        
            self.get_est_price_psp22_token1_to_psp22_token2(1 * 10u128.pow(12))    
        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }


        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(&self,of: AccountId) -> Balance {
            let of_balance = self.balances.get(&of).unwrap_or(0);

            of_balance
        }
        ///function to get contract PSP22 token2 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token2_reserve(&self) -> Balance {
            let balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());
            balance
        }
        ///function to get contract PSP22 token1 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token1_reserve(&self) -> Balance {
            
            let balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());
            balance
        }



    	#[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number

        }
        
        #[ink(message,payable)]
        pub fn check_diffrenece(&mut self,value1: u128,value2: u128) -> u128 {

            let abs_dif = value1.abs_diff(value2);

            let abs_dif_nominated = abs_dif * 10u128.pow(12);

            let diff = 100 * (abs_dif_nominated / ((value1+value2) / 2)) ;

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