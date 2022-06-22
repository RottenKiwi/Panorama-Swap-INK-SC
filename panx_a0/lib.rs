#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
extern crate chrono;
#[cfg(not(feature = "ink-as-dependency"))]



#[ink::contract]
pub mod panx_a0 {
    

    use chrono::prelude::*;
    use ink_storage::traits::SpreadAllocate;

    
    #[cfg(not(feature = "ink-as-dependency"))]
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
    pub struct PanxA0 {

        //Number of overall transactions (Not including LP provision)
        transasction_number: i64,
        //Deployer address
        manager: AccountId,
        //Panx contract address
        panx_psp22: AccountId,
        //A0 coin reserve
        a0_reserve:Balance,
        //PANX token reserve
        panx_reserve: Balance,
        //LP fee
        fee: Balance,
        //Total LP token supply
        total_supply: Balance,
        //LP token balances of LP providers
        balances: ink_storage::Mapping<AccountId, Balance>,
        //Logging trans into hashmap (Need to delete)
        transactions: ink_storage::Mapping<Balance, String>,
        //Logging trans into vec
        trans_log: Vec<String>,
        //Logging panx volume into vec
        panx_volume_log: Vec<String>,
        //Logging panx volume into vec
        a0_volume_log: Vec<String>,
        //Logging LP provision into vec
        lp_provision_log: Vec<String>,
        //Hashmap of LP providers
        lp_providers: ink_storage::Mapping<AccountId, Balance>,
    }

    impl PanxA0 {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(panx_contract:AccountId, fee: Balance) -> Self {
            
            let me = ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.panx_psp22 = panx_contract;  
                contract.manager = Self::env().caller();
               
            });
            
            me
            
        }


        //Fun to return the price for 1 PANX
	    #[ink(message)]
        pub fn get_price_for_1_panx(&self)-> Balance {
            
            //init U128 to use as operator.
            let opr:Balance = 10;
            //nominating 1 to base 12 (10^12)
            let value = 1 * opr.pow(12);
            //formula to calculate the price
            let amount_out = (self.env().balance() * value) / (self.get_panx_balance() + value);

            return amount_out

        }
        
        //fun to return the current PANX TO AO price including LP fee
        #[ink(message)]
        pub fn get_price_panx_to_a0_with_lp_fee(&self, value: Balance)-> Balance {

            //Reduct LP fee from the amount in
            let amount_in_with_fees =  (value * 997) / 1000;
            //calc how much amount to give for given value (price)
            let amount_out = (self.env().balance() * amount_in_with_fees) / (self.get_panx_balance() + amount_in_with_fees);

            return amount_out
            
            

        }

        //fun to return the current PANX TO AO price excluding LP fee 
        #[ink(message)]
        pub fn get_price_panx_to_a0(&self, value: Balance)-> Balance {

            //calc how much amount to give for given value (price)
            let amount_out = (self.env().balance() * value) / (self.get_panx_balance() + value);

            return amount_out                        

        }

        //fun to return the current A0 TO PANX price including LP fee 
        #[ink(message,payable)]
        pub fn get_price_a0_to_panx_with_lp_fee(&mut self) -> Balance { 
            

            //amount of A0 given by user
            let amount_in = self.env().transferred_value();
            //Reduct LP fee from the amount in
            let amount_in_with_feed =  (amount_in * 997) / 1000;
            //calc how much amount to give for given value (price)
            let amount_out = (self.get_panx_balance() * amount_in_with_feed) / (self.env().balance() + amount_in_with_feed);

            return amount_out
        }

        //fun to return the current A0 TO PANX price excluding LP fee
        #[ink(message,payable)]
        pub fn get_price_a0_to_panx(&mut self) -> Balance { 
            
            //amount of A0 given by user
            let amount_in = self.env().transferred_value();
            //calc how much amount to give for given value (price)
            let amount_out = (self.get_panx_balance() * amount_in) / (self.env().balance() + amount_in);
            
            return amount_out
        }
        //fun to get the price impact when swapping PANX
        #[ink(message)]
        pub fn get_price_impact_panx_to_a0(&self,value: u128) -> Balance {
            
            //Reduct LP fee from the amount in
            let amount_in_with_fees =  (value * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.env().balance()  * amount_in_with_fees) / (self.get_panx_balance() + amount_in_with_fees);
            //calc the price impact using the amount_out tokens
            let amount_out_with_impact = ((self.env().balance() - amount_out ) * amount_in_with_fees) / ((self.get_panx_balance() + value ) + amount_in_with_fees);
            
            amount_out_with_impact

        }


        //fun to get the price impact when swapping A0
        #[ink(message,payable)]
        pub fn get_price_impact_a0_to_panx(&self) -> Balance {
            
         
            //amount of A0 given by user
            let amount_in = self.env().transferred_value();
            //Reduct LP fee from the amount in
            let amount_in_with_feed =  (amount_in * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.get_panx_balance() * amount_in_with_feed) / (self.env().balance() + amount_in_with_feed);
            //calc the price impact using the amount_out tokens
            let amount_out_with_impact = ((self.get_panx_balance() - amount_out ) * amount_in_with_feed) / ((self.env().balance() + amount_in ) + amount_in_with_feed);
            
            amount_out_with_impact


        }



        


        //fun tp swap PANX to A0
        #[ink(message)]
        pub fn swap_panx_to_a0(&mut self,amount_to_transfer: u128, amount_to_validate: u128,slippage: u128) {
            


            //dont forget to approve() :)


            //cross contract call to PANX contract to transfer PANX to the SWAP contract
            let response = PSP22Ref::transfer_from_builder(&self.panx_psp22, self.env().caller(), Self::env().account_id(), amount_to_transfer, ink_prelude::vec![]).call_flags(CallFlags::default().set_allow_reentry(true)).fire();
            //Reduct LP fee from the amount in
            let amount_in_with_fees =  (amount_to_transfer * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.env().balance() * amount_in_with_fees) / (self.get_panx_balance() + amount_in_with_fees);
            //Validating slippage
            assert!(((amount_to_validate - amount_out) / 100) < slippage );
            //fun to transfer A0 to swapper
            if self.env().transfer(self.env().caller(), amount_out).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }


            let date_as_string = &self.get_current_timestamp_as_utc_string();

            //fetch price of one token to log swap trans
            let price_of_one_token = self.get_price_for_1_panx();
            //casting string on price of 1 token
            let mut str_to_log = price_of_one_token.to_string();
            //building a whole string of recorded trans
            str_to_log = str_to_log +", "+ date_as_string+";";
            //pushing log into logs vec
            self.trans_log.push(str_to_log.to_string());


            let mut panx_amount_to_log = amount_to_validate.to_string();
            //building a whole string of recorded trans
            panx_amount_to_log = panx_amount_to_log +", "+ date_as_string+", type: SELL;";
            //pushing log into logs vec
            self.panx_volume_log.push(panx_amount_to_log.to_string());


            let mut a0_amount_to_log = amount_out.to_string();
            //building a whole string of recorded trans
            a0_amount_to_log = a0_amount_to_log +", "+ date_as_string+", type: BUY;";
            //pushing log into logs vec
            self.a0_volume_log.push(a0_amount_to_log.to_string());

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

        }

        
        //fun to swap A0 to PANX
        #[ink(message,payable)]
        pub fn swap_a0_to_panx(&mut self,amount_to_validate: u128,slippage: u128) {
            

            //amount of transferred A0
            let amount_in = self.env().transferred_value();

            //Reduct LP fee from the amount in
            let amount_in_with_feed =  (amount_in * 997) / 1000;
            //calc how much tokens to give to swapper
            let amount_out = (self.get_panx_balance() * amount_in_with_feed) / (self.env().balance() + amount_in_with_feed);
            //Validating slippage
            assert!(((amount_to_validate - amount_out) / 100) < slippage );

            //cross contract call to PANX contract to transfer PANX to the swapper
            let response = PSP22Ref::transfer(&self.panx_psp22, self.env().caller(), amount_out, ink_prelude::vec![]);


            let date_as_string = &self.get_current_timestamp_as_utc_string();
            
            
            //fetch price of one token to log swap trans
            let price_of_one_token = self.get_price_for_1_panx();
            //casting string on price of 1 token
            let mut str_to_log = price_of_one_token.to_string();
            //building a whole string of recorded trans
            str_to_log = str_to_log +", "+date_as_string+";";
            //pushing log into logs vec
            self.trans_log.push(str_to_log.to_string());

            let mut panx_amount_to_log = amount_out.to_string();
            //building a whole string of recorded trans
            panx_amount_to_log = panx_amount_to_log +", "+ date_as_string+", type: BUY;";
            //pushing log into logs vec
            self.panx_volume_log.push(panx_amount_to_log.to_string());


            let mut a0_amount_to_log = amount_in.to_string();
            //building a whole string of recorded trans
            a0_amount_to_log = a0_amount_to_log +", "+ date_as_string+", type: SELL;";
            //pushing log into logs vec
            self.a0_volume_log.push(a0_amount_to_log.to_string());

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            
        }

        //fun to add LP to PANX/A0 pair
        #[ink(message,payable)]
        pub fn add_to_pool(&mut self,panx_deposit_amount:Balance)  {

            //A0 deposited amount
            let a0_deposit_amount = self.env().transferred_value();
            //cross contract call to PANX contract to transfer PANX to the SWAP contract
            PSP22Ref::transfer_from_builder(&self.panx_psp22, self.env().caller(), Self::env().account_id(), panx_deposit_amount, ink_prelude::vec![]).call_flags(ink_env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").expect("Transfer failed");
            


            let bal_a0 = self.get_a0_balance();
            
            let bal_panx = self.get_panx_balance();

            let d0 = a0_deposit_amount;

            let d1 = panx_deposit_amount;

            //init LP shares variable (shares to give to user)
            let mut shares:Balance = 0;

            //if its the caller first deposit 
            if self.total_supply > 0 {


                shares = self._min((d0 * self.total_supply) / self.get_a0_balance(), (d1 * self.total_supply) / self.get_panx_balance());
            }
            //if its not the first LP deposit
            else{
                shares = d0 * d1;
                shares = shares.sqrt();
            }

            //validating that share is more than 0
            assert!(shares > 0);

            //caller current shares (if any)
            let current_shares = self.get_lp_token_of(self.env().caller());
            //adding caller to LP providers
            self.lp_providers.insert(self.env().caller(), &(current_shares + shares));
            //increasing LP balance of caller (mint)
            self.balances.insert(self.env().caller(), &(current_shares + shares));
            //adding to over LP tokens (mint)
            self.total_supply += shares;


            let date_as_string = &self.get_current_timestamp_as_utc_string();
            
            
            //building a whole string of recorded trans
            let lp_log = "PANX: ".to_owned()+&panx_deposit_amount.to_string()+", AO: "+&a0_deposit_amount.to_string()+", share: "+&shares.to_string() +", date: "+ date_as_string+", type: ADD;";
            //pushing log into logs vec
            self.lp_provision_log.push(lp_log.to_string());
        }
    

        //fun to withdraw ALL LP and burn LP tokens
        #[ink(message,payable)]
        pub fn withdraw(&mut self)  {
           
            //caller address
            let caller = self.env().caller();
            //caller total LP shares
            let caller_shares = self.balances.get(&caller).unwrap_or(0);
            //calc amount of A0 to give 
            let amount_of_a0_to_give = (caller_shares * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_panx_to_give = (caller_shares * self.get_panx_balance()) / self.total_supply;

            
            //reducing caller LP token balance
            self.balances.insert(caller, &(caller_shares - caller_shares));
            //reducing over LP token supply (burn)
            self.total_supply -= caller_shares;

            //cross contract call to PANX contract to approve PANX to give to caller
            let response_1 = PSP22Ref::approve(&self.panx_psp22, caller,amount_of_panx_to_give);
            //cross contract call to PANX contract to transfer PANX to the caller
            let response_2 = PSP22Ref::transfer(&self.panx_psp22, caller, amount_of_panx_to_give, ink_prelude::vec![]);
            
            //fun to transfer A0 to the caller
            if self.env().transfer(self.env().caller(), amount_of_a0_to_give).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }


            let date_as_string = &self.get_current_timestamp_as_utc_string();
            
            
            //building a whole string of recorded trans
            let lp_log = "PANX: ".to_owned()+&amount_of_panx_to_give.to_string()+", AO: "+&amount_of_a0_to_give.to_string()+", share: "+&caller_shares.to_string() +", date: "+ date_as_string+", type: REMOVE;";
            //pushing log into logs vec
            self.lp_provision_log.push(lp_log.to_string());


        }

        //fun to withdraw SPECIFIC LP and burn LP tokens
        #[ink(message,payable)]
        pub fn withdraw_specific_amount(&mut self, share_amount: u128)  {
           

            //caller address
            let caller = self.env().caller();
            //caller total LP shares
            let caller_shares = self.balances.get(&caller).unwrap_or(0);

            //Checks if users has the amount of specific shares
            assert!(caller_shares >= share_amount);

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (share_amount * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_panx_to_give = (share_amount * self.get_panx_balance()) / self.total_supply;

            
            //reducing caller LP token balance
            self.balances.insert(caller, &(caller_shares - share_amount));
            //reducing over LP token supply (burn)
            self.total_supply -= share_amount;


            //cross contract call to PANX contract to approve PANX to give to caller
            let balance1 = PSP22Ref::approve(&self.panx_psp22, caller,amount_of_panx_to_give);
            //cross contract call to PANX contract to transfer PANX to the caller
            let res1 = PSP22Ref::transfer(&self.panx_psp22, caller, amount_of_panx_to_give, ink_prelude::vec![]);
            
            //fun to transfer A0 to the caller
            if self.env().transfer(self.env().caller(), amount_of_a0_to_give).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }

            let date_as_string = &self.get_current_timestamp_as_utc_string();
            
            
            //building a whole string of recorded trans
            let lp_log = "PANX: ".to_owned()+&amount_of_panx_to_give.to_string()+", AO: "+&amount_of_a0_to_give.to_string()+", share: "+&caller_shares.to_string() +", date: "+ date_as_string+", type: REMOVE;";
            //pushing log into logs vec
            self.lp_provision_log.push(lp_log.to_string());



        }

        

        //fun to get user tokens with given share_amount
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, share_amount: u128) -> (Balance,Balance) {

            //caller address
            let caller = self.env().caller();        
            //amount of shares given
            let caller_shares = share_amount;

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (share_amount * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_panx_to_give = (share_amount * self.get_panx_balance()) / self.total_supply;
        

            (amount_of_a0_to_give,amount_of_panx_to_give)
        
        }

        //fun to get expected shares amount
        #[ink(message,payable)]
        pub fn get_expected_lp_token_amount(&mut self,panx_deposit_amount:Balance) -> Balance {

            //A0 deposited amount
            let a0_deposit_amount = self.env().transferred_value();
           
            let bal_a0 = self.get_a0_balance();
            
            let bal_panx = self.get_panx_balance();

            let d0 = a0_deposit_amount;

            let d1 = panx_deposit_amount;

            //init LP shares variable (shares to give to user)
            let mut shares:Balance = 0;

            //if its the caller first deposit 
            if self.total_supply > 0 {


                shares = self._min((d0 * self.total_supply) / self.get_a0_balance(), (d1 * self.total_supply) / self.get_panx_balance());
            }
            //if its not the first LP deposit
            else{
                shares = d0 * d1;
                shares = shares.sqrt();
            }

            shares

            
        }

        //fun to get caller locked (pooled) tokens
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let user = account_id;
            //get account LP tokens 
            let user_shares = self.balances.get(&user).unwrap_or(0);

            //calc amount of A0 to give 
            let amount_of_a0_to_give = (user_shares * self.get_a0_balance()) / self.total_supply;
            //calc amount of PANX to give 
            let amount_of_panx_to_give = (user_shares * self.get_panx_balance()) / self.total_supply;

           
            (amount_of_panx_to_give,amount_of_a0_to_give)

            
        }


        //fun to get the bigger var
        #[ink(message)]
        pub fn _min(&self,val1:Balance,val2:Balance ) -> Balance {
            

            if val1 <= val2 {
                return val1
            }
            else{
                return val2
            }
        }

        //fun to get swap contract address
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }
        
        //fun to get price for 1 PANX
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
            
            let opr:Balance = 10;

            self.get_price_panx_to_a0(1 * opr.pow(12))

            
        }

        //fun to return the LP trans log
        #[ink(message)]
        pub fn get_lp_trans_log(&self) -> String {
            let mut str_to_send: String = "".to_string();
            
            for x in &self.lp_provision_log{

                str_to_send = str_to_send + x;
            }
            str_to_send.to_string()
        }

        //fun to return the trans log
        #[ink(message)]
        pub fn get_trans_log(&self) -> String {
            let mut str_to_send: String = "".to_string();
            
            for x in &self.trans_log{

                str_to_send = str_to_send + x;
            }
            str_to_send.to_string()
        }

        

        //fun to return the panx log
        #[ink(message)]
        pub fn get_panx_volume_log(&self) -> String {
            let mut str_to_send: String = "".to_string();
            
            for x in &self.panx_volume_log{

                str_to_send = str_to_send + x;
            }
            str_to_send.to_string()
        }

        //fun to return the A) log
        #[ink(message)]
        pub fn get_a0_volume_log(&self) -> String {
            let mut str_to_send: String = "".to_string();
            
            for x in &self.a0_volume_log{

                str_to_send = str_to_send + x;
            }
            str_to_send.to_string()
        }
        //fun to get total LP supply
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        //fun to get A0 reserve 
        #[ink(message)]
        pub fn get_a0_balance(&self) -> Balance {
            
            let amount = self.env().balance();
            amount
        }
        //fun to get LP tokens of specific address
        #[ink(message)]
        pub fn get_lp_token_of(&self,of: AccountId) -> Balance {
            let of_balance = self.balances.get(&of).unwrap_or(0);

            of_balance
        }
        //fun to get PANX reserve 
        #[ink(message)]
        pub fn get_panx_balance(&self) -> Balance {
            let balance1 = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());
            balance1
        }
        //fun to get current fee amount
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            let amount = self.fee;
            amount
        }
        //fun to get current timestamp in second
        #[ink(message)]
        pub fn get_current_timestamp(&self) -> u64 {
            let bts = self.env().block_timestamp() / 1000;
            bts
        }
        //fun to get current timestamp as UTC date
        #[ink(message)]
        pub fn get_current_timestamp_as_utc_string(&self) -> String {
            let bts = self.env().block_timestamp() / 1000;
            
            let naive_datetime = NaiveDateTime::from_timestamp(bts as i64, 0);
            let datetime_again: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
            datetime_again.to_string()
        }

    	#[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number

        }


 
    }
}