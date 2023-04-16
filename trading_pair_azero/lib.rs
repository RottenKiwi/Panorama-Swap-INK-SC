#![cfg_attr(not(feature = "std"), no_std)]

pub use self::trading_pair_azero::{
	TradingPairAzero,
	TradingPairAzeroRef,
};


#[ink::contract]
pub mod trading_pair_azero {

    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,  // Importing PSP22Ref trait from openbrush contracts
        },
    };
    use ink::storage::Mapping;  // Importing Mapping from ink storage
    use ink::env::CallFlags;  // Importing CallFlags from ink env
    use ink::prelude::vec;  // Importing vec from ink prelude

    #[ink(storage)]
    pub struct TradingPairAzero {

        // Number of transactions
        transasction_number: i64,
        // Account ID for the PSP22 token
        psp22_token: AccountId,
        // Fee amount
        fee: Balance,
        // Total supply of the token
        total_supply: Balance,
        // Balances of individual accounts
        balances: Mapping<AccountId, Balance>,
        // Account ID for the Panx contract
        panx_contract: AccountId,
        // LP token allowances between accounts
        lp_tokens_allowances: Mapping<(AccountId, AccountId), Balance>,
        // Account ID for the vault
        vault: AccountId,
        // Traders fee amount
        traders_fee: Balance,
        // PSP22 LP fee vault balance
        psp22_lp_fee_vault: Balance,
        // Azero LP fee vault balance
        azero_lp_fee_vault: Balance,
        // Overall generated PSP22 fee by the contract
        contract_overall_generated_psp22_fee: Balance,
        // Overall generated Azero fee by the contract
        contract_overall_generated_azero_fee: Balance,
        // PSP22 tokens to give in a day to each account
        psp22_to_give_in_a_day: Mapping<AccountId, Balance>,
        // Azero tokens to give in a day to each account
        azero_to_give_in_a_day: Mapping<AccountId, Balance>,
        // Overall staking rewards for each account
        account_overall_staking_rewards: Mapping<AccountId, (Balance, Balance)>,
        // Overall LP fee rewards for each account
        account_overall_lp_fee_rewards: Mapping<AccountId, (Balance, Balance)>,
        // Last redeemed timestamp for each account
        last_redeemed: Mapping<AccountId, u64>,
        // Staking percentage for LP tokens
        staking_percentage: Balance,
        // Actual LP fee amount
        actual_lp_fee: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum TradingPairErrors {
        CallerInsufficientPSP22Balance, // Error code for insufficient PSP22 balance in caller wallet
        CallerInsufficientAZEROBalance,  // Error code for insufficient PSP22 balance in caller wallet
        NotEnoughAllowance,  // Error code for not enough allowance
        Overflow,  // Error code for overflow
        ZeroSharesGiven,  // Error code for zero shares given
        SlippageTolerance,  // Error code for slippage tolerance
        PSP22TransferFromFailed,  // Error code for failed PSP22 transferFrom
        PSP22TransferFailed,  // Error code for failed PSP22 transfer
        A0TransferFailed,  // Error code for failed AZERO transfer
        CallerInsufficientLPBalance,  // Error code for insufficient LP balance in caller
        ContractOutOfA0,  // Error code for contract out of pooled AZERO tokens
        ContractOutOfPSP22,  // Error code for contract out of pooled PSP22 tokens
        NotEnoughOwnerLPAllowance,  // Error code for not enough allowance for LP tokens by owner
        ZeroDaysPassed,  // Error code for zero days passed
        ZeroDailyPSP22,  // Error code for zero daily PSP22 tokens
        ZeroDailyA0,  // Error code for zero daily AZERO tokens
        UpdateIncentiveProgramError,  // Error code for update incentive program error
        RemoveLpIncentiveProgramError,  // Error code for remove LP incentive program error

    }

    #[ink(event)]
    pub struct LiquidityPoolProvision {
        provider: AccountId,  // Address of the provider who deposited the liquidity
        a0_deposited_amount: Balance,  // Amount of AZERO tokens deposited by the provider
        psp22_deposited_amount: Balance,  // Amount of PSP22 tokens deposited by the provider
        shares_given: Balance  // Amount of LP tokens (shares) given to the provider in return
    }

    #[ink(event)]
    pub struct LiquidityPoolWithdrawal {
        caller: AccountId,  // Address of the caller who initiated the liquidity withdrawal
        shares_given: Balance,  // Amount of LP tokens (shares) being withdrawn
        a0_given_amount: Balance,  // Amount of AZERO tokens given to the caller as part of the withdrawal
        psp22_given_amount: Balance,  // Amount of PSP22 tokens given to the caller as part of the withdrawal
        new_shares_balance: Balance  // Updated balance of LP tokens (shares) after the withdrawal
    }

    #[ink(event)]
    pub struct A0Swap {
        caller: AccountId,  // Address of the caller who initiated the A0 token swap
        a0_deposited_amount: Balance,  // Amount of AZERO tokens deposited by the caller for the swap
        psp22_given_amount: Balance,  // Amount of PSP22 tokens given to the caller as part of the swap
        psp22_given_to_vault: Balance,  // Amount of PSP22 tokens sent to the vault as part of the swap
    }

    #[ink(event)]
    pub struct PSP22Swap {
        caller: AccountId,  // Address of the caller who initiated the PSP22 token swap
        psp22_deposited_amount: Balance,  // Amount of PSP22 tokens deposited by the caller for the swap
        a0_given_amount: Balance,  // Amount of AZERO tokens given to the caller as part of the swap
        a0_given_to_vault: Balance,  // Amount of AZERO tokens sent to the vault as part of the swap
    }


    impl TradingPairAzero {
        #[ink(constructor)]
        pub fn new(
            psp22_contract: AccountId,  // Address of the PSP22 token contract
            fee: Balance,  // Fee to be charged for LP providers
            panx_contract: AccountId,  // Address of the PANX token contract
            vault: AccountId  // Address of the vault where traders fees are sent
        ) -> Self {

            let transasction_number: i64 = 0;  // Number of transactions initiated
            let balances = Mapping::default();  // Mapping to store user balances
            let lp_tokens_allowances = Mapping::default();  // Mapping to store LP token allowances
            let psp22_token = psp22_contract;  // Address of the PSP22 token contract
            let total_supply: Balance = 0;  // Total supply of LP tokens
            let traders_fee: Balance = 2500000000000 / 10u128.pow(12);  // Fee to be charged to traders
            let psp22_lp_fee_vault: Balance = 0;  // Total PSP22 LP fees sent to the LP vault
            let azero_lp_fee_vault: Balance = 0;  // Total AZERO LP fees sent to the LP vault
            let contract_overall_generated_psp22_fee: Balance = 0;  // Total PSP22 fees generated by the contract
            let contract_overall_generated_azero_fee: Balance = 0;  // Total AZERO fees generated by the contract
            let psp22_to_give_in_a_day = Mapping::default();  // Mapping to store daily PSP22 fees to be given
            let azero_to_give_in_a_day = Mapping::default();  // Mapping to store daily AZERO fees to be given
            let account_overall_staking_rewards = Mapping::default();  // Mapping to store overall staking rewards for accounts
            let account_overall_lp_fee_rewards = Mapping::default();  // Mapping to store overall LP fee rewards for accounts
            let last_redeemed = Mapping::default();  // Mapping to store last redeemed time for accounts
            let staking_percentage = 3;  // Percentage of fees to be distributed as staking rewards
            let actual_lp_fee = fee;  // Actual LP fee to be charged

            // Return a new instance of TradingPairAzero with initialized variables

            Self {
                transasction_number,
                psp22_token,
                fee,
                total_supply,
                balances,
                panx_contract,
                lp_tokens_allowances,
                vault,
                traders_fee,
                psp22_lp_fee_vault,
                azero_lp_fee_vault,
                contract_overall_generated_psp22_fee,
                contract_overall_generated_azero_fee,
                psp22_to_give_in_a_day,
                azero_to_give_in_a_day,
                account_overall_staking_rewards,
                account_overall_lp_fee_rewards,
                last_redeemed,
                staking_percentage,
                actual_lp_fee

            }

            
        }

       ///function to provide liquidity to a PSP22/A0 trading pair contract.
       #[ink(message,payable)]
       pub fn provide_to_pool(
            &mut self,
            psp22_deposit_amount: Balance, // Amount of PSP22 tokens to be deposited
            expected_lp_tokens: Balance, // Expected amount of LP tokens to be received
            slippage: Balance // Slippage tolerance percentage
        ) -> Result<(), TradingPairErrors> { // Function returns a Result with an error type TradingPairErrors or a unit type ()
        
            let caller = self.env().caller(); // Get the address of the caller
        
            let caller_current_balance: Balance = PSP22Ref::balance_of( // Get the current balance of PSP22 tokens for the caller
                &self.psp22_token,
                caller
            );
        
            if caller_current_balance < psp22_deposit_amount { // If caller's PSP22 balance is less than the deposit amount, return an error
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            }
        
            let contract_allowance: Balance = PSP22Ref::allowance( // Get the allowance granted by the caller to this contract for PSP22 tokens
                &self.psp22_token,
                caller,
                Self::env().account_id()
            );
        
            if contract_allowance < psp22_deposit_amount { // If contract's allowance is less than the deposit amount, return an error
                return Err(TradingPairErrors::NotEnoughAllowance);
            }
        
            let mut shares: Balance = 0; // Initialize shares variable to 0
        
            if self.total_supply == 0 { // If total LP token supply is 0, set shares to a fixed value of 1000 * 10^12
                shares = 1000u128 * 10u128.pow(12);
            }
        
            if self.total_supply > 0 { // If total LP token supply is greater than 0, calculate shares based on the transaction value and reserve balance
        
                let reserve_before_transaction = self.get_a0_balance() - self.env().transferred_value();
        
                match (self.env().transferred_value() * self.total_supply).checked_div(reserve_before_transaction) { // Calculate shares using transferred value and total supply
                    Some(result) => {
                        shares = result;
                    }
                    None => {
                        return Err(TradingPairErrors::Overflow); // If overflow occurs during calculation, return an error
                    }
                };
            }
        
            if shares <= 0 { // If shares is less than or equal to 0, return an error difference
                return Err(TradingPairErrors::ZeroSharesGiven);
            }
        
            let percentage_diff = self.check_difference(expected_lp_tokens, shares).unwrap(); // Calculate the percentage difference between expected LP tokens and calculated shares
        
            // Validate slippage tolerance
            if percentage_diff > slippage.try_into().unwrap() { // If percentage difference is greater than slippage tolerance, return an error
                return Err(TradingPairErrors::SlippageTolerance);
            }
        
            let current_shares: Balance = self.get_lp_token_of(caller); // Get the current LP tokens balance of the caller
        
            let new_caller_shares: Balance; // Initialize new caller shares variable
        
            // Calculate the new caller shares by adding current shares and calculated shares
            match current_shares.checked_add(shares) {
                Some(result) => {
                    new_caller_shares = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow); // If overflow occurs during calculation, return an error
                }
            };

            // Perform a cross-contract call to the PSP22 token contract to transfer `psp22_deposit_amount` tokens from `caller` to the current contract's account ID
            if PSP22Ref::transfer_from_builder(&self.psp22_token,caller,Self::env().account_id(),psp22_deposit_amount,vec![]) 
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(TradingPairErrors::PSP22TransferFromFailed);
                    }

        // Get the balance of `caller` after the PSP22 token transfer
        let caller_balance_after_transfer:Balance = PSP22Ref::balance_of(
            &self.psp22_token,
            caller
        );

        // Check if the caller's balance didn't change after the PSP22 token transfer, indicating insufficient balance
        if caller_current_balance == caller_balance_after_transfer {
            // If so, return an error indicating insufficient PSP22 balance for the caller
            return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
        }

        // Increase the LP balance of `caller` (mint) by inserting `new_caller_shares` into `self.balances`
        self.balances.insert(caller, &(new_caller_shares));

        // Add `shares` to the total supply of LP tokens (mint)
        self.total_supply += shares;

        // Update the incentive program for `caller`, and if it fails, return an error
        if self.update_incentive_program(caller).is_err(){
            return Err(TradingPairErrors::UpdateIncentiveProgramError);
        }

        // Emit an event indicating the liquidity pool provision details
        Self::env().emit_event(LiquidityPoolProvision{
            provider:caller,
            a0_deposited_amount:self.env().transferred_value(),
            psp22_deposited_amount:psp22_deposit_amount,
            shares_given:shares
        });

        // Return a successful result

        Ok(())

       }

       ///function to withdraw specific amount of LP share tokens and receive AZERO coins and PSP22 tokens. 
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(
            &mut self,
            shares: Balance //number of shares the caller wants to withdraw
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

            //amount of PSP22 tokens to give to the caller
            let psp22_amount_to_give = self.get_psp22_withdraw_tokens_amount(shares).unwrap();

            //amount of A0 to give to the caller
            let a0_amount_to_give = self.get_a0_withdraw_tokens_amount(shares).unwrap();

            //amount of PSP22 tokens the caller earned from the LP fee
            let psp22_fee_amount_to_give = self.get_psp22_lp_fee_tokens(shares).unwrap();

            //amount of AZERO tokens the caller earned from the LP fee
            let a0_fee_amount_to_give = self.get_a0_lp_fee_tokens(shares).unwrap();

            
            let new_caller_lp_shares:Balance; // Initialize new_caller_lp_shares variable to 0

            //calculation to determine the new amount of caller LP shares.
            match caller_shares.checked_sub(shares) {
                Some(result) => {
                    new_caller_lp_shares = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let actual_psp22_amount_to_give:Balance; // Initialize actual_psp22_amount_to_give variable to 0

            //calculation to determine the actual PSP22 tokens amount to caller with LP fees.
            match psp22_amount_to_give.checked_add(psp22_fee_amount_to_give) {
                Some(result) => {
                    actual_psp22_amount_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let actual_azero_amount_to_give:Balance; // Initialize actual_azero_amount_to_give variable to 0


            //calculation to determine the actual AZERO tokens amount to caller with LP fees.
            match a0_amount_to_give.checked_add(a0_fee_amount_to_give) {
                Some(result) => {
                    actual_azero_amount_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
           
            //cross contract call to PSP22 contract to transfer PSP2 tokens to the caller
            if PSP22Ref::transfer(&self.psp22_token,caller,actual_psp22_amount_to_give,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }

            //function to transfer A0 to the caller
            if self.env().transfer(caller, actual_azero_amount_to_give).is_err() {
                return Err(TradingPairErrors::A0TransferFailed);
            }

            //update caller's incentive program claim percentage according to the new LP share tokens
            if self.remove_lp(new_caller_lp_shares).is_err() {
                return Err(TradingPairErrors::RemoveLpIncentiveProgramError);
            }

            //reducing caller total LP share tokens balance
            self.balances.insert(caller, &(new_caller_lp_shares));

            //reducing overall LP token supply
            self.total_supply -= shares;

            if self.total_supply == 0 {

                //cross contract call to PSP22 contract to transfer PSP2 tokens to the caller
                if PSP22Ref::transfer(&self.psp22_token,caller,self.get_psp22_balance(),vec![]).is_err(){
                    return Err(TradingPairErrors::PSP22TransferFailed);
                }

                //function to transfer A0 to the caller
                if self.env().transfer(caller, self.get_a0_balance()).is_err() {
                    return Err(TradingPairErrors::A0TransferFailed);
                }


            }

            let (current_overall_psp22_lp_rewards,current_overall_azero_lp_rewards) = self.account_overall_lp_fee_rewards
                .get(&caller)
                .unwrap_or((0,0));

            self.account_overall_lp_fee_rewards
                .insert(
                    &caller,
                    &(current_overall_psp22_lp_rewards + psp22_fee_amount_to_give,
                    current_overall_azero_lp_rewards + a0_fee_amount_to_give));

            //reducing the given PSP22 tokens from LP fee from the total PSP22 LP vault
            self.psp22_lp_fee_vault = self.psp22_lp_fee_vault - psp22_fee_amount_to_give;

            //reducing the given AZERO tokens from LP fee from the total AZERO LP vault
            self.azero_lp_fee_vault = self.azero_lp_fee_vault - a0_fee_amount_to_give;

            //emit LP withdrawal event
            Self::env().emit_event(LiquidityPoolWithdrawal{
                caller:caller,
                shares_given:shares,
                a0_given_amount:a0_amount_to_give,
                psp22_given_amount:psp22_amount_to_give,
                new_shares_balance:new_caller_lp_shares
            });

                    // Return a successful result

            Ok(())



       }


        ///function to get the amount of withdrawable PSP22 and A0 by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(
            &self,
            shares_amount: Balance
        )   -> Result<(Balance,Balance), TradingPairErrors> {

            let amount_of_a0_to_give: Balance; // Amount of A0 tokens to give to the caller.
            let actual_a0_balance = self.get_a0_balance(); // Get the actual balance of A0 tokens.
            
            // Calculate the amount of A0 tokens to give to the caller.
            match (shares_amount * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow); // Return an error if overflow occurs.
                }
            };
            
            let mut actual_psp22_balance = self.get_psp22_balance(); // Get the actual balance of PSP22 tokens.
            
            // Divide actual PSP22 balance by 10^12 to get balance in whole numbers.
            match actual_psp22_balance.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_psp22_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow); // Return an error if overflow occurs.
                }
            };
            
            let amount_of_psp22_to_give: Balance; // Amount of PSP22 tokens to give to the caller.
            
            // Calculate the amount of PSP22 tokens to give to the caller.
            match (shares_amount * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow); // Return an error if overflow occurs.
                }
            };
            
            let actual_amount_of_psp22_to_give: Balance; // Actual amount of PSP22 tokens to give to the caller.
            
            // Multiply the amount of PSP22 tokens to give by 10^12 to get the actual amount in balance.
            match amount_of_psp22_to_give.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    actual_amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow); // Return an error if overflow occurs.
                }
            };
            
            Ok((amount_of_a0_to_give, actual_amount_of_psp22_to_give)) // Return the calculated amounts of A0 and PSP22 tokens to give to the caller.

        }
            
            

        ///function to get the amount of withdrawable PSP22 and A0 by given number of LP shares with LP fees.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount_with_lp(
            &self,
            shares_amount: Balance
        ) -> Result<(Balance,Balance), TradingPairErrors> {

            let amount_of_a0_to_give:Balance;

            let actual_a0_balance = self.get_a0_balance();

            //calculating the amount of A0 to give to the caller.
            match (shares_amount * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let azero_amount_from_fees:Balance;

            //calculating the amount of A0 fees to give to the caller.
            match (shares_amount * self.azero_lp_fee_vault).checked_div(self.total_supply) {
                Some(result) => {
                    azero_amount_from_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let mut actual_psp22_balance = self.get_psp22_balance();

            match actual_psp22_balance.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_psp22_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let amount_of_psp22_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let psp22_amount_from_fees:Balance;

            //calculating the amount of PSP22 fees to give to the caller.
            match (shares_amount * self.psp22_lp_fee_vault).checked_div(self.total_supply) {
                Some(result) => {
                    psp22_amount_from_fees = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let actual_amount_of_psp22_to_give:Balance;

            match amount_of_psp22_to_give.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    actual_amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

        
            Ok(((amount_of_a0_to_give + azero_amount_from_fees),(actual_amount_of_psp22_to_give + psp22_amount_from_fees)))

        }

        ///function to get the amount of withdrawable pooled PSP22 tokens by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(
            &mut self,
            shares_amount: Balance
        )   -> Result<Balance, TradingPairErrors> {

            let amount_of_psp22_to_give:Balance;

            let mut actual_psp22_balance = self.get_psp22_balance();

            match actual_psp22_balance.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_psp22_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let actual_amount_of_psp22_to_give:Balance;

            match amount_of_psp22_to_give.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    actual_amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            Ok(actual_amount_of_psp22_to_give)
        
        }

        ///function to get the amount of PSP22 LP fee tokens by number of shares
        #[ink(message)]
        pub fn get_psp22_lp_fee_tokens(
            &mut self,
            shares_amount: Balance
        )   -> Result<Balance, TradingPairErrors> {

            let amount_of_psp22_fees_to_give:Balance;

            //calculating the amount of PSP22 to give to the caller.
            match (shares_amount * self.psp22_lp_fee_vault).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_fees_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            Ok(amount_of_psp22_fees_to_give)
        
        }

        ///function to get the percentage difference between the PSP22 pooled tokens without LP fee and with LP fees 
        #[ink(message)]
        pub fn get_psp22_difference_by_percentage(
            &mut self,
        )   -> Result<Balance, TradingPairErrors> {

            //caller address 
            let caller = self.env().caller();

            //caller total LP shares
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let amount_of_psp22_fees:Balance = self.get_psp22_lp_fee_tokens(caller_shares).unwrap();

            //amount of PSP22 to give to the caller
            let psp22_amount_without_fees = self.get_psp22_withdraw_tokens_amount(caller_shares).unwrap();

            let psp22_amount_with_fees = psp22_amount_without_fees + amount_of_psp22_fees;


            let percentage_diff:Balance = self.check_difference(
                psp22_amount_without_fees,
                psp22_amount_with_fees)
                .unwrap();



            Ok(percentage_diff)
        
        }

        ///function to get the amount of withdrawable pooled AZERO coins by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_a0_withdraw_tokens_amount(
            &self,
            shares_amount: Balance
        )   -> Result<Balance, TradingPairErrors> {

            let amount_of_a0_to_give:Balance;

            let actual_a0_balance = self.get_a0_balance();


            //calculating the amount of A0 to give to the caller.
            match (shares_amount * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            
            Ok(amount_of_a0_to_give)
        
        }

        ///function to get the amount of A0 LP fee tokens by number of shares
       #[ink(message)]
       pub fn get_a0_lp_fee_tokens(
           &self,
           shares_amount: Balance
       )   -> Result<Balance, TradingPairErrors> {

           let amount_of_a0_fees_to_give:Balance;

           //calculating the amount of LP fee A0 to give to the caller.
           match (shares_amount * self.azero_lp_fee_vault).checked_div(self.total_supply) {
               Some(result) => {
                amount_of_a0_fees_to_give = result;
               }
               None => {
                   return Err(TradingPairErrors::Overflow);
               }
           };

           
           Ok(amount_of_a0_fees_to_give)
       
       }

        ///function to get the percentage difference between the AZERO pooled coins without LP fee and with LP fees 
        #[ink(message)]
        pub fn get_a0_difference_by_percentage(
            &mut self,
        )   -> Result<Balance, TradingPairErrors> {

            //caller address 
            let caller = self.env().caller();

            //caller total LP shares
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let amount_of_a0_fees:Balance = self.get_a0_lp_fee_tokens(caller_shares).unwrap();

            //amount of PSP22 to give to the caller
            let a0_amount_without_fees = self.get_a0_withdraw_tokens_amount(caller_shares).unwrap();

            let a0_amount_with_fees = a0_amount_without_fees + amount_of_a0_fees;


            let percentage_diff:Balance = self.check_difference(
                a0_amount_without_fees,
                a0_amount_with_fees)
                .unwrap();

            Ok(percentage_diff)
        
        }
        
        ///function to get the callers pooled PSP22 and A0.
        #[ink(message)]
        pub fn get_account_locked_tokens(
            &self,
            account_id:AccountId
        )   -> Result<(Balance,Balance), TradingPairErrors> {
           
            //caller address
            let caller = account_id;
            //get caller LP tokens 
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let mut amount_of_a0_to_give:Balance = 0;

            let mut amount_of_psp22_to_give:Balance = 0;


            if caller_shares <= 0 {

                return Ok((amount_of_psp22_to_give,amount_of_a0_to_give))
                 
            }


            let actual_a0_balance = self.get_a0_balance();


            //calculating the amount of A0 to give to the caller.
            match (caller_shares * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };


            let mut actual_psp22_balance = self.get_psp22_balance();

            match actual_psp22_balance.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_psp22_balance = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the amount of PSP22 to give to the caller.
            match (caller_shares * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let actual_amount_of_psp22_to_give:Balance;

            match amount_of_psp22_to_give.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    actual_amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };
           
            Ok((actual_amount_of_psp22_to_give,amount_of_a0_to_give))

            
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
            let percentage_diff:Balance = self.check_difference(
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

            let actual_lp_fee:Balance;

            //calculating the actual LP fee
            match (self.fee / (10u128.pow(12))).checked_mul(10) {
                Some(result) => {
                    actual_lp_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let a0_amount_out_for_lp_vault:Balance;

            //calculating the amount of A0 coins to allocate to the lp vault
            match  (a0_amount_out_for_caller_before_traders_fee * actual_lp_fee).checked_div(1000u128)  {
                Some(result) => {
                    a0_amount_out_for_lp_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let new_azero_lp_fee_vault:Balance;

            match  self.azero_lp_fee_vault.checked_add(a0_amount_out_for_lp_vault)  {
                Some(result) => {
                    new_azero_lp_fee_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            self.azero_lp_fee_vault = new_azero_lp_fee_vault;

            let new_contract_overall_generated_azero_fee:Balance;

            match  self.contract_overall_generated_azero_fee.checked_add(a0_amount_out_for_lp_vault)  {
                Some(result) => {
                    new_contract_overall_generated_azero_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            self.contract_overall_generated_azero_fee = new_contract_overall_generated_azero_fee;


            //calculating the final amount of A0 coins to give to the caller after reducing traders fee
            match  a0_amount_out_for_caller_before_traders_fee.checked_sub(a0_amount_out_for_vault + a0_amount_out_for_lp_vault) {
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
        )   -> Result<(), TradingPairErrors> {

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
            let percentage_diff:Balance = self.check_difference(
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

            let actual_lp_fee:Balance;

            //calculating the actual LP fee
            match (self.fee / (10u128.pow(12))).checked_mul(10) {
                Some(result) => {
                    actual_lp_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let psp22_amount_out_for_lp_vault:Balance;
            
            //calculating the amount of PSP22 tokens to allocate to the lp vault
            match (psp22_amount_out_for_caller_before_traders_fee * actual_lp_fee).checked_div(1000u128) {
                Some(result) => {
                    psp22_amount_out_for_lp_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            let new_psp22_lp_fee_vault:Balance;

            match self.psp22_lp_fee_vault.checked_add(psp22_amount_out_for_lp_vault) {
                Some(result) => {
                    new_psp22_lp_fee_vault = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            self.psp22_lp_fee_vault = new_psp22_lp_fee_vault;

            let new_contract_overall_generated_psp22_fee:Balance;

            match self.contract_overall_generated_psp22_fee.checked_add(psp22_amount_out_for_lp_vault) {
                Some(result) => {
                    new_contract_overall_generated_psp22_fee = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            self.contract_overall_generated_psp22_fee = new_contract_overall_generated_psp22_fee;

            //calculating the final amount of PSP22 tokens to give to the caller after reducing traders fee
            match psp22_amount_out_for_caller_before_traders_fee.checked_sub(psp22_amount_out_for_vault + psp22_amount_out_for_lp_vault) {
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

        ///function to add caller to the LP incentive program
        #[ink(message)]
        pub fn update_incentive_program(
            &mut self,
            caller:AccountId
        )   -> Result<(), TradingPairErrors> {

            let account_shares_balance:Balance = self.balances.get(&caller).unwrap_or(0);

            //amount of PSP22 to give to the caller without LP fee
            let caller_locked_psp22_balance = self.get_psp22_withdraw_tokens_amount(account_shares_balance).unwrap();

            //amount of A0 to give to the caller without LP fee
            let caller_locked_azero_balance = self.get_a0_withdraw_tokens_amount(account_shares_balance).unwrap();

            //calc how many tokens to give in a day
            let psp22_amount_to_give_each_day:Balance;

            let azero_amount_to_give_each_day:Balance;

            //calculating the amount of daily PSP22 to give to the user
            match ((caller_locked_psp22_balance * self.staking_percentage) / 100u128 ).checked_div(365) {
                 Some(result) => {
                    psp22_amount_to_give_each_day = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //calculating the amount of daily AZERO to give to the user
            match ((caller_locked_azero_balance * self.staking_percentage) / 100u128 ).checked_div(365) {
                Some(result) => {
                    azero_amount_to_give_each_day = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
            };

            //insert the daily amount of PSP22 tokens and AZERO to give to the caller
            self.psp22_to_give_in_a_day.insert(caller,&psp22_amount_to_give_each_day);

            self.azero_to_give_in_a_day.insert(caller,&azero_amount_to_give_each_day);

            self.last_redeemed.insert(caller, &self.get_current_timestamp());

            Ok(())

        }

       ///function to get caller redeemable amount of pooled PSP22 and A0
       #[ink(message)]
       pub fn get_redeemable_amount(
           &mut self,
       )   -> Result<(Balance,Balance), TradingPairErrors> {

           
            //call address 
            let caller = self.env().caller();
            //current timestamp
            let current_tsp = self.get_current_timestamp();

            let account_shares_balance:Balance = self.balances.get(&caller).unwrap_or(0);

            //amount of PSP22 to give to the caller without LP fee
            let caller_locked_psp22_balance = self.get_psp22_withdraw_tokens_amount(account_shares_balance).unwrap();

            //amount of A0 to give to the caller without LP fee
            let caller_locked_azero_balance = self.get_a0_withdraw_tokens_amount(account_shares_balance).unwrap();

            //last time caller redeemed tokens
            let last_redeemed = self.last_redeemed.get(caller).unwrap_or(0);

            //the amount of daily PSP22 tokens to give ot the caller
            let psp22_to_give_each_day:Balance = self.psp22_to_give_in_a_day.get(caller).unwrap_or(0);

            //the amount of daily AZERO tokens to give ot the caller
            let azero_to_give_each_day:Balance = self.azero_to_give_in_a_day.get(caller).unwrap_or(0);

            // Declare a variable to hold the difference in days between current timestamp and last redeemed timestamp
            let days_difference:u64; 
            
            // Calculate the difference in days by dividing the difference between current timestamp and last redeemed timestamp by 86400 (number of seconds in a day)
            match (current_tsp - last_redeemed).checked_div(86400) { 
                Some(result) => {
                    days_difference = result; 
                }
                None => {
                    // If the division results in overflow, return an error of StakingErrors::Overflow
                    return Err(TradingPairErrors::Overflow); 
                }
            };

            //making sure that caller has more then 0 pooled PSP22 tokens
            if caller_locked_psp22_balance <= 0 {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance);
            }

            //making sure that caller has more then 0 pooled AZERO tokens
            if caller_locked_azero_balance <= 0 {
                return Err(TradingPairErrors::CallerInsufficientAZEROBalance);
            }

            //making sure that caller has more than 0 daily PSP22 tokens to claim
            if psp22_to_give_each_day <= 0 {
                return Err(TradingPairErrors::ZeroDailyPSP22);
            }
       
            //making sure that caller has more than 0 daily AZERO tokens to claim
            if azero_to_give_each_day <= 0 {
                return Err(TradingPairErrors::ZeroDailyA0);
            }



            //The amount of PSP22 tokens and AZERO to give to the caller
            let psp22_redeemable_amount:Balance = psp22_to_give_each_day * days_difference as u128;

            let azero_redeemable_amount:Balance = azero_to_give_each_day * days_difference as u128;

            Ok((psp22_redeemable_amount,azero_redeemable_amount))

           

       }

        ///function for caller to redeem LP incentive tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(
            &mut self
        )   -> Result<(), TradingPairErrors> {

            
            //caller address
            let caller = self.env().caller();
            //caller timestamp
            let current_tsp = self.get_current_timestamp();

            let (psp22_redeemable_amount,azero_redeemable_amount) = self.get_redeemable_amount().unwrap_or((0,0));

            //cross contract call to PSP22 contract to transfer PSP22 to caller
            if PSP22Ref::transfer(&self.psp22_token,caller,psp22_redeemable_amount,vec![]).is_err(){
                return Err(TradingPairErrors::PSP22TransferFailed);
            }

            //function to transfer A0 to the caller.
            if self.env().transfer(
                caller,
                azero_redeemable_amount)
                .is_err() {
                    return Err(TradingPairErrors::A0TransferFailed);
            }

            let (current_account_overall_psp22_staking_rewards, current_account_overall_azero_staking_rewards) = self.account_overall_staking_rewards
                .get(&caller)
                .unwrap_or((0,0));

            self.account_overall_staking_rewards
                .insert(
                    &caller,
                    &((current_account_overall_psp22_staking_rewards + psp22_redeemable_amount),
                    (current_account_overall_azero_staking_rewards + azero_redeemable_amount)));

            //Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(caller,&current_tsp);

            Ok(())

        }
    
        ///function to reduce the incentive program rewards allocation after LP removal.
        #[ink(message)]
        pub fn remove_lp(
            &mut self,
            new_shares:Balance,
        )   -> Result<(), TradingPairErrors> {

            //caller address
            let caller = self.env().caller();

            if new_shares == 0 {

                if self.redeem_redeemable_amount().is_err() {
                    return Err(TradingPairErrors::RemoveLpIncentiveProgramError);
                }

                //insert the daily amount of PSP22 and AZERO tokens to give to the caller
                self.psp22_to_give_in_a_day.insert(caller,&0);

                self.azero_to_give_in_a_day.insert(caller,&0);

            }

            if new_shares > 0 {

                if self.redeem_redeemable_amount().is_err() {
                    return Err(TradingPairErrors::RemoveLpIncentiveProgramError);
                }

                //amount of pooled PSP22 tokens by number of LP shares with LP fee
                let caller_locked_psp22_balance = self.get_psp22_withdraw_tokens_amount(new_shares).unwrap();

                //amount of pooled AZERO coins by number of LP shares with LP fee
                let caller_locked_azero_balance = self.get_a0_withdraw_tokens_amount(new_shares).unwrap();

                let new_psp22_amount_to_give_each_day:Balance;

                let new_a0_amount_to_give_each_day:Balance;

                //calculating the amount of daily PSP22 to give to the user
                match ((caller_locked_psp22_balance * self.staking_percentage) / 100u128 ).checked_div(365) {
                    Some(result) => {
                        new_psp22_amount_to_give_each_day = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow);
                }
                };

                //calculating the amount of daily A0 to give to the user
                match ((caller_locked_azero_balance * self.staking_percentage) / 100u128 ).checked_div(365) {
                    Some(result) => {
                            new_a0_amount_to_give_each_day = result;
                    }
                    None => {
                        return Err(TradingPairErrors::Overflow);
                    }
                };

                //insert the daily amount of PSP22 and AZERO tokens to give to the caller
                self.psp22_to_give_in_a_day.insert(caller,&new_psp22_amount_to_give_each_day);

                self.azero_to_give_in_a_day.insert(caller,&new_a0_amount_to_give_each_day);

            }



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

        ///function to get the amount of tokens to give to caller each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_caller(
            &mut self,
            caller:AccountId
        )-> (Balance,Balance)  {
        
           let psp22_daily_amount:Balance = self.psp22_to_give_in_a_day.get(&caller).unwrap_or(0);
           let a0_daily_amount:Balance = self.azero_to_give_in_a_day.get(&caller).unwrap_or(0);

           (psp22_daily_amount,a0_daily_amount)

        }

        #[ink(message)]
        pub fn get_generated_lp_fees(
            &mut self,
        )-> (Balance,Balance)  {
        
           let psp22_lp_fees:Balance = self.psp22_lp_fee_vault;
           let azero_lp_fees:Balance = self.azero_lp_fee_vault;

           (psp22_lp_fees,azero_lp_fees)

        }

        #[ink(message)]
        pub fn get_account_overall_staking_rewards(
            &self,
            owner: AccountId,
        )-> (Balance,Balance)  {
        
           let (psp22_overall_amount, azero_overall_amount) = self.account_overall_staking_rewards.get(&owner).unwrap_or((0,0));


           (psp22_overall_amount,azero_overall_amount)

        }

        #[ink(message)]
        pub fn get_account_overall_lp_fee_rewards(
            &self,
            owner: AccountId,
        )-> (Balance,Balance)  {
        
           let (psp22_overall_amount, azero_overall_amount) = self.account_overall_lp_fee_rewards.get(&owner).unwrap_or((0,0));


           (psp22_overall_amount,azero_overall_amount)

        }

        //function to get the contract's overall generated LP fees
        #[ink(message)]
        pub fn get_contract_overall_generated_fee(
            &mut self,
        )-> (Balance,Balance)  {
        
            let psp22_lp_fees:Balance = self.contract_overall_generated_psp22_fee;
            let azero_lp_fees:Balance = self.contract_overall_generated_azero_fee;

            (psp22_lp_fees,azero_lp_fees)

        }


        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(
            &self
        ) -> AccountId {

            Self::env().account_id()

        }
        
        ///function to fetch current price for one PSP22
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

        ///function to get trading contract AZERO balance
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

        //function to get the total number of swaps
    	#[ink(message)]
        pub fn get_transactions_num(
            &self
        ) -> i64 {

            self.transasction_number

        }
        
        ///function to calculate the percentage between values.
        #[ink(message,payable)]
        pub fn check_difference(
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

        ///function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(
            &self
        ) -> u64 {

            let time_stamp_in_seconds = self.env().block_timestamp() / 1000;
            time_stamp_in_seconds

        }

        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });
  
        }
 
    }
}