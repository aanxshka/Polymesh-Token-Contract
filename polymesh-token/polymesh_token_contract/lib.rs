#![cfg_attr(not(feature = "std"), no_std, no_main)] //ensures that the contract can run in Substrate's no-std environment 
//(This is necessary because Substrate-based blockchains run in a Wasm runtime, which is a minimal execution environment with no operating system.)


#[ink::contract]
mod polymesh_token_contract {

    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct PolymeshTokenContract {
        total_supply: Balance, //stores the totaly supply of the token
        balances: Mapping<AccountId, Balance>, //maps account IDs to their token balances
        owner: AccountId, //stores the account that deployed the contract for minting and burning priveleges
    }

    //event triggered on successful token transfers
    #[ink(event)] 
    pub struct Transfer {
        #[ink(topic)] //queryable field for sender
        from: Option<AccountId>, //sender's account
        #[ink(topic)] //queryable field for recipient 
        to: Option<AccountId>, //recipeint's account
        #[ink(topic)] //queryable field for amount
        value: Balance, //amount of tokens transferred
    }

    //implementation block for the Polymesh token contract 
    impl PolymeshTokenContract {
        #[ink(constructor)] //this is a construcutor function called once during deployment
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller(); //gets the account that deployed the contract
            let mut balances = Mapping::default(); //initializes the balances mapping
            balances.insert(&caller, &initial_supply); //assigns the intitial supply to the deployer

            Self {
                total_supply: initial_supply,
                balances,
                owner: caller,
            }
        }

        //returns the total supply of the token
        #[ink(message)] //public function callable by external accounts
        pub fn balance_of(&self, account: AccountId) -> Balance {
            self.balances.get(account).unwrap_or(0) //fetches the balance or defaults to 0 if not set
        }

        //transfers tokens from the caller's account to another account
        #[ink(message)]
        pub fn transfer(&mut self, to:AccountId, value:Balance) -> bool {
            let from = self.env().caller(); //gets the caller's account
            let from_balance = self.balance_of(from);
            assert!(from_balance >= value, "Insufficient balance");
            let to_balance = self.balance_of(to);

            if let Some(new_balance) = from_balance.checked_sub(value) {
                self.balances.insert(from, &new_balance);
            } else {
                panic!("Insufficient balance for transfer");
            }

            if let Some(new_balance) = to_balance.checked_add(value) {
                self.balances.insert(&to, &new_balance);
            } else {
                panic!("Balance overflow for recipient");
            }            

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            true //returns true if successful
        }

        //mints new toekns and assigns them to a specific account 
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value:Balance) {
            assert_eq!(self.env().caller(), self.owner, "Only owner can mint");

            if let Some(new_total) = self.total_supply.checked_add(value) {
                self.total_supply = new_total;
            } else {
                panic!("Total supply overflow");
            }            

            let current_balance = self.balance_of(to);

            if let Some(new_balance) = current_balance.checked_add(value) {
                self.balances.insert(&to, &new_balance);
            } else {
                panic!("Balance overflow for recipient");
            }            

            self.env().emit_event(Transfer {
                from: None, 
                to: Some(to),
                value,
            });
        }

        //burns tokens from the caller's account 
        #[ink(message)] 
        pub fn burn(&mut self, value: Balance) {
            let caller = self.env().caller();
            let current_balance = self.balance_of(caller);
            assert!(current_balance >= value, "Insufficient balance to burn");

            if let Some(new_total) = self.total_supply.checked_sub(value) {
                self.total_supply = new_total;
            } else {
                panic!("Total supply underflow");
            }

            if let Some(new_balance) = current_balance.checked_sub(value) {
                self.balances.insert(&caller, &new_balance);
            } else {
                panic!("Insufficient balance for burn");
            }            

            self.env().emit_event(Transfer { 
                from: Some(caller),
                to: None,
                value,
            });
        }
    }
}
