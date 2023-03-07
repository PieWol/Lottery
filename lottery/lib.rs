#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod lottery {
    // Import the `Mapping` type
use ink::storage::Mapping;


    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Lottery {
        /// Stores a single `bool` value on the storage.
        /// used to determine state of the lottery. 
        /// true meaning tickets can be bought.
        /// false meaning a closed lottery waiting for the drawing of the winner.
        open: bool,
        // Store the winner
        winner: Option<AccountId>,
        // Store a mapping from AccountIds to a u32
        tickets: Mapping<AccountId, u32>,
        // Amount of funds to be won. Winner takes all.
        jackpot: u32,
        // Block of the first ticket purchase
        starting_block: Option<u32>
        
        
    }



    

    impl Lottery {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool ) -> Self {
            let tickets = Mapping::default();
            Self { 
                open: true, 
                winner: None,
                tickets,
                jackpot: 0,
                starting_block: None
                
                 }
        }



        /// Retrieve the ticket amount of the caller.
        #[ink(message)]
        pub fn get_tickets(&self) -> Option<u32> {
            let caller = self.env().caller();
            self.tickets.get(caller)
        }

        /// Retrieve the ticket amount of any account.
        #[ink(message)]
        pub fn get_tickets_by_account(&self, account: AccountId) -> Option<u32> {
            self.tickets.get(account)
        }

        /// Buy Lottery tickets
        #[ink(message, payable)]
        pub fn transfer(&mut self) {
            // set the starting block of the lottery
            let starting_block = self.env().block_number();
            let caller = self.env().caller();
            let tickets = self.tickets.get(caller).unwrap_or(0);
            let endowment = self.env().transferred_value() as u32;
            self.tickets.insert(caller, &(tickets + endowment));
        }


        /// Simply returns the current state of the lottery.
        #[ink(message)]
        pub fn lottery_state(&self) -> bool {
            self.open
        }

        /// Simply returns the current jackpot size of the lottery. This should also represent the amount of tickets that have been purchased so far.
        #[ink(message)]
        pub fn jackpot_size(&self) -> u32 {
            self.jackpot
        }
    }
    /* 
    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let lottery = Lottery::new(true);
            assert_eq!(lottery.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut lottery = Lottery::new(false);
            assert_eq!(lottery.get(), false);
            lottery.flip();
            assert_eq!(lottery.get(), true);
        }
    }*/
}
