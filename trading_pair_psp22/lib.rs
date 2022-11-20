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

           let mut psp22_token1_deposit = psp22_token1_deposit_amount;

           let mut psp22_token2_deposit = psp22_token2_deposit_amount;

           //init LP shares variable (shares to give to provider)

           let user_current_balance_token1 = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());

           if user_current_balance_token1 < psp22_token1_deposit {
            panic!(
                 "Caller does not have enough PSP22_1 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22_1 tokens."
            )
            }

           let user_current_balance_token2 = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());

           if user_current_balance_token2 < psp22_token2_deposit {
            panic!(
                 "Caller does not have enough PSP22_2 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22_2 tokens."
            )
            }

            //psp22 token1 starting balance
            let contract_token1_starting_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());
            
            //psp22 token2 starting balance
            let contract_token2_starting_balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());

           let contract_token1_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());
           //making sure trading pair contract has enough allowance.
           if contract_token1_allowance < psp22_token1_deposit {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22_1 tokens."
            )
            }

           let contract_token2_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());
           //making sure trading pair contract has enough allowance.
           if contract_token2_allowance < psp22_token2_deposit {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22_2 tokens."
            )
            }

           //cross contract call to psp22 token1 contract to transfer psp22 token1 to the Pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), psp22_token1_deposit, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22_1 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_1 tokens amount."
           )
           }
           //cross contract call to psp22 token2 contract to transfer psp22 token2 to the Pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), psp22_token2_deposit, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22_2 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_2 tokens amount."
           )
           }

            //psp22 token1 closing balance
           let contract_token1_closing_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());

            //psp22 token2 closing balance
            let contract_token2_closing_balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());

            //get psp22 token1 deposited amount (considering tokens with internal tax)
            psp22_token1_deposit = contract_token1_closing_balance - contract_token1_starting_balance;

            //get psp22 token2 deposited amount (considering tokens with internal tax)
            psp22_token2_deposit = contract_token2_closing_balance - contract_token2_starting_balance;

           let mut shares:Balance = 0;
          
           //if its the pool first deposit
           if self.total_supply == 0 {
            
            shares = 1000 * 10u128.pow(12);

           }

           //if its not the first LP deposit
           if self.total_supply > 0{

            //Shares to give to provider
            shares = (psp22_token1_deposit * self.total_supply) / self.get_psp22_token1_reserve();

             
           }

           //validating that shares is greater than 0
           if shares <= 0 {
            panic!(
                 "Expected given liquidity pool SHARES are equal to 0,
                 cannot proceed with liquidity pool provision."
            )
            }

           //function to return the precenrage diff between the expected lp token that was shown in the front-end and the final shares amount.
           let precentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares);

           //Validating slippage
           if precentage_diff > slippage.try_into().unwrap() {
            panic!(
                "The percentage difference is bigger than the given slippage,
                kindly re-adjust the slippage settings."
            )
            }
          

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
           let caller_shares = self.balances.get(&caller).unwrap_or(0);

           //Validating that the caller has the given number of shares.
           if caller_shares < shares {
            panic!(
                 "Caller does not have enough liquidity pool SHARES to withdraw,
                  kindly lower the liquidity pool SHARES withdraw amount."
            )
            }

           //Amount of psp22 token1 to give to the caller
           let psp22_token1_amount_to_give = self.get_psp22_token1_withdraw_tokens_amount(shares);
           //Amount of psp22 token1 to give to the caller
           let psp22_token2_amount_to_give = self.get_psp22_token2_withdraw_tokens_amount(shares);
           
           //cross contract call to PSP22 token1 contract to transfer PSP22 token1 to the caller
           if PSP22Ref::transfer(&self.psp22_token1_address, caller, psp22_token1_amount_to_give, ink_prelude::vec![]).is_err() {
            panic!(
                "Error in PSP22_1 transfer cross contract call function, kindly re-adjust withdraw shares amount."
            )
            }
           
           //cross contract call to PSP22 token2 contract to transfer PSP22 token2 to the caller
           if PSP22Ref::transfer(&self.psp22_token2_address, caller, psp22_token2_amount_to_give, ink_prelude::vec![]).is_err() {
            panic!(
                "Error in PSP22_2 transfer cross contract call function, kindly re-adjust withdraw shares amount."
            )
            }

           //reducing caller LP token balance_caller_shares
           self.balances.insert(caller, &(caller_shares - shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;



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
        pub fn get_expected_lp_token_amount(&self,psp22_token1_deposit:Balance) -> Balance {

           //init LP shares variable (shares to give to user)
           let mut shares:Balance = 0;
           
           //if its the caller first deposit 
           if self.total_supply == 0 {

               shares = 1000 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

               //Shares to give to provider
               shares = (psp22_token1_deposit * self.total_supply) / self.get_psp22_token1_reserve();
             
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

           //validating if user has more than 3500 PANX
           if user_current_balance >= 3500 * 10u128.pow(12){

            if self.fee  <= 1400000000000 {
                 amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) / 2));
            }

            if self.fee  > 1400000000000 {
                 amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) - 1));
            }
         }


            let amount_out = (amount_in_with_fees * self.get_psp22_token2_reserve()) / ((self.get_psp22_token1_reserve() * 100) + amount_in_with_fees);
            
            return amount_out                        

        }


        ///function to get the amount of PSP22 token1 the caller will get for given PSP22 token2 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token2_to_psp22_token1(&self, amount_in: Balance)-> Balance {

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            //Init variable
            let mut amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            //validating if user has more than 3500 PANX
            if user_current_balance >= 3500 * 10u128.pow(12){

                if self.fee  <= 1400000000000 {
                     amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) / 2));
                }
 
                if self.fee  > 1400000000000 {
                     amount_in_with_fees = amount_in * (100 - ((self.fee / 10u128.pow(12)) - 1));
                }
             }


            let amount_out = (amount_in_with_fees * self.get_psp22_token1_reserve()) / ((self.get_psp22_token2_reserve() * 100) + amount_in_with_fees);
            
            return amount_out                        

        }

        ///function to get the estimated price impact for given psp22 token1 amount
        #[ink(message)]
        pub fn get_price_impact_psp22_token1_to_psp22_token2(&self,amount_in: u128) -> Balance {
            

            //calc how much tokens to give to swapper
            let psp22_token2_amount_out = self.get_est_price_psp22_token1_to_psp22_token2(amount_in);

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            let amount_out = (amount_in_with_fees * (self.get_psp22_token2_reserve() - psp22_token2_amount_out)) / (((self.get_psp22_token1_reserve() + amount_in_with_fees ) * 100) + amount_in_with_fees);

            
            return amount_out    

        }
        ///function to get the estimated price impact for given psp22 token2 amount
        #[ink(message,payable)]
        pub fn get_price_impact_psp22_token2_to_psp22_token1(&self,amount_in:Balance) -> Balance {
            
         
            //calc how much tokens to give to swapper
            let psp22_token1_amount_out = self.get_est_price_psp22_token2_to_psp22_token1(amount_in);

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_fees = amount_in * (100 - (self.fee / 10u128.pow(12)));

            let amount_out = (amount_in_with_fees * (self.get_psp22_token1_reserve() - psp22_token1_amount_out)) / (((self.get_psp22_token2_reserve() + amount_in_with_fees ) * 100) + amount_in_with_fees);
            
            return amount_out  


        }

        
        ///function to swap psp22 token1 to psp22 token2
        #[ink(message)]
        pub fn swap_psp22_token1(&mut self,psp22_token1_amount_to_swap: u128, amount_to_validate: u128,slippage: u128) {

            let mut token1_amount_to_swap = psp22_token1_amount_to_swap;

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());
            //making sure user has more or equal to the amount he transfers.
            if user_current_balance < token1_amount_to_swap {
                panic!(
                    "Caller balance is lower than the amount of PSP22_1 token he wishes to trasnfer,
                    kindly lower your deposited PSP22_1 tokens amount."
                )
            }

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            if contract_allowance < token1_amount_to_swap {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22_1 tokens before swapping."
                )
            }

            //get balance before transfer
            let contract_starting_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());

            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), token1_amount_to_swap, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
                panic!(
                    "Error in PSP22_1 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_1 tokens."
               )
               
            }

            //get balance after transfer
            let contract_closing_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());

            //get actual deposit (considering tokens with internal txn fees)
            token1_amount_to_swap = contract_closing_balance - contract_starting_balance;

            //the amount of A0 to give to the caller.
            let amount_out = self.get_est_price_psp22_token1_to_psp22_token2(token1_amount_to_swap);

            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            if precentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            //fun to transfer PSP22 token2 to caller
            if PSP22Ref::transfer(&self.psp22_token2_address, self.env().caller(), amount_out, ink_prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_2 transfer cross contract call function, kindly re-adjust PSP22_1 deposit amount."
                )
            }


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

        }


        ///function to swap psp22 token2 to psp22 token1
        #[ink(message,payable)]
        pub fn swap_psp22_token2(&mut self,psp22_token2_amount_to_swap: u128, amount_to_validate: u128,slippage: u128) {
            
            let mut token2_amount_to_swap = psp22_token2_amount_to_swap;

            //fetching user current PSP22 balance
            let user_current_balance = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());
            //making sure user has more or equal to the amount he transfers.
            if user_current_balance < token2_amount_to_swap {
                panic!(
                    "Caller balance is lower than the amount of PSP22_2 token he wishes to trasnfer,
                    kindly lower your deposited PSP22_2 tokens amount."
                )
            }

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            if contract_allowance < token2_amount_to_swap {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22_2 tokens before swapping."
                )
            }

            //get balance before transfer
            let contract_starting_balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());

            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), token2_amount_to_swap, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
                panic!(
                    "Error in PSP22_2 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_2 tokens."
               )
            }

            //get balance after transfer
            let contract_closing_balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());

            //get actual deposit (considering tokens with internal txn fees)
            token2_amount_to_swap = contract_closing_balance - contract_starting_balance;

            //the amount of A0 to give to the caller.
            let amount_out = self.get_est_price_psp22_token2_to_psp22_token1(token2_amount_to_swap);

            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff = self.check_diffrenece(amount_to_validate,amount_out);

            //Validating slippage
            if precentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            //function to transfer PSP22 token2 to swapper
            if PSP22Ref::transfer(&self.psp22_token1_address, self.env().caller(), amount_out, ink_prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_1 transfer cross contract call function, kindly re-adjust PSP22_2 deposit amount."
                )
            }

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
        pub fn get_lp_token_of(&self,account: AccountId) -> Balance {
            let account_balance = self.balances.get(&account).unwrap_or(0);
            account_balance
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
 
    }
}
