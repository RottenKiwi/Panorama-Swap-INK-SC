#![cfg_attr(not(feature = "std"), no_std)]


pub use self::multi_sig::{
	MultiSig,
	MultiSigRef,
};

#[ink::contract]
pub mod multi_sig {

    use openbrush::{
        contracts::{

            traits::{psp22::PSP22Ref},
        },
    };
    use ink::{storage::Mapping};
    use ink::env::CallFlags;
    use ink::prelude::{
        vec,
        vec::Vec};
    use ink::storage::Lazy;




    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",derive(scale_info::TypeInfo, 
     ink::storage::traits::StorageLayout))]
    pub struct WalletTransaction {

        creator:AccountId,
        approvers: Vec<AccountId>,
        approved: Vec<AccountId>,
        rejected: Vec<AccountId>,
        number_of_approvals:u64,
        number_of_approvals_needed:u64,
        psp22_token_to_transfer:AccountId,
        psp22_amount_to_transfer:Balance,
        recipient:AccountId,
        transaction_number:u64,
        completed_transaction:bool,
        date: Balance

    }



    #[ink(storage)]
    pub struct MultiSig {
        
        deployer: AccountId,
        wallet_participants: Vec<AccountId>,
        number_of_participants: u64,
        number_of_transactions: u64,
        wallet_tokens: Mapping<AccountId,Balance>,
        wallet_transactions: Mapping<u64,WalletTransaction>,
        traders_fee:Balance,
        vault: AccountId,
        


    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum MultiSigErrors {
        CallerNotDeployer,
        CallerNotParticipant,
        NotEnoughAllowance,
        PSP22TransferFromFailed,
        PSP22TransferFailed,
        TransactionDoesntExists,
        TokenDoesntExistsInWallet,
        NoEnoughTokenBalance,
        TransactionNotActive,
        ParticipantAlreadyExists,
        ParticipantAlreadyApproved,
        ParticipantAlreadyRejected,
        Overflow,
        CallerInsufficientPSP22Balance,
        
        


    }

    #[ink(event)]
    pub struct ParticipantAdded {
        caller:AccountId,
        participant:AccountId
    }

    #[ink(event)]
    pub struct TokenAdded {
        caller:AccountId,
        psp22_token_address:AccountId,
        psp22_amount_added:Balance
    }

    #[ink(event)]
    pub struct NewTransaction {
        creator:AccountId,
        approvers: Vec<AccountId>,
        number_of_approvals_needed:u64,
        psp22_token_to_transfer:AccountId,
        psp22_amount_to_transfer:Balance,
        recipient:AccountId,
        transaction_number:u64,
        date: Balance,
        
    }

    #[ink(event)]
    pub struct TransactionApproved {
        creator: AccountId,
        transaction_number: u64,
        number_of_approvals: u64,
        date: Balance
    }

    #[ink(event)]
    pub struct TransactionRejected {
        creator: AccountId,
        transaction_number: u64,
        number_of_rejects: u64,
        date: Balance
    }


    impl MultiSig {
        /// Creates a new multi-sig wallet contract.
        #[ink(constructor)]
        pub fn new(
            vault:AccountId
        )   -> Self {
            
            let deployer = Self::env().caller();
            let mut wallet_participants = Vec::new();
            let number_of_participants = 1;
            let number_of_transactions = 0;
            let wallet_tokens = Mapping::default();
            let wallet_transactions = Mapping::default();
            wallet_participants.push(deployer);
            let traders_fee:Balance = 25;
            
            Self{

                deployer,
                wallet_participants,
                number_of_participants,
                number_of_transactions,
                wallet_tokens,
                wallet_transactions,
                traders_fee,
                vault
            }
            
        }



        ///function to add participants to the multi-sig wallet.
        #[ink(message)]
        pub fn add_participants_to_wallet(
            &mut self,
            participant_to_add:AccountId
        )   -> Result<(), MultiSigErrors>  {

            let caller = Self::env().caller();

            //making sure that the caller is the deployer
            if caller != self.deployer  {
                return Err(MultiSigErrors::CallerNotDeployer);
            }

            //loop over all wallet participants to make that the given new participant is not present
            for participant in &self.wallet_participants {

                //making sure that the caller is a participant
                if participant_to_add == *participant  {

                    return Err(MultiSigErrors::ParticipantAlreadyExists);

                }
                
            }

            //insert new participant to participants map
            self.wallet_participants.push(participant_to_add);

            //increasing the amount of participants after adding a new one
            self.number_of_participants = self.number_of_participants + 1;

            Self::env().emit_event(ParticipantAdded{
                caller:caller,
                participant:participant_to_add
            });

            Ok(())

        
        }

        ///function to add tokens to the multi-sig wallet.
        #[ink(message)]
        pub fn add_token_to_wallet(
            &mut self,
            psp22_token_address:AccountId,
            psp22_amount_to_add:Balance
        )   -> Result<(), MultiSigErrors>   {

            let caller = Self::env().caller();

            let mut caller_is_participant:bool = false;

            //loop over all wallet participants to make that the caller is a participant
            for participant in &self.wallet_participants {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_participant = true;

                }
                
            }

            //throw error is the caller is not a wallet participant
            if caller_is_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let contract_allowance:Balance = PSP22Ref::allowance(
                &psp22_token_address,
                caller,
                Self::env().account_id()
            );
            
            //making sure that the multi-sig wallet contract has enough allowance.
            if contract_allowance < psp22_amount_to_add {
                return Err(MultiSigErrors::NotEnoughAllowance);
            }

            let caller_balance_before_transfer:Balance = PSP22Ref::balance_of(
                &psp22_token_address,
                caller
            );

            //cross contract call to psp22 contract to transfer psp22 token to the multi-sig wallet. 
            if PSP22Ref::transfer_from_builder(&psp22_token_address,self.env().caller(),Self::env().account_id(),psp22_amount_to_add,vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .is_err(){
                        return Err(MultiSigErrors::PSP22TransferFromFailed);
            }
            
            let caller_balance_after_transfer:Balance = PSP22Ref::balance_of(
                &psp22_token_address,
                caller
            );

            if caller_balance_before_transfer == caller_balance_after_transfer {
                return Err(MultiSigErrors::CallerInsufficientPSP22Balance);
            }
           
            self.wallet_tokens.insert(psp22_token_address,&psp22_amount_to_add);
           
            Self::env().emit_event(TokenAdded{
                caller:caller,
                psp22_token_address:psp22_token_address,
                psp22_amount_added:psp22_amount_to_add
            });

            Ok(())

        
        }

        ///function to create new multi-sig transaction.
        #[ink(message)]
        pub fn create_new_transaction(
            &mut self,
            number_of_approvals:u64,
            psp22_token_address_to_transfer:AccountId,
            amount_of_psp22_to_transfer:Balance,
            recipient_address:AccountId
        )   -> Result<(), MultiSigErrors>   {

            let caller = Self::env().caller();

            let current_date = self.get_current_timestamp();

            let multi_sig_given_psp22_balance = self.wallet_tokens.get(psp22_token_address_to_transfer).unwrap_or(0);

            //validating if the multi-sig wallet holds the given PSP22 token
            if multi_sig_given_psp22_balance <= 0 {
                return Err(MultiSigErrors::TokenDoesntExistsInWallet); 
            }

            //validating if the multi-sig wallet holds enough of the given PSP22 token
            if multi_sig_given_psp22_balance < amount_of_psp22_to_transfer {
                return Err(MultiSigErrors::NoEnoughTokenBalance); 
            }

            let mut caller_is_participant:bool = false;

            let current_transactions_number = self.number_of_transactions;
        
            let mut transaction_approvers:Vec<AccountId> = Vec::new();

            let approved_participants:Vec<AccountId> = Vec::new();

            let rejected_participants:Vec<AccountId> = Vec::new();

            //loop over all wallet participants to make that the caller is a participant
            for participant in &self.wallet_participants {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_participant = true;

                }

                transaction_approvers.push(*participant);

                
            }

            //throw error is the caller is not a wallet participant
            if caller_is_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let new_transaction_number= current_transactions_number + 1;
            
            //Update `number_of_transactions` in storage so that we can return exact no of transaction in `get_number_of_transactions()`
            self.number_of_transactions = new_transaction_number;

            let transaction =  WalletTransaction {

                creator: caller,
                approvers: transaction_approvers.clone(),
                approved: approved_participants,
                rejected: rejected_participants,
                number_of_approvals: 0,
                number_of_approvals_needed: number_of_approvals,
                psp22_token_to_transfer: psp22_token_address_to_transfer,
                psp22_amount_to_transfer: amount_of_psp22_to_transfer,
                recipient: recipient_address,
                transaction_number: new_transaction_number,
                completed_transaction: false,
                date: current_date

            };

            self.wallet_transactions.insert(new_transaction_number,&transaction);

            Self::env().emit_event(NewTransaction{
                creator:caller,
                approvers:transaction_approvers.clone(),
                number_of_approvals_needed:number_of_approvals,
                psp22_token_to_transfer:psp22_token_address_to_transfer,
                psp22_amount_to_transfer:amount_of_psp22_to_transfer,
                recipient:recipient_address,
                transaction_number: new_transaction_number,
                date: current_date
            });
        
            Ok(())
        
        }

        ///function to approve a transaction by caller
        #[ink(message)]
        pub fn approve_transaction(
            &mut self,
            number_of_transaction:u64
        )   -> Result<(), MultiSigErrors>   {

            let caller = Self::env().caller();

            let current_time = self.get_current_timestamp();

            let mut caller_is_participant:bool = false;

            //loop over all wallet participants to make that the caller is a participant
            for participant in &self.wallet_participants {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_participant = true;

                }
                
            }

            //throw error is the caller is not a wallet participant
            if caller_is_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let mut caller_is_transaction_participant:bool = false;

            let mut transaction:WalletTransaction;

            match self.get_wallet_transaction(number_of_transaction) {
                Ok(located_transaction) => {
                    transaction = located_transaction;
                }
                Err(error) =>{
                    return Err(error);
                }
            };

            //maing sure transaction is still active
            if transaction.completed_transaction == true { 
                return Err(MultiSigErrors::TransactionNotActive);
            }

            //loop over all the transaction participants to make that the caller is a participant
            for participant in &transaction.approvers {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_transaction_participant = true;

                }
                
            }

            //throw error is the caller is not a transaction participant
            if caller_is_transaction_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let mut caller_approved_before:bool = false;

            //loop over all the transaction approved participants to make sure that
            //caller didnt approved before
            for participant in &transaction.approved {

                //validating if the caller approved before
                if caller == *participant  {

                    caller_approved_before = true;

                }
                
            }

            if caller_approved_before == true{
                return Err(MultiSigErrors::ParticipantAlreadyApproved);
            }

            //adding caller to the approvers vec 
            transaction.approvers.push(caller);

            //increase the number of over all transaction approvals
            let new_number_of_approvals = transaction.number_of_approvals + 1;

            transaction.number_of_approvals = new_number_of_approvals;

            //Update mapping after adding `new_number_of_approvals` in `number_of_approvals`
            self.wallet_transactions.insert(number_of_transaction, &transaction);

            
            let transaction_number = transaction.transaction_number;

            //making sure that the number of approvals reached the number of approvals needed to send
            //the transaction, if so, we send it
            if transaction.number_of_approvals_needed == transaction.number_of_approvals {

                //send transaction
                match self.send_transaction(transaction) {
                    Result::Ok(()) => {

                    }
                    Result::Err(error) =>{
                        return Err(error);
                    }
                };

            }

            Self::env().emit_event(TransactionApproved{
                creator: caller,
                transaction_number: transaction_number,
                number_of_approvals: new_number_of_approvals,
                date: current_time
            });

            Ok(())

        
        }

        ///function to reject a transaction by caller
        #[ink(message)]
        pub fn reject_transaction(
            &mut self,
            number_of_transaction:u64
        )   -> Result<(), MultiSigErrors>   {

            let caller = Self::env().caller();

            let current_time = self.get_current_timestamp();

            let mut caller_is_participant:bool = false;

            //loop over all wallet participants to make that the caller is a participant
            for participant in &self.wallet_participants {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_participant = true;

                }
                
            }

            //throw error is the caller is not a wallet participant
            if caller_is_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let mut caller_is_transaction_participant:bool = false;

            let mut transaction:WalletTransaction;

            match self.get_wallet_transaction(number_of_transaction) {
                Ok(located_transaction) => {
                    transaction = located_transaction;
                }
                Err(error) =>{
                    return Err(error);
                }
            };

            //maing sure transaction is still active
            if transaction.completed_transaction == true { 
                return Err(MultiSigErrors::TransactionNotActive);
            }

            //loop over all the transaction participants to make that the caller is a participant
            for participant in &transaction.approvers {

                //making sure that the caller is a participant
                if caller == *participant  {

                    caller_is_transaction_participant = true;

                }
                
            }

            //throw error is the caller is not a transaction participant
            if caller_is_transaction_participant == false{
                return Err(MultiSigErrors::CallerNotParticipant);
            }

            let mut caller_rejected_before:bool = false;

            //loop over all the transaction rejected participants to make sure that
            //caller didnt reject before
            for participant in &transaction.rejected {

                //validating if the caller rejected before
                if caller == *participant  {

                    caller_rejected_before = true;

                }
                
            }

            if caller_rejected_before == true{
                return Err(MultiSigErrors::ParticipantAlreadyRejected);
            }

            //adding caller to the rejected vec 
            transaction.rejected.push(caller);

            let transaction_number = transaction.transaction_number;

            let number_of_rejects:u64 = transaction.rejected.len().try_into().unwrap();

            Self::env().emit_event(TransactionRejected{
                creator: caller,
                transaction_number: transaction_number,
                number_of_rejects: number_of_rejects,
                date: current_time
            });


            Ok(())

        
        }

        ///function to send confirmed transaction
        fn send_transaction(
            &mut self,
            mut transaction: WalletTransaction
        )   -> Result<(), MultiSigErrors>   {

            let transaction_recipient_address = transaction.recipient;

            let transaction_psp22_token_address = transaction.psp22_token_to_transfer;

            let transaction_psp22_token_amount_before_traders_fee = transaction.psp22_amount_to_transfer;

            let psp22_amount_for_vault:Balance;

            //calculating the amount of tokens to allocate to the vault account
            match  (transaction_psp22_token_amount_before_traders_fee * self.traders_fee).checked_div(1000u128)  {
                Some(result) => {
                    psp22_amount_for_vault = result;
                }
                None => {
                    return Err(MultiSigErrors::Overflow);
                }
            };

            //cross contract call to PSP22 contract to transfer tokens to vault account
            if PSP22Ref::transfer(&transaction_psp22_token_address,self.vault,psp22_amount_for_vault,vec![]).is_err(){
                return Err(MultiSigErrors::PSP22TransferFailed);
            }

            let actual_psp22_amount_to_transfer_to_recipient:Balance;

            //calculating the amount of tokens to allocate to the recipient
            match  transaction_psp22_token_amount_before_traders_fee.checked_sub(psp22_amount_for_vault)  {
                Some(result) => {
                    actual_psp22_amount_to_transfer_to_recipient = result;
                }
                None => {
                    return Err(MultiSigErrors::Overflow);
                }
            };

            //cross contract call to PSP22 contract to transfer tokens to transaction recipient
            if PSP22Ref::transfer(&transaction_psp22_token_address,transaction_recipient_address,actual_psp22_amount_to_transfer_to_recipient,vec![]).is_err(){
                return Err(MultiSigErrors::PSP22TransferFailed);
            }


            transaction.completed_transaction = true;
            
            //Update mapping so that `completed_transaction` get updated in transaction
            self.wallet_transactions.insert(transaction.transaction_number, &transaction);


            let psp22_current_balance:Balance = self.wallet_tokens
                .get(&transaction_psp22_token_address)
                .unwrap_or(0);

            let new_psp22_balance:Balance;

            match psp22_current_balance.checked_sub(transaction_psp22_token_amount_before_traders_fee) {
                Some(result) => {
                    new_psp22_balance = result;
                }
                None => {
                    return Err(MultiSigErrors::Overflow);
                }
            };

            self.wallet_tokens.insert(
                transaction_psp22_token_address,
                &new_psp22_balance
            );

            Ok(())

        }

        #[ink(message)]
        pub fn get_wallet_participants(
            &self
        )   -> Vec<AccountId> {

            self.wallet_participants.clone()

        }

        #[ink(message)]
        pub fn get_wallet_deployer(
            &self
        )   -> AccountId {

            self.deployer

        }

        #[ink(message)]
        pub fn get_number_of_transactions(
            &self
        )   -> u64 {

            self.number_of_transactions

        }

        ///function to get multi_sig contract address (self)
        #[ink(message)]
        pub fn get_account_id(
            &self
        ) -> AccountId {

            Self::env().account_id()

        }

        ///function to get current timpstamp in seconds
        fn get_current_timestamp(
            &self
        )   -> Balance {

            let bts = self.env().block_timestamp() / 1000;
            bts.into()

        }

        #[ink(message)]
        pub fn get_wallet_transaction(
            &self,
            number_of_transaction:u64
        )   -> Result<WalletTransaction, MultiSigErrors> {

            let transaction: WalletTransaction;

            match self.wallet_transactions.get(number_of_transaction) {
                Some(result) => {
                    transaction = result;
                }
                None => {
                    return Err(MultiSigErrors::TransactionDoesntExists);
                }
            };

            Ok(transaction)

        }
 
    }
}