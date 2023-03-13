#![cfg_attr(not(feature = "std"), no_std)]
use ink::env::Environment;
/// This is an example of how an ink! contract may call the Substrate
/// runtime function `RandomnessCollectiveFlip::random_seed`. See the
/// file `runtime/chain-extension-example.rs` for that implementation.
///
/// Here we define the operations to interact with the Substrate runtime.
#[ink::chain_extension]
pub trait FetchRandom {
    type ErrorCode = RandomReadErr;

    /// Note: this gives the operation a corresponding `func_id` (1101 in this case),
    /// and the chain-side chain extension will get the `func_id` to do further operations.
    #[ink(extension = 1101)]
    fn fetch_random(subject: [u8; 32]) -> [u8; 32];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomReadErr {
    FailGetRandomSource,
}

impl ink::env::chain_extension::FromStatusCode for RandomReadErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailGetRandomSource),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = ink::primitives::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = FetchRandom;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod lottery {

    // Import the `Mapping` type
    use super::RandomReadErr;
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Lottery {
        // Store the winner
        winner: Option<AccountId>,
        // Store all entrants. used to traverse the mapping lateron.
        entrants: ink::prelude::vec::Vec<AccountId>,
        // Store all tickets, whereas each ticket is represented by a reference to the AccountId that it belongs to.
        tickets: Mapping<AccountId, u32>,
        // Amount of funds to be won. Winner takes all.
        jackpot: u32,
        // Block that marks end of tickets being purchaseable.
        ending_block: u32,
        // Block that the drawing of the winner will occur in. ~1hr after end of lottery.
        drawing_block: u32,
    }

    impl Lottery {
        /// Constructor that initializes the lottery.
        #[ink(constructor)]
        pub fn new_lottery() -> Self {
            let mut lottery = Self::new();
            Lottery::start_lottery(&mut lottery);
            lottery
        }

        fn new() -> Self {
            let tickets = Mapping::default();
            Self {
                winner: None,
                entrants: ink::prelude::vec![],
                tickets,
                jackpot: 0,
                ending_block: 0,
                drawing_block: 0,
            }
        }

        // separately pay out the jackpot as soon as a winner has been drawn.
        // this transfers out all the funds the contract holds and terminates its existence.
        #[ink(message)]
        pub fn payout(&self) {
            assert!(self.winner.is_some());
            self.env().terminate_contract(self.winner.unwrap());
        }

        // set the ending and drawing block of the lottery based on the current block number
        fn start_lottery(&mut self) {
            let block = self.env().block_number();

            self.ending_block = block + 100800;
            self.drawing_block = block + 100900;
        }

        /// Retrieve the ticket amount of the caller.
        #[ink(message)]
        pub fn get_tickets(&self) -> Option<u32> {
            let caller = self.env().caller();
            self.tickets.get(caller)
        }

        #[ink(message)]
        pub fn add_entrant(&mut self, entrant: AccountId) {
            self.entrants.push(entrant);
        }

        /// Retrieve the ticket amount of any account.
        #[ink(message)]
        pub fn get_tickets_by_account(&self, account: AccountId) -> Option<u32> {
            self.tickets.get(account)
        }

        /// Draw the winner of the lottery
        #[ink(message)]
        pub fn draw_winner(&mut self) -> Result<(), RandomReadErr> {
            // make sure drawing block has been reached and no winner is yet determined.
            assert!(self.env().block_number() >= self.drawing_block);
            assert!(self.winner.is_none());

            let bytes: [u8; 4] = self.jackpot.to_be_bytes();
            let subject: [u8; 32] = core::array::from_fn(|i| bytes[i % 4]);
            let rand: [u8; 32] = self.env().extension().fetch_random(subject)?;

            // add the 1 to the current jackpot size so even the last entry has a chance of winning.
            let mut winning_number =
                (rand[0] * rand[1] * rand[2] * rand[3]) as u32 % self.jackpot_size() + 1;
            let map = &self.tickets;
            let mut winner = None;

            for entrant in &self.entrants {
                if winning_number <= map.get(entrant).unwrap() {
                    use ink::prelude::borrow::ToOwned;
                    winner = Some(entrant.to_owned());
                    break;
                }

                winning_number -= map.get(entrant).unwrap();
            }
            self.winner = winner;
            Ok(())
        }

        /// Buy Lottery tickets
        #[ink(message, payable)]
        pub fn purchase_tickets(&mut self, desired_amount: u32) {
            // check for lottery being open
            assert!(self.env().block_number() < self.ending_block);

            let caller = self.env().caller();
            let tickets = self.tickets.get(caller).unwrap_or(0);
            let endowment = self.env().transferred_value() as u32;
            assert!(endowment > desired_amount);
            self.tickets.insert(caller, &(tickets + desired_amount));
            self.jackpot += desired_amount;
            self.add_entrant(caller);
        }

        /// Return the winner as an Optional
        #[ink(message)]
        pub fn get_winner(&self) -> Option<AccountId> {
            self.winner
        }

        /// Fetch price of one ticket
        #[ink(message)]
        pub fn get_ticket_price(&self) -> ink::prelude::string::String {
            use ink::prelude::string::*;
            "Ticket costs exactly 1 token".to_string()
        }
        /// Simply returns the current state of the lottery. true means lottery is open and tickets can be bought.
        #[ink(message)]
        pub fn lottery_is_open(&self) -> bool {
             self.env().block_number() < self.ending_block
        }

        /// Simply returns the current jackpot size of the lottery. This should also represent the amount of tickets that have been purchased so far.
        #[ink(message)]
        pub fn jackpot_size(&self) -> u32 {
            self.jackpot
        }
    }
    
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
            let lottery = Lottery::new();
            assert_eq!(lottery.winner, None);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let lottery = Lottery::new();
            assert_eq!(lottery.entrants.len() , 0);
           // lottery.flip();
           // assert_eq!(lottery.get(), true);
        }
    }
}
