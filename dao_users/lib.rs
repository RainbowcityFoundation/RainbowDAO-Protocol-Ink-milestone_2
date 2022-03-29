#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn_trait_bound)]
extern crate alloc;
use ink_lang as ink;
pub use self::dao_users::{
    DaoUsers
};
#[allow(unused_imports)]
#[ink::contract]
mod dao_users {
    use alloc::string::String;
    use ink_prelude::vec::Vec;
    use dao_setting::DaoSetting;
    use erc20::Erc20;
    use ink_prelude::collections::BTreeMap;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        lazy::Lazy,
        traits::{
            PackedLayout,
            SpreadLayout,
        }
    };
    /// store a user info
    /// addr:the address of user
    /// expire_time : the expire of user
    /// role : the role of user
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]

    #[derive(Debug)]
    pub struct User {
        addr : AccountId,
        expire_time:u128,
        role:u64
    }
    /// store a group info
    /// id:the id of group
    /// name:the name of group
    /// join_directly:Join directly
    /// is_open:Open or not
    /// users:HashMap of user's address of bool
    /// manager:the manager of group
    #[derive(scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout)]
    #[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]

    #[derive(Debug)]
    pub struct Group {
        id:u128,
        name:String,
        join_directly:bool,
        is_open:bool,
        users:BTreeMap<AccountId,bool>,
        manager:AccountId
    }

    ///All users in Dao are stored here
    /// user:hashmap of user's address and userinfo
    /// setting_addr:the address of setting
    /// group:hashmap of group'id and group info
    /// user_group:hashmap of user address , group id and bool
    #[ink(storage)]
    pub struct DaoUsers {
         user:StorageHashMap<AccountId,User>,
       // user_referer:StorageHashMap<AccountId,AccountId>,
       // length:u128,
        setting_addr:AccountId,
        group:StorageHashMap<u128,Group>,
        user_group:StorageHashMap<(AccountId,u128),bool>,
        group_index:u128
    }

    impl DaoUsers {
        #[ink(constructor)]
        pub fn new(setting_addr:AccountId) -> Self {
            Self {
                user:StorageHashMap::new(),
                setting_addr,
                group:StorageHashMap::new(),
                user_group:StorageHashMap::new(),
                group_index:0
            }
        }
        /// add a group
        /// name:the name of group
        /// join_directly:Join directly
        /// is_open:Open or not
        #[ink(message)]
        pub fn add_group(
            &mut self,
            name:String,
            join_directly:bool,
            is_open:bool,
            manager:AccountId
        ) -> bool {
            let index = self.group_index.clone() + 1;
            let mut user = BTreeMap::new();
            user.insert(self.env().caller(),true);
            let group = Group{
                id:index,
                name,
                join_directly,
                is_open,
                users:user.clone(),
                manager
            };
            self.group_index += 1;
            self.group.insert(index,group);
            true
        }
        /// join the dao
        #[ink(message)]
        pub fn join(&mut self) ->bool {
            let default_user = User{addr:AccountId::default(),expire_time:0,role:0};
            let user = self.user.get(&Self::env().caller()).unwrap_or(&default_user).clone();
            assert!(user.addr == AccountId::default());
            let  setting_instance: DaoSetting = ink_env::call::FromAccountId::from_account_id(self.setting_addr);
            let condition =  setting_instance.get_conditions();
            let fee_limit = setting_instance.get_fee_setting();
            if condition == 2 {
                let mut erc20_instance: Erc20 = ink_env::call::FromAccountId::from_account_id(fee_limit.token);
                assert_eq!(erc20_instance.balance_of(self.env().caller()) >= fee_limit.fee_limit, true);
                erc20_instance.transfer_from(Self::env().caller(),AccountId::default(),fee_limit.fee_limit);
                self.user.insert(Self::env().caller(),User{addr:Self::env().caller(),expire_time:0,role:0});
            }else{
                self.user.insert(Self::env().caller(),User{addr:Self::env().caller(),expire_time:0,role:0});
            }
            true
        }
        /// Check whether the user has joined
        #[ink(message)]
        pub fn verify_user(&mut self,index:u128,user:AccountId) -> bool {
            let  group =  self.group.get_mut(&index).unwrap();
            assert_eq!(group.id > 0, true);
            group.users.insert(user,true);
            true
        }
        /// Check whether the user has joined
        #[ink(message)]
        pub fn init_user(&mut self,user:AccountId) -> bool {
            self.user.insert(user,User{addr:user,expire_time:0,role:0});
            true
        }

        /// join a group
        /// index:the id of group
        #[ink(message)]
        pub fn join_group(&mut self,index:u128) -> bool {
            let  group =  self.group.get_mut(&index).unwrap();
            let caller = Self::env().caller();
            assert_eq!(group.id > 0, true);
            // let mut user_group = self.user_group.get_mut(&(caller,index)).unwrap();
            if group.join_directly == false {
                group.users.insert(caller,false);
            }else{
                group.users.insert(caller,true);
            }
            self.user_group.insert((caller,index),true);
            true
        }
        /// show all user of dao
        #[ink(message)]
        pub fn list_user(&self) -> Vec<User> {
            let mut user_vec = Vec::new();
            let mut iter = self.user.values();
            let mut user = iter.next();
            while user.is_some() {
                user_vec.push(user.unwrap().clone());
                user = iter.next();
            }
            user_vec
        }
        /// show all group of dao
        #[ink(message)]
        pub fn list_group(&self) -> Vec<Group> {
            let mut group_vec = Vec::new();
            let mut iter = self.group.values();
            let mut group = iter.next();
            while group.is_some() {
                group_vec.push(group.unwrap().clone());
                group = iter.next();
            }
            group_vec
        }
        /// close a group
        /// id:the id of group
        #[ink(message)]
        pub fn close_group(&mut self,id:u128) -> bool {
            let mut group =  self.group.get_mut(&id).unwrap();
            group.is_open = false;
            true
        }
    }


    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        /// We test a simple use case of our contract.
        #[ink::test]
        fn test_add_group() {
            let mut dao_users = DaoUsers::new(AccountId::from([0x01; 32]));
            assert!(dao_users.add_group(String::from("test"),true,true,AccountId::from([0x01; 32])) == true);
        }
        #[ink::test]
        fn test_init_user(){
            let mut dao_users = DaoUsers::new(AccountId::from([0x01; 32]));
            assert!(dao_users.init_user(AccountId::from([0x01; 32])) == true);
        }
        #[ink::test]
        fn test_list_user(){
            let mut dao_users = DaoUsers::new(AccountId::from([0x01; 32]));
            dao_users.init_user(AccountId::from([0x01; 32]));
            let mut vec = Vec::new();
            let group = User{addr:AccountId::from([0x01; 32]),expire_time:0,role:0};
            vec.push(group);
            let list = dao_users.list_user();
            assert!(vec[0].addr == list[0].addr);
        }
        #[ink::test]
        fn test_join_group(){
            let mut dao_users = DaoUsers::new(AccountId::from([0x01; 32]));
            dao_users.add_group(String::from("test"),true,true,AccountId::from([0x01; 32]));
            assert!(dao_users.join_group(1) == true);
        }
        #[ink::test]
        fn test_list_group(){
            let mut dao_users = DaoUsers::new(AccountId::from([0x01; 32]));
            dao_users.add_group(String::from("test"),true,true,AccountId::from([0x01; 32]));
            let mut vec = Vec::new();
            let mut user = BTreeMap::new();
            user.insert(AccountId::from([0x01; 32]),true);
            let group = Group{
                id:1,
                name:String::from("test"),
                join_directly:true,
                is_open:true,
                users:user.clone(),
                manager:AccountId::from([0x01; 32])
            };
            vec.push(group);
            let list = dao_users.list_group();
            assert!(vec[0].id == list[0].id);
        }
    }
}
