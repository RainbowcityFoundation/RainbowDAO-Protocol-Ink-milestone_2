#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn_trait_bound)]
extern crate alloc;
pub use self::dao_proposal::DaoProposal;

use ink_lang as ink;
#[allow(unused_imports)]
#[ink::contract]
mod dao_proposal {
    use ink_env::call::{
        build_call,
        utils::ReturnType,
        ExecutionInput,
    };

    use erc20::Erc20;
    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use ink_prelude::collections::BTreeMap;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
        collections::HashMap as StorageHashMap,
    };
    use scale::Output;
    struct CallInput<'a>(&'a [u8]);

    impl<'a> scale::Encode for CallInput<'a> {
        fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// The Voting details of a person
    /// has_voted:Whether to vote
    /// support:Is it supported
    /// votes:Number of votes cast
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    #[derive(Debug)]
    pub struct Receipt {
        has_voted: bool,
        support: u64,
        votes: u128,
    }

     /// Details of the proposal
     /// proposal_id:proposal's id
     /// title:proposal's title
     /// desc:proposal's content
     /// start_block:proposal's start block
     /// end_block:proposal's end block
     /// for_votes:Number of support votes
     /// against_votes:Number of against votes
     /// canceled:it is cancel
     /// executed:it is executed
     /// receipts:Voting details
     /// transaction:Proposal implementation details
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    #[derive(Debug)]
    pub struct Proposal {
        proposal_id: u64,
        title: String,
        desc: String,
        start_block: u32,
        end_block: u32,
        for_votes: u128,
        against_votes: u128,
        owner: AccountId,
        canceled: bool,
        executed: bool,
        receipts: BTreeMap<AccountId, Receipt>,
        transaction: Transaction,
        category:u32,
        publicity_votes:u128,
        publicity_delay:u32
    }

    ///Restrictions on initiating proposals
    ///fee_open:Open charge limit
    ///fee_number:Charge quantity
    ///fee_token:Charging token
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    #[derive(Debug)]
    pub struct Limit {
        fee_open:bool,
        fee_number:u128,
        fee_token:AccountId
    }
    /// Voting validity settings
    /// category:the category of the settings
    /// vote_scale:Voting rate setting
    /// entrust_scale:Entrust rate setting
    /// support_scale:Support rate setting
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    #[derive(Debug)]
    pub struct VoteEffective {
        category:u32,
        vote_scale:u128,
        entrust_scale:u128,
        support_scale:u128
    }


    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    #[derive(Debug)]
    pub struct Transaction {
        /// The `AccountId` of the contract that is called in this transaction.
        callee: AccountId,
        /// The selector bytes that identifies the function of the callee that should be called.
        selector: [u8; 4],
        /// The SCALE encoded parameters that are passed to the called function.
        input: Vec<u8>,
        /// The amount of chain balance that is transferred to the callee.
        transferred_value: Balance,
        /// Gas limit for the execution of the call.
        gas_limit: u64,
    }


    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: u64,
        #[ink(topic)]
        creator: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ProposalState {
        Canceled,
        Pending,
        Active,
        Defeated,
        Succeeded,
        Executed,
        Expired,
        Publicity,
        Queued,
    }

    /// This is a proposal in Dao
    /// creator:the creator of the contract
    /// owner:the owner of the contract
    /// proposals:HashMap of the proposal'id and proposal
    /// voting_delay:Voting buffer
    /// voting_period:Voting time
    /// proposal_length:Total number of proposals
    /// erc20_addr:the addr of erc20
    /// limit:the limit of create proposal
    /// vote_effective:the effective of vote
    #[ink(storage)]
    pub struct DaoProposal {
        creator:AccountId,
        owner: AccountId,
        proposals: StorageHashMap<u64, Proposal>,
        voting_delay: u32,
        voting_period: u32,
        proposal_length: u64,
        erc20_addr: AccountId,
        limit:Limit,
        vote_effective:VoteEffective
    }

    impl DaoProposal {
        #[ink(constructor)]
        pub fn new(creator:AccountId, erc20_addr: AccountId) -> Self {
            Self {
                creator,
                owner: Self::env().caller(),
                proposals: StorageHashMap::new(),
                voting_delay: 1,
                voting_period: 259200, //3 days
                proposal_length: 0,
                erc20_addr,
                limit:Limit{
                    fee_open:false,
                    fee_number:1,
                    fee_token:AccountId::default()
                },
                vote_effective:VoteEffective{
                    category:0,
                    vote_scale:0,
                    entrust_scale:0,
                    support_scale:0
                }
            }
        }

        /// Set requirements for initiating proposals
        #[ink(message)]
        pub fn set_permission(&mut self,limit:Limit) -> bool {
            assert!(self.env().caller() == self.creator);
            self.limit = limit;

            true
        }
        /// Set the conditions for successful proposal
        #[ink(message)]
        pub fn set_vote_effective(&mut self,vote_effective:VoteEffective) -> bool {
            assert!(self.env().caller() == self.creator);
            self.vote_effective = vote_effective;
            true
        }


        /// Create a new proposal
        /// #Fields
        /// title:proposal's title
        /// desc:proposal's content
        /// category:proposal's category
        /// start_block:proposal's start_block
        /// end_block:proposal's end_block
        /// publicity_delay:Date of publication of the proposal
        /// transaction:proposal's transaction
        #[ink(message)]
        pub fn propose(
            &mut self,
            title: String,
            desc: String,
            category:u32,
            start_block:u32,
            end_block:u32,
            transaction: Transaction,
            publicity_delay:u32,
        ) -> bool {
            assert!(start_block > self.env().block_number());
            assert!(end_block > start_block);
            assert!(category < 4);
            let limit = &self.limit;
            if limit.fee_open {
                let mut erc20_instance: Erc20 = ink_env::call::FromAccountId::from_account_id(limit.fee_token);
                erc20_instance.transfer_from(Self::env().caller(),AccountId::default(),limit.fee_number);
            }

            let proposal_id = self.proposal_length.clone() + 1;
            self.proposal_length += 1;
            let proposal_info = Proposal {
                category,
                proposal_id,
                title,
                desc,
                start_block,
                end_block,
                for_votes: 0,
                against_votes: 0,
                owner: Self::env().caller(),
                canceled: false,
                executed: false,
                receipts: BTreeMap::new(),
                transaction,
                publicity_votes:0,
                publicity_delay
            };
            self.proposals.insert(proposal_id, proposal_info);
            self.env().emit_event(ProposalCreated {
                proposal_id,
                creator: self.env().caller(),
            });
            true
        }
         /// Show state of proposal
         /// proposal_id:proposal's id
        #[ink(message)]
        pub fn state(&self, proposal_id: u64) -> ProposalState {
            let proposal: Proposal = self.proposals.get(&proposal_id).unwrap().clone();
            let block_number = self.env().block_number();
            let effective:VoteEffective = self.vote_effective.clone();
            let mut failed = false;
            let erc20_instance: Erc20 = ink_env::call::FromAccountId::from_account_id(self.erc20_addr);
            let token_info = erc20_instance.query_info();
            let all_vote = proposal.for_votes + proposal.against_votes;
            if effective.category == 1 {
                if all_vote / token_info.total_supply * 100 <= effective.vote_scale {
                    failed = true;
                }
            }else if effective.category == 3 {
                if proposal.for_votes / all_vote * 100 <= effective.support_scale {
                    failed = true;
                }
            }
            if proposal.canceled { return ProposalState::Canceled; }
            else if block_number <= proposal.start_block { return ProposalState::Pending; }
            else if block_number <= proposal.end_block { return ProposalState::Active; }
            else if failed { return ProposalState::Defeated; }
            else if proposal.executed { return ProposalState::Executed; }
            else if block_number > proposal.end_block { return ProposalState::Expired; }
            else if block_number < proposal.end_block + proposal.publicity_delay { return ProposalState::Publicity; }
            else if proposal.publicity_votes > proposal.for_votes{ return ProposalState::Defeated; }
            else { return ProposalState::Queued; }
        }
        /// Set a proposal to cancel
        /// proposal_id:proposal's id
        #[ink(message)]
        pub fn cancel(&mut self, proposal_id: u64) -> bool {
            let proposal = self.proposals.get_mut(&proposal_id).unwrap();
            assert!(proposal.owner == Self::env().caller());
            proposal.canceled = true;
            true
        }
        /// Vote for the publicity period
        /// proposal_id:proposal's id
        #[ink(message)]
        pub fn public_vote(&mut self, proposal_id: u64) -> bool {
            let block_number = self.env().block_number();
            let caller = Self::env().caller();
            let proposal = self.proposals.get_mut(&proposal_id).unwrap();
            assert!(proposal.end_block < block_number);
            assert!(proposal.end_block + proposal.publicity_delay > block_number);
            let erc20_instance: Erc20 = ink_env::call::FromAccountId::from_account_id(self.erc20_addr);
            let votes = erc20_instance.get_current_votes(caller);
            proposal.publicity_votes = votes;
            true
        }
        /// Vote on a proposal
        /// proposal_id:proposal's id
        /// support:Is it supported
        #[ink(message)]
        pub fn cast_vote(&mut self, proposal_id: u64, support: u64) -> bool {
            let caller = Self::env().caller();
            assert!(self.state(proposal_id) == ProposalState::Active);
            assert!(support <= 1);
            let  proposal = self.proposals.get_mut(&proposal_id).unwrap();
            let  default_receipt = Receipt{has_voted:false, support: 0, votes:0};
            let  receipts = proposal.receipts.get(&caller).unwrap_or(&default_receipt);
            assert!(receipts.has_voted == false);
            let erc20_instance: Erc20 = ink_env::call::FromAccountId::from_account_id(self.erc20_addr);
            let votes = erc20_instance.get_current_votes(caller);
            if support == 1 {
                proposal.for_votes += votes;
            } else {
                proposal.against_votes += votes;
            }
            proposal.receipts.insert(caller,Receipt{has_voted:true, support: support, votes:votes});
            true
        }
        /// Show all proposals
        #[ink(message)]
        pub fn list_proposals(&self) -> Vec<Proposal> {
            let mut proposal_vec = Vec::new();
            let mut iter = self.proposals.values();
            let mut proposal = iter.next();
            while proposal.is_some() {
                proposal_vec.push(proposal.unwrap().clone());
                proposal = iter.next();
            }
            proposal_vec
        }
        /// Show a proposal by id
        #[ink(message)]
        pub fn get_proposal_by_id(&self, proposal_id: u64) -> Proposal {
            let proposal: Proposal = self.proposals.get(&proposal_id).unwrap().clone();
            proposal
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn test_proposal() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let mut govnance_dao = DaoProposal::new(
                AccountId::from([0x01; 32]),
                AccountId::from([0x01; 32]),
            );
            let mut vec = Vec::new();
            vec.push(1);
            let select: [u8; 4] = [1, 2, 3, 4];
            govnance_dao.propose(String::from("test"), String::from("test"),3,4,5, Transaction {
                callee: accounts.alice,
                selector: select,
                input: vec,
                transferred_value: 0,
                gas_limit: 1000000 },
                1
            );
            let proposal: Proposal = govnance_dao.get_proposal_by_id(1);
            assert!(proposal.title == String::from("test"));
        }
        #[ink::test]
        fn test_cancel() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let mut govnance_dao = DaoProposal::new(
                AccountId::from([0x01; 32]),
                AccountId::from([0x01; 32]),
            );
            let mut vec = Vec::new();
            vec.push(1);
            let select: [u8; 4] = [1, 2, 3, 4];
            govnance_dao.propose(String::from("test"), String::from("test"),3,4,5, Transaction {
                callee: accounts.alice,
                selector: select,
                input: vec,
                transferred_value: 0,
                gas_limit: 1000000 },
                                 1
            );
            assert!(govnance_dao.cancel(1) == true);
        }
        #[ink::test]
        fn test_set_permission() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let mut govnance_dao = DaoProposal::new(
                accounts.alice,
                accounts.alice,
            );
            let limit =  Limit{
                fee_open:false,
                fee_number:1,
                fee_token:AccountId::default()
            };
            assert!(govnance_dao.set_permission(limit) == true);
        }
        #[ink::test]
        fn test_set_vote_effective() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let mut govnance_dao = DaoProposal::new(
                accounts.alice,
                accounts.alice,
            );
            let limit =  Limit{
                fee_open:false,
                fee_number:1,
                fee_token:AccountId::default()
            };
            let vote_effective = VoteEffective{
                category:0,
                vote_scale:0,
                entrust_scale:0,
                support_scale:0
            };
            assert!(govnance_dao.set_vote_effective(vote_effective) == true);
        }
    }
}
