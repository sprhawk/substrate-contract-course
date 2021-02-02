#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {

    use ink_storage::collections::HashMap as StorageHashMap;
    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        balances: StorageHashMap<AccountId, Balance>,
        allowance: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBallance,
    }
    pub type Result<T> = core::result::Result<T, Error>;
    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);

            Self {
                total_supply,
                balances,
                allowance: StorageHashMap::new(),
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            *self.allowance.get(&(owner, spender)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            self.transfer_helper(who, to, value)
        }

        fn transfer_helper(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBallance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer { from, to, value });
            Ok(())
        }

        #[ink(message)]
        pub fn transer_from(&mut self, from: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            self.transfer_helper(from, who, value)
        }

        #[ink(message)]
        pub fn burn(&mut self, value: Balance) {
            let who = Self::env().caller();
            let balance = self.balance_of(who);
            if balance < value {
                self.balances.insert(who, 0);
            } else {
                self.balances.insert(who, balance - value);
            }
        }

        #[ink(message)]
        pub fn issue(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let current_balance = self.balance_of(to);
            self.balances.insert(to, current_balance + value);
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink_lang as ink;
        #[ink::test]
        fn create_contract_works() {
            let erc20 = Erc20::new(1000);
            assert_eq!(1000, erc20.total_supply());
        }

        #[ink::test]
        fn get_good_balance() {
            let erc20 = Erc20::new(1000);
            assert_eq!(erc20.balance_of(AccountId::from([0x1; 32])), 1000);
            assert_eq!(erc20.balance_of(AccountId::from([0x2; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut erc20 = Erc20::new(1000);
            let owner = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x2; 32]);
            assert_eq!(erc20.transer(to, 100), Ok(()));
            assert_eq!(erc20.balance_of(to), 100);
            assert_eq!(erc20.balance_of(owner), 900);
        }

        #[ink::test]
        fn transfer_failed_for_lower_balance() {
            let mut erc20 = Erc20::new(100);
            let to = AccountId::from([0x2; 32]);
            assert_eq!(erc20.transer(to, 200), Err(Error::InsufficientBallance));
        }
        #[ink::test]
        fn transfer_from_works() {
            let mut erc20 = Erc20::new(1000);
            let owner = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x2; 32]);
            erc20.transer(to, 200).unwrap();
            assert_eq!(erc20.transer_from(to, 100), Ok(()));
            assert_eq!(erc20.balance_of(to), 100);
            assert_eq!(erc20.balance_of(owner), 900);
        }

        #[ink::test]
        fn transfer_from_failed_for_lower_balance() {
            let mut erc20 = Erc20::new(1000);
            let to = AccountId::from([0x2; 32]);
            erc20.transer(to, 100).unwrap();
            assert_eq!(
                erc20.transer_from(to, 200),
                Err(Error::InsufficientBallance)
            );
        }

        #[ink::test]
        fn burn_works() {
            let mut erc20 = Erc20::new(1000);
            let owner = AccountId::from([0x1; 32]);
            erc20.burn(100);
            assert_eq!(erc20.balance_of(owner), 900);
            erc20.burn(1000);
            assert_eq!(erc20.balance_of(owner), 0);
        }

        #[ink::test]
        fn issue_works() {
            let owner = AccountId::from([0x1; 32]);
            let to = AccountId::from([0x2; 32]);
            let mut erc20 = Erc20::new(1000);
            erc20.issue(to, 100).unwrap();
            assert_eq!(erc20.balance_of(owner), 1000);
            assert_eq!(erc20.balance_of(to), 100);
        }
    }
}
