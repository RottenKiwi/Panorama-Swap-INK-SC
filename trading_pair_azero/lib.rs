#![cfg_attr(not(feature = "std"), no_std)]

#[openbrush::implementation(PSP22)]
#[openbrush::contract]
pub mod trading_pair_azero {

    use ink::env::CallFlags; // Importing CallFlags from ink env
    use ink::prelude::vec; // Importing vec from ink prelude
    use ink::storage::Mapping; // Importing Mapping from ink storage
    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,
        },
        traits::Storage,
    };
    use primitive_types::U256;

    
    #[ink(storage)]
    #[derive(Storage)]
    pub struct TradingPairAzero {
        #[storage_field]
        psp22: psp22::Data,
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
        // Overall staking rewards for each account
        account_overall_staking_rewards: Mapping<AccountId, Balance>,
        // Overall LP fee rewards for each account
        account_overall_lp_fee_rewards: Mapping<AccountId, (Balance, Balance)>,
        // Last redeemed timestamp for each account
        last_redeemed: Mapping<AccountId, u64>,
        // Staking percentage for LP tokens
        staking_percentage: Balance,
        // LP lock timestamp
        lp_lock_timestamp: u64,
        // Deployer account address
        deployer: AccountId
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum TradingPairErrors {
        CallerInsufficientPSP22Balance, /* Error code for insufficient PSP22 balance in caller wallet */
        CallerInsufficientAZEROBalance, /* Error code for insufficient PSP22 balance in caller wallet */
        NotEnoughAllowance,             // Error code for not enough allowance
        Overflow,                       // Error code for overflow
        ZeroSharesGiven,                // Error code for zero shares given
        SlippageTolerance,              // Error code for slippage tolerance
        PSP22TransferFromFailed,        // Error code for failed PSP22 transferFrom
        PSP22TransferFailed,            // Error code for failed PSP22 transfer
        A0TransferFailed,               // Error code for failed AZERO transfer
        CallerInsufficientLPBalance,    // Error code for insufficient LP balance in caller
        ContractOutOfA0,                // Error code for contract out of pooled AZERO tokens
        ContractOutOfPSP22,             // Error code for contract out of pooled PSP22 tokens
        NotEnoughOwnerLPAllowance, // Error code for not enough allowance for LP tokens by owner
        ZeroDaysPassed,            // Error code for zero days passed
        ZeroDailyPSP22,            // Error code for zero daily PSP22 tokens
        ZeroDailyA0,               // Error code for zero daily AZERO tokens
        UpdateIncentiveProgramError, // Error code for update incentive program error
        RemoveLpIncentiveProgramError, // Error code for remove LP incentive program error
        LpStillLocked,             // Error code for remove LP before the lock date
    }

    #[ink(event)]
    pub struct LiquidityPoolProvision {
        provider: AccountId, // Address of the provider who deposited the liquidity
        a0_deposited_amount: Balance, // Amount of AZERO tokens deposited by the provider
        psp22_deposited_amount: Balance, // Amount of PSP22 tokens deposited by the provider
        shares_given: Balance, // Amount of LP tokens (shares) given to the provider in return
    }

    #[ink(event)]
    pub struct LiquidityPoolWithdrawal {
        caller: AccountId, // Address of the caller who initiated the liquidity withdrawal
        shares_given: Balance, // Amount of LP tokens (shares) being withdrawn
        a0_given_amount: Balance, /* Amount of AZERO tokens given to the caller as part of the withdrawal */
        psp22_given_amount: Balance, /* Amount of PSP22 tokens given to the caller as part of the withdrawal */
        new_shares_balance: Balance, // Updated balance of LP tokens (shares) after the withdrawal
    }

    #[ink(event)]
    pub struct A0Swap {
        caller: AccountId, // Address of the caller who initiated the A0 token swap
        a0_deposited_amount: Balance, // Amount of AZERO tokens deposited by the caller for the swap
        psp22_given_amount: Balance, /* Amount of PSP22 tokens given to the caller as part of the swap */
        psp22_given_to_vault: Balance, /* Amount of PSP22 tokens sent to the vault as part of the swap */
    }

    #[ink(event)]
    pub struct PSP22Swap {
        caller: AccountId, // Address of the caller who initiated the PSP22 token swap
        psp22_deposited_amount: Balance, /* Amount of PSP22 tokens deposited by the caller for the swap */
        a0_given_amount: Balance, // Amount of AZERO tokens given to the caller as part of the swap
        a0_given_to_vault: Balance, // Amount of AZERO tokens sent to the vault as part of the swap
    }

    #[overrider(PSP22)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
        self.lp_tokens_allowances
        .get(&(owner, spender))
        .unwrap_or(0)
    }

    #[overrider(PSP22)]
    fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
        let caller = self.get_caller_id();

        self.lp_tokens_allowances
            .insert((caller, spender), &(value));

        Ok(())
    }

    #[overrider(PSP22)]
    fn transfer(
        &mut self,
        to: AccountId,
        value: Balance,
        _data: Vec<u8>,
    ) -> Result<(), PSP22Error> {
        let caller = self.get_caller_id();

        let caller_shares: Balance = self.balances.get(&caller).unwrap_or(0);

        let recipient_shares: Balance = self.balances.get(&to).unwrap_or(0);

        if caller_shares < value {
            return Err(PSP22Error::InsufficientBalance)
        }

        let new_caller_lp_balance: Balance = caller_shares - value;

        let new_recipient_lp_balance: Balance = recipient_shares + value;

        self.balances.insert(caller, &(new_caller_lp_balance));

        self.balances.insert(to, &(new_recipient_lp_balance));

        Ok(())
    }

    #[overrider(PSP22)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: Balance,
        _data: Vec<u8>,
    ) -> Result<(), PSP22Error> {
        let caller = self.get_caller_id();

        let allowance = psp22::PSP22::allowance(self, from, caller);
        

        if allowance < value {
            return Err(PSP22Error::InsufficientAllowance)
        }

        let from_shares: Balance = self.balances.get(&from).unwrap_or(0);

        let recipient_shares: Balance = self.balances.get(&to).unwrap_or(0);

        if from_shares < value {
            return Err(PSP22Error::InsufficientBalance)
        }

        let new_from_lp_balance: Balance = from_shares - value;

        let new_recipient_lp_balance: Balance = recipient_shares + value;

        self.balances.insert(from, &(new_from_lp_balance));

        self.balances.insert(to, &(new_recipient_lp_balance));

        let new_allowance = allowance - value;

        self.lp_tokens_allowances
            .insert((from, caller), &(new_allowance));

        Ok(())
    }

    #[overrider(PSP22)]
    fn balance_of(&self, owner: AccountId) -> Balance {
        self.balances.get(&owner).unwrap_or(0)
    }

    #[overrider(PSP22)]
    fn total_supply(&self) -> Balance {
        self.total_supply
    }
    

    impl TradingPairAzero {
        #[ink(constructor)]
        pub fn new(
            psp22_contract: AccountId, // Address of the PSP22 token contract
            fee: Balance,              // Fee to be charged for LP providers
            panx_contract: AccountId,  // Address of the PANX token contract
            vault: AccountId,          // Address of the vault where traders fees are sent
            lp_lock_timestamp: u64,    // Lp lock timestamp
            deployer: AccountId
        ) -> Self {
            let psp22: psp22::Data = Default::default();
            let transasction_number: i64 = 0; // Number of transactions initiated
            let balances = Mapping::default(); // Mapping to store user balances
            let lp_tokens_allowances = Mapping::default(); // Mapping to store LP token allowances
            let psp22_token = psp22_contract; // Address of the PSP22 token contract
            let total_supply: Balance = 0; // Total supply of LP tokens
            let traders_fee: Balance = 2500000000000 / 10u128.pow(12); // Fee to be charged to traders
            let psp22_lp_fee_vault: Balance = 0; // Total PSP22 LP fees sent to the LP vault
            let azero_lp_fee_vault: Balance = 0; // Total AZERO LP fees sent to the LP vault
            let contract_overall_generated_psp22_fee: Balance = 0; // Total PSP22 fees generated by the contract
            let contract_overall_generated_azero_fee: Balance = 0; // Total AZERO fees generated by the contract
            let psp22_to_give_in_a_day = Mapping::default(); // Mapping to store daily PSP22 fees to be given
            let account_overall_staking_rewards = Mapping::default(); // Mapping to store overall staking rewards for accounts
            let account_overall_lp_fee_rewards = Mapping::default(); // Mapping to store overall LP fee rewards for accounts
            let last_redeemed = Mapping::default(); // Mapping to store last redeemed time for accounts
            let staking_percentage = 2; // Percentage of fees to be distributed as staking rewards

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
                account_overall_staking_rewards,
                account_overall_lp_fee_rewards,
                last_redeemed,
                staking_percentage,
                lp_lock_timestamp,
                psp22,
                deployer
            }
        }

        /// function to provide liquidity to a PSP22/A0 trading pair contract.
        #[ink(message, payable)]
        pub fn provide_to_pool(
            &mut self,
            psp22_deposit_amount: Balance, // Amount of PSP22 tokens to be deposited
            a0_deposit_amount: Balance,    // Amount of AZERO coins to be deposited
            expected_lp_tokens: Balance,   // Expected amount of LP tokens to be received
            slippage: Balance,             // Slippage tolerance percentage
        ) -> Result<(), TradingPairErrors> {
            // Function returns a Result with an error type TradingPairErrors or a unit type ()

            let caller = self.env().caller(); // Get the address of the caller

            let caller_current_balance: Balance = PSP22Ref::balance_of(
                // Get the current balance of PSP22 tokens for the caller
                &self.psp22_token,
                caller,
            );

            if caller_current_balance < psp22_deposit_amount {
                // If caller's PSP22 balance is less than the deposit amount, return an error
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance)
            }

            let contract_allowance: Balance = PSP22Ref::allowance(
                // Get the allowance granted by the caller to this contract for PSP22 tokens
                &self.psp22_token,
                caller,
                Self::env().account_id(),
            );

            if contract_allowance < psp22_deposit_amount {
                // If contract's allowance is less than the deposit amount, return an error
                return Err(TradingPairErrors::NotEnoughAllowance)
            }

            let mut shares: U256 = U256::from(0); // Initialize shares variable to 0

            if self.total_supply == 0 {
                shares =
                    U256::from(self.env().transferred_value()) * U256::from(psp22_deposit_amount);

                match shares.checked_div(U256::from(10u128.pow(12))) {
                    Some(result) => {
                        shares = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };
            }

            if self.total_supply > 0 {
                let reserve_before_transaction =
                    self.get_a0_balance() - self.env().transferred_value();

                let coin_product = (self.env().transferred_value() * self.total_supply)
                    / reserve_before_transaction;

                let psp22_product =
                    (psp22_deposit_amount * self.total_supply) / self.get_psp22_balance();

                shares = U256::from(self._min(coin_product, psp22_product));

                let psp22_amount_needed_to_deposit =
                    self.get_psp22_amount_for_lp(a0_deposit_amount, reserve_before_transaction);

                let a0_amount_needed_to_deposit =
                    self.get_a0_amount_for_lp(psp22_deposit_amount, reserve_before_transaction);

                let psp22_deposit_percentage_diff = self
                    .check_difference(psp22_deposit_amount, psp22_amount_needed_to_deposit)
                    .unwrap();

                let a0_deposit_percentage_diff = self
                    .check_difference(a0_amount_needed_to_deposit, a0_deposit_amount)
                    .unwrap();

                if psp22_deposit_percentage_diff > slippage && a0_deposit_percentage_diff > slippage
                {
                    return Err(TradingPairErrors::SlippageTolerance)
                }
            }

            if shares <= U256::from(0) {
                // If shares is less than or equal to 0, return an error difference
                return Err(TradingPairErrors::ZeroSharesGiven)
            }

            let percentage_diff = self
                .check_difference(expected_lp_tokens, shares.as_u128())
                .unwrap(); // Calculate the percentage difference between expected LP tokens and calculated shares

            // Validate slippage tolerance
            if percentage_diff > slippage.try_into().unwrap() {
                // If percentage difference is greater than slippage tolerance, return an error
                return Err(TradingPairErrors::SlippageTolerance)
            }

            let current_shares: Balance = self.get_lp_token_of(caller); // Get the current LP tokens balance of the caller

            let new_caller_shares: Balance; // Initialize new caller shares variable

            // Calculate the new caller shares by adding current shares and calculated shares
            match current_shares.checked_add(shares.as_u128()) {
                Some(result) => {
                    new_caller_shares = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow) // If overflow occurs during calculation, return an error
                }
            };

            // Perform a cross-contract call to the PSP22 token contract to transfer `psp22_deposit_amount` tokens from `caller` to the current contract's account ID
            if PSP22Ref::transfer_from_builder(
                &self.psp22_token,
                caller,
                Self::env().account_id(),
                psp22_deposit_amount,
                vec![],
            )
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFromFailed)
            }

            // Get the balance of `caller` after the PSP22 token transfer
            let caller_balance_after_transfer: Balance =
                PSP22Ref::balance_of(&self.psp22_token, caller);

            // Check if the caller's balance didn't change after the PSP22 token transfer, indicating insufficient balance
            if caller_current_balance == caller_balance_after_transfer {
                // If so, return an error indicating insufficient PSP22 balance for the caller
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance)
            }

            // Increase the LP balance of `caller` (mint) by inserting `new_caller_shares` into `self.balances`
            self.balances.insert(caller, &(new_caller_shares));

            //self._mint_to(caller, shares.as_u128());
            psp22::Internal::_mint_to(self,caller, shares.as_u128());

            // Add `shares` to the total supply of LP tokens (mint)
            self.total_supply += shares.as_u128();

            // Update the incentive program for `caller`, and if it fails, return an error
            if self.update_incentive_program(caller).is_err() {
                return Err(TradingPairErrors::UpdateIncentiveProgramError)
            }

            // Emit an event indicating the liquidity pool provision details
            Self::env().emit_event(LiquidityPoolProvision {
                provider: caller,
                a0_deposited_amount: self.env().transferred_value(),
                psp22_deposited_amount: psp22_deposit_amount,
                shares_given: shares.as_u128(),
            });

            // Return a successful result

            Ok(())
        }

        /// function to withdraw specific amount of LP share tokens and receive AZERO coins and PSP22 tokens.
        #[ink(message, payable)]
        pub fn withdraw_specific_amount(
            &mut self,
            shares: Balance, // number of shares the caller wants to withdraw
        ) -> Result<(), TradingPairErrors> {
            
            // caller address
            let caller = self.env().caller();

            if self.get_current_timestamp() < self.lp_lock_timestamp && caller == self.deployer {
                return Err(TradingPairErrors::LpStillLocked)
            }

            // throw error is the caller tries to withdraw 0 LP shares
            if shares <= 0 {
                return Err(TradingPairErrors::ZeroSharesGiven)
            }



            // caller total LP shares
            let caller_shares: Balance = self.balances.get(&caller).unwrap_or(0);

            // validating that the caller has more than the given number of shares.
            if caller_shares < shares {
                return Err(TradingPairErrors::CallerInsufficientLPBalance)
            }

            // amount of PSP22 tokens to give to the caller
            let psp22_amount_to_give = self.get_psp22_withdraw_tokens_amount(shares).unwrap();

            // amount of A0 to give to the caller
            let a0_amount_to_give = self.get_a0_withdraw_tokens_amount(shares).unwrap();

            // amount of PSP22 tokens the caller earned from the LP fee
            let psp22_fee_amount_to_give = self.get_psp22_lp_fee_tokens(shares).unwrap();

            // amount of AZERO tokens the caller earned from the LP fee
            let a0_fee_amount_to_give = self.get_a0_lp_fee_tokens(shares).unwrap();

            // Initialize new_caller_lp_shares variable to 0
            let new_caller_lp_shares: Balance;

            // calculation to determine the new amount of caller LP shares.
            match caller_shares.checked_sub(shares) {
                Some(result) => {
                    new_caller_lp_shares = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            // cross contract call to PSP22 contract to transfer PSP2 tokens to the caller
            if PSP22Ref::transfer(&self.psp22_token, caller, psp22_amount_to_give, vec![]).is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFailed)
            }

            // function to transfer A0 to the caller
            if self.env().transfer(caller, a0_amount_to_give).is_err() {
                return Err(TradingPairErrors::A0TransferFailed)
            }

            // reducing caller total LP share tokens balance
            self.balances.insert(caller, &(new_caller_lp_shares));
            //self._burn_from(caller, shares);
            psp22::Internal::_burn_from(self,caller, shares);

            // reducing overall LP token supply
            self.total_supply -= shares;

            if self.total_supply == 0 {
                // cross contract call to PSP22 contract to transfer PSP2 tokens to the caller
                if PSP22Ref::transfer(&self.psp22_token, caller, self.get_psp22_balance(), vec![])
                    .is_err()
                {
                    return Err(TradingPairErrors::PSP22TransferFailed)
                }

                // function to transfer A0 to the caller
                if self.env().transfer(caller, self.get_a0_balance()).is_err() {
                    return Err(TradingPairErrors::A0TransferFailed)
                }
            }

            // update caller's incentive program claim percentage according to the new LP share tokens
            if self.remove_lp(new_caller_lp_shares).is_err() {
                return Err(TradingPairErrors::RemoveLpIncentiveProgramError)
            }

            let (current_overall_psp22_lp_rewards, current_overall_azero_lp_rewards) = self
                .account_overall_lp_fee_rewards
                .get(&caller)
                .unwrap_or((0u128, 0u128));

            self.account_overall_lp_fee_rewards.insert(
                &caller,
                &(
                    current_overall_psp22_lp_rewards + psp22_fee_amount_to_give,
                    current_overall_azero_lp_rewards + a0_fee_amount_to_give,
                ),
            );

            // reducing the given PSP22 tokens from LP fee from the total PSP22 LP vault
            self.psp22_lp_fee_vault = self.psp22_lp_fee_vault - psp22_fee_amount_to_give;

            // reducing the given AZERO tokens from LP fee from the total AZERO LP vault
            self.azero_lp_fee_vault = self.azero_lp_fee_vault - a0_fee_amount_to_give;

            // emit LP withdrawal event
            Self::env().emit_event(LiquidityPoolWithdrawal {
                caller,
                shares_given: shares,
                a0_given_amount: a0_amount_to_give,
                psp22_given_amount: psp22_amount_to_give,
                new_shares_balance: new_caller_lp_shares,
            });

            // Return a successful result
            Ok(())
        }

        /// function to get the amount of withdrawable PSP22 and A0 by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(
            &mut self,
            shares_amount: Balance,
        ) -> Result<(Balance, Balance), TradingPairErrors> {
            let mut amount_of_a0_to_give: Balance; // Amount of A0 tokens to give to the caller.

            let actual_a0_balance = self.get_a0_balance(); // Get the actual balance of A0 tokens.

            // Calculate the amount of A0 tokens to give to the caller.
            match (shares_amount * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow) // Return an error if overflow occurs.
                }
            };

            let actual_psp22_balance = self.get_psp22_balance(); // Get the actual balance of PSP22 tokens.

            let mut amount_of_psp22_to_give: Balance; // Amount of PSP22 tokens to give to the caller.

            // Calculate the amount of PSP22 tokens to give to the caller.
            match (shares_amount * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => {
                    return Err(TradingPairErrors::Overflow) // Return an error if overflow occurs.
                }
            };

            // amount of PSP22 tokens the caller earned from the LP fee
            let psp22_fee_amount_to_give = self.get_psp22_lp_fee_tokens(shares_amount).unwrap();

            // amount of AZERO tokens the caller earned from the LP fee
            let a0_fee_amount_to_give = self.get_a0_lp_fee_tokens(shares_amount).unwrap();

            amount_of_psp22_to_give -= psp22_fee_amount_to_give;

            amount_of_a0_to_give -= a0_fee_amount_to_give;

            Ok((amount_of_a0_to_give, amount_of_psp22_to_give)) // Return the calculated amounts of A0 and PSP22 tokens to give to the caller.
        }

        /// function to get the amount of withdrawable PSP22 and A0 by given number of LP shares with LP fees.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount_with_lp(
            &self,
            shares_amount: Balance,
        ) -> Result<(Balance, Balance), TradingPairErrors> {
            let amount_of_a0_to_give: Balance;

            let actual_a0_balance = self.get_a0_balance();

            // calculating the amount of A0 to give to the caller.
            match (shares_amount * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let actual_psp22_balance = self.get_psp22_balance();

            let amount_of_psp22_to_give: Balance;

            // calculating the amount of PSP22 to give to the caller.
            match (shares_amount * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok((amount_of_a0_to_give, amount_of_psp22_to_give))
        }

        /// function to get the amount of withdrawable pooled PSP22 tokens by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_psp22_withdraw_tokens_amount(
            &self,
            shares_amount: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let amount_of_psp22_to_give: U256;

            let actual_psp22_balance = self.get_psp22_balance();

            // calculating the amount of PSP22 to give to the caller.
            match (U256::from(shares_amount) * U256::from(actual_psp22_balance))
                .checked_div(U256::from(self.total_supply))
            {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let amount_of_psp22_to_give_u128 = amount_of_psp22_to_give.as_u128();

            Ok(amount_of_psp22_to_give_u128)
        }

        /// function to get the amount of PSP22 LP fee tokens by number of shares
        #[ink(message)]
        pub fn get_psp22_lp_fee_tokens(
            &mut self,
            shares_amount: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let amount_of_psp22_fees_to_give: U256;

            // calculating the amount of PSP22 to give to the caller.
            match (U256::from(shares_amount) * U256::from(self.psp22_lp_fee_vault))
                .checked_div(U256::from(self.total_supply))
            {
                Some(result) => {
                    amount_of_psp22_fees_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let amount_of_psp22_to_give_u128 = amount_of_psp22_fees_to_give.as_u128();

            Ok(amount_of_psp22_to_give_u128)
        }

        /// function to get the percentage difference between the PSP22 pooled tokens without LP fee and with LP fees
        #[ink(message)]
        pub fn get_psp22_difference_by_percentage(&mut self) -> Result<Balance, TradingPairErrors> {
            // caller address
            let caller = self.env().caller();

            // caller total LP shares
            let caller_shares: Balance = self.balances.get(&caller).unwrap_or(0);

            let amount_of_psp22_fees: Balance =
                self.get_psp22_lp_fee_tokens(caller_shares).unwrap();

            // amount of PSP22 to give to the caller
            let psp22_amount_without_fees = self
                .get_psp22_withdraw_tokens_amount(caller_shares)
                .unwrap();

            let psp22_amount_with_fees = psp22_amount_without_fees + amount_of_psp22_fees;

            let percentage_diff: Balance = self
                .check_difference(psp22_amount_without_fees, psp22_amount_with_fees)
                .unwrap();

            Ok(percentage_diff)
        }

        /// function to get the amount of withdrawable pooled AZERO coins by given number of LP shares without LP fees.
        #[ink(message)]
        pub fn get_a0_withdraw_tokens_amount(
            &self,
            shares_amount: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let amount_of_a0_to_give: U256;

            let actual_a0_balance = self.get_a0_balance();

            // calculating the amount of A0 to give to the caller.
            match (U256::from(shares_amount) * U256::from(actual_a0_balance))
                .checked_div(U256::from(self.total_supply))
            {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let amount_of_a0_to_give_u128 = amount_of_a0_to_give.as_u128();

            Ok(amount_of_a0_to_give_u128)
        }

        /// function to get the amount of A0 LP fee tokens by number of shares
        #[ink(message)]
        pub fn get_a0_lp_fee_tokens(
            &self,
            shares_amount: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let amount_of_a0_fees_to_give: Balance;

            // calculating the amount of LP fee A0 to give to the caller.
            match (shares_amount * self.azero_lp_fee_vault).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_fees_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(amount_of_a0_fees_to_give)
        }

        /// function to get the percentage difference between the AZERO pooled coins without LP fee and with LP fees
        #[ink(message)]
        pub fn get_a0_difference_by_percentage(&mut self) -> Result<Balance, TradingPairErrors> {
            // caller address
            let caller = self.env().caller();

            // caller total LP shares
            let caller_shares: Balance = self.balances.get(&caller).unwrap_or(0);

            let amount_of_a0_fees: Balance = self.get_a0_lp_fee_tokens(caller_shares).unwrap();

            // amount of PSP22 to give to the caller
            let a0_amount_without_fees = self.get_a0_withdraw_tokens_amount(caller_shares).unwrap();

            let a0_amount_with_fees = a0_amount_without_fees + amount_of_a0_fees;

            let percentage_diff: Balance = self
                .check_difference(a0_amount_without_fees, a0_amount_with_fees)
                .unwrap();

            Ok(percentage_diff)
        }

        /// function to get the callers pooled PSP22 and A0.
        #[ink(message)]
        pub fn get_account_locked_tokens(
            &self,
            account_id: AccountId,
        ) -> Result<(Balance, Balance), TradingPairErrors> {
            // caller address
            let caller = account_id;
            // get caller LP tokens
            let caller_shares: Balance = self.balances.get(&caller).unwrap_or(0);

            let mut amount_of_a0_to_give: Balance = 0;

            let mut amount_of_psp22_to_give: Balance = 0;

            if caller_shares <= 0 {
                return Ok((amount_of_psp22_to_give, amount_of_a0_to_give))
            }

            let actual_a0_balance = self.get_a0_balance();

            // calculating the amount of A0 to give to the caller.
            match (caller_shares * actual_a0_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_a0_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let mut actual_psp22_balance = self.get_psp22_balance();

            match actual_psp22_balance.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_psp22_balance = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            // calculating the amount of PSP22 to give to the caller.
            match (caller_shares * actual_psp22_balance).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let actual_amount_of_psp22_to_give: Balance;

            match amount_of_psp22_to_give.checked_mul(10u128.pow(12)) {
                Some(result) => {
                    actual_amount_of_psp22_to_give = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok((actual_amount_of_psp22_to_give, amount_of_a0_to_give))
        }

        // function to get the expected amount of LP shares by given A0 amount.
        #[ink(message)]
        pub fn get_expected_lp_token_amount(
            &self,
            a0_deposit_amount: Balance,
            psp22_deposit_amount: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let mut shares: Balance = 0;

            // if its the trading pair first deposit
            if self.total_supply == 0 {
                // calculating the amount of shares to give to the provider if its the first LP deposit overall
                shares = a0_deposit_amount * psp22_deposit_amount;
                match shares.checked_div(10u128.pow(12)) {
                    Some(result) => {
                        shares = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };
            }

            // if its not the first LP deposit
            if self.total_supply > 0 {
                // calculating the amount of shares to give to the provider if its not the first LP deposit
                match (a0_deposit_amount * self.total_supply).checked_div(self.get_a0_balance()) {
                    Some(result) => {
                        shares = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };
            }

            Ok(shares)
        }

        /// function to get the amount of A0 the caller will get for 1 PSP22 token.
        #[ink(message)]
        pub fn get_price_for_one_psp22(&self) -> Result<Balance, TradingPairErrors> {
            let amount_out = self
                .get_est_price_psp22_to_a0(1u128 * (10u128.pow(12)))
                .unwrap();

            Ok(amount_out)
        }

        /// function to get the amount of A0 the caller will get for given PSP22 amount.
        #[ink(message)]
        pub fn get_est_price_psp22_to_a0(
            &self,
            psp22_amount_in: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let caller = self.env().caller();

            // fetching caller current PSP22 balance
            let caller_current_balance: Balance = PSP22Ref::balance_of(&self.panx_contract, caller);

            let mut psp22_amount_in_with_lp_fees: U256 = U256::from(0);

            // reducting the LP fee from the PSP22 amount in
            match U256::from(psp22_amount_in)
                .checked_mul(U256::from(100u128 * 10u128.pow(12)) - (self.fee))
            {
                Some(result) => {
                    psp22_amount_in_with_lp_fees = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let tokens_to_validate: Balance = 3500u128 * 10u128.pow(12);

            // validating if caller has more than 3500 PANX to verify if the caller is eligible for the incentive program
            if caller_current_balance >= tokens_to_validate {
                if self.fee <= 1400000000000u128 {
                    // reducting HALF of the LP fee from PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match U256::from(psp22_amount_in)
                        .checked_mul(U256::from(100u128 * 10u128.pow(12)) - (self.fee / 2u128))
                    {
                        Some(result) => {
                            psp22_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }

                if self.fee > 1400000000000u128 {
                    // reducting (LP fee - 1) of the LP fee from PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match U256::from(psp22_amount_in).checked_mul(
                        U256::from(100u128 * 10u128.pow(12))
                            - (self.fee - (1u128 * 10u128.pow(12))),
                    ) {
                        Some(result) => {
                            psp22_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }
            }

            psp22_amount_in_with_lp_fees = psp22_amount_in_with_lp_fees / 10u128.pow(12);

            let mut numerator: U256 = U256::from(0);
            let mut denominator: U256 = U256::from(0);
            let a0_amount_out: Balance;

            match U256::from(psp22_amount_in_with_lp_fees)
                .checked_mul(U256::from(self.get_a0_balance()))
            {
                Some(result) => {
                    numerator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match (U256::from(self.get_psp22_balance()) * U256::from(100))
                .checked_add(U256::from(psp22_amount_in_with_lp_fees))
            {
                Some(result) => {
                    denominator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match numerator.checked_div(denominator) {
                Some(result) => {
                    a0_amount_out = result.as_u128();
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(a0_amount_out)
        }

        /// function to get the amount of PSP22 the caller will get for given A0 amount (swap use)
        #[ink(message, payable)]
        pub fn get_est_price_a0_to_psp22_for_swap(
            &self,
            a0_amout_in: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let caller = self.env().caller();

            let a0_reserve_before: Balance;

            // calculating the A0 contract reserve before the transaction
            match self.get_a0_balance().checked_sub(a0_amout_in) {
                Some(result) => {
                    a0_reserve_before = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let caller_current_balance: Balance = PSP22Ref::balance_of(&self.panx_contract, caller);

            let mut a0_amount_in_with_lp_fees: U256 = U256::from(0);

            // reducting the LP fee from the A0 amount in
            match U256::from(a0_amout_in)
                .checked_mul(U256::from(100u128 * 10u128.pow(12)) - self.fee)
            {
                Some(result) => {
                    a0_amount_in_with_lp_fees = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let tokens_to_validate = 3500u128 * 10u128.pow(12);

            // validating if the caller has more than 3500 PANX
            if caller_current_balance >= tokens_to_validate {
                if self.fee <= 1400000000000 {
                    // reducting HALF of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match U256::from(a0_amout_in)
                        .checked_mul(U256::from(100u128 * 10u128.pow(12)) - (self.fee / 2u128))
                    {
                        Some(result) => {
                            a0_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }

                if self.fee > 1400000000000 {
                    // reducting (LP fee - 1) of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match U256::from(a0_amout_in).checked_mul(
                        U256::from(100u128 * 10u128.pow(12))
                            - (self.fee - (1u128 * 10u128.pow(12))),
                    ) {
                        Some(result) => {
                            a0_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }
            }

            a0_amount_in_with_lp_fees = a0_amount_in_with_lp_fees / 10u128.pow(12);

            let mut numerator: U256 = U256::from(0);
            let mut denominator: U256 = U256::from(0);
            let a0_amount_out: Balance;

            match U256::from(a0_amount_in_with_lp_fees)
                .checked_mul(U256::from(self.get_psp22_balance()))
            {
                Some(result) => {
                    numerator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match (U256::from(a0_reserve_before) * U256::from(100))
                .checked_add(U256::from(a0_amount_in_with_lp_fees))
            {
                Some(result) => {
                    denominator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match numerator.checked_div(denominator) {
                Some(result) => {
                    a0_amount_out = result.as_u128();
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(a0_amount_out)
        }

        /// function to get the amount of PSP22 the caller will get for given A0 amount (front-end use)
        #[ink(message, payable)]
        pub fn get_est_price_a0_to_psp22(
            &self,
            a0_amount_in: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let caller = self.env().caller();

            let caller_current_balance: Balance = PSP22Ref::balance_of(&self.panx_contract, caller);

            let mut a0_amount_in_with_lp_fees: U256 = U256::from(0);

            // reducting the LP fee from the A0 amount in
            match U256::from(a0_amount_in)
                .checked_mul(U256::from(100u128 * 10u128.pow(12)) - self.fee)
            {
                Some(result) => {
                    a0_amount_in_with_lp_fees = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let tokens_to_validate = 3500u128 * 10u128.pow(12);

            // validating if the caller has more than 3500 PANX
            if caller_current_balance >= tokens_to_validate {
                if self.fee <= 1400000000000 {
                    // reducting HALF of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                    match U256::from(a0_amount_in)
                        .checked_mul(U256::from(100u128 * 10u128.pow(12)) - (self.fee / 2u128))
                    {
                        Some(result) => {
                            a0_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }

                if self.fee > 1400000000000 {
                    // reducting (LP fee - 1) of the LP fee from the A0 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                    match U256::from(a0_amount_in).checked_mul(
                        U256::from(100u128 * 10u128.pow(12))
                            - (self.fee - (1u128 * 10u128.pow(12))),
                    ) {
                        Some(result) => {
                            a0_amount_in_with_lp_fees = result;
                        }
                        None => return Err(TradingPairErrors::Overflow),
                    };
                }
            }

            a0_amount_in_with_lp_fees = a0_amount_in_with_lp_fees / 10u128.pow(12);

            let mut numerator: U256 = U256::from(0);
            let mut denominator: U256 = U256::from(0);

            let a0_amount_out: Balance;

            match U256::from(a0_amount_in_with_lp_fees)
                .checked_mul(U256::from(self.get_psp22_balance()))
            {
                Some(result) => {
                    numerator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match (U256::from(self.get_a0_balance()) * U256::from(100))
                .checked_add(U256::from(a0_amount_in_with_lp_fees))
            {
                Some(result) => {
                    denominator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match numerator.checked_div(denominator) {
                Some(result) => {
                    a0_amount_out = result.as_u128();
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(a0_amount_out)
        }

        /// function to get the estimated price impact for given psp22 token amount
        #[ink(message)]
        pub fn get_price_impact_psp22_to_a0(
            &self,
            psp22_amount_in: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            // fetching the amount of A0 the caller WOULD get if he would swap
            let current_amount_out = self.get_est_price_psp22_to_a0(psp22_amount_in).unwrap();

            let mut psp22_amount_in_with_lp_fees: U256 = U256::from(0);

            // reducting the LP fee from the PSP22 amount in
            match U256::from(psp22_amount_in)
                .checked_mul(U256::from(100u128 * 10u128.pow(12)) - (self.fee))
            {
                Some(result) => {
                    psp22_amount_in_with_lp_fees = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            psp22_amount_in_with_lp_fees = psp22_amount_in_with_lp_fees / 10u128.pow(12);

            let mut numerator: U256 = U256::from(0);
            let mut denominator: U256 = U256::from(0);
            let future_a0_amount_out: Balance;

            match U256::from(psp22_amount_in_with_lp_fees)
                .checked_mul(U256::from(self.get_a0_balance() - current_amount_out))
            {
                Some(result) => {
                    numerator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match (U256::from(self.get_psp22_balance() + psp22_amount_in) * U256::from(100))
                .checked_add(U256::from(psp22_amount_in_with_lp_fees))
            {
                Some(result) => {
                    denominator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match numerator.checked_div(denominator) {
                Some(result) => {
                    future_a0_amount_out = result.as_u128();
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(future_a0_amount_out)
        }

        /// function to get the estimated price impact for given A0 amount
        #[ink(message)]
        pub fn get_price_impact_a0_to_psp22(
            &mut self,
            a0_amount_in: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let current_amount_out = self.get_est_price_a0_to_psp22(a0_amount_in).unwrap();

            let mut a0_amount_in_with_lp_fees: U256 = U256::from(0);

            // reducting the LP fee from the A0 amount in
            match U256::from(a0_amount_in)
                .checked_mul(U256::from(100u128 * 10u128.pow(12)) - self.fee)
            {
                Some(result) => {
                    a0_amount_in_with_lp_fees = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            a0_amount_in_with_lp_fees = a0_amount_in_with_lp_fees / 10u128.pow(12);

            let mut numerator: U256 = U256::from(0);
            let mut denominator: U256 = U256::from(0);
            let future_psp22_amount_out: Balance;

            match U256::from(a0_amount_in_with_lp_fees)
                .checked_mul(U256::from(self.get_psp22_balance() - current_amount_out))
            {
                Some(result) => {
                    numerator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match (U256::from(self.get_a0_balance() + a0_amount_in) * U256::from(100))
                .checked_add(U256::from(a0_amount_in_with_lp_fees))
            {
                Some(result) => {
                    denominator = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            match numerator.checked_div(denominator) {
                Some(result) => {
                    future_psp22_amount_out = result.as_u128();
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            Ok(future_psp22_amount_out)
        }

        /// function to swap PSP22 to A0
        #[ink(message)]
        pub fn swap_psp22(
            &mut self,
            psp22_amount_to_transfer: Balance,
            a0_amount_to_validate: Balance,
            slippage: Balance,
        ) -> Result<(), TradingPairErrors> {
            let caller = self.env().caller();

            let contract_a0_current_balance = self.get_a0_balance();

            // making sure that the contract has more than 0 A0 coins.
            if contract_a0_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfA0)
            }

            let contract_psp22_current_balance: Balance = self.get_psp22_balance();

            // making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP22)
            }

            let caller_current_balance: Balance = PSP22Ref::balance_of(&self.psp22_token, caller);

            // making sure that the caller has more or equal the amount he wishes to transfers.
            if caller_current_balance < psp22_amount_to_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance)
            }

            let contract_allowance: Balance =
                PSP22Ref::allowance(&self.psp22_token, caller, Self::env().account_id());

            // making sure that the trading pair contract has enough allowance.
            if contract_allowance < psp22_amount_to_transfer {
                return Err(TradingPairErrors::NotEnoughAllowance)
            }

            // the amount of A0 to give to the caller before traders fee.
            let a0_amount_out_for_caller_before_traders_fee: Balance = self
                .get_est_price_psp22_to_a0(psp22_amount_to_transfer)
                .unwrap();

            // percentage dif between given A0 amount (from front-end) and acutal final AO amount
            let percentage_diff: Balance = self
                .check_difference(
                    a0_amount_to_validate,
                    a0_amount_out_for_caller_before_traders_fee,
                )
                .unwrap();

            // validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance)
            }

            let actual_a0_amount_out_for_caller: Balance;

            let a0_amount_out_for_vault: Balance;

            // calculating the amount of A0 coins to allocate to the vault account
            match (a0_amount_out_for_caller_before_traders_fee * self.traders_fee)
                .checked_div(1000u128)
            {
                Some(result) => {
                    a0_amount_out_for_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let actual_lp_fee: Balance;

            // calculating the actual LP fee
            match (self.fee / (10u128.pow(12))).checked_mul(10) {
                Some(result) => {
                    actual_lp_fee = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let a0_amount_out_for_lp_vault: Balance;

            // calculating the amount of A0 coins to allocate to the lp vault
            match (a0_amount_out_for_caller_before_traders_fee * actual_lp_fee)
                .checked_div(1000u128)
            {
                Some(result) => {
                    a0_amount_out_for_lp_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let new_azero_lp_fee_vault: Balance;

            match self
                .azero_lp_fee_vault
                .checked_add(a0_amount_out_for_lp_vault)
            {
                Some(result) => {
                    new_azero_lp_fee_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            self.azero_lp_fee_vault = new_azero_lp_fee_vault;

            let new_contract_overall_generated_azero_fee: Balance;

            match self
                .contract_overall_generated_azero_fee
                .checked_add(a0_amount_out_for_lp_vault)
            {
                Some(result) => {
                    new_contract_overall_generated_azero_fee = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            self.contract_overall_generated_azero_fee = new_contract_overall_generated_azero_fee;

            // calculating the final amount of A0 coins to give to the caller after reducing traders fee
            match a0_amount_out_for_caller_before_traders_fee
                .checked_sub(a0_amount_out_for_vault + a0_amount_out_for_lp_vault)
            {
                Some(result) => {
                    actual_a0_amount_out_for_caller = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let psp22_amount_out_for_vault: Balance;

            // calculating the amount of PSP22 tokens to allocate to the vault account
            match (psp22_amount_to_transfer * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    psp22_amount_out_for_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            // cross contract call to psp22 contract to transfer psp22 token to the Pair contract
            if PSP22Ref::transfer_from_builder(
                &self.psp22_token,
                caller,
                Self::env().account_id(),
                psp22_amount_to_transfer,
                vec![],
            )
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFromFailed)
            }

            let caller_balance_after_transfer: Balance =
                PSP22Ref::balance_of(&self.psp22_token, caller);

            if caller_current_balance == caller_balance_after_transfer {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance)
            }

            // cross contract call to PSP22 contract to transfer PSP22 to the vault
            if PSP22Ref::transfer(
                &self.psp22_token,
                self.vault,
                psp22_amount_out_for_vault,
                vec![],
            )
            .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFailed)
            }

            // function to transfer A0 to the caller.
            if self
                .env()
                .transfer(caller, actual_a0_amount_out_for_caller)
                .is_err()
            {
                return Err(TradingPairErrors::A0TransferFailed)
            }

            // function to transfer A0 to the vault.
            if self
                .env()
                .transfer(self.vault, a0_amount_out_for_vault)
                .is_err()
            {
                return Err(TradingPairErrors::A0TransferFailed)
            }

            // increase num of trans
            self.transasction_number = self.transasction_number + 1;
            Self::env().emit_event(PSP22Swap {
                caller,
                psp22_deposited_amount: psp22_amount_to_transfer,
                a0_given_amount: actual_a0_amount_out_for_caller,
                a0_given_to_vault: a0_amount_out_for_vault,
            });

            Ok(())
        }

        /// function to swap A0 to PSP22
        #[ink(message, payable)]
        pub fn swap_a0(
            &mut self,
            psp22_amount_to_validate: Balance,
            slippage: Balance,
        ) -> Result<(), TradingPairErrors> {
            let caller = self.env().caller();

            let contract_a0_current_balance = self.get_a0_balance();

            // making sure that the contract has more than 0 A0 coins.
            if contract_a0_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfA0)
            }

            let contract_psp22_current_balance: Balance = self.get_psp22_balance();

            // making sure that the contract has more than 0 PSP22 tokens.
            if contract_psp22_current_balance <= 0 {
                return Err(TradingPairErrors::ContractOutOfPSP22)
            }

            // amount of PSP22 tokens to give to caller before traders fee.
            let psp22_amount_out_for_caller_before_traders_fee: Balance = self
                .get_est_price_a0_to_psp22_for_swap(self.env().transferred_value())
                .unwrap();

            // percentage dif between given PSP22 amount (from front-end) and the acutal final PSP22 amount.
            let percentage_diff: Balance = self
                .check_difference(
                    psp22_amount_to_validate,
                    psp22_amount_out_for_caller_before_traders_fee,
                )
                .unwrap();

            // validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                return Err(TradingPairErrors::SlippageTolerance)
            }

            let psp22_amount_out_for_vault: Balance;

            let actual_psp22_amount_out_for_caller: Balance;

            // calculating the amount of PSP22 tokens to allocate to the vault account
            match (psp22_amount_out_for_caller_before_traders_fee * self.traders_fee)
                .checked_div(1000u128)
            {
                Some(result) => {
                    psp22_amount_out_for_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let actual_lp_fee: Balance;

            // calculating the actual LP fee
            match (self.fee / (10u128.pow(12))).checked_mul(10) {
                Some(result) => {
                    actual_lp_fee = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let psp22_amount_out_for_lp_vault: Balance;

            // calculating the amount of PSP22 tokens to allocate to the lp vault
            match (psp22_amount_out_for_caller_before_traders_fee * actual_lp_fee)
                .checked_div(1000u128)
            {
                Some(result) => {
                    psp22_amount_out_for_lp_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let new_psp22_lp_fee_vault: Balance;

            match self
                .psp22_lp_fee_vault
                .checked_add(psp22_amount_out_for_lp_vault)
            {
                Some(result) => {
                    new_psp22_lp_fee_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            self.psp22_lp_fee_vault = new_psp22_lp_fee_vault;

            let new_contract_overall_generated_psp22_fee: Balance;

            match self
                .contract_overall_generated_psp22_fee
                .checked_add(psp22_amount_out_for_lp_vault)
            {
                Some(result) => {
                    new_contract_overall_generated_psp22_fee = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            self.contract_overall_generated_psp22_fee = new_contract_overall_generated_psp22_fee;

            // calculating the final amount of PSP22 tokens to give to the caller after reducing traders fee
            match psp22_amount_out_for_caller_before_traders_fee
                .checked_sub(psp22_amount_out_for_vault + psp22_amount_out_for_lp_vault)
            {
                Some(result) => {
                    actual_psp22_amount_out_for_caller = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            let a0_amount_out_for_vault: Balance;

            // calculating the amount of A0 coins to allocate to the vault account
            match (self.env().transferred_value() * self.traders_fee).checked_div(1000u128) {
                Some(result) => {
                    a0_amount_out_for_vault = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            // cross contract call to PSP22 contract to transfer PSP22 to the caller
            if PSP22Ref::transfer(
                &self.psp22_token,
                caller,
                actual_psp22_amount_out_for_caller,
                vec![],
            )
            .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFailed)
            }

            // cross contract call to PSP22 contract to transfer PSP22 to the vault
            if PSP22Ref::transfer(
                &self.psp22_token,
                self.vault,
                psp22_amount_out_for_vault,
                vec![],
            )
            .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFailed)
            }

            // function to transfer A0 to the vault.
            if self
                .env()
                .transfer(self.vault, a0_amount_out_for_vault)
                .is_err()
            {
                return Err(TradingPairErrors::A0TransferFailed)
            }

            // increase num of trans
            self.transasction_number = self.transasction_number + 1;

            Self::env().emit_event(A0Swap {
                caller,
                a0_deposited_amount: self.env().transferred_value(),
                psp22_given_amount: actual_psp22_amount_out_for_caller,
                psp22_given_to_vault: psp22_amount_out_for_vault,
            });

            Ok(())
        }

        /// function to add caller to the LP incentive program
        fn update_incentive_program(&mut self, caller: AccountId) -> Result<(), TradingPairErrors> {
            let account_shares_balance: Balance = self.balances.get(&caller).unwrap_or(0);

            // amount of PSP22 to give to the caller without LP fee
            let caller_locked_psp22_balance = self
                .get_psp22_withdraw_tokens_amount(account_shares_balance)
                .unwrap();

            // calc how many tokens to give in a day
            let psp22_amount_to_give_each_day: Balance;

            // calculating the amount of daily PSP22 to give to the user
            match ((caller_locked_psp22_balance * self.staking_percentage) / 100u128)
                .checked_div(365)
            {
                Some(result) => {
                    psp22_amount_to_give_each_day = result;
                }
                None => return Err(TradingPairErrors::Overflow),
            };

            // insert the daily amount of PSP22 tokens and AZERO to give to the caller
            self.psp22_to_give_in_a_day
                .insert(caller, &psp22_amount_to_give_each_day);

            self.last_redeemed
                .insert(caller, &self.get_current_timestamp());

            Ok(())
        }

        /// function to get caller redeemable amount of pooled PSP22
        #[ink(message)]
        pub fn get_psp22_redeemable_amount(&mut self) -> Result<Balance, TradingPairErrors> {
            // call address
            let caller = self.env().caller();
            // current timestamp
            let current_tsp = self.get_current_timestamp();

            let account_shares_balance: Balance = self.balances.get(&caller).unwrap_or(0);

            // amount of PSP22 to give to the caller without LP fee
            let caller_locked_psp22_balance = self
                .get_psp22_withdraw_tokens_amount(account_shares_balance)
                .unwrap_or(0);

            // last time caller redeemed tokens
            let last_redeemed: u64 = self.last_redeemed.get(caller).unwrap_or(0);

            // the amount of daily PSP22 tokens to give ot the caller
            let psp22_to_give_each_day: Balance =
                self.psp22_to_give_in_a_day.get(caller).unwrap_or(0);

            // Declare a variable to hold the difference in days between current timestamp and last redeemed timestamp
            let days_difference: u64;

            // Calculate the difference in days by dividing the difference between current timestamp and last redeemed timestamp by 86400 (number of seconds in a day)
            match (current_tsp - last_redeemed).checked_div(86400) {
                Some(result) => {
                    days_difference = result;
                }
                None => {
                    // If the division results in overflow, return an error of StakingErrors::Overflow
                    return Err(TradingPairErrors::Overflow)
                }
            };

            // making sure that caller has more then 0 pooled PSP22 tokens
            if caller_locked_psp22_balance <= 0 {
                return Err(TradingPairErrors::CallerInsufficientPSP22Balance)
            }

            // making sure that caller has more than 0 daily PSP22 tokens to claim
            if psp22_to_give_each_day <= 0 {
                return Err(TradingPairErrors::ZeroDailyPSP22)
            }

            // The amount of PSP22 tokens and AZERO to give to the caller
            let psp22_redeemable_amount: Balance = psp22_to_give_each_day * days_difference as u128;

            Ok(psp22_redeemable_amount)
        }

        /// function for caller to redeem LP incentive tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(&mut self) -> Result<(), TradingPairErrors> {
            // caller address
            let caller = self.env().caller();
            // caller timestamp
            let current_tsp = self.get_current_timestamp();

            let psp22_redeemable_amount = self.get_psp22_redeemable_amount().unwrap_or(0);

            // cross contract call to PSP22 contract to transfer PSP22 to caller
            if PSP22Ref::transfer(&self.psp22_token, caller, psp22_redeemable_amount, vec![])
                .is_err()
            {
                return Err(TradingPairErrors::PSP22TransferFailed)
            }

            let current_account_overall_psp22_staking_rewards = self
                .account_overall_staking_rewards
                .get(&caller)
                .unwrap_or(0);

            self.account_overall_staking_rewards.insert(
                &caller,
                &(current_account_overall_psp22_staking_rewards + psp22_redeemable_amount),
            );

            // Making sure to set his last redeem to current timestamp
            self.last_redeemed.insert(caller, &current_tsp);

            Ok(())
        }

        /// function to reduce the incentive program rewards allocation after LP removal.
        fn remove_lp(&mut self, new_shares: Balance) -> Result<(), TradingPairErrors> {
            // caller address
            let caller = self.env().caller();

            if new_shares == 0 {
                if self.redeem_redeemable_amount().is_err() {
                    return Err(TradingPairErrors::RemoveLpIncentiveProgramError)
                }

                // insert the daily amount of PSP22 and AZERO tokens to give to the caller
                self.psp22_to_give_in_a_day.insert(caller, &0);
            }

            if new_shares > 0 {
                if self.redeem_redeemable_amount().is_err() {
                    return Err(TradingPairErrors::RemoveLpIncentiveProgramError)
                }

                // amount of pooled PSP22 tokens by number of LP shares with LP fee
                let caller_locked_psp22_balance =
                    self.get_psp22_withdraw_tokens_amount(new_shares).unwrap();

                let new_psp22_amount_to_give_each_day: Balance;

                // calculating the amount of daily PSP22 to give to the user
                match ((caller_locked_psp22_balance * self.staking_percentage) / 100u128)
                    .checked_div(365)
                {
                    Some(result) => {
                        new_psp22_amount_to_give_each_day = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };

                // insert the daily amount of PSP22 and AZERO tokens to give to the caller
                self.psp22_to_give_in_a_day
                    .insert(caller, &new_psp22_amount_to_give_each_day);
            }

            Ok(())
        }

        /// function to get the amount of tokens to give to caller each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_caller(&mut self, caller: AccountId) -> Balance {
            let psp22_daily_amount: Balance = self.psp22_to_give_in_a_day.get(&caller).unwrap_or(0);

            psp22_daily_amount
        }

        #[ink(message)]
        pub fn get_generated_lp_fees(&mut self) -> (Balance, Balance) {
            let psp22_lp_fees: Balance = self.psp22_lp_fee_vault;
            let azero_lp_fees: Balance = self.azero_lp_fee_vault;

            (psp22_lp_fees, azero_lp_fees)
        }

        #[ink(message)]
        pub fn get_account_overall_staking_rewards(&self, owner: AccountId) -> Balance {
            let psp22_overall_amount = self
                .account_overall_staking_rewards
                .get(&owner)
                .unwrap_or(0);

            psp22_overall_amount
        }

        #[ink(message)]
        pub fn get_account_overall_lp_fee_rewards(&self, owner: AccountId) -> (Balance, Balance) {
            let (psp22_overall_amount, azero_overall_amount) = self
                .account_overall_lp_fee_rewards
                .get(&owner)
                .unwrap_or((0, 0));

            (psp22_overall_amount, azero_overall_amount)
        }

        // function to get the contract's overall generated LP fees
        #[ink(message)]
        pub fn get_contract_overall_generated_fee(&mut self) -> (Balance, Balance) {
            let psp22_lp_fees: Balance = self.contract_overall_generated_psp22_fee;
            let azero_lp_fees: Balance = self.contract_overall_generated_azero_fee;

            (psp22_lp_fees, azero_lp_fees)
        }

        /// function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }

        #[ink(message)]
        pub fn get_deployer_account(&self) -> AccountId {
            self.deployer
        }

        /// function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_caller_id(&self) -> AccountId {
            self.env().caller()
        }

        /// function to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
            let current_price = self
                .get_est_price_psp22_to_a0(1u128 * 10u128.pow(12))
                .unwrap();

            current_price
        }

        /// function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }

        /// function to get trading contract AZERO balance
        #[ink(message)]
        pub fn get_a0_balance(&self) -> Balance {
            let a0_balance = self.env().balance();
            a0_balance
        }

        /// function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(&self, account: AccountId) -> Balance {
            self.balances.get(&account).unwrap_or(0)
        }

        // function to get contract PSP22 reserve (self)
        #[ink(message)]
        pub fn get_psp22_balance(&self) -> Balance {
            let psp22_balance: Balance =
                PSP22Ref::balance_of(&self.psp22_token, Self::env().account_id());
            psp22_balance
        }

        /// function to get current fee
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            let fee: Balance = self.fee;
            fee
        }

        // function to get the total number of swaps
        #[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number
        }

        /// function to calculate the percentage between values.
        #[ink(message, payable)]
        pub fn check_difference(
            &mut self,
            value1: Balance,
            value2: Balance,
        ) -> Result<Balance, TradingPairErrors> {
            let mut percentage_difference: Balance = 0;

            if value1 > value2 {
                percentage_difference = (value1 - value2) * (10u128.pow(12));
                match (percentage_difference / value2).checked_mul(100u128) {
                    Some(result) => {
                        percentage_difference = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };
            }

            if value2 > value1 {
                percentage_difference = (value2 - value1) * (10u128.pow(12));
                match (percentage_difference / value1).checked_mul(100u128) {
                    Some(result) => {
                        percentage_difference = result;
                    }
                    None => return Err(TradingPairErrors::Overflow),
                };
            }

            Ok(percentage_difference)
        }

        #[ink(message)]
        pub fn get_psp22_amount_for_lp(
            &self,
            a0_deposit_amount: Balance,
            a0_contract_balance: Balance,
        ) -> Balance {
            let psp22_amount_to_deposit = ((self.get_psp22_balance() * (10u128.pow(12)))
                / a0_contract_balance
                * a0_deposit_amount)
                / (10u128.pow(12));
            psp22_amount_to_deposit
        }

        #[ink(message)]
        pub fn get_a0_amount_for_lp(
            &self,
            psp22_deposit_amount: Balance,
            a0_contract_balance: Balance,
        ) -> Balance {
            let a0_amount_to_deposit = ((a0_contract_balance * (10u128.pow(12)))
                / self.get_psp22_balance()
                * psp22_deposit_amount)
                / (10u128.pow(12));

            a0_amount_to_deposit
        }

        /// function to get current timpstamp in seconds
        #[ink(message)]
        pub fn get_current_timestamp(&self) -> u64 {
            let time_stamp_in_seconds = self.env().block_timestamp() / 1000;
            time_stamp_in_seconds
        }

        /// function to get LP lock timestamp
        #[ink(message)]
        pub fn get_lp_lock_timestamp(&self) -> u64 {
            self.lp_lock_timestamp
        }

        /// function to get LP lock timestamp
        fn _min(&self, value1: Balance, value2: Balance) -> Balance {
            if value1 < value2 {
                return value1
            } else {
                return value2
            }
        }
    }

    /// ink! end-to-end (E2E) tests
    ///
    /// cargo test --features e2e-tests -- --nocapture
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        use my_psp22::my_psp22::MyPsp22Ref;
        use openbrush::{
            contracts::psp22::{
                extensions::metadata::*,
                psp22_external::PSP22,
            },
            traits::Storage,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Helper to get Bob's account_id from `ink_e2e::bob()` PairSigner
        fn get_bob_account_id() -> AccountId {
            let bob = ink_e2e::bob::<ink_e2e::PolkadotConfig>();
            let bob_account_id_32 = bob.account_id();
            let bob_account_id = AccountId::try_from(bob_account_id_32.as_ref()).unwrap();

            bob_account_id
        }

        fn get_alice_account_id() -> AccountId {
            let alice = ink_e2e::alice::<ink_e2e::PolkadotConfig>();
            let alice_account_id_32 = alice.account_id();
            let alice_account_id = AccountId::try_from(alice_account_id_32.as_ref()).unwrap();

            alice_account_id
        }

        fn get_charlie_account_id() -> AccountId {
            let charlie = ink_e2e::charlie::<ink_e2e::PolkadotConfig>();
            let charlie_account_id_32 = charlie.account_id();
            let charlie_account_id = AccountId::try_from(charlie_account_id_32.as_ref()).unwrap();

            charlie_account_id
        }

        /// Tests included in "provide_to_pool_works":
        /// 1. provide_to_pool
        /// 2. get_a0_balance
        /// 3. get_psp22_balance
        /// 4. get_amount_to_give_each_day_to_caller
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn provide_to_pool_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Create a new instance of MyPsp22Ref contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate MyPsp22Ref contract and get the account ID
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a new instance of TradingPairAzeroRef contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate TradingPairAzeroRef contract and get the account ID
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build an `approve` message for MyPsp22Ref contract to approve TradingPairAzeroRef contract to use tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call `approve_psp22_to_provide_lp` message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build a `provide_to_pool` message for TradingPairAzeroRef contract to provide liquidity to the pool
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            let amount: u128 = 10000000000000;

            // Call `provide_to_tpa` message
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a `get_a0_balance` message for TradingPairAzeroRef contract to get the balance of a0
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call `get_a0_balance` message
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            // Assert the returned balance of a0
            assert_eq!(get_a0_res.return_value(), 10001000000000);

            // Build a `get_psp22_balance` message for TradingPairAzeroRef contract to get the balance of psp22
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call `get_psp22_balance` message
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_res failed");

            // Assert the returned balance of psp22
            assert_eq!(get_psp22_res.return_value(), 100000000000000, "get_res");

            // Build a `get_amount_to_give_each_day_to_caller` message for TradingPairAzeroRef contract
            // to get the amount to give each day to the caller
            let get_amount_to_give_each_day_to_caller = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_amount_to_give_each_day_to_caller(get_alice_account_id())
            });

            // Call `get_amount_to_give_each_day_to_caller` message
            let get_amount_to_give_each_day_to_caller_res = client
                .call(
                    &ink_e2e::alice(),
                    get_amount_to_give_each_day_to_caller,
                    0,
                    None,
                )
                .await
                .expect("get_amount_to_give_each_day_to_caller_res failed");

            // Assert the returned amount to give each day to the caller
            assert_eq!(
                get_amount_to_give_each_day_to_caller_res.return_value(),
                5479452054
            );

            // Return Ok(())
            Ok(())
        }

        /// Tests included in "withdraw_from_pool":
        /// 1. provide_to_pool
        /// 2. get_amount_to_give_each_day_to_caller
        /// 3. get_a0_balance
        /// 4. get_psp22_balance
        /// 5. get_lp_token_of
        /// 6. withdraw_specific_amount
        /// 7. get_account_overall_staking_rewards
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn withdraw_from_pool(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Create a new instance of MyPsp22Ref contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate MyPsp22Ref contract and get the account ID
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a new instance of TradingPairAzeroRef contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate TradingPairAzeroRef contract and get the account ID
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build an `approve` message for MyPsp22Ref contract to approve TradingPairAzeroRef contract to use tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call `approve_psp22_to_provide_lp` message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build a `provide_to_pool` message for TradingPairAzeroRef contract to provide liquidity to the pool
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            let amount: u128 = 10000000000000;

            // Call `provide_to_tpa` message
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a `get_amount_to_give_each_day_to_caller` message for TradingPairAzeroRef contract
            // to get the amount to give each day to the caller
            let get_amount_to_give_each_day_to_caller = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_amount_to_give_each_day_to_caller(get_alice_account_id())
            });

            // Call `get_amount_to_give_each_day_to_caller` message
            let get_amount_to_give_each_day_to_caller_res = client
                .call(
                    &ink_e2e::alice(),
                    get_amount_to_give_each_day_to_caller,
                    0,
                    None,
                )
                .await
                .expect("get_amount_to_give_each_day_to_caller_res failed");

            // Assert the returned amount to give each day to the caller
            assert_eq!(
                get_amount_to_give_each_day_to_caller_res.return_value(),
                5479452054
            );

            // Build a `get_a0_balance` message for TradingPairAzeroRef contract to get the balance of a0
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call `get_a0_balance` message
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            // Assert the returned balance of a0
            assert_eq!(get_a0_res.return_value(), 10001000000000);

            // Build a `get_psp22_balance` message for TradingPairAzeroRef contract to get the balance of psp22
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call `get_psp22_balance` message
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_balance failed");

            // Assert the returned balance of psp22
            assert_eq!(get_psp22_res.return_value(), 100000000000000);

            // Build a `get_lp_token_of` message for TradingPairAzeroRef contract to get the LP token balance
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Call `get_lp_token_of` message
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_token_of failed");

            // Assert the returned LP token balance
            assert_eq!(get_lp_share_res.return_value(), 1000000000000000);

            let amount: u128 = 500000000000000;

            // Build a `withdraw_specific_amount` message for TradingPairAzeroRef contract to withdraw a specific amount from the pool
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            // Call `withdraw_from_pool` message
            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw failed");

            // Build a `get_amount_to_give_each_day_to_caller` message for TradingPairAzeroRef contract
            // to get the updated amount to give each day to the caller
            let get_amount_to_give_each_day_to_caller = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_amount_to_give_each_day_to_caller(get_alice_account_id())
            });

            // Call `get_amount_to_give_each_day_to_caller` message
            let get_amount_to_give_each_day_to_caller_res = client
                .call(
                    &ink_e2e::alice(),
                    get_amount_to_give_each_day_to_caller,
                    0,
                    None,
                )
                .await
                .expect("get_amount_to_give_each_day_to_caller_res failed");

            // Assert the updated amount to give each day to the caller
            assert_eq!(
                get_amount_to_give_each_day_to_caller_res.return_value(),
                2739726027
            );

            // Build a `get_lp_token_of` message for TradingPairAzeroRef contract to get the updated LP token balance
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Call `get_lp_token_of` message
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get failed");

            // Assert the updated LP token balance
            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Build a `get_a0_balance` message for TradingPairAzeroRef contract to get the updated balance of a0
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call `get_a0_balance` message
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get failed");

            // Assert the updated balance of a0
            assert_eq!(get_a0_res.return_value(), 5000500000000);

            // Build a `get_psp22_balance` message for TradingPairAzeroRef contract to get the updated balance of psp22
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call `get_psp22_balance` message
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get failed");

            // Assert the updated balance of psp22
            assert_eq!(get_psp22_res.return_value(), 50000000000000);

            // Build a `get_account_overall_staking_rewards` message for TradingPairAzeroRef contract
            // to get the overall staking rewards for the account
            let get_account_overall_staking_rewards = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_account_overall_staking_rewards(get_alice_account_id())
            });

            // Call `get_account_overall_staking_rewards` message
            let get_account_overall_staking_rewards_res = client
                .call(
                    &ink_e2e::alice(),
                    get_account_overall_staking_rewards,
                    0,
                    None,
                )
                .await
                .expect("get failed");

            // Assert the overall staking rewards for the account
            assert_eq!(get_account_overall_staking_rewards_res.return_value(), 0);

            //

            Ok(())
        }

        /// Tests included in "get_withdraw_tokens_amount_works"
        /// 1. provide_to_pool
        /// 2. get_a0_balance
        /// 3. get_psp22_balance
        /// 4. get_withdraw_tokens_amount
        /// 5. get_lp_token_of
        /// 6. withdraw_specific_amount
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_withdraw_tokens_amount_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // PSP22 token constructor object
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate new PSP22 token using the constructor
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Trading pair constructor object
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate new TradingPairAzero contract using the constructor
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build PSP22 approve message: Alice approves the trading pair for 100 PSP22 tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call the PSP22 approve message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build TPA's provide to pool message:
            // Parameters: 100 PSP22 tokens, 1000 expected LP token shares, 0.5 slippage
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            // Amount of native coin to transfer in the provide to pool call
            let amount: u128 = 10000000000000;

            // Call provide to pool function
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build get A0 (Azero/native coin) balance call to fetch new TPA A0 balance after LP provision
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_res failed");

            // Validate that the new A0 balance is correct
            assert_eq!(get_a0_res.return_value(), 10001000000000);

            // Build get PSP22 balance call to fetch new TPA PSP22 balance after LP provision
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_res failed");

            // Validate that the new PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 100000000000000);

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_withdraw_tokens_amount(1000000000000000)
            });

            // Call and fetch the results
            let get_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_withdraw_tokens_amount, 0, None)
                .await
                .expect("get_withdraw_tokens_amount_res failed");

            // Fetch the A0 (Native coin) and PSP22 tokens balances from 'get_withdraw_tokens_amount_res'
            let Some((a0_coins, psp22_tokens)) = get_withdraw_tokens_amount_res.return_value().ok() else { panic!("test") };

            // Validate that TPA really holds the PSP22 tokens that we sent
            assert_eq!(psp22_tokens, 100000000000000);

            // Validate that TPA really holds the native tokens that we sent
            assert_eq!(a0_coins, 10001000000000);

            // LP share amount to withdraw (500 x 10^12)
            let amount: u128 = 500000000000000;

            // Build the withdraw from pool function with the specified amount
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            // Call the withdraw from pool function
            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Build the get lp share token balance message with the LP provider AccountId
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Get the lp share tokens after withdrawal
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get failed");

            // Validate that the amount of the remaining LP share tokens is correct
            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Build the get A0 balance message to see the remaining native coin balance after withdrawal
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call the get A0 message and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get failed");

            // Validate that the remaining native coin balance is correct
            assert_eq!(get_a0_res.return_value(), 5000500000000);

            // Build the get PSP22 balance message to see the remaining PSP22 balance after withdrawal
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call the get PSP22 message and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get failed");

            // Validate that the remaining PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 50000000000000);

            // Get the withdraw tokens amount by given shares after withdrawal
            let get_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_withdraw_tokens_amount(500000000000000)
            });

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_withdraw_tokens_amount, 0, None)
                .await
                .expect("get_withdraw_tokens_amount_res failed");

            // Fetch the A0 (Native coin) and PSP22 tokens balances from 'get_withdraw_tokens_amount_res'
            let Some((a0_coins, psp22_tokens)) = get_withdraw_tokens_amount_res.return_value().ok() else { panic!("get withdraw failed") };

            assert_eq!(psp22_tokens, 50000000000000);

            assert_eq!(a0_coins, 5000500000000);

            //

            Ok(())
        }

        /// Tests included in 'get_psp22_withdraw_tokens_amount_works'
        /// 1. provide_to_pool
        /// 2. get_psp22_balance
        /// 3. get_psp22_withdraw_tokens_amount
        /// 4. withdraw_specific_amount
        /// 5. get_lp_token_of
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_psp22_withdraw_tokens_amount_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // PSP22 token constructor object
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate new PSP22 token using the constructor
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Trading pair constructor object
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate new tpa contract using the constructor
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build psp22 approve message: Alice approves trading pair for 100 psp22 tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call the psp22 approve message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build tpa's provide to pool message: Provide 100 PSP22 tokens, 1000 expected LP token shares, and 0.5 as slippage
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            // Amount of native coin to transfer in the provide to pool call
            let amount: u128 = 10000000000000;

            // Call the provide to pool function
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build get PSP22 balance call to fetch new tpa PSP22 balance after LP provision
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_res failed");

            // Validate that the new PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 100000000000000);

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_psp22_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_psp22_withdraw_tokens_amount(1000000000000000)
            });

            // Call and fetch the results
            let get_psp22_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_psp22_withdraw_tokens_amount, 0, None)
                .await
                .expect("get_psp22_withdraw_tokens_amount_res failed");

            // Fetch the get_psp22_withdraw_tokens_amount_res result
            let Some(psp22_tokens) = get_psp22_withdraw_tokens_amount_res.return_value().ok() else { panic!("test") };

            // Validate that tpa really holds the PSP22 tokens that we sent
            assert_eq!(psp22_tokens, 100000000000000);

            // LP share amount to withdraw (500 x 10^12)
            let amount: u128 = 500000000000000;

            // Build the withdraw from pool function with the amount that we stated above
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            // Call the withdraw from pool function
            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Build the get lp share token balance message with the LP provider AccountId
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Get the lp share tokens after withdrawal
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_res failed");

            // Validate that the amount of the remaining LP share tokens is correct
            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Build the get PSP22 balance message to see remaining PSP22 balance after withdrawal
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call the get PSP22 message and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_res failed");

            // Validate that the remaining PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 50000000000000);

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_psp22_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_psp22_withdraw_tokens_amount(1000000000000000)
            });

            // Call and fetch the results
            let get_psp22_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_psp22_withdraw_tokens_amount, 0, None)
                .await
                .expect("get failed");

            let Some(psp22_tokens) = get_psp22_withdraw_tokens_amount_res.return_value().ok() else { panic!("test") };

            // Validate that tpa really holds the PSP22 tokens that we sent
            assert_eq!(psp22_tokens, 100000000000000);

            Ok(())
        }

        /// Tests included in 'get_a0_withdraw_tokens_amount_works'
        /// 1. provide_to_pool
        /// 2. get_a0_balance
        /// 3. get_a0_withdraw_tokens_amount
        /// 4. withdraw_specific_amount
        /// 5. get_lp_token_of
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_a0_withdraw_tokens_amount_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // PSP22 token constructor object
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate new PSP22 token using the constructor
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Trading pair constructor object
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate new TPA contract using the constructor
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build PSP22 approve message: Alice approves trading pair for 100 PSP22 tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call the PSP22 approve message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build TPA's provide to pool message:
            // Parameters: 100 PSP22 tokens, 1000 as expected LP tokens shares, and 0.5 as slippage
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            // Amount of native coin to transfer in the provide to pool call
            let amount: u128 = 10000000000000;

            // Call the provide to pool function
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build the get A0 balance call to fetch new TPA A0 balance after LP provision
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_res failed");

            // Validate that the new PSP22 balance is correct
            assert_eq!(get_a0_res.return_value(), 10001000000000);

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_a0_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_a0_withdraw_tokens_amount(1000000000000000)
            });

            // Call and fetch the result
            let get_a0_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_a0_withdraw_tokens_amount, 0, None)
                .await
                .expect("get_a0_withdraw_tokens_amount_res failed");

            let Some(a0_tokens) = get_a0_withdraw_tokens_amount_res.return_value().ok() else { panic!("test") };

            // Validate that TPA really holds the PSP22 tokens that we sent
            assert_eq!(a0_tokens, 10001000000000);

            // LP share amount to withdraw (500 x 10^12)
            let amount: u128 = 500000000000000;

            // Build the withdraw from pool function with the specified amount
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            // Call the withdraw from pool function
            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Build the get LP share token balance message with the LP provider AccountId
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Get the LP share tokens after withdrawal
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_res failed");

            // Validate that the amount of the remaining LP share tokens is correct
            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Build the get PSP22 balance message to see the remaining PSP22 balance after withdrawal
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call the get PSP22 message and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_res failed");

            // Validate that the remaining PSP22 balance is correct
            assert_eq!(get_a0_res.return_value(), 5000500000000);

            // Build get_withdraw_tokens message to fetch the withdrawable tokens by given shares
            let get_a0_withdraw_tokens_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_a0_withdraw_tokens_amount(1000000000000000)
            });

            // Call and fetch the result
            let get_a0_withdraw_tokens_amount_res = client
                .call(&ink_e2e::alice(), get_a0_withdraw_tokens_amount, 0, None)
                .await
                .expect("get_a0_withdraw_tokens_amount_res failed");

            let Some(a0_tokens) = get_a0_withdraw_tokens_amount_res.return_value().ok() else { panic!("test") };

            // Validate that TPA really holds the PSP22 tokens that we sent
            assert_eq!(a0_tokens, 10001000000000);

            Ok(())
        }

        /// Tests included in 'get_account_locked_tokens_works'
        /// 1. provide_to_pool
        /// 2. get_a0_balance
        /// 3. get_psp22_balance
        /// 4. get_account_locked_tokens
        /// 5. withdraw_specific_amount
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_account_locked_tokens_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // PSP22 token constructor object
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            // Instantiate new PSP22 token using the constructor
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Trading pair constructor object
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            // Instantiate new TPA contract using the constructor
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Build PSP22 approve message: Alice approves trading pair for 100 PSP22 tokens
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            // Call the PSP22 approve message
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Build TPA's provide to pool message: (100 PSP22 tokens, 1000 as expected LP tokens shares, and 0.5 as slippage)
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            // Amount of native coin to transfer in the provide to pool call
            let amount: u128 = 10000000000000;

            // Call the provide to pool function
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build the get A0 balance call to fetch new TPA A0 balance after LP provision
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            // Validate that the new A0 balance is correct
            assert_eq!(get_a0_res.return_value(), 10001000000000);

            // Build the get PSP22 balance call to fetch new TPA PSP22 balance after LP provision
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_balance failed");

            // Validate that the new PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 100000000000000);

            // Build get_account_locked_tokens message to fetch the locked tokens by given account ID
            let get_account_locked_tokens = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_account_locked_tokens(get_alice_account_id())
            });

            // Call and fetch the result
            let get_account_locked_tokens_res = client
                .call(&ink_e2e::alice(), get_account_locked_tokens, 0, None)
                .await
                .expect("get_account_locked_tokens failed");

            // Fetch locked PSP22 tokens and A0 (Native coin) balances
            let Some((psp22_tokens, a0_coins)) = get_account_locked_tokens_res.return_value().ok() else { panic!("test") };

            // Validate that TPA really holds the PSP22 tokens that we sent
            assert_eq!(a0_coins, 10001000000000);

            // Validate that TPA really holds the native tokens that we sent
            assert_eq!(psp22_tokens, 100000000000000);

            // LP share amount to withdraw (500 x 10^12)
            let amount: u128 = 500000000000000;

            // Build the withdraw from pool function with the specified amount
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            // Call the withdraw from pool function
            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Build the get A0 balance message to see the remaining native coin balance after withdrawal
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            // Call the get A0 message and fetch the result
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            // Validate that the remaining native coin balance is correct
            assert_eq!(get_a0_res.return_value(), 5000500000000);

            // Build the get PSP22 balance message to see the remaining PSP22 balance after withdrawal
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            // Call the get PSP22 message and fetch the result
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_balance failed");

            // Validate that the remaining PSP22 balance is correct
            assert_eq!(get_psp22_res.return_value(), 50000000000000);

            // Build get_account_locked_tokens message to fetch the locked tokens by given account ID
            let get_account_locked_tokens = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_account_locked_tokens(get_alice_account_id())
            });

            // Call and fetch the result
            let get_account_locked_tokens_res = client
                .call(&ink_e2e::alice(), get_account_locked_tokens, 0, None)
                .await
                .expect("get_account_locked_tokens failed");

            let Some((psp22_tokens, a0_coins)) = get_account_locked_tokens_res.return_value().ok() else { panic!("test") };

            // Validate that TPA really holds the PSP22 tokens that remains
            assert_eq!(psp22_tokens, 50000000000000);

            // Validate that TPA really holds the A0 coins that remains
            assert_eq!(a0_coins, 5000500000000);

            Ok(())
        }

        /// Tests included in 'get_price_for_one_psp22_works':
        /// 1. provide_to_pool
        /// 2. get_price_for_one_psp22
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_price_for_one_psp22_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the price for one PSP22 token
            let get_price_for_one_psp22 = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_price_for_one_psp22());

            // Call and fetch the results
            let get_price_for_one_psp22_res = client
                .call(&ink_e2e::alice(), get_price_for_one_psp22, 0, None)
                .await
                .expect("get_price_for_one_psp22 failed");

            // Retrieve the price for one PSP22 token
            let Some(price) = get_price_for_one_psp22_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 99019801980);

            //

            Ok(())
        }

        /// Tests included in 'get_est_price_psp22_to_a0_works'
        /// 1. provide_to_pool
        /// 2. get_est_price_psp22_to_a0
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_est_price_psp22_to_a0_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the estimated price of PSP22 tokens to A0 tokens
            let get_est_price_psp22_to_a0 = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_est_price_psp22_to_a0(1000000000000));

            // Call and fetch the results
            let get_est_price_psp22_to_a0_res = client
                .call(&ink_e2e::alice(), get_est_price_psp22_to_a0, 0, None)
                .await
                .expect("get_est_price_psp22_to_a0 failed");

            // Retrieve the estimated price of PSP22 tokens to A0 tokens
            let Some(price) = get_est_price_psp22_to_a0_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 99019801980);

            //

            Ok(())
        }

        /// Tests included in 'get_est_price_a0_to_psp22_for_swap_works'
        /// 1. provide_to_pool
        /// 2. get_est_price_a0_to_psp22_for_swap
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_est_price_a0_to_psp22_for_swap_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the estimated price of A0 tokens to PSP22 tokens for swapping
            let get_est_price_a0_to_psp22_for_swap = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_est_price_a0_to_psp22_for_swap(1000000000000)
            });

            // Call and fetch the results
            let get_est_price_a0_to_psp22_for_swap_res = client
                .call(
                    &ink_e2e::alice(),
                    get_est_price_a0_to_psp22_for_swap,
                    0,
                    None,
                )
                .await
                .expect("get_est_price_a0_to_psp22_for_swap failed");

            // Retrieve the estimated price of A0 tokens to PSP22 tokens for swapping
            let Some(price) = get_est_price_a0_to_psp22_for_swap_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 9999000099990);

            //

            Ok(())
        }

        /// Test included in 'get_expected_lp_token_amount_works'
        /// 1. get_expected_lp_token_amount
        /// 2. provide_to_pool
        /// 3. get_lp_token_of
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_expected_lp_token_amount_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let amount: u128 = 10000000000000;

            // Build a message to get the expected amount of LP tokens for the given amount
            let get_expected_lp_token_amount = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_expected_lp_token_amount(amount));

            // Call and fetch the results
            let get_expected_lp_token_amount_res = client
                .call(&ink_e2e::alice(), get_expected_lp_token_amount, 0, None)
                .await
                .expect("get_expected_lp_token_amount failed");

            // Retrieve the expected amount of LP tokens
            let Some(expected_lp_shares) = get_expected_lp_token_amount_res.return_value().ok() else { panic!("test") };

            // Assert the expected amount of LP tokens
            assert_eq!(expected_lp_shares, 1000000000000000);

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the LP token balance of Alice's account
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            // Call and fetch the results
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_balance failed");

            // Assert the LP token balance of Alice's account
            assert_eq!(get_lp_share_res.return_value(), 1000000000000000);

            Ok(())
        }

        /// Test included in 'get_est_price_a0_to_psp22_works'
        /// 1. provide_to_pool
        /// 2. get_est_price_a0_to_psp22
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_est_price_a0_to_psp22_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the estimated price from A0 to PSP22
            let get_est_price_a0_to_psp22 = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_est_price_a0_to_psp22(1000000000000));

            // Call and fetch the results
            let get_est_price_a0_to_psp22_res = client
                .call(&ink_e2e::alice(), get_est_price_a0_to_psp22, 0, None)
                .await
                .expect("get_est_price_a0_to_psp22 failed");

            // Retrieve the price
            let Some(price) = get_est_price_a0_to_psp22_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 9090082719752);

            Ok(())
        }

        /// Test included in 'get_price_impact_psp22_to_a0_works'
        /// 1. provide_to_pool
        /// 2. get_price_impact_psp22_to_a0
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_price_impact_psp22_to_a0_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the price impact from PSP22 to A0
            let get_price_impact_psp22_to_a0 = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_price_impact_psp22_to_a0(1000000000000)
            });

            // Call and fetch the results
            let get_price_impact_psp22_to_a0_res = client
                .call(&ink_e2e::alice(), get_price_impact_psp22_to_a0, 0, None)
                .await
                .expect("get_price_impact_psp22_to_a0 failed");

            // Retrieve the price
            let Some(price) = get_price_impact_psp22_to_a0_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 96116878086);

            Ok(())
        }

        /// Tests included in 'get_price_impact_a0_to_psp22_works'
        /// 1. provide_to_pool
        /// 2. get_price_impact_a0_to_psp22
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn get_price_impact_a0_to_psp22_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Create a PSP22 contract instance
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create a TradingPairAzero contract instance
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve PSP22 contract to provide liquidity to TradingPairAzero contract
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzero contract
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Build a message to get the price impact from A0 to PSP22
            let get_price_impact_a0_to_psp22 = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_price_impact_a0_to_psp22(1000000000000)
            });

            // Call and fetch the results
            let get_price_impact_a0_to_psp22_res = client
                .call(&ink_e2e::alice(), get_price_impact_a0_to_psp22, 0, None)
                .await
                .expect("get_price_impact_a0_to_psp22 failed");

            // Retrieve the price
            let Some(price) = get_price_impact_a0_to_psp22_res.return_value().ok() else { panic!("test") };

            // Assert the expected price
            assert_eq!(price, 7505697448706);

            Ok(())
        }

        /// Tests included in 'swap_psp22_works'
        /// 1. provide_to_pool
        /// 2. get_est_price_psp22_to_a0
        /// 3. swap_psp22
        /// 4. get_a0_balance
        /// 5. get_generated_lp_fees
        /// 6. get_a0_difference_by_percentage
        /// 7. get_account_overall_lp_fee_rewards
        /// 8. withdraw_specific_amount
        /// 9. get_psp22_balance
        /// 10. get_a0_lp_fee_tokens
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn swap_psp22_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate MyPsp22Ref contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Instantiate TradingPairAzeroRef contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve MyPsp22Ref to provide liquidity to TradingPairAzeroRef
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide liquidity to TradingPairAzeroRef
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Get the balance of MyPsp22Ref for Alice account
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_alice_account_id()));

            // Call and fetch the results
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
            let psp22_balance = psp22_balance_of_res.return_value();

            // Verify the balance of MyPsp22Ref for Alice account
            assert_eq!(psp22_balance, 9900000000000000);

            // Approve MyPsp22Ref to provide liquidity to TradingPairAzeroRef again
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Get estimated price of MyPsp22Ref to Azero token in TradingPairAzeroRef
            let get_est_price_psp22_to_a0 = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_est_price_psp22_to_a0(1000000000000));

            // Call and fetch the results
            let get_est_price_psp22_to_a0_res = client
                .call(&ink_e2e::alice(), get_est_price_psp22_to_a0, 0, None)
                .await
                .expect("get_est_price_psp22_to_a0 failed");

            let Some(price) = get_est_price_psp22_to_a0_res.return_value().ok() else { panic!("failed!") };

            // Verify the estimated price of MyPsp22Ref to Azero token
            assert_eq!(price, 99019801980);

            // Swap MyPsp22Ref for Azero token
            let swap_psp22 = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.swap_psp22(1000000000000, price, 500000000000)
                },
            );
            client
                .call(&ink_e2e::alice(), swap_psp22, 0, None)
                .await
                .expect("calling `swap_psp22` failed");

            // Get the balance of MyPsp22Ref for Alice account after swapping
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_alice_account_id()));

            // Call and fetch the results
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");

            let psp22_balance = psp22_balance_of_res.return_value();

            // Verify the balance of MyPsp22Ref for Alice account after swapping
            assert_eq!(psp22_balance, 9899000000000000);

            // Get the balance of Azero token in TradingPairAzeroRef
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());
            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            // Verify the balance of Azero token in TradingPairAzeroRef
            assert_eq!(get_a0_res.return_value(), 9902970396039);

            // Get the balance of MyPsp22Ref in TradingPairAzeroRef
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());
            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_balance failed");

            // Verify the balance of MyPsp22Ref in TradingPairAzeroRef
            assert_eq!(get_psp22_res.return_value(), 100998000000000);

            // Get the balance of MyPsp22Ref for Charlie account after swapping
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_charlie_account_id()));

            // Call and fetch the results
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
            let psp22_balance = psp22_balance_of_res.return_value();

            // Verify the balance of MyPsp22Ref for Charlie account after swapping
            assert_eq!(psp22_balance, 2000000000);

            // Get the generated LP fees in TradingPairAzeroRef
            let get_generated_lp_fees = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_generated_lp_fees());

            // Call and fetch the results
            let get_generated_lp_fees_res = client
                .call(&ink_e2e::alice(), get_generated_lp_fees, 0, None)
                .await
                .expect("get_generated_lp_fees failed");

            let (psp22_fees, a0_fees) = get_generated_lp_fees_res.return_value();

            // Verify the generated LP fees in TradingPairAzeroRef
            assert_eq!(psp22_fees, 0);

            assert_eq!(a0_fees, 990198019);

            // Get the percentage difference of Azero LP fees
            let get_a0_difference_by_percentage = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_a0_difference_by_percentage());

            // Call and fetch the results
            let get_a0_difference_by_percentage_res = client
                .call(&ink_e2e::alice(), get_a0_difference_by_percentage, 0, None)
                .await
                .expect("get_a0_difference_by_percentage failed");
            let Some(a0_lp_fee_diff) = get_a0_difference_by_percentage_res.return_value().ok() else { panic!("test") };

            // Verify the percentage difference of Azero LP fees
            assert_eq!(a0_lp_fee_diff, 9998500200);

            let amount: u128 = 500000000000000;

            // Withdraw a specific amount from the pool
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Get the overall LP fee rewards for Alice account
            let get_account_overall_lp_fee_rewards = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_account_overall_lp_fee_rewards(get_alice_account_id())
            });

            // Call and fetch the results
            let get_account_overall_lp_fee_rewards_res = client
                .call(
                    &ink_e2e::alice(),
                    get_account_overall_lp_fee_rewards,
                    0,
                    None,
                )
                .await
                .expect("get_account_overall_lp_fee_rewards failed");

            let (psp22_fees, a0_fees) = get_account_overall_lp_fee_rewards_res.return_value();

            // Verify the overall LP fee rewards for Alice account
            assert_eq!(psp22_fees, 0);

            assert_eq!(a0_fees, 495099009);

            // Get A0 LP fee tokens
            let get_a0_lp_fee_tokens = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_lp_fee_tokens(amount));

            let get_a0_lp_fee_tokens_res = client
                .call(&ink_e2e::alice(), get_a0_lp_fee_tokens, 0, None)
                .await
                .expect("get_a0_lp_fee_tokens failed");

            let Some(a0_lp_fee) = get_a0_lp_fee_tokens_res.return_value().ok() else {
                panic!("failed!")
            };

            assert_eq!(a0_lp_fee, 495099010);

            Ok(())
        }

        /// Tests included in 'swap_a0_works'
        /// 1. provide_to_pool
        /// 2. swap_a0
        /// 3. get_a0_balance
        /// 4. get_psp22_balance
        /// 5. get_generated_lp_fees
        /// 6. get_contract_overall_generated_fee
        /// 7. get_psp22_difference_by_percentage
        /// 8. withdraw_specific_amount
        /// 9. get_account_overall_lp_fee_rewards
        /// 10.get_psp22_lp_fee_tokens
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn swap_a0_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Create and initialize MyPsp22 contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Create and initialize TradingPairAzero contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve MyPsp22 to provide LP to TradingPairAzero
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide LP to TradingPairAzero
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            let amount: u128 = 10000000000000;

            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Fetch the estimated price for swap
            let get_est_price_a0_to_psp22_for_swap = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_est_price_a0_to_psp22_for_swap(100000000000)
            });

            let get_est_price_a0_to_psp22_for_swap_res = client
                .call(
                    &ink_e2e::alice(),
                    get_est_price_a0_to_psp22_for_swap,
                    0,
                    None,
                )
                .await
                .expect("get_est_price_a0_to_psp22_for_swap failed");

            let Some(price) = get_est_price_a0_to_psp22_for_swap_res.return_value().ok() else { panic!("failed!") };

            assert_eq!(price, 999900009999);

            // Perform the swap
            let swap_a0 = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.swap_a0(price, 1000000000000));

            let amount: u128 = 100000000000;

            client
                .call(&ink_e2e::alice(), swap_a0, amount, None)
                .await
                .expect("calling `swap_a0` failed");

            // Check MyPsp22 balance
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_alice_account_id()));

            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");

            let psp22_balance = psp22_balance_of_res.return_value();

            assert_eq!(psp22_balance, 9900978120978120);

            // Get TradingPairAzero a0 balance
            let get_a0_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_a0_balance());

            let get_a0_res = client
                .call(&ink_e2e::alice(), get_a0_balance, 0, None)
                .await
                .expect("get_a0_balance failed");

            assert_eq!(get_a0_res.return_value(), 10100800000000);

            // Get TradingPairAzero MyPsp22 balance
            let get_psp22_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_balance());

            let get_psp22_res = client
                .call(&ink_e2e::alice(), get_psp22_balance, 0, None)
                .await
                .expect("get_psp22_balance failed");

            assert_eq!(get_psp22_res.return_value(), 99019899019900);

            // Check MyPsp22 balance for Charlie
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_charlie_account_id()));

            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");

            let psp22_balance = psp22_balance_of_res.return_value();

            assert_eq!(psp22_balance, 1980001980);

            // Get generated LP fees
            let get_generated_lp_fees = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_generated_lp_fees());

            let get_generated_lp_fees_res = client
                .call(&ink_e2e::alice(), get_generated_lp_fees, 0, None)
                .await
                .expect("get_generated_lp_fees failed");

            let (psp22_fees, a0_fees) = get_generated_lp_fees_res.return_value();

            assert_eq!(psp22_fees, 9900009900);
            assert_eq!(a0_fees, 0);

            // Get overall generated fees by the contract
            let get_contract_overall_generated_fee = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_contract_overall_generated_fee());

            let get_contract_overall_generated_fee_res = client
                .call(
                    &ink_e2e::alice(),
                    get_contract_overall_generated_fee,
                    0,
                    None,
                )
                .await
                .expect("get_contract_overall_generated_fee failed");

            let (psp22_fees, a0_fees) = get_contract_overall_generated_fee_res.return_value();

            assert_eq!(psp22_fees, 9900009900);
            assert_eq!(a0_fees, 0);

            // Get MyPsp22 difference by percentage
            let get_psp22_difference_by_percentage = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| trading_pair_azero.get_psp22_difference_by_percentage());

            let get_psp22_difference_by_percentage_res = client
                .call(
                    &ink_e2e::alice(),
                    get_psp22_difference_by_percentage,
                    0,
                    None,
                )
                .await
                .expect("get_psp22_difference_by_percentage failed");

            let Some(psp22_lp_fee_diff) = get_psp22_difference_by_percentage_res.return_value().ok() else {
                panic!("failed!")
            };

            assert_eq!(psp22_lp_fee_diff, 9997500600);

            // Withdraw from the pool
            let withdraw_from_pool = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.withdraw_specific_amount(amount));

            client
                .call(&ink_e2e::alice(), withdraw_from_pool, 0, None)
                .await
                .expect("withdraw_from_pool failed");

            // Get account overall LP fee rewards
            let get_account_overall_lp_fee_rewards = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.get_account_overall_lp_fee_rewards(get_alice_account_id())
            });

            let get_account_overall_lp_fee_rewards_res = client
                .call(
                    &ink_e2e::alice(),
                    get_account_overall_lp_fee_rewards,
                    0,
                    None,
                )
                .await
                .expect("get_account_overall_lp_fee_rewards failed");

            let (psp22_fees, a0_fees) = get_account_overall_lp_fee_rewards_res.return_value();

            assert_eq!(psp22_fees, 990000);
            assert_eq!(a0_fees, 0);

            // Get PSP22 LP fee tokens
            let get_psp22_lp_fee_tokens = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| trading_pair_azero.get_psp22_lp_fee_tokens(amount));

            let get_psp22_lp_fee_tokens_res = client
                .call(&ink_e2e::alice(), get_psp22_lp_fee_tokens, 0, None)
                .await
                .expect("get_psp22_lp_fee_tokens failed");

            let Some(psp22_lp_fee) = get_psp22_lp_fee_tokens_res.return_value().ok() else {
                panic!("failed!")
            };

            assert_eq!(psp22_lp_fee, 990000);

            Ok(())
        }

        /// Tests included in 'transfer_lp_tokens_works'
        /// 1. provide_to_tpa
        /// 2. get_lp_token_of
        /// 3. transfer_lp_tokens
        /// 4. get_lp_token_of
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn transfer_lp_tokens_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate MyPsp22 contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Instantiate TradingPairAzero contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id.clone(),
                1000000000000,
                psp22_acc_id.clone(),
                get_charlie_account_id(),
            );
            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve MyPsp22 contract to provide LP tokens to TradingPairAzero
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id.clone(), 100000000000000));
            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide LP tokens to TradingPairAzero
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );
            let amount: u128 = 10000000000000;
            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Check LP token balance for Alice
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_balance failed");

            assert_eq!(get_lp_share_res.return_value(), 1000000000000000);

            // Transfer LP tokens from Alice to Bob
            let transfer_lp_tokens = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.transfer_lp_tokens(get_bob_account_id(), 500000000000000)
                },
            );
            client
                .call(&ink_e2e::alice(), transfer_lp_tokens, 0, None)
                .await
                .expect("calling `transfer_lp_tokens` failed");

            // Check LP token balance for Alice after transfer
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_balance failed");

            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Check LP token balance for Bob after transfer
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_bob_account_id())
                });
            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get failed");

            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            Ok(())
        }

        /// Tests included in 'transfer_lp_tokens_from_to_works'
        /// 1. provide_to_pool
        /// 2. get_lp_token_of
        /// 3. approve_lp_tokens
        /// 4. get_lp_tokens_allowance
        /// 5. transfer_lp_tokens_from_to
        /// 6.
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn transfer_lp_tokens_from_to_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // Instantiate MyPsp22 contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );

            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Instantiate TradingPairAzero contract
            let tpa_constructor = TradingPairAzeroRef::new(
                psp22_acc_id,
                1000000000000,
                psp22_acc_id,
                get_charlie_account_id(),
            );

            let tpa_acc_id = client
                .instantiate(
                    "trading_pair_azero",
                    &ink_e2e::alice(),
                    tpa_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Approve MyPsp22 to provide LP tokens to TradingPairAzero
            let approve_psp22_to_provide_lp = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.approve(tpa_acc_id, 100000000000000));

            client
                .call(&ink_e2e::alice(), approve_psp22_to_provide_lp, 0, None)
                .await
                .expect("calling `approve_psp22_to_provide_lp` failed");

            // Provide LP tokens to TradingPairAzero
            let provide_to_tpa = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.provide_to_pool(
                        100000000000000,
                        1000000000000000,
                        500000000000,
                    )
                },
            );

            let amount: u128 = 10000000000000;

            client
                .call(&ink_e2e::alice(), provide_to_tpa, amount, None)
                .await
                .expect("calling `provide_to_tpa` failed");

            // Get LP share balance of Alice
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get_lp_share_balance failed");

            assert_eq!(get_lp_share_res.return_value(), 1000000000000000);

            // Approve LP tokens for Bob
            let approve_lp_tokens = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone()).call(
                |trading_pair_azero| {
                    trading_pair_azero.approve_lp_tokens(get_bob_account_id(), 500000000000000)
                },
            );

            client
                .call(&ink_e2e::alice(), approve_lp_tokens, 0, None)
                .await
                .expect("calling `approve_lp_tokens` failed");

            // Get LP tokens allowance from Alice to Bob
            let get_lp_tokens_allowance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero
                        .get_lp_tokens_allowance(get_alice_account_id(), get_bob_account_id())
                });

            let get_lp_tokens_allowance_res = client
                .call(&ink_e2e::alice(), get_lp_tokens_allowance, 0, None)
                .await
                .expect("get_lp_tokens_allowance failed");

            assert_eq!(get_lp_tokens_allowance_res.return_value(), 500000000000000);

            // Transfer LP tokens from Alice to Bob
            let transfer_lp_tokens_from_to = build_message::<TradingPairAzeroRef>(
                tpa_acc_id.clone(),
            )
            .call(|trading_pair_azero| {
                trading_pair_azero.transfer_lp_tokens_from_to(
                    get_alice_account_id(),
                    get_bob_account_id(),
                    500000000000000,
                )
            });

            client
                .call(&ink_e2e::alice(), transfer_lp_tokens_from_to, 0, None)
                .await
                .expect("calling `transfer_lp_tokens_from_to` failed");

            // Get LP share balance of Alice after transfer
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_alice_account_id())
                });

            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get failed");

            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            // Get LP share balance of Bob after transfer
            let get_lp_share_balance = build_message::<TradingPairAzeroRef>(tpa_acc_id.clone())
                .call(|trading_pair_azero| {
                    trading_pair_azero.get_lp_token_of(get_bob_account_id())
                });

            let get_lp_share_res = client
                .call(&ink_e2e::alice(), get_lp_share_balance, 0, None)
                .await
                .expect("get failed");

            assert_eq!(get_lp_share_res.return_value(), 500000000000000);

            Ok(())
        }
    }
}
