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
    use ink::env::CallFlags;
    use ink::prelude::vec;

    
    #[ink(storage)]
    pub struct TradingPairAzero {

        transasction_number: i64,
        psp22_token: AccountId,
        fee: Balance,
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        panx_contract: AccountId,
        lp_tokens_allowances: Mapping<(AccountId,AccountId), Balance>,
        vault: AccountId,
        traders_fee:Balance


    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum TradingPairErrors {
        CallerInsufficientPSP22Balance,
        NotEnoughAllowance,
        Overflow,
        ZeroSharesGiven,
        SlippageTolerance,
        PSP22TransferFromFailed,
        PSP22TransferFailed,
        A0TransferFailed,
        CallerInsufficientLPBalance,
        ContractOutOfA0,
        ContractOutOfPSP22,
        NotEnoughOwnerLPAllowance

    }

    #[ink(event)]
    pub struct LiquidityPoolProvision {
        provider:AccountId,
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
        psp22_given_amount:Balance,
        psp22_given_to_vault:Balance,

    }

    #[ink(event)]
    pub struct PSP22Swap {
        caller:AccountId,
        psp22_deposited_amount:Balance,
        a0_given_amount:Balance,
        a0_given_to_vault:Balance
  
    }


    impl TradingPairAzero {
        #[ink(constructor)]
        pub fn new(
            psp22_contract:AccountId,
            fee: Balance,panx_contract:AccountId,
            vault:AccountId
        ) -> Self {


            let transasction_number:i64 = 0;
            let balances = Mapping::default();
            let lp_tokens_allowances = Mapping::default();
            let psp22_token = psp22_contract;
            let total_supply:Balance = 0;
            let traders_fee:Balance = 25;

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
       pub fn provide_to_pool(
            &mut self,
            psp22_deposit_amount:Balance,
            excpeted_lp_tokens:Balance,
            slippage:Balance
        )   -> Result<(), TradingPairErrors> {

            let caller = self.env().caller();

            //fetching caller current psp22 balance  
            let caller_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token,
                caller
            );

            //making sure that caller current PSP22 balance is greater than the deposit amount.
            if caller_current_balance < psp22_deposit_amount {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            } 

            let contract_allowance:Balance = PSP22Ref::allowance(
                &self.psp22_token,
                caller,
                Self::env().account_id()
            );


            //making sure that trading pair contract has enough allowance.
            if contract_allowance < psp22_deposit_amount {
                return Err(TradingPairErrors::NotEnoughAllowance);
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
                        return Err(TradingPairErrors::Overflow);
                    }
                };
             
            }

            //validating that shares is greater than 0
            if shares <= 0 {
                return Err(TradingPairErrors::ZeroSharesGiven);
            }

            //function to return the percentage diff between the expected lp token
            //that was shown in the front-end and the final shares amount.
            let percentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares).unwrap();

            //validating slippage    
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance);
            }


            let current_shares:Balance = self.get_lp_token_of(caller);

            let new_caller_shares:Balance;

            //calculating the current caller shares with the new provided shares.
            match current_shares.checked_add(shares) {
                Some(result) => {
                    new_caller_shares = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            //cross contract call to psp22 contract to transfer psp22 token to the pair contract
            if PSP22Ref::transfer_from_builder(&self.psp22_token,caller,Self::env().account_id(),psp22_deposit_amount,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP22TransferFromFailed);
                    }

            let caller_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &self.psp22_token,
                caller
            );

            if caller_current_balance == caller_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            }
                
                    
            //increasing LP balance of caller (mint)
            self.balances.insert(caller, &(new_caller_shares));
            //adding to over LP tokens (mint)
            self.total_supply += shares;

            Self::env().emit_event(LiquidityPoolProvision{
                provider:caller,
                a0_deposited_amount:self.env().transferred_value(),
                psp22_deposited_amount:psp22_deposit_amount,
                shares_given:shares
            });

            Ok(())



       }

       ///function to withdraw specific amount of LP share tokens.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(
            &mut self,
            shares: Balance
        )   -> Result<(), TradingPairErrors>  {

            //throw error is the caller tries to withdraw 0 LP shares
            if shares <= 0 {
                return Err(TradingPairErrors::ZeroSharesGiven);
            }
          
            //caller address 
            let caller = self.env().caller();

            //caller total LP shares
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            //validating that the caller has more than the given number of shares.
            if caller_shares < shares {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
            }

            //amount of PSP22 to give to the caller
            let psp22_amount_to_give = self.get_psp22_withdraw_tokens_amount(shares).unwrap();

            //amount of A0 to give to the caller
            let a0_amount_to_give = self.get_a0_withdraw_tokens_amount(shares).unwrap();

            let new_caller_lp_shares:Balance;

            //calculating the current caller shares with the new provided shares.
            match caller_shares.checked_sub(shares) {
                Some(result) => {
                    new_caller_lp_shares = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
           
            //cross contract call to PSP22 contract to transfer PSP2 to the caller.
            if PSP22Ref::transfer(&self.psp22_token,caller,psp22_amount_to_give,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }

            //function to transfer A0 to the caller
            if self.env().transfer(caller, a0_amount_to_give).is_err() {
                return Err(TradingPairErrors::A0TransferFailed);
            }

            //reducing caller LP token balance
            self.balances.insert(caller, &(new_caller_lp_shares));
            //reducing over LP token supply (burn)
            self.total_supply -= shares;

            Self::env().emit_event(LiquidityPoolWithdrawal{
                caller:caller,
                shares_given:shares,
                a0_given_amount:a0_amount_to_give,
                psp22_given_amount:psp22_amount_to_give,
                new_shares_balance:new_caller_lp_shares
            });

            Ok(())



       }


        ///funtion to get the amount of withdrawable PSP22 and A0 by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(
            &self,
            shares_amount: Balance
        ) -> Result<(Balance,Balance), TradingPairErrors> {

            let amount_of_a0_to_give:Balance;

            //calculating the amount of A0 to give to the caller.
            match (shares_amount * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let amount_of_psp22_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
        
            Ok((amount_of_a0_to_give,amount_of_psp22_to_give))


        
        }


        ///funtion to get the amount of withdrawable PSP22 by given number of LP shares.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(
            &self,
            shares_amount: Balance
        ) -> Result<Balance, TradingPairErrors> {

            let amount_of_psp22_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(amount_of_psp22_to_give)
        
        }

        ///funtion to get the amount of withdrawable A0 by given number of LP shares.
        #[ink(message)]
        pub fn get_a0_withdraw_tokens_amount(
            &self,
            shares_amount: Balance
        ) -> Result<Balance, TradingPairErrors> {


            let amount_of_a0_to_give:Balance;

            //calculating the amount of A0 to give to the caller.
            match (shares_amount * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(amount_of_a0_to_give)
        
        }

        
        ///function to get the callers pooled PSP22 and A0.
        #[ink(message)]
        pub fn get_account_locked_tokens(
            &self,
            account_id:AccountId
        ) -> Result<(Balance,Balance), TradingPairErrors> {
           
            //account address
            let caller = account_id;
            //get account LP tokens 
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let mut amount_of_a0_to_give:Balance = 0;

            let mut amount_of_psp22_to_give:Balance = 0;


            if caller_shares <= 0 {

                return Ok((amount_of_psp22_to_give,amount_of_a0_to_give))
                 
            }

            
            //calculating the amount of A0 to give to the caller.
            match (caller_shares * self.get_a0_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the amount of PSP22 to give to the caller.
            match (caller_shares * self.get_psp22_balance()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
           
            Ok((amount_of_psp22_to_give,amount_of_a0_to_give))

            
        }

        //function to get the expected amount of LP shares by given A0 amount.
        #[ink(message)]
        pub fn get_expected_lp_token_amount(
            &self,
            a0_deposit_amount:Balance
        ) -> Result<Balance, TradingPairErrors> {


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
                    return Err(TradingPairErrors::Overflow);
                }
                };

            }

            Ok(shares)
            
        }
 

        ///function to get the amount of A0 the caller will get for 1 PSP22 token.
        #[ink(message)]
        pub fn get_price_for_one_psp22(
            &self
        ) -> Result<Balance, TradingPairErrors> {

            let amount_out = self.get_est_price_psp22_to_a0(1u128 * (10u128.pow(12))).unwrap();

            Ok(amount_out)
        }

        ///function to get the amount of A0 the caller will get for given PSP22 amount.
        #[ink(message)]
        pub fn get_est_price_psp22_to_a0(
            &self,
            psp22_amount_in:Balance
        ) -> Result<Balance, TradingPairErrors> {

            let caller = self.env().caller();

            //fetching caller current PSP22 balance
            let caller_current_balance:Balance = PSP22Ref::balance_of(
                &self.panx_contract,
                caller
            );

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the PSP22 amount in
            match psp22_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
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
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(a0_amount_out)  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (swap use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22_for_swap(
            &self,
            a0_amout_in:Balance
        ) -> Result<Balance, TradingPairErrors> { 

            let caller = self.env().caller();

            let a0_reserve_before:Balance;

            //calculating the A0 contract reserve before the transaction
            match self.get_a0_balance().checked_sub(a0_amout_in) {
                Some(result) => {
                    a0_reserve_before = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let caller_current_balance:Balance = PSP22Ref::balance_of(
                &self.panx_contract,
                caller
            );

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amout_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
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
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(amount_out)  

        }

        ///function to get the amount of PSP22 the caller will get for given A0 amount (front-end use)
        #[ink(message,payable)]
        pub fn get_est_price_a0_to_psp22(
            &self,
            a0_amout_in:Balance
        ) -> Result<Balance, TradingPairErrors> {

            let caller = self.env().caller();

            let caller_current_balance:Balance = PSP22Ref::balance_of(
                &self.panx_contract,
                caller
            );

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amout_in.checked_mul(100u128 - actual_fee){
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
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
                            return Err(TradingPairErrors::Overflow);
                        }
                    };
                }
            }

            let amount_out:Balance;

            //calculating the final PSP22 amount to transfer to the caller
            match (amount_in_with_lp_fees * self.get_psp22_balance()).checked_div((self.get_a0_balance() * 100u128) + amount_in_with_lp_fees){
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(amount_out)  

        }

        ///function to get the estimated price impact for given psp22 token amount
        #[ink(message)]
        pub fn get_price_impact_psp22_to_a0(
            &self,
            psp22_amount_in:Balance
        ) -> Result<Balance, TradingPairErrors> {

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //fetching the amount of A0 the caller WOULD get if he would swap
            let current_amount_out = self.get_est_price_psp22_to_a0(psp22_amount_in).unwrap();

            let amount_in_with_fees:Balance;

            //reducting the LP fee from the PSP22 amount in
            match psp22_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
    
            let future_ao_amount_out:Balance;

            //calculating the final future A0 amount to transfer to the caller.
            match (amount_in_with_fees * (self.get_a0_balance() - current_amount_out)).checked_div(((self.get_psp22_balance() + psp22_amount_in) * 100) + amount_in_with_fees) {
                Some(result) => {
                    future_ao_amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(future_ao_amount_out)
    
        }
        
        ///function to get the estimated price impact for given A0 amount
        #[ink(message)]
        pub fn get_price_impact_a0_to_psp22(
            &mut self,
            a0_amount_in:Balance
        ) -> Result<Balance, TradingPairErrors> {
            
            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let current_amount_out = self.get_est_price_a0_to_psp22(a0_amount_in).unwrap();

            let amount_in_with_fees:Balance;

            //reducting the LP fee from the A0 amount in
            match a0_amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let future_psp22_amount_out:Balance;

            //calculating the final future PSP22 amount to transfer to the caller.
            match (amount_in_with_fees * (self.get_psp22_balance() - current_amount_out)).checked_div(((self.get_a0_balance() + a0_amount_in)* 100) + amount_in_with_fees) {
                Some(result) => {
                    future_psp22_amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(future_psp22_amount_out)


        }

        
        ///function to swap PSP22 to A0
        #[ink(message)]
        pub fn swap_psp22(
            &mut self,
            psp22_amount_to_transfer: Balance,
            a0_amount_to_validate: Balance,
            slippage: Balance
        ) -> Result<(), TradingPairErrors> {

            let caller = self.env().caller();

            let contract_a0_current_balance = self.get_a0_balance();

            //making sure that the contract has more than 0 A0 coins.
            if contract_a0_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfA0);
            }

            let contract_psp22_current_balance:Balance = self.get_psp22_balance();

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP22);
            }

            let caller_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token,
                caller
            );

            //making sure that the caller has more or equal the amount he wishes to transfers.
            if caller_current_balance < psp22_amount_to_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            }
            
            let contract_allowance:Balance = PSP22Ref::allowance(
                &self.psp22_token,
                caller,
                Self::env().account_id()
            );
            
            //making sure that the trading pair contract has enough allowance.
            if contract_allowance < psp22_amount_to_transfer {
                return Err(TradingPairErrors::NotEnoughAllowance);
            }
            
            //the amount of A0 to give to the caller before traders fee.
            let a0_amount_out_for_caller_before_traders_fee:Balance = self.get_est_price_psp22_to_a0(
                psp22_amount_to_transfer)
                .unwrap();

            //percentage dif between given A0 amount (from front-end) and acutal final AO amount
            let percentage_diff:Balance = self.check_diffrenece(
                a0_amount_to_validate,
                a0_amount_out_for_caller_before_traders_fee)
                .unwrap();

            //validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance);
            }

            let actual_a0_amount_out_for_caller:Balance;

            let a0_amount_out_for_vault:Balance;

            //calculating the amount of A0 coins to allocate to the vault account
            match  (a0_amount_out_for_caller_before_traders_fee * self.traders_fee).checked_div(1000u128)  {
                Some(result) => {
                    a0_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the final amount of A0 coins to give to the caller after reducing traders fee
            match  a0_amount_out_for_caller_before_traders_fee.checked_sub(a0_amount_out_for_vault) {
                Some(result) => {
                    actual_a0_amount_out_for_caller = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let psp22_amount_out_for_vault:Balance;

            //calculating the amount of PSP22 tokens to allocate to the vault account
            match  (psp22_amount_to_transfer * self.traders_fee).checked_div(1000u128)  {
                Some(result) => {
                    psp22_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //cross contract call to psp22 contract to transfer psp22 token to the Pair contract
            if PSP22Ref::transfer_from_builder(&self.psp22_token,caller,Self::env().account_id(),psp22_amount_to_transfer,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP22TransferFromFailed);
            }

            let caller_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &self.psp22_token,
                caller
            );

            if caller_current_balance == caller_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            }

            //cross contract call to PSP22 contract to transfer PSP22 to the vault
            if PSP22Ref::transfer(&self.psp22_token,self.vault,psp22_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }


            //function to transfer A0 to the caller.
            if self.env().transfer(
                caller,
                actual_a0_amount_out_for_caller)
                .is_err() {
                    return Err(TradingPairErrors::A0TransferFailed);
            }

            //function to transfer A0 to the vault.
            if self.env().transfer(
                self.vault,
                a0_amount_out_for_vault)
                .is_err() {
                    return Err(TradingPairErrors::A0TransferFailed);
            }

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            Self::env().emit_event(PSP22Swap{
                caller:caller,
                psp22_deposited_amount:psp22_amount_to_transfer,
                a0_given_amount:actual_a0_amount_out_for_caller,
                a0_given_to_vault:a0_amount_out_for_vault
            });

            Ok(())


        }


        ///function to swap A0 to PSP22
        #[ink(message,payable)]
        pub fn swap_a0(
            &mut self,
            psp22_amount_to_validate: Balance,
            slippage: Balance
        ) -> Result<(), TradingPairErrors> {

            let caller = self.env().caller();

            let contract_a0_current_balance = self.get_a0_balance();

            //making sure that the contract has more than 0 A0 coins.
            if contract_a0_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfA0);
            }

            let contract_psp22_current_balance:Balance = self.get_psp22_balance();

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP22);
            }
            
            //amount of PSP22 tokens to give to caller before traders fee.
            let psp22_amount_out_for_caller_before_traders_fee:Balance = self.get_est_price_a0_to_psp22_for_swap(
                self.env().transferred_value())
                .unwrap();

            //percentage dif between given PSP22 amount (from front-end) and the acutal final PSP22 amount.
            let percentage_diff:Balance = self.check_diffrenece(
                psp22_amount_to_validate,
                psp22_amount_out_for_caller_before_traders_fee)
                .unwrap();

            //validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance);
            }


            let psp22_amount_out_for_vault:Balance;
            
            let actual_psp22_amount_out_for_caller:Balance;

            //calculating the amount of PSP22 tokens to allocate to the vault account
            match (psp22_amount_out_for_caller_before_traders_fee * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    psp22_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            //calculating the final amount of PSP22 tokens to give to the caller after reducing traders fee
            match psp22_amount_out_for_caller_before_traders_fee.checked_sub(psp22_amount_out_for_vault) {
                Some(result) => {
                    actual_psp22_amount_out_for_caller = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let a0_amount_out_for_vault:Balance;

            //calculating the amount of A0 coins to allocate to the vault account
            match (self.env().transferred_value() * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    a0_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //cross contract call to PSP22 contract to transfer PSP22 to the caller
            if PSP22Ref::transfer(&self.psp22_token,caller,actual_psp22_amount_out_for_caller,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }

            //cross contract call to PSP22 contract to transfer PSP22 to the vault
            if PSP22Ref::transfer(&self.psp22_token,self.vault,psp22_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }

            //function to transfer A0 to the vault.
            if self.env().transfer(self.vault,a0_amount_out_for_vault).is_err() {
                    return Err(TradingPairErrors::A0TransferFailed);
            }


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

            Self::env().emit_event(A0Swap{
                caller:caller,
                a0_deposited_amount:self.env().transferred_value(),
                psp22_given_amount:actual_psp22_amount_out_for_caller,
                psp22_given_to_vault:psp22_amount_out_for_vault
            });

            Ok(())
            
        }


        ///function used to transfer LP share tokens from caller to recipient.
        #[ink(message)]
        pub fn transfer_lp_tokens(
            &mut self,
            recipient:AccountId,
            shares_to_transfer: Balance
        ) -> Result<(), TradingPairErrors>  {

            let caller = self.env().caller();

            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let recipient_shares:Balance = self.balances.get(&recipient).unwrap_or(0);
        
            if caller_shares < shares_to_transfer {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
            }

            let new_caller_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match caller_shares.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_caller_lp_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let new_recipient_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match recipient_shares.checked_add(shares_to_transfer) {
                Some(result) => {
                    new_recipient_lp_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            self.balances.insert(caller, &(new_caller_lp_balance));
 
            self.balances.insert(recipient, &(new_recipient_lp_balance));

            Ok(())


        }

        ///function used to approve the amount of LP token shares for the spender to spend from owner.
        #[ink(message)]
        pub fn approve_lp_tokens(
            &mut self,
            spender:AccountId,
            shares_to_approve: Balance
        ) -> Result<(), TradingPairErrors>  {

           let caller = self.env().caller();

           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           if caller_shares < shares_to_approve {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
           }

           if shares_to_approve >= u128::MAX {
                return Err(TradingPairErrors::Overflow);
           }


           self.lp_tokens_allowances.insert((caller,spender), &(shares_to_approve));

           Ok(())

        }

        //function to transfer LP share tokens FROM owner TO receipent
        #[ink(message)]
        pub fn transfer_lp_tokens_from_to(
            &mut self,
            owner:AccountId,
            to:AccountId,
            shares_to_transfer: Balance
        ) -> Result<(), TradingPairErrors>  {

           let spender = self.env().caller();

           let owner_shares:Balance = self.balances.get(&owner).unwrap_or(0);

           let to_shares:Balance = self.balances.get(&to).unwrap_or(0);

           let allowance:Balance = self.get_lp_tokens_allowance(owner,spender);

           if allowance < shares_to_transfer {
                return Err(TradingPairErrors::NotEnoughOwnerLPAllowance);
           }

           if owner_shares < shares_to_transfer {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
           }

           let new_owner_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match owner_shares.checked_sub(shares_to_transfer) {
               Some(result) => {
                   new_owner_lp_balance = result;
               }
               None => {
                return Err(TradingPairErrors::Overflow);
               }
           };

           let new_to_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match to_shares.checked_add(shares_to_transfer) {
               Some(result) => {
                new_to_lp_balance = result;
               }
               None => {
                return Err(TradingPairErrors::Overflow);
               }
           };

           let new_allowance:Balance;

           //calculating spender new allowance amount
           match allowance.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_allowance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

           self.balances.insert(owner, &(new_owner_lp_balance));

           self.lp_tokens_allowances.insert((owner,spender), &(new_allowance));

           self.balances.insert(to, &(new_to_lp_balance));

           Ok(())
    
         
        }
         
        //function to get the allowance of spender from the owner
        #[ink(message)]
        pub fn get_lp_tokens_allowance(
            &self,
            owner: AccountId,
            spender: AccountId
        ) -> Balance   {

            self.lp_tokens_allowances.get(&(owner,spender)).unwrap_or(0)

        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(
            &self
        ) -> AccountId {

            Self::env().account_id()

        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(
            &self
        ) -> Balance {

            let current_price = self.get_est_price_psp22_to_a0(1u128 * 10u128.pow(12)).unwrap();

            current_price
        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(
            &self
        ) -> Balance {

            self.total_supply

        }

        ///function to get contract A0 balance
        #[ink(message)]
        pub fn get_a0_balance(
            &self
        ) -> Balance {

            let a0_balance = self.env().balance();
            a0_balance

        }

        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(
            &self,account: AccountId
        ) -> Balance {

            let account_balance:Balance = self.balances.get(&account).unwrap_or(0);
            account_balance

        }

        //function to get contract PSP22 reserve (self)
        #[ink(message)]
        pub fn get_psp22_balance(
            &self
        ) -> Balance {

            let psp22_balance:Balance = PSP22Ref::balance_of(&self.psp22_token, Self::env().account_id());
            psp22_balance

        }

        ///function to get current fee 
        #[ink(message)]
        pub fn get_fee(
            &self
        ) -> Balance {

            let fee:Balance = self.fee;
            fee

        }


    	#[ink(message)]
        pub fn get_transactions_num(
            &self
        ) -> i64 {

            self.transasction_number

        }
        
        ///function to calculate the percentage between values.
        #[ink(message,payable)]
        pub fn check_diffrenece(
            &mut self,
            value1: Balance,
            value2: Balance
        ) -> Result<Balance, TradingPairErrors>  {

            let absolute_difference = value1.abs_diff(value2);


            let absolute_difference_nominated = absolute_difference * (10u128.pow(12));


            let percentage_difference:Balance;

            match 100u128.checked_mul(absolute_difference_nominated / ((value1+value2) / 2)) {
                Some(result) => {
                    percentage_difference = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(percentage_difference)
            
        }



 
    }
}