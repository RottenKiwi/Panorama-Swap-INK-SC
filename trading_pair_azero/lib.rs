#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;



pub use self::trading_pair_azero::{
	TradingPairAzero,
	TradingPairAzeroRef,
};


#[ink::contract]
pub mod trading_pair_azero {
    
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };
    use ink_env::CallFlags;

    
    
    #[ink(storage)]
    #[derive(SpreadAllocate)]
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
        balances: ink_storage::Mapping<AccountId, Balance>,
        //PANX contract address
        panx_contract: AccountId,
        //Store accounts LP tokens allowances
        lp_tokens_allowances: ink_storage::Mapping<(AccountId,AccountId), Balance>,
    }


    impl TradingPairAzero {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(psp22_contract:AccountId, fee: u128,panx_contract:AccountId) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.psp22_token = psp22_contract;  
                contract.fee = fee;
                contract.panx_contract = panx_contract;
               
            });
            
            me
            
        }

       ///function to add liquidity to PSP22/A0 pair.
       ///If its the first liquidity provision, the provider will always receive 1000 LP shares.
       ///We validate that the provider gets the correct amount of shares as displayed to him in front-end (add LP UI) and never gets 0 shares.
       #[ink(message,payable)]
       pub fn provide_to_pool(&mut self,a0_amount_in:Balance,psp22_deposit_amount:Balance,excpeted_lp_tokens:Balance,slippage:Balance)  {

           //fetching user current psp22 balance  
           let user_current_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, self.env().caller());

           //making sure user current balance is greater than the deposit amount.
           if user_current_balance < psp22_deposit_amount {
            panic!(
                 "Caller does not have enough PSP22 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22 tokens."
            )
            }

           let contract_allowance:Balance = PSP22Ref::allowance(&self.psp22_token, self.env().caller(),Self::env().account_id());
           //making sure trading pair contract has enough allowance.
           if contract_allowance < psp22_deposit_amount {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22 tokens."
            )
            }

           //init LP shares variable (shares to give to provider)
           let mut shares:Balance = 0;
           
           //if its the pool first deposit
           if self.total_supply == 0 {

               shares = 1000 * 10u128.pow(12);

           }

           //if its not the first LP deposit
           if self.total_supply > 0{


               //We need to sub the incoming amount of A0 by the current A0 reserve (current reserve includes incoming A0)
               let reserve_before_transaction:Balance = self.get_a0_balance() - a0_amount_in;
               //Shares to give to provider
               shares = (a0_amount_in * self.total_supply) / reserve_before_transaction;

             
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


           //cross contract call to psp22 contract to transfer psp22 token to the Pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), psp22_deposit_amount, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22 transferFrom cross contract call function, kindly re-adjust your deposited PSP22 tokens."
           )
           }
           
                       

           //caller current shares (if any)
           let current_shares:Balance = self.get_lp_token_of(self.env().caller());
           //increasing LP balance of caller (mint)
           self.balances.insert(self.env().caller(), &(current_shares + shares));
           //adding to over LP tokens (mint)
           self.total_supply += shares;



       }

       ///Function to withdraw specific amount of LP tokens given from the front-end.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, shares: Balance)  {
          
           //caller address 
           let caller = self.env().caller();
           //caller total LP shares
           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           //Validating that the caller has the given number of shares.
           if caller_shares < shares {
            panic!(
                 "Caller does not have enough liquidity pool SHARES to withdraw,
                  kindly lower the liquidity pool SHARES withdraw amount."
            )
            }

           //Amount of psp22 to give to the caller
           let psp22_amount_to_give:Balance = self.get_psp22_withdraw_tokens_amount(shares);
           //Amount of psp22 to give to the caller
           let a0_amount_to_give:Balance = self.get_a0_withdraw_tokens_amount(shares);
           
           //cross contract call to PANX contract to transfer PANX to the caller
           PSP22Ref::transfer(&self.psp22_token, caller, psp22_amount_to_give, ink_prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 tokens to caller : {:?}",
                error
            )
            });
           //fun to transfer A0 to the caller
           if self.env().transfer(self.env().caller(), a0_amount_to_give).is_err() {
               panic!(
                   "requested transfer failed. this can be the case if the contract does not\
                    have sufficient free funds or if the transfer would have brought the\
                    contract's balance below minimum balance."
               )
           }

           //reducing caller LP token balance
           self.balances.insert(caller, &(caller_shares - shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;


       }


        ///funtion to get amount of withdrable PSP22/A0 tokens by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, share_amount: Balance) -> (Balance,Balance) {



            //calc amount of A0 to give 
            let amount_of_a0_to_give:Balance = (share_amount * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_psp22_to_give:Balance = (share_amount * self.get_psp22_balance()) / self.total_supply;
        

            (amount_of_a0_to_give,amount_of_psp22_to_give)
        
        }


        ///function to get the amount of withdrawable psp22 tokens by given shares.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(&self, share_amount: Balance) -> Balance {

            //calc amount of PANX to give 
            let amount_of_psp22_to_give:Balance = (share_amount * self.get_psp22_balance()) / self.total_supply;

            amount_of_psp22_to_give
        
        }

        ///function to get the amount of withdrawable A0 by given LP shares.
        #[ink(message)]
        pub fn get_a0_withdraw_tokens_amount(&self, share_amount: Balance) -> Balance {

            //calc amount of A0 to give 
            let amount_of_a0_to_give:Balance = (share_amount * self.get_a0_balance()) / self.total_supply;
            
            amount_of_a0_to_give
        
        }

        
        ///function to get caller pooled PSP22 tokens and A0
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let user = account_id;
            //get account LP tokens 
            let user_shares:Balance = self.balances.get(&user).unwrap_or(0);

            //calc amount of A0 to give 
            let amount_of_a0_to_give:Balance = (user_shares * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_psp22_to_give:Balance = (user_shares * self.get_psp22_balance()) / self.total_supply;

           
            (amount_of_psp22_to_give,amount_of_a0_to_give)

            
        }

        //function to get expected amount of LP shares.
        #[ink(message)]
        pub fn get_expected_lp_token_amount(&self,a0_deposit_amount:Balance) -> Balance {

           //init LP shares variable (shares to give to user)
           let mut shares:Balance = 0;
           
           //if its the caller first deposit 
           if self.total_supply == 0 {

               shares = 1000 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

               //Shares to give to provider
               shares = (a0_deposit_amount * self.total_supply) / self.get_a0_balance();   

           }

            shares
            
        }
 

        ///function to get the amount of A0 given for 1 PSP22 token
	    #[ink(message)]
        pub fn get_price_for_one_psp22(&self)-> Balance {

            let one_token:Balance = 1*10u128.pow(12);

            let amount_out:Balance = self.get_est_price_psp22_to_a0(one_token);

            amount_out
        }

        ///function to get the amount of A0 the caller will get for given PSP22 amount
        #[ink(message)]
        pub fn get_est_price_psp22_to_a0(&self, amount_in:Balance)-> Balance {

            //fetching user current PSP22 balance
            let user_current_balance:Balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance = self.fee / 10u128.pow(12);

            //Init variable
            let mut amount_in_with_fees:Balance = amount_in * (100 - (actual_fee));

            let tokens_to_validate:Balance = 3500 * 10u128.pow(12);

            //validating if user has more than 3500 PANX
            if user_current_balance >= tokens_to_validate{

                if self.fee  <= 1400000000000 {
                     amount_in_with_fees = amount_in * (100 - ((actual_fee) / 2));
                }
 
                if self.fee  > 1400000000000 {
                     amount_in_with_fees = amount_in * (100 - ((actual_fee) - 1));
                }
             }


            let amount_out:Balance = (amount_in_with_fees * self.get_a0_balance()) / ((self.get_psp22_balance() * 100) + amount_in_with_fees);
            
            return amount_out                        

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (swap use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22_for_swap(&self,ao_amount_to_transfer:Balance) -> Balance { 

            //We need to calc the A0 reserve before swapping.
            let a0_reserve_before:Balance = self.get_a0_balance() - ao_amount_to_transfer;

            //fetching user current PSP22 balance
            let user_current_balance:Balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance = self.fee / 10u128.pow(12);

            //Init variable
            let mut amount_in_with_fees:Balance = ao_amount_to_transfer * (100 - (actual_fee));

            let tokens_to_validate:Balance = 3500 * 10u128.pow(12);

            //validating if user has more than 3500 PANX
            if user_current_balance >= tokens_to_validate{

                if self.fee  <= 1400000000000 {
                     amount_in_with_fees = ao_amount_to_transfer * (100 - ((actual_fee) / 2));
                }
 
                if self.fee  > 1400000000000 {
                     amount_in_with_fees = ao_amount_to_transfer * (100 - ((actual_fee) - 1));
                }
             }


            let amount_out:Balance = (amount_in_with_fees * self.get_psp22_balance()) / ((a0_reserve_before * 100) + amount_in_with_fees);

            return amount_out  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (front-end use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22(&self,ao_amount_to_transfer:Balance) -> Balance { 

            let actual_fee:Balance = self.fee / 10u128.pow(12);

            //calc the amount_in with the current fees to transfer to the LP providers.
            let amount_in_with_fees:Balance = ao_amount_to_transfer * (100 - (actual_fee));

            let amount_out:Balance = (amount_in_with_fees * self.get_psp22_balance()) / ((self.get_a0_balance() * 100) + amount_in_with_fees);
            
            return amount_out  

        }

        ///function to get the estimated price impact for given psp22 token amount
        #[ink(message)]
        pub fn get_price_impact_psp22_to_a0(&self,psp22_amount_in:Balance) -> Balance {

            let actual_fee:Balance = self.fee / 10u128.pow(12);

            let current_amount_out:Balance = self.get_est_price_psp22_to_a0(psp22_amount_in);
    
            //Reduct LP fee from the amount in
            let amount_in_with_fees:Balance = psp22_amount_in * (100 - (actual_fee));
    
            let future_amount_out:Balance = (amount_in_with_fees * (self.get_a0_balance() - current_amount_out)) / (((self.get_psp22_balance() + psp22_amount_in) * 100) + amount_in_with_fees);
            
            future_amount_out
    
        }
        ///function to get the estimated price impact for given A0 amount
        #[ink(message)]
        pub fn get_price_impact_a0_to_psp22(&mut self,a0_amount_in:Balance) -> Balance {
            
            let actual_fee:Balance = self.fee / 10u128.pow(12);

            let current_amount_out:Balance = self.get_est_price_a0_to_psp22(a0_amount_in);

            //calc the amount_in with the current fees to transfer to the LP providers.
            let amount_in_with_fees:Balance = a0_amount_in * (100 - (actual_fee));

            let future_amount_out:Balance = (amount_in_with_fees * (self.get_psp22_balance() - current_amount_out)) / (((self.get_a0_balance() + a0_amount_in)* 100) + amount_in_with_fees);

            future_amount_out


        }

        
        ///function to swap PSP22 Tokens to A0
        #[ink(message)]
        pub fn swap_psp22(&mut self,psp22_amount_to_transfer: Balance, a0_amount_to_validate: Balance,slippage: Balance) {

            //fetching user current PSP22 balance
            let user_current_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, self.env().caller());
            //making sure user has more or equal to the amount he transfers.
            if user_current_balance < psp22_amount_to_transfer {
                panic!(
                    "Caller balance is lower than the amount of PSP22 token he wishes to trasnfer,
                    kindly lower your deposited PSP22 tokens amount."
                )
            }
    

            let contract_allowance:Balance = PSP22Ref::allowance(&self.psp22_token, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            if contract_allowance < psp22_amount_to_transfer {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22 tokens before swapping."
                )
            }
            
            //the amount of A0 to give to the caller.
            let a0_amount_out:Balance = self.get_est_price_psp22_to_a0(psp22_amount_to_transfer);

           //cross contract call to psp22 contract to transfer psp22 token to the Pair contract
           if PSP22Ref::transfer_from_builder(&self.psp22_token, self.env().caller(), Self::env().account_id(), psp22_amount_to_transfer, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22 transferFrom cross contract call function, kindly re-adjust your deposited PSP22 tokens."
           )
           }

            //precentage dif between given A0 amount (from front-end) and acutal final AO amount
            let precentage_diff:Balance = self.check_diffrenece(a0_amount_to_validate,a0_amount_out);

            //Validating slippage
            if precentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            //function to transfer A0 to the caller.
            if self.env().transfer(self.env().caller(), a0_amount_out).is_err() {
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
        pub fn swap_a0(&mut self,a0_amount_in:Balance,psp22_amount_to_validate: Balance,slippage: Balance) {
            
            //amount of PSP22 tokens to give to caller.
            let psp22_amount_out:Balance = self.get_est_price_a0_to_psp22_for_swap(a0_amount_in);

            //precentage dif between given PSP22 amount (from front-end) and acutal final PSP22 amount
            let precentage_diff:Balance = self.check_diffrenece(psp22_amount_to_validate,psp22_amount_out);

            //Validating slippage
            if precentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }
            //cross contract call to PSP22 contract to transfer PSP22 to the swapper
            PSP22Ref::transfer(&self.psp22_token, self.env().caller(), psp22_amount_out, ink_prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 tokens to caller : {:?}",
                    error
                )
            });


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            
            
        }


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

            self.balances.insert(caller, &(caller_shares - shares_to_transfer));
 
            self.balances.insert(recipient, &(recipient_shares + shares_to_transfer));


        }

        #[ink(message)]
        pub fn approve_lp_tokens(&mut self, spender:AccountId,shares_to_approve: Balance)  {

           let caller = self.env().caller();

           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           if caller_shares < shares_to_approve {
            panic!(
                "Cannot approve LP tokens, owner LP token balance is lower than the given shares to approve."
            )
           }

           self.lp_tokens_allowances.insert((caller,spender), &(shares_to_approve));

            
         
        }

        #[ink(message)]
        pub fn transfer_lp_tokens_from_to(&mut self,owner:AccountId,spender:AccountId,shares_to_transfer: Balance)  {


           let owner_shares:Balance = self.balances.get(&owner).unwrap_or(0);

           let spender_shares:Balance = self.balances.get(&spender).unwrap_or(0);

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

           self.balances.insert(owner, &(owner_shares - shares_to_transfer));

           self.lp_tokens_allowances.insert((owner,spender), &(allowance - shares_to_transfer));

           self.balances.insert(spender, &(spender_shares + shares_to_transfer));
    
         
        }
         
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

            let one_token:Balance = 1 * 10u128.pow(12);
        
            self.get_est_price_psp22_to_a0(one_token)    
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
        
        #[ink(message,payable)]
        pub fn check_diffrenece(&mut self,value1: Balance,value2: Balance) -> Balance {

            let absolute_difference:Balance = value1.abs_diff(value2);

            let absolute_difference_nominated:Balance = absolute_difference * 10u128.pow(12);

            let percentage_difference:Balance = 100 * (absolute_difference_nominated / ((value1+value2) / 2));

            percentage_difference 

      
            
        }

 
    }
}