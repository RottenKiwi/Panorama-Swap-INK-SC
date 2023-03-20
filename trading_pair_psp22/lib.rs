#![cfg_attr(not(feature = "std"), no_std)]

pub use self::trading_pair_psp22::{
	TradingPairPsp22,
	TradingPairPsp22Ref,
};


#[ink::contract]
pub mod trading_pair_psp22 {
    
    
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

    use ink::storage::Mapping;
    use ink::env::CallFlags;
    use ink::prelude::vec;


        
    
    #[ink(storage)]
    pub struct TradingPairPsp22 {

        transasction_number: i64,
        psp22_token1_address: AccountId,
        psp22_token2_address: AccountId,
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
        CallerInsufficientPSP1Balance,
        CallerInsufficientPSP2Balance,
        NotEnoughPSP1Allowance,
        NotEnoughPSP2Allowance,
        Overflow,
        ZeroSharesGiven,
        SlippageTolerance,
        PSP1TransferFromFailed,
        PSP2TransferFromFailed,
        PSP1TransferFailed,
        PSP2TransferFailed,
        A0TransferFailed,
        CallerInsufficientLPBalance,
        ContractOutOfPSP1,
        ContractOutOfPSP2,
        NotEnoughOwnerLPAllowance
    }

    #[ink(event)]
    pub struct LiquidityPoolProvision {
        from:AccountId,
        psp22_1_deposited_amount:Balance,
        psp22_2_deposited_amount: Balance,
        shares_given:Balance
    }

    #[ink(event)]
    pub struct LiquidityPoolWithdrawal {
        caller:AccountId,
        shares_given:Balance,
        psp22_1_given_amount:Balance,
        psp22_2_given_amount: Balance,
        new_shares_balance:Balance
    }
    #[ink(event)]
    pub struct PSP22Token1Swap {
        caller:AccountId,
        psp22_1_deposited_amount:Balance,
        psp22_2_given_amount: Balance,
        psp22_2_given_amount_for_vault:Balance
    }
    #[ink(event)]
    pub struct PSP22Token2Swap {
        caller:AccountId,
        psp22_2_deposited_amount:Balance,
        psp22_1_given_amount: Balance,
        psp22_1_given_amount_for_vault:Balance
    }




    impl TradingPairPsp22 {
        #[ink(constructor)]
        pub fn new(
            psp22_token1_contract:AccountId,
            psp22_token2_contract:AccountId,
            fee: Balance,
            panx_contract:AccountId,
            vault:AccountId
        ) -> Self {
            

            let transasction_number:i64 = 0;
            let psp22_token1_address = psp22_token1_contract;
            let psp22_token2_address = psp22_token2_contract;
            let balances = Mapping::default();
            let lp_tokens_allowances = Mapping::default();
            let total_supply = 0;
            let traders_fee:Balance = 25;

            Self {
                transasction_number,
                psp22_token1_address,
                psp22_token2_address,
                fee,
                total_supply,
                balances,
                panx_contract,
                lp_tokens_allowances,
                vault,
                traders_fee
            }

            
        }

       ///function to provide liquidity to a PSP22/PSP22 trading pair contract.
       #[ink(message,payable)]
       pub fn provide_to_pool(
        &mut self,
        psp22_token1_deposit_amount:Balance,
        psp22_token2_deposit_amount:Balance,
        excpeted_lp_tokens:Balance,
        slippage:Balance
        )  -> Result<(), TradingPairErrors>  {

            let caller = self.env().caller();

            let mut shares:Balance = 0;

            let caller_current_balance_token1 = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                caller
            );

            //making sure that caller current PSP22 1 token balance is greater than the deposit amount.
            if caller_current_balance_token1 < psp22_token1_deposit_amount {
                return Err(TradingPairErrors::CallerInsufficientPSP1Balance);
            }

            let caller_current_balance_token2 = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                caller
            );

            //making sure that caller current PSP22 2 token balance is greater than the deposit amount.
            if caller_current_balance_token2 < psp22_token2_deposit_amount {
                return Err(TradingPairErrors::CallerInsufficientPSP2Balance);
            }

            let contract_token1_allowance = PSP22Ref::allowance(
                &self.psp22_token1_address,
                caller,
                Self::env().account_id()
            );

            //making sure that the trading pair contract has enough PSP22 1 token allowance.
            if contract_token1_allowance < psp22_token1_deposit_amount {
                return Err(TradingPairErrors::NotEnoughPSP1Allowance);
            }

            let contract_token2_allowance = PSP22Ref::allowance(
                &self.psp22_token2_address,
                caller,
                Self::env().account_id()
            );

           //making sure that the trading pair contract has enough PSP22 2 token allowance.
           if contract_token2_allowance < psp22_token2_deposit_amount {
                return Err(TradingPairErrors::NotEnoughPSP2Allowance);        
            }

           let contract_psp22_1_starting_balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                Self::env().account_id()
            );

            //cross contract call to psp22 1 token contract to transfer psp22 1 token to the trading pair contract.
            if PSP22Ref::transfer_from_builder(&self.psp22_token1_address,caller,Self::env().account_id(),psp22_token1_deposit_amount,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP1TransferFromFailed);
            }

            let caller_psp1_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                caller
            );

            if caller_current_balance_token1 == caller_psp1_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP1Balance);
            }

            let contract_psp22_1_closing_balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                Self::env().account_id()
            );

            let mut actual_psp22_1_deposit_amount:Balance = 0;
        
            //calculating the actual amount of PSP22 1 token  deposited amount (some PSP22 tokens might have internal tax)
            match contract_psp22_1_closing_balance.checked_sub(contract_psp22_1_starting_balance) {
                Some(result) => {
                    actual_psp22_1_deposit_amount = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let contract_psp22_2_starting_balance = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                Self::env().account_id()
            );

            //cross contract call to psp22 2 token contract to transfer psp22 2 token to the trading pair contract.
            if PSP22Ref::transfer_from_builder(&self.psp22_token2_address,caller,Self::env().account_id(),psp22_token2_deposit_amount,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP2TransferFromFailed);
           }

        let caller_psp2_balance_after_transfer:Balance = PSP22Ref::balance_of(
            &self.psp22_token2_address,
            caller
        );

        if caller_current_balance_token2 == caller_psp2_balance_after_transfer {
            return Err(TradingPairErrors::CallerInsufficientPSP2Balance);
        }

            let contract_psp22_2_closing_balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                Self::env().account_id()
            );
          
            let mut actual_psp22_2_deposit_amount:Balance = 0;

            //calculating the actual amount of PSP22 2 token deposited amount (some PSP22 tokens might have internal tax)
            match contract_psp22_2_closing_balance.checked_sub(contract_psp22_2_starting_balance) {
                Some(result) => {
                    actual_psp22_2_deposit_amount = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //if its the pool first deposit
            if self.total_supply == 0 {
            
                //calculating the amount of shares to give to the provider if its the first LP deposit overall
                shares = 1000u128 * 10u128.pow(12);

            }

            //if its not the first LP deposit
            if self.total_supply > 0{

                //calculating the amount of shares to give to the provider if its not the LP deposit
                match (actual_psp22_1_deposit_amount * self.total_supply).checked_div(self.get_psp22_token1_reserve() - actual_psp22_1_deposit_amount) {
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

                if PSP22Ref::transfer(&self.psp22_token1_address,caller,actual_psp22_1_deposit_amount,vec![]).is_err(){
                    return Err(TradingPairErrors::PSP1TransferFailed);
                }

                if PSP22Ref::transfer(&self.psp22_token2_address,caller,actual_psp22_2_deposit_amount,vec![]).is_err(){
                    return Err(TradingPairErrors::PSP2TransferFailed);
                }

                return Err(TradingPairErrors::ZeroSharesGiven);

            }

            //function to return the percentage diff between the expected lp token that was shown in the front-end and the final shares amount.
            let percentage_diff = self.check_diffrenece(
                excpeted_lp_tokens,shares)
                .unwrap();

            //validating slippage
            if percentage_diff > slippage.try_into().unwrap() {

                if PSP22Ref::transfer(&self.psp22_token1_address,caller,actual_psp22_1_deposit_amount,vec![]).is_err(){
                    return Err(TradingPairErrors::PSP2TransferFailed);
                }

                if PSP22Ref::transfer(&self.psp22_token2_address,caller,actual_psp22_2_deposit_amount,vec![]).is_err(){
                    return Err(TradingPairErrors::PSP2TransferFailed);
                }


                return Err(TradingPairErrors::SlippageTolerance);

            }

            //caller current shares (if any)
            let current_shares = self.get_lp_token_of(caller);

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



            //increasing LP balance of caller (mint)
            self.balances.insert(caller, &(new_caller_shares));
            //adding to over LP tokens (mint)
            self.total_supply += shares;

            Self::env().emit_event(LiquidityPoolProvision{
                from:caller,
                psp22_1_deposited_amount:actual_psp22_1_deposit_amount,
                psp22_2_deposited_amount:actual_psp22_2_deposit_amount,
                shares_given:shares
            });

            
            Ok(())



       }

       ///function to withdraw specific amount of LP share tokens.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(
            &mut self,
            shares: Balance
        )  -> Result<(), TradingPairErrors>   {

            //throw error is the caller tries to withdraw 0 LP shares
            if shares <= 0 {
                return Err(TradingPairErrors::ZeroSharesGiven);
            }
          
            //caller address
            let caller = self.env().caller();

            //caller total LP shares
            let caller_shares = self.balances.get(&caller).unwrap_or(0);

            //Validating that the caller has the given number of shares.
            if caller_shares < shares {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
            }

            //Amount of psp22 token1 to give to the caller
            let psp22_token1_amount_to_give = self.get_psp22_token1_withdraw_tokens_amount(shares).unwrap();
            //Amount of psp22 token1 to give to the caller
            let psp22_token2_amount_to_give = self.get_psp22_token2_withdraw_tokens_amount(shares).unwrap();

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
           
            //cross contract call to PSP22 token1 contract to transfer PSP22 token1 to the caller
            if PSP22Ref::transfer(&self.psp22_token1_address,caller,psp22_token1_amount_to_give,vec![]).is_err(){
                return Err(TradingPairErrors::PSP1TransferFailed);
            }
           
            //cross contract call to PSP22 token2 contract to transfer PSP22 token2 to the caller
            if PSP22Ref::transfer(&self.psp22_token2_address,caller,psp22_token2_amount_to_give,vec![]).is_err(){
                return Err(TradingPairErrors::PSP1TransferFailed);
            }

            //reducing caller LP token balance_caller_shares
            self.balances.insert(caller, &(caller_shares - shares));
            //reducing over LP token supply (burn)
            self.total_supply -= shares;

            Self::env().emit_event(LiquidityPoolWithdrawal{
                caller:caller,
                shares_given:shares,
                psp22_1_given_amount:psp22_token1_amount_to_give,
                psp22_2_given_amount:psp22_token2_amount_to_give,
                new_shares_balance:new_caller_lp_shares
            });

            Ok(())


       }

        
        ///funtion to get amount of withdrable PSP22/PSP22 tokens by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(
            &self,
            share_amount: Balance
        )  -> Result<(Balance,Balance), TradingPairErrors> {

            
            let mut amount_of_psp22_token1_to_give:Balance;

            //calculating the amount of PSP22 tokens 1 to give to the caller
            match (share_amount * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the amount of PSP22 tokens 2 to give to the caller
            let mut amount_of_psp22_token2_to_give:Balance;

            match (share_amount * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
        

            Ok((amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give))
        
        }


        ///function to get the amount of withdrawable PSP22 token1 by given shares.
        #[ink(message)]
        pub fn get_psp22_token1_withdraw_tokens_amount(
            &self,
            share_amount: Balance
        )  -> Result<Balance, TradingPairErrors> {

       
            let amount_of_psp22_token1_to_give:Balance;

            //calculating the amount of PSP22 tokens 1 to give to the caller
            match (share_amount * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
        

            Ok(amount_of_psp22_token1_to_give)
        
        }

        ///function to get the amount of withdrawable PSP22 token2 by given LP shares.
        #[ink(message)]
        pub fn get_psp22_token2_withdraw_tokens_amount(
            &self,
            share_amount: Balance
        )  -> Result<Balance, TradingPairErrors> {

        
            //calculating the amount of PSP22 tokens 2 to give to the caller
            let amount_of_psp22_token2_to_give:Balance;

            match (share_amount * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
        

            Ok(amount_of_psp22_token2_to_give)
        
        
        }

        
        ///function to get caller pooled PSP22 token1 and PSP22 token2 amounts
        #[ink(message)]
        pub fn get_account_locked_tokens(
            &self,
            account_id:AccountId
        )  -> Result<(Balance,Balance), TradingPairErrors> {
           
            //account address
            let caller = account_id;
            //get account LP tokens 
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);


            let mut amount_of_psp22_token1_to_give:Balance = 0;

            let mut amount_of_psp22_token2_to_give:Balance = 0;

            if caller_shares <= 0 {

                return Ok((amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give));
                 
            }

            //calculating the amount of locked PSP22 token 1 of given caller
            match (caller_shares * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the amount of locked PSP22 token 2 of given caller
            match (caller_shares * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
        

            Ok((amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give))

            
        }

        //function to get expected amount of LP shares.
        #[ink(message,payable)]
        pub fn get_expected_lp_token_amount(
            &self,
            psp22_token1_deposit_amount:Balance
        )  -> Result<Balance, TradingPairErrors>  {

            //init LP shares variable (shares to give to caller)
            let mut shares:Balance = 0;
           
            //if its the caller first deposit 
            if self.total_supply == 0 {

                //calculating the amount of shares to give to the provider if its the first LP deposit overall
                shares = 1000u128 * 10u128.pow(12);

            }
           
            //if its not the first LP deposit
            if self.total_supply > 0{

            //calculating the amount of shares to give to the provider if its not the LP deposit
            match (psp22_token1_deposit_amount * self.total_supply).checked_div(self.get_psp22_token1_reserve()) {
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
 

        ///function to get the amount of PSP22 token2 given for 1 PSP22 token1
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token1(
            &self
        )  -> Result<Balance, TradingPairErrors> {
            
            //formula to calculate the price
            let amount_out = self.get_est_price_psp22_token1_to_psp22_token2(1u128 * 10u128.pow(12)).unwrap();

            Ok(amount_out)

        }

        ///function to get the amount of PSP22 token1 given for 1 PSP22 token2
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token2(
            &self
        )  -> Result<Balance, TradingPairErrors> {
            


            //formula to calculate the price
            let amount_out:Balance = self.get_est_price_psp22_token2_to_psp22_token1(1u128 * 10u128.pow(12)).unwrap();

            Ok(amount_out)

        }

        ///function to get the amount of PSP22 token2 the caller will get for given PSP22 token1 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token1_to_psp22_token2(
            &self,
            amount_in: Balance
        )  -> Result<Balance, TradingPairErrors> {

            let caller = self.env().caller();

            let caller_current_balance = PSP22Ref::balance_of(
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

            //reducting the LP fee from the deposited PSP22 1 tokens 
            match amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //validating if caller has more than 3500 PANX to check if caller is eligible for the incentive program 
            if caller_current_balance >= 3500u128 * 10u128.pow(12){

                if self.fee  <= 1400000000000u128 {

                    //reducting HALF of the LP fee from the PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match amount_in.checked_mul(100u128 - (actual_fee / 2u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            return Err(TradingPairErrors::Overflow);
                        }
                    };

                }

                if self.fee  > 1400000000000u128 {

                    //reducting (LP fee - 1) of the LP fee from the PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match amount_in.checked_mul(100u128 - (actual_fee - 1u128)) {
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

            //calculating the final PSP22 2 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_psp22_token2_reserve()).checked_div((self.get_psp22_token1_reserve() * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(amount_out)                        

        }


        ///function to get the amount of PSP22 token1 the caller will get for given PSP22 token2 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token2_to_psp22_token1(
            &self,
            amount_in: Balance
        )  -> Result<Balance, TradingPairErrors> {

            let caller = self.env().caller();

            let caller_current_balance = PSP22Ref::balance_of(
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

            //reducting the LP fee from the deposited PSP22 2 tokens 
            match amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //validating if caller has more than 3500 PANX
            if caller_current_balance >= 3500 * 10u128.pow(12){

                if self.fee  <= 1400000000000u128 {

                    match amount_in.checked_mul(100 - (actual_fee / 2u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            return Err(TradingPairErrors::Overflow);
                        }
                    };

                }
 
                if self.fee  > 1400000000000u128 {

                    match amount_in.checked_mul(100 - (actual_fee - 1u128)) {
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

            //calculating the final PSP22 1 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_psp22_token1_reserve()).checked_div((self.get_psp22_token2_reserve() * 100) + amount_in_with_lp_fees){
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(amount_out)

        }

        ///function to get the estimated price impact for given psp22 token1 amount
        #[ink(message)]
        pub fn get_price_impact_psp22_token1_to_psp22_token2(
            &self,
            amount_in: Balance
        )  -> Result<Balance, TradingPairErrors> {
            
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

            //fetching the amount of PSP22 2 tokens the caller WOULD get if he would swap
            let psp22_token2_amount_out:Balance = self.get_est_price_psp22_token1_to_psp22_token2(amount_in).unwrap();

            //reducting the LP fee from the PSP22 amount in
            let amount_in_with_lp_fees:Balance;

            match amount_in.checked_mul(100 - (self.fee / 10u128.pow(12))) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let amount_out:Balance;

            //calculating the final PSP22 2 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * (self.get_psp22_token2_reserve() - psp22_token2_amount_out)).checked_div(((self.get_psp22_token1_reserve() + amount_in_with_lp_fees ) * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            
            Ok(amount_out)    

        }
        ///function to get the estimated price impact for given psp22 token2 amount
        #[ink(message,payable)]
        pub fn get_price_impact_psp22_token2_to_psp22_token1(
            &self,
            amount_in:Balance
        )  -> Result<Balance, TradingPairErrors> {

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
            
            //fetching the amount of PSP22 1 tokens the caller WOULD get if he would swap
            let psp22_token1_amount_out = self.get_est_price_psp22_token2_to_psp22_token1(amount_in).unwrap();

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_lp_fees:Balance;

            match amount_in.checked_mul(100 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let amount_out:Balance;

            //calculating the final PSP22 1 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * (self.get_psp22_token1_reserve() - psp22_token1_amount_out)).checked_div(((self.get_psp22_token2_reserve() + amount_in_with_lp_fees ) * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
            
            Ok(amount_out)  


        }

        
        ///function to swap psp22 token1 to psp22 token2
        #[ink(message)]
        pub fn swap_psp22_token1(
            &mut self,
            psp22_token1_amount_to_swap: Balance,
            amount_to_validate: Balance,
            slippage: Balance
        )  -> Result<(), TradingPairErrors> {

            let caller = self.env().caller();

            let contract_psp22_token1_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                Self::env().account_id()
            );

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_token1_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP1);
            }

            let contract_psp22_token2_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                Self::env().account_id()
            );

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_token2_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP2);

            }

            let caller_current_balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                caller
            );

            //making sure caller has more or equal to the amount he transfers.
            if caller_current_balance < psp22_token1_amount_to_swap {
                return Err(TradingPairErrors::CallerInsufficientPSP1Balance);
            }

            let contract_allowance = PSP22Ref::allowance(
                &self.psp22_token1_address,
                caller,
                Self::env().account_id()
            );

            //making sure trading pair contract has enough allowance.
            if contract_allowance < psp22_token1_amount_to_swap {
                return Err(TradingPairErrors::NotEnoughPSP1Allowance);
            }
            
            //amount of PSP22 tokens 2 to give to caller before traders fee.
            let psp22_token2_amount_out_for_caller_before_traders_fee = self.get_est_price_psp22_token1_to_psp22_token2(
                psp22_token1_amount_to_swap)
                .unwrap();

            let actual_psp22_token2_amount_out_for_caller:Balance;

            let psp22_token2_amount_out_for_vault:Balance;

            //Calculating the amount to allocate to the vault account
            match (psp22_token2_amount_out_for_caller_before_traders_fee * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    psp22_token2_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //Calculating the final amount of psp22 tokens 2 to give to the caller after reducing traders fee
            match psp22_token2_amount_out_for_caller_before_traders_fee.checked_sub(psp22_token2_amount_out_for_vault) {
                Some(result) => {
                    actual_psp22_token2_amount_out_for_caller = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let psp22_token1_amount_out_for_vault:Balance;

            //calculating the amount of PSP22 token 1 to allocate to the vault account
            match  (psp22_token1_amount_to_swap * self.traders_fee).checked_div(1000u128)  {
                Some(result) => {
                    psp22_token1_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //percentage dif between given PSP22 2 tokens amount from front-end and acutal final PSP22 2 tokens amount
            let percentage_diff = self.check_diffrenece(
                amount_to_validate,
                psp22_token2_amount_out_for_caller_before_traders_fee)
                .unwrap();

            //Validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance);
            }

            //cross contract call to PSP22 1 contract to transfer PSP22 1 tokens to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token1_address,caller,Self::env().account_id(),psp22_token1_amount_to_swap,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP1TransferFromFailed);
            }

            let caller_psp1_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                caller
            );
    
            if caller_current_balance == caller_psp1_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP1Balance);
            }



            //function to transfer PSP22 2 tokens to caller
            if PSP22Ref::transfer(&self.psp22_token2_address,caller,actual_psp22_token2_amount_out_for_caller,vec![]).is_err(){
                return Err(TradingPairErrors::PSP2TransferFailed);
            }

            //function to transfer PSP22 2 tokens to vault
            if PSP22Ref::transfer(&self.psp22_token2_address,self.vault,psp22_token2_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP2TransferFailed);
            }

            //function to transfer PSP22 tokens 1 to vault
            if PSP22Ref::transfer(&self.psp22_token1_address,self.vault,psp22_token1_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP1TransferFailed);
            }


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

            Self::env().emit_event(PSP22Token1Swap{
                caller:caller,
                psp22_1_deposited_amount:psp22_token1_amount_to_swap,
                psp22_2_given_amount:actual_psp22_token2_amount_out_for_caller,
                psp22_2_given_amount_for_vault:psp22_token2_amount_out_for_vault
            });

            Ok(())

        }


        ///function to swap psp22 token2 to psp22 token1
        #[ink(message,payable)]
        pub fn swap_psp22_token2(
            &mut self,
            psp22_token2_amount_to_swap: Balance,
            amount_to_validate: Balance,
            slippage: Balance
        )  -> Result<(), TradingPairErrors> {

            let caller = self.env().caller();

            let contract_psp22_token1_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token1_address,
                Self::env().account_id()
            );

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_token1_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP1);
            }

            let contract_psp22_token2_current_balance:Balance = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                Self::env().account_id()
            );

            //making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_token2_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP2);
            }
            
            let caller_current_balance = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                caller
            );

            //making sure caller has more or equal to the amount he transfers.
            if caller_current_balance < psp22_token2_amount_to_swap {
                return Err(TradingPairErrors::CallerInsufficientPSP2Balance);
            }

            let contract_allowance = PSP22Ref::allowance(
                &self.psp22_token2_address,
                caller,
                Self::env().account_id()
            );

            //making sure trading pair contract has enough allowance.
            if contract_allowance < psp22_token2_amount_to_swap {
                return Err(TradingPairErrors::NotEnoughPSP2Allowance);
            }


            //amount of PSP22 tokens 1 to give to caller before traders fee.
            let psp22_token1_amount_out_for_caller_before_traders_fee = self.get_est_price_psp22_token2_to_psp22_token1(
                psp22_token2_amount_to_swap)
                .unwrap();

            let actual_psp22_token1_amount_out_for_caller:Balance;

            let psp22_token1_amount_out_for_vault:Balance;

            //calculating the amount of PSP22 1 tokens to allocate to the vault account
            match (psp22_token1_amount_out_for_caller_before_traders_fee * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    psp22_token1_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the final amount of psp22 tokens 1 to give to the caller after reducing traders fee.
            match psp22_token1_amount_out_for_caller_before_traders_fee.checked_sub(psp22_token1_amount_out_for_vault) {
                Some(result) => {
                    actual_psp22_token1_amount_out_for_caller = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
    
            let psp22_token2_amount_out_for_vault:Balance;

            //calculating the amount of PSP22 token 1 to allocate to the vault account
            match  (psp22_token2_amount_to_swap * self.traders_fee).checked_div(1000u128)  {
                Some(result) => {
                    psp22_token2_amount_out_for_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //percentage dif between given PSP22 1 token amount (from front-end) and acutal final PSP22 1 token amount
            let percentage_diff = self.check_diffrenece(
                    amount_to_validate,
                    psp22_token1_amount_out_for_caller_before_traders_fee
                ).unwrap();

            //Validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance);
            }


            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token2_address,caller,Self::env().account_id(),psp22_token2_amount_to_swap,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP2TransferFromFailed);
            }

            let caller_psp2_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &self.psp22_token2_address,
                caller
            );
    
            if caller_current_balance == caller_psp2_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP2Balance);
            }


            //function to transfer PSP22 1 token to caller
            if PSP22Ref::transfer(&self.psp22_token1_address,caller,actual_psp22_token1_amount_out_for_caller,vec![]).is_err(){
                return Err(TradingPairErrors::PSP1TransferFailed);
            }


            //function to transfer PSP22 1 token to vault
            if PSP22Ref::transfer(&self.psp22_token1_address,self.vault,psp22_token1_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP1TransferFailed);
            }


            //function to transfer PSP22 2 token to vault
            if PSP22Ref::transfer(&self.psp22_token2_address,self.vault,psp22_token2_amount_out_for_vault,vec![]).is_err(){
                return Err(TradingPairErrors::PSP2TransferFailed);
            }

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

            Self::env().emit_event(PSP22Token2Swap{
                caller:self.env().caller(),
                psp22_2_deposited_amount:psp22_token2_amount_to_swap,
                psp22_1_given_amount:actual_psp22_token1_amount_out_for_caller,
                psp22_1_given_amount_for_vault:psp22_token1_amount_out_for_vault
            });

            Ok(())

            
        }
        
        ///function used to transfer LP share tokens from caller to recipient.
        #[ink(message)]
        pub fn transfer_lp_tokens(
            &mut self,
            recipient:AccountId,
            shares_to_transfer: Balance
        )  -> Result<(), TradingPairErrors> {

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
        )  -> Result<(), TradingPairErrors>  {

            let caller = self.env().caller();

            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            if caller_shares < shares_to_approve {
                return Err(TradingPairErrors::CallerInsufficientLPBalance);
            }

            //overflow validation
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
        )  -> Result<(), TradingPairErrors>  {

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
        pub fn get_lp_tokens_allowance(
            &self,
            owner: AccountId,
            spender: AccountId
        ) -> Balance {

            self.lp_tokens_allowances.get(&(owner,spender)).unwrap_or(0)

        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(
            &self
        ) -> AccountId {

            Self::env().account_id()

        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_fee(
            &self
        ) -> Balance {

            self.fee

        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(
            &self
        ) -> Balance {
        
            self.get_est_price_psp22_token1_to_psp22_token2(100u128 * 10u128.pow(12)).unwrap()

        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(
            &self
        ) -> Balance {

            self.total_supply

        }


        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(
            &self,
            account: AccountId
        ) -> Balance {

            let account_balance = self.balances.get(&account).unwrap_or(0);
            account_balance

        }
        ///function to get contract PSP22 token2 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token2_reserve(
            &self
        ) -> Balance {

            let balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());
            balance

        }
        ///function to get contract PSP22 token1 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token1_reserve(
            &self
        ) -> Balance {
            
            let balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());
            balance

        }



    	#[ink(message)]
        pub fn get_transactions_num(
            &self
        ) -> i64 {

            self.transasction_number

        }
        
        #[ink(message,payable)]
        pub fn check_diffrenece(
            &mut self,
            value1: Balance,
            value2: Balance
        )  -> Result<Balance, TradingPairErrors> {

            let abs_dif = value1.abs_diff(value2);

            let abs_dif_nominated = abs_dif * 10u128.pow(12);

            let diff:Balance;

            match 100u128.checked_mul(abs_dif_nominated / ((value1+value2) / 2)) {
                Some(result) => {
                    diff = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(diff)
            
        }
 
    }
}