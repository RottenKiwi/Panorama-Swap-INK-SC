#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]


#[ink::contract]
pub mod vesting_contract {

    // Import the `PSP22Ref` trait from the `traits::psp22` module in the `openbrush` crate
    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,
        },
    };
    // Import the `Mapping` struct from the `storage` module in the `ink` crate
    use ink::storage::Mapping;
    // Import the `vec` module from the `prelude` module in the `ink` crate
    use ink::prelude::vec;

    #[ink(storage)]
    pub struct VestingContract {
        
    // Declare a field named `manager` of type `AccountId`
    manager: AccountId,
    // Declare a field named `panx_psp22` of type `AccountId`
    panx_psp22: AccountId,
    // Declare a field named `started_date_in_timestamp` of type `Balance`
    started_date_in_timestamp: Balance,
    // Declare a field named `balances` of type `Mapping<AccountId, Balance>`
    balances: Mapping<AccountId, Balance>,
    // Declare a field named `collected_tge` of type `Mapping<AccountId, Balance>`
    collected_tge: Mapping<AccountId, Balance>,
    // Declare a field named `panx_to_give_in_a_day` of type `Mapping<AccountId, Balance>`
    panx_to_give_in_a_day: Mapping<AccountId, Balance>,
    // Declare a field named `last_redeemed` of type `Mapping<AccountId, Balance>`
    last_redeemed: Mapping<AccountId, Balance>,
    
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    /// Represents the possible errors that can occur in the vesting program.
    pub enum VestingProgramErrors {

    // Error indicating that the caller is not the manager.
    CallerIsNotManager,
    // Error indicating that the caller has already collected the TGE (Token Generation Event).
    CallerCollectedTGEAlready,
    // Error indicating that the caller has insufficient locked tokens.
    CallerInsufficientLockedTokens,
    // Error indicating that zero days have passed.
    ZeroDaysPassed,
    // Error indicating a failed PSP22 transfer.
    PSP22TransferFailed,
    // Error indicating an overflow has occurred.
    Overflow,

    }

    /// Represents the `AddToVestingProgram` event.
    #[ink(event)]
    pub struct AddToVestingProgram {

    // The account that is added to the vesting program.
    added_account: AccountId,
    // The total vesting amount for the added account.
    total_vesting_amount: Balance,
    // The PANX (Token) amount to give each day in the vesting program.
    panx_amount_to_give_each_day: Balance,

    }

    /// Represents the `CollectedTGE` event.
    #[ink(event)]
    pub struct CollectedTGE {

    // The caller account that collected the TGE (Token Generation Event).
    caller: AccountId,
    // The amount of PANX (Token) given to the caller.
    panx_given_amount: Balance,
    // The new balance of the caller after collecting the TGE.
    caller_new_balance: Balance,

    }
    // Represents the `Redeem` event.
    #[ink(event)]
    pub struct Redeem {

    // The caller account that initiated the redemption.
    caller: AccountId,
    // The amount of PANX (Token) given to the caller during redemption.
    panx_given_amount: Balance,
    // The new balance of the caller after the redemption.
    caller_new_balance: Balance,

    }

    impl VestingContract {

        /// Constructor for the VestingContract.
        #[ink(constructor)]
        pub fn new(
            panx_contract: AccountId,
        )   -> Self {

            // Set the PANX contract account as the PANX PSP22.
            let panx_psp22 = panx_contract;
            // Get the current block timestamp and convert it into a Balance type.
            let started_date_in_timestamp: Balance = Self::env().block_timestamp().into();
            // Set the manager as the caller of the contract.
            let manager = Self::env().caller();
            // Create a default Mapping for balances.
            let balances = Mapping::default();
            // Create a default Mapping for collected TGE (Token Generation Event).
            let collected_tge = Mapping::default();
            // Create a default Mapping for PANX to give in a day.
            let panx_to_give_in_a_day = Mapping::default();
            // Create a default Mapping for last redeemed.
            let last_redeemed = Mapping::default();

            Self {
                // Assign the values to the struct fields.
                manager,
                panx_psp22,
                started_date_in_timestamp,
                balances,
                collected_tge,
                panx_to_give_in_a_day,
                last_redeemed,
            }
        }


        /// Adds an account to the vesting program.
        #[ink(message)]
        pub fn add_to_vesting(
            &mut self,
            account: AccountId,
            panx_to_give_overall: Balance,
        ) -> Result<(), VestingProgramErrors> {

            // Making sure the caller is the manager (Only manager can add to the vesting program)
            if self.env().caller() != self.manager {
                return Err(VestingProgramErrors::CallerIsNotManager);
            }

            // Get the current balance of the account
            let account_balance = self.balances.get(&account).unwrap_or(0);

            // Variable to hold the new vesting PANX amount
            let new_vesting_panx_amount: Balance;

            // Calculate the new vesting amount of the caller
            match account_balance.checked_add(panx_to_give_overall) {
                Some(result) => {
                    new_vesting_panx_amount = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Insert the new vesting amount for the account
            self.balances.insert(account, &(new_vesting_panx_amount));

            // Variable to hold the PANX amount to give each day
            let panx_amount_to_give_each_day: Balance;

            // Calculate how much PANX tokens the caller needs to get each day
            match new_vesting_panx_amount.checked_div(365) {
                Some(result) => {
                    panx_amount_to_give_each_day = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Insert the PANX amount to give each day for the account
            self.panx_to_give_in_a_day
                .insert(account, &panx_amount_to_give_each_day);

            // Allow the account to collect TGE (Token Generation Event)
            self.collected_tge.insert(account, &0);

            // Insert the current date as the last redeem date for the account
            self.last_redeemed
                .insert(account, &self.get_current_timestamp());

            // Emit the AddToVestingProgram event
            Self::env().emit_event(AddToVestingProgram {
                added_account: self.env().caller(),
                total_vesting_amount: panx_to_give_overall,
                panx_amount_to_give_each_day,
            });

            Ok(())
            
        }


        /// Collects the TGE (Token Generation Event) tokens for the caller.
        #[ink(message)]
        pub fn collect_tge_tokens(&mut self) -> Result<(), VestingProgramErrors> {

            // Get the caller's account ID
            let caller = self.env().caller();

            // Get the caller's current balance
            let caller_current_balance: Balance = self.balances.get(&caller).unwrap_or(0);

            // Make sure the caller hasn't already redeemed TGE tokens
            if self.collected_tge.get(&caller).unwrap_or(0) != 0 {
                return Err(VestingProgramErrors::CallerCollectedTGEAlready);
            }

            // Make sure the caller has more than 0 locked tokens
            if caller_current_balance <= 0 {
                return Err(VestingProgramErrors::CallerInsufficientLockedTokens);
            }

            // Variable to hold the caller's locked PANX after deducting TGE amount (10%)
            let caller_locked_panx_after_tge: Balance;

            // Calculate the caller's balance after reducing the TGE amount (10%)
            match (caller_current_balance * 900).checked_div(1000) {
                Some(result) => {
                    caller_locked_panx_after_tge = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Variable to hold the amount of PANX to give to the caller
            let amount_of_panx_to_give: Balance;

            // Calculate the amount of PANX to give to the caller
            match caller_current_balance.checked_sub(caller_locked_panx_after_tge) {
                Some(result) => {
                    amount_of_panx_to_give = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Transfer the TGE tokens to the caller
            if PSP22Ref::transfer(&self.panx_psp22, caller, amount_of_panx_to_give, vec![]).is_err() {
                return Err(VestingProgramErrors::PSP22TransferFailed);
            }

            // Deduct the amount of PANX to give from the overall vesting amount
            self.balances.insert(caller, &(caller_current_balance - amount_of_panx_to_give));

            // Change the caller's collected TGE status to 1 to prevent calling it again
            self.collected_tge.insert(caller, &1);

            // Emit the CollectedTGE event
            Self::env().emit_event(CollectedTGE {
                caller,
                panx_given_amount: amount_of_panx_to_give,
                caller_new_balance: caller_current_balance - amount_of_panx_to_give,
            });

            Ok(())

        }


        /// Retrieves the caller's redeemable amount of tokens.
        #[ink(message)]
        pub fn get_redeemable_amount(&mut self) -> Result<Balance, VestingProgramErrors> {

            // Get the caller's account ID
            let caller = self.env().caller();

            // Get the current date in timestamp
            let current_date_in_tsp = self.get_current_timestamp();

            // Get the caller's total vesting amount
            let caller_total_vesting_amount: Balance = self.get_account_total_vesting_amount(caller);

            // Get the date of the last redeem in timestamp
            let date_of_last_redeem_in_tsp: Balance = self.last_redeemed.get(caller).unwrap_or(0);

            // Get the PANX amount to give each day to the caller
            let panx_to_give_each_day: Balance = self.panx_to_give_in_a_day.get(caller).unwrap_or(0);

            // Variable to hold the number of days difference between the last redeem date and the current date
            let days_difference: Balance;

            // Calculate the number of days difference between the last redeem date and the current date
            match (current_date_in_tsp - date_of_last_redeem_in_tsp).checked_div(86400) {
                Some(result) => {
                    days_difference = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Make sure that at least 24 hours have passed since the last redeem
            if days_difference <= 0 {
                return Err(VestingProgramErrors::ZeroDaysPassed);
            }

            // Make sure that the caller has more than 0 PANX to redeem
            if caller_total_vesting_amount <= 0 {
                return Err(VestingProgramErrors::CallerInsufficientLockedTokens);
            }

            // Variable to hold the redeemable amount of PANX for the caller
            let mut redeemable_amount: Balance;

            // Calculate the amount of PANX the caller needs to get
            match panx_to_give_each_day.checked_mul(days_difference) {
                Some(result) => {
                    redeemable_amount = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // If the caller has fewer tokens than the daily amount, give them the remaining tokens
            if redeemable_amount > caller_total_vesting_amount {
                redeemable_amount = caller_total_vesting_amount;
            }

            Ok(redeemable_amount)

        }


        /// Function for the caller to redeem their redeemable tokens.
        #[ink(message)]
        pub fn redeem_redeemable_amount(&mut self) -> Result<(), VestingProgramErrors> {

            // Get the caller's account ID
            let caller = self.env().caller();

            // Get the current date in timestamp
            let current_date_in_tsp = self.get_current_timestamp();

            // Get the caller's total vesting amount
            let caller_total_vesting_amount = self.get_account_total_vesting_amount(caller);

            // Get the redeemable amount for the caller
            let redeemable_amount = self.get_redeemable_amount().unwrap();

            // Set the new redeem date for the caller
            self.last_redeemed.insert(caller, &current_date_in_tsp);

            // Variable to hold the caller's new vesting amount
            let caller_new_vesting_amount: Balance;

            // Calculate the caller's new total vesting amount
            match caller_total_vesting_amount.checked_sub(redeemable_amount) {
                Some(result) => {
                    caller_new_vesting_amount = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            // Deduct from the overall vesting amount for the caller
            self.balances.insert(caller, &(caller_new_vesting_amount));

            // Cross-contract call to PANX contract to transfer PANX tokens to the caller
            if PSP22Ref::transfer(&self.panx_psp22, caller, redeemable_amount, vec![]).is_err() {
                return Err(VestingProgramErrors::PSP22TransferFailed);
            }

            // Emit the Redeem event
            Self::env().emit_event(Redeem {
                caller: caller,
                panx_given_amount: redeemable_amount,
                caller_new_balance: caller_total_vesting_amount - redeemable_amount,
            });

            Ok(())

        }



        /// Function to get the caller's total locked tokens.
        #[ink(message)]
        pub fn get_account_total_vesting_amount(
            &mut self,
            account: AccountId,
        ) -> Balance {

            // Get the account balance
            let account_balance: Balance = self.balances.get(&account).unwrap_or(0);

            // Return the account balance as the total vesting amount
            account_balance

        }

        /// Function to get the caller's last redeem timestamp.
        #[ink(message)]
        pub fn get_account_last_redeem(
            &mut self,
            account: AccountId,
        ) -> Balance {

            // Get the timestamp of the last redeem for the account
            let timestamp = self.last_redeemed.get(&account).unwrap_or(0);

            // Return the timestamp
            timestamp

        }


        /// Function to get the amount of tokens to give to the account each day.
        #[ink(message)]
        pub fn get_amount_to_give_each_day_to_account(
            &mut self,
            account: AccountId,
        ) -> Balance {

            // Get the account balance from the panx_to_give_in_a_day mapping
            let account_balance: Balance = self.panx_to_give_in_a_day.get(&account).unwrap_or(0);

            // Return the account balance as the amount to give each day
            account_balance
        }

        /// Function to get the PANX reserve of the vesting contract.
        #[ink(message)]
        pub fn get_vesting_contract_panx_reserve(
            &self
        ) -> Balance {

            // Get the PANX reserve of the vesting contract by calling balance_of on the PANX contract
            let vesting_panx_reserve = PSP22Ref::balance_of(&self.panx_psp22, Self::env().account_id());

            // Return the vesting PANX reserve
            vesting_panx_reserve

        }

        ///function to get account TGE collection status
        #[ink(message)]
        pub fn user_tge_collection_status(
            &mut self,
            account:AccountId
        )   ->Balance  {

            let tge_status = self.collected_tge.get(account).unwrap_or(0);
            tge_status


        }

        ///funtion to get the started date since issuing the vesting contract in timpstamp
        #[ink(message)]
        pub fn get_started_date(
            &self
        )   -> Balance {

            let timestamp = self.started_date_in_timestamp;

            timestamp

        }

                
        /// Function to get the current timestamp in seconds.
        #[ink(message)]
        pub fn get_current_timestamp(
            &self
        ) -> Balance {

            // Get the current block timestamp from the environment and convert it to seconds
            let bts = self.env().block_timestamp() / 1000;

            // Convert the timestamp to Balance and return
            bts.into()
        }

        /// Function to get the number of days passed since contract deployment.
        #[ink(message)]
        pub fn get_days_passed_since_issue(
            &self
        ) -> Result<Balance, VestingProgramErrors> {

            // Get the current timestamp in seconds
            let current_tsp: Balance = (self.env().block_timestamp() / 1000).into();

            // Calculate the difference in days between the current timestamp and the started date
            let days_diff: Balance;
            match (current_tsp - self.started_date_in_timestamp).checked_div(86400) {
                Some(result) => {
                    days_diff = result;
                }
                None => {
                    return Err(VestingProgramErrors::Overflow);
                }
            };

            Ok(days_diff)
        }
    }


    /// ink! end-to-end (E2E) tests
    ///
    /// cargo test --features e2e-tests -- --nocapture
    ///
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        use openbrush::contracts::psp22::psp22_external::PSP22;
        use my_psp22::my_psp22::MyPsp22Ref;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        fn get_bob_account_id() -> AccountId {
            let bob = ink_e2e::bob::<ink_e2e::PolkadotConfig>();
            let bob_account_id_32 = bob.account_id();
            let bob_account_id = AccountId::try_from(bob_account_id_32.as_ref()).unwrap();

            bob_account_id
        }
        
        ///Tests included in "add_accout_with_tge":
        /// 1. get_vesting_contract_panx_reserve
        /// 2. add_to_vesting
        /// 3. user_tge_collection_status
        /// 4. get_amount_to_give_each_day_to_account_res
        /// 5. get_account_total_vesting_amount
        #[ink_e2e::test( additional_contracts = "../my_psp22/Cargo.toml" )]
        async fn add_account_with_tge(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        
            // Create a new instance of MyPsp22Ref
            let psp22_constructor = MyPsp22Ref::new(10000000000000000, Some(String::from("TOKEN").into()), Some(String::from("TKN").into()), 12);
        
            // Instantiate the MyPsp22 contract and obtain the account ID
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
            // Create a new instance of VestingContractRef
            let vesting_constructor = VestingContractRef::new(psp22_acc_id);
        
            // Instantiate the VestingContract and obtain the account ID
            let vesting_acc_id = client
                .instantiate("vesting_contract", &ink_e2e::alice(), vesting_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
        
            // Build a transfer_to_vesting message
            let transfer_to_vesting = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.transfer(vesting_acc_id, 1000000000000000, vec![]));
        
            // Call the transfer_to_vesting message
            client
                .call(&ink_e2e::alice(), transfer_to_vesting, 0, None)
                .await
                .expect("calling `transfer_to_vesting` failed");
        
            // Build a get_vesting_contract_panx_reserve message
            let get_vesting_contract_panx_reserve = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_vesting_contract_panx_reserve());
        
            // Call the get_vesting_contract_panx_reserve message
            let get_vesting_contract_panx_reserve_res = client
                .call(&ink_e2e::alice(), get_vesting_contract_panx_reserve, 0, None)
                .await
                .expect("get_vesting_contract_panx_reserve failed");
        
            // Assert that the return value of get_vesting_contract_panx_reserve is 1000000000000000
            assert_eq!(get_vesting_contract_panx_reserve_res.return_value(), 1000000000000000);
        
            // Build an add_to_vesting message
            let add_to_vesting = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.add_to_vesting(get_bob_account_id(), 1000000000000000));
        
            // Call the add_to_vesting message
            client
                .call(&ink_e2e::alice(), add_to_vesting, 0, None)
                .await
                .expect("calling `add_to_vesting` failed");
        
            // Build a user_tge_collection_status message
            let user_tge_collection_status = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.user_tge_collection_status(get_bob_account_id()));
        
            // Call the user_tge_collection_status message
            let user_tge_collection_status_res = client
                .call(&ink_e2e::alice(), user_tge_collection_status, 0, None)
                .await
                .expect("user_tge_collection_status failed");
        
            // Assert that the return value of user_tge_collection_status is 0
            assert_eq!(user_tge_collection_status_res.return_value(), 0);
        
            // Build a get_amount_to_give_each_day_to_account message
            let get_amount_to_give_each_day_to_account = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_amount_to_give_each_day_to_account(get_bob_account_id()));
        
            // Call the get_amount_to_give_each_day_to_account message
            let get_amount_to_give_each_day_to_account_res = client
                .call(&ink_e2e::alice(), get_amount_to_give_each_day_to_account, 0, None)
                .await
                .expect("get_amount_to_give_each_day_to_account failed");
        
            // Assert that the return value of get_amount_to_give_each_day_to_account is 2739726027397
            assert_eq!(get_amount_to_give_each_day_to_account_res.return_value(), 2739726027397);
        
            // Build a get_current_timestamp message
            let get_current_timestamp = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_current_timestamp());
        
            // Call the get_current_timestamp message
            let get_current_timestamp_res = client
                .call(&ink_e2e::alice(), get_current_timestamp, 0, None)
                .await
                .expect("get_current_timestamp failed");
        
            // Build a get_account_last_redeem message
            let get_account_last_redeem = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_account_last_redeem(get_bob_account_id()));
        
            // Call the get_account_last_redeem message
            let get_account_last_redeem_res = client
                .call(&ink_e2e::alice(), get_account_last_redeem, 0, None)
                .await
                .expect("get_account_last_redeem failed");
        
            // Assert that the return value of get_account_last_redeem is equal to the return value of get_current_timestamp
            // Might get an error here because the timestamp is very narrow, re-run the e2e and check again
            assert_eq!(get_account_last_redeem_res.return_value(), get_current_timestamp_res.return_value());
        
            // Build a psp22_balance_of message
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the psp22_balance_of message
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the return value of psp22_balance_of
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the psp22_balance is 0
            assert_eq!(psp22_balance, 0);
        
            // Build a collect_tge_tokens message
            let collect_tge_tokens = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.collect_tge_tokens());
        
            // Call the collect_tge_tokens message
            client
                .call(&ink_e2e::bob(), collect_tge_tokens, 0, None)
                .await
                .expect("calling `collect_tge_tokens` failed");
        
            // Build another get_current_timestamp message
            let get_current_timestamp = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_current_timestamp());
        
            // Call the get_current_timestamp message
            let get_current_timestamp_res = client
                .call(&ink_e2e::alice(), get_current_timestamp, 0, None)
                .await
                .expect("get_current_timestamp failed");
        
            // Build another get_account_last_redeem message
            let get_account_last_redeem = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_account_last_redeem(get_bob_account_id()));
        
            // Call the get_account_last_redeem message
            let get_account_last_redeem_res = client
                .call(&ink_e2e::alice(), get_account_last_redeem, 0, None)
                .await
                .expect("get_account_last_redeem failed");
        
            // Assert that the return value of get_account_last_redeem is equal to the return value of get_current_timestamp
            // Might get an error here because the timestamp is very narrow, re-run the e2e and check again
            assert_eq!(get_account_last_redeem_res.return_value(), get_current_timestamp_res.return_value());
        
            // Build another psp22_balance_of message
            let psp22_balance_of = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.balance_of(get_bob_account_id()));
        
            // Call the psp22_balance_of message
            let psp22_balance_of_res = client
                .call(&ink_e2e::alice(), psp22_balance_of, 0, None)
                .await
                .expect("psp22_balance_of failed");
        
            // Get the return value of psp22_balance_of
            let psp22_balance = psp22_balance_of_res.return_value();
        
            // Assert that the psp22_balance is 100000000000000
            assert_eq!(psp22_balance, 100000000000000);
        
            // Build a get_account_total_vesting_amount message
            let get_account_total_vesting_amount = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_account_total_vesting_amount(get_bob_account_id()));
        
            // Call the get_account_total_vesting_amount message
            let get_account_total_vesting_amount_res = client
                .call(&ink_e2e::alice(), get_account_total_vesting_amount, 0, None)
                .await
                .expect("get_account_total_vesting_amount failed");
        
            // Assert that the return value of get_account_total_vesting_amount is 900000000000000
            assert_eq!(get_account_total_vesting_amount_res.return_value(), 900000000000000);
        
            Ok(())
        }

        ///Tests included in "daily_claim":
        /// 1. get_redeemable_amount
        /// 2. get_vesting_contract_panx_reserve
        /// 3. add_to_vesting
        /// 4. get_redeemable_amount
        #[ink_e2e::test(additional_contracts = "../my_psp22/Cargo.toml")]
        async fn daily_claim(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Create a new instance of the MyPsp22 contract
            let psp22_constructor = MyPsp22Ref::new(
                10000000000000000,
                Some(String::from("TOKEN").into()),
                Some(String::from("TKN").into()),
                12,
            );
        
            // Instantiate the MyPsp22 contract
            let psp22_acc_id = client
                .instantiate("my_psp22", &ink_e2e::alice(), psp22_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
            // Create a new instance of the VestingContract
            let vesting_constructor = VestingContractRef::new(psp22_acc_id);
        
            // Instantiate the VestingContract
            let vesting_acc_id = client
                .instantiate("vesting_contract", &ink_e2e::alice(), vesting_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;
        
            // Build a transfer_to_vesting message
            let transfer_to_vesting = build_message::<MyPsp22Ref>(psp22_acc_id.clone())
                .call(|my_psp22| my_psp22.transfer(vesting_acc_id, 1000000000000000, vec![]));
        
            // Call the transfer_to_vesting message
            client
                .call(&ink_e2e::alice(), transfer_to_vesting, 0, None)
                .await
                .expect("calling `transfer_to_vesting` failed");
        
            // Build a get_vesting_contract_panx_reserve message
            let get_vesting_contract_panx_reserve =
                build_message::<VestingContractRef>(vesting_acc_id.clone())
                    .call(|vesting_contract| vesting_contract.get_vesting_contract_panx_reserve());
        
            // Call the get_vesting_contract_panx_reserve message
            let get_vesting_contract_panx_reserve_res = client
                .call(&ink_e2e::alice(), get_vesting_contract_panx_reserve, 0, None)
                .await
                .expect("get_vesting_contract_panx_reserve failed");
        
            // Assert that the return value of get_vesting_contract_panx_reserve is 1000000000000000
            assert_eq!(get_vesting_contract_panx_reserve_res.return_value(), 1000000000000000);
        
            // Build an add_to_vesting message
            let add_to_vesting = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.add_to_vesting(get_bob_account_id(), 1000000000000000));
        
            // Call the add_to_vesting message
            client
                .call(&ink_e2e::alice(), add_to_vesting, 0, None)
                .await
                .expect("calling `add_to_vesting` failed");
        
            // Build a get_redeemable_amount message
            let get_redeemable_amount = build_message::<VestingContractRef>(vesting_acc_id.clone())
                .call(|vesting_contract| vesting_contract.get_redeemable_amount());
        
            // Assert that calling get_redeemable_amount with Bob's account fails
            assert!(client
                .call(&ink_e2e::bob(), get_redeemable_amount, 0, None)
                .await
                .is_err());
        
            Ok(())

        }

    }

}