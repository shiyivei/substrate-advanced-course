#![cfg_attr(not(feature = "std"), no_std)]

// use ink_lang as ink;
// use scale::{Decode, Encode};

#[ink::contract]
mod erc20 {
    // 8个方法两个event

    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct ERC20 {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        approval: Mapping<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        to: Option<AccountId>,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        owner: AccountId,
        spender: AccountId,
        amount: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientApproval,
    }

    // 实现一些公共方法
    impl ERC20 {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();

            let sender = Self::env().caller();

            balances.insert(&sender, &total_supply);

            ink::env::debug_println!(
                "balance in constructor, Account: {:?} | Balance: {:?}",
                sender,
                balances.get(&sender)
            );

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(sender),
                amount: total_supply,
            });

            Self {
                total_supply,
                balances,
                approval: Default::default(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, who: AccountId) -> Balance {
            // let balance = self.balances.get(&who).unwrap_or_default();

            // ink::env::debug_println!("balance of account: {:?} | Balance: {:?}", who, balance);

            // balance.unwrap_or_default()

            self.balances.get(&who).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approval(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.approval.get(&(owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, amount: Balance) -> Result<(), Error> {
            let owner = self.env().caller();

            self.approval.insert((owner, to), &amount);

            self.env().emit_event(Approval {
                owner,
                spender: to,
                amount,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: AccountId,
            amount: Balance,
        ) -> core::result::Result<(), Error> {
            let from = self.env().caller();
            self.inner_transfer(from, to, amount)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let approval = self.approval(from, caller);
            if approval < amount {
                return Err(Error::InsufficientApproval);
            }

            self.inner_transfer(from, to, amount)?;
            self.approval.insert(&(from, caller), &(approval - amount));

            // let balance = self.balances.get(&who).unwrap_or_default();

            self.approval.insert(&(from, caller), &(approval - amount));

            Ok(())
        }

        pub fn inner_transfer(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), Error> {
            let from_balance = self.balance_of(from);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(&from, &(from_balance - amount));

            let to_balance = self.balance_of(to);
            self.balances.insert(&to, &(to_balance + amount));

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                amount,
            });

            Ok(())
        }
    }
}
