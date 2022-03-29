#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn_trait_bound)]
extern crate alloc;
use ink_lang as ink;
pub use self::dao_setting::{
    DaoSetting
};
#[allow(unused_imports)]
#[ink::contract]
mod dao_setting {
    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use ink_prelude::collections::BTreeMap;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };

    /// Fee limit for joining Dao
    /// time_limit:How long is the total limit
    /// fee_limit:the number of fee
    /// token:the token of limit
    #[derive(
    Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Default
    )]
    #[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub struct FeeConditions {
        pub  time_limit:u128,
        pub  fee_limit:u128,
        pub  token:AccountId
    }

    /// Other limit for joining Dao
    /// use_token:Whether to enable token restriction
    /// use_nft:Whether to enable nft restriction
    /// token:the token of limit
    /// token_balance_limit:the balance of limit
    /// nft:the nft address
    /// nft_balance_limit:the balance of limit
    /// nft_time_limit:Remaining time limit of NFT
    #[derive(
    Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Default
    )]
    #[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub struct OtherConditions {
        pub use_token:bool,
        pub use_nft:bool,
        pub token:AccountId,
        pub token_balance_limit:u128,
        pub nft:AccountId,
        pub nft_balance_limit:u128,
        pub nft_time_limit:u128
    }
    ///creator:the creator's address
    ///owner:the manager's address
    /// fee_limit:the fee limit info
    /// other_limit:the other limit info
    /// conditions:Specific restriction type
    #[ink(storage)]
    pub struct DaoSetting {
        creator:AccountId,
        owner:AccountId,
        fee_limit:FeeConditions,
        other_limit:OtherConditions,
        conditions : u64,
    }

    impl DaoSetting {
        #[ink(constructor)]
        pub fn new(creator:AccountId) -> Self {
            Self {
                creator,
                owner:Self::env().caller(),
                fee_limit:FeeConditions{
                    time_limit:0,
                    fee_limit:0,
                    token:AccountId::default()
                },
                other_limit:OtherConditions{
                    use_token:false,
                    use_nft:false,
                    token:AccountId::default(),
                    token_balance_limit:0,
                    nft:AccountId::default(),
                    nft_balance_limit:0,
                    nft_time_limit:0
                },
                conditions:0,
            }
        }

        ///Get what restrictions to use
        #[ink(message)]
        pub fn get_conditions(&self) -> u64 {
            self.conditions
        }
        ///Get fee limit
        #[ink(message)]
        pub fn get_fee_setting(&self) -> FeeConditions { self.fee_limit.clone() }
        ///Get other limit
        #[ink(message)]
        pub fn get_other_setting(&self) -> OtherConditions {
            self.other_limit.clone()
        }
        ///set join limit
        #[ink(message)]
        pub fn set_join_limit(&mut self,conditions:u64,other_conditions:OtherConditions,fee_conditions:FeeConditions) -> bool {
            let owner = self.env().caller();
            assert_eq!(owner == self.creator, true);
            if conditions == 2 {
                self.fee_limit = fee_conditions;
            }else if conditions == 4 {
                self.other_limit = other_conditions;
            } else if conditions == 6 {
                self.fee_limit = fee_conditions;
                self.other_limit = other_conditions;
            }
            self.conditions = conditions;

            true
        }
    }


    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        #[ink::test]
        fn test_get_conditions() {
            let dao_setting = DaoSetting::new(AccountId::from([0x01; 32]));
            assert!(dao_setting.get_conditions() == 0);
        }
        #[ink::test]
        fn test_get_other_setting(){
            let dao_setting = DaoSetting::new(AccountId::from([0x01; 32]));
            assert!(dao_setting.get_other_setting().token_balance_limit == 0);
        }
        #[ink::test]
        fn test_set_join(){
            let alice = ink_env::test::default_accounts::<Environment>()
                .unwrap().alice;
            let mut dao_setting = DaoSetting::new(alice);
            let fee_limit = FeeConditions{
                time_limit:0,
                fee_limit:0,
                token:AccountId::default()
            };
            let other_limit = OtherConditions{
                use_token:false,
                use_nft:false,
                token:AccountId::default(),
                token_balance_limit:0,
                nft:AccountId::default(),
                nft_balance_limit:0,
                nft_time_limit:0
            };
            assert!(dao_setting.set_join_limit(2,other_limit,fee_limit) == true);
        }
    }
}
