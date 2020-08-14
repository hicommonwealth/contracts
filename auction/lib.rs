#![cfg_attr(not(feature = "std"), no_std)]
//cargo +nightly test 
use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod auction {
    use ink_core::storage;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct Auction {
        /// Stores a single `bool` value on the storage.
        beneficiary: storage::Value<AccountId>, // this is the account that created the auction
        highest_bidder: storage::Value<AccountId>, //highest bidder
        highest_bid: storage::Value<u32>, // highest bid amount (u32 means it can only be positive)
        ended: storage::Value<bool>, // is the auction over
        pending_returns: storage::HashMap<AccountId, u32>, // Allowed withdrawals of previous bids
    }


    // events
    #[ink(event)]
    struct Created {
        #[ink(topic)]
        beneficiary: Option<AccountId>,
        #[ink(topic)]
        starting_bid: u32,
    }
    

    #[ink(event)]
    struct New_Higher_Bid {
        #[ink(topic)]
        previous_highest_bidder: Option<AccountId>,
        #[ink(topic)]
        previous_highest_bid: u32,
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: u32,
    }

    #[ink(event)]
    struct Withdrawal {
        #[ink(topic)]
        account: Option<AccountId>,
        #[ink(topic)]
        amount: u32,
    }


     #[ink(event)]
    struct Ended {
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: u32,
    }

    impl Auction {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        fn new(&mut self, init_value: u32) {
            self.beneficiary.set(self.env().caller());
            self.highest_bidder.set(self.env().caller());
            self.highest_bid.set(init_value);
            self.ended.set(false);
            // emit event
            self.env().emit_event(Created {
                beneficiary: Some(self.env().caller()),
                starting_bid: init_value,
            });
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(0)
        }

        // Simply returns the current value of our highestBid.
        #[ink(message)]
        fn get_highest_bid(&self) -> u32 {
            *self.highest_bid
        }

        // Simply returns the current value of our highestBidder.
        #[ink(message)]
        fn get_highest_bidder(&self) -> AccountId {
            *self.highest_bidder
        }

        // beneficiary can end bidding
        #[ink(message)]
        fn get_beneficiary(&self) -> AccountId {
            * self.beneficiary
        }

        //is the auction over
        #[ink(message)]
        fn is_over(&self) -> bool {
            *self.ended
        }

        //is the auction over
        #[ink(message)]
        fn my_withdrawl_balance(&self) -> u32 {
            *self.pending_returns.get(&self.env().caller()).unwrap_or(&0)
        }

         #[ink(message)]
        fn end(&mut self) -> bool {
            if self.beneficiary != self.env().caller() {
                return false
            }
            self.ended.set(true);
            // emit event
            self.env().emit_event(Ended {
                highest_bidder: Some(self.get_highest_bidder()),
                highest_bid: self.get_highest_bid(),
            });
            true
        }

        //returns the pending amount of an account
         #[ink(message)]
        fn curr_pendding_amount(&self, id: AccountId) -> u32 {
            *self.pending_returns.get(&id).unwrap_or(&0)
        }


         #[ink(message)] 
        fn bid(&mut self, amount: u32) -> bool {
            let previous_highest_bid = self.get_highest_bid();
            // if the bid is not higher than the current highest, then return false
            //if the bid is made after the voting closes, return false
            if amount < previous_highest_bid || *self.ended{
                return false;
            }

            // Sending back the money by simply using a command is a security risk
            // because it could execute an untrusted contract.
            // It is safer to let the recipients withdraw their money themselves.
            let previous_highest_bidder = self.get_highest_bidder();
            let curr_pending = self.curr_pendding_amount(previous_highest_bidder);
            // the curr_pendding_amount is there in case the highest bidder befor had other pending money
            self.pending_returns.insert(previous_highest_bidder, 
                                        previous_highest_bid + curr_pending);


            // change the highest bidder to the new highest bidder
            self.highest_bid.set(amount);
            self.highest_bidder.set(self.env().caller());

            // emit event
            self.env().emit_event(New_Higher_Bid {
                previous_highest_bidder: Some(previous_highest_bidder),
                previous_highest_bid: previous_highest_bid,
                highest_bidder: Some(self.get_highest_bidder()),
                highest_bid: self.get_highest_bid(),
            });

            true
        }

         #[ink(message)]
        fn withdraw(&mut self) -> bool {
            let sender = self.env().caller();
            let amount = self.curr_pendding_amount(self.env().caller());
            if amount == 0 {
                return false
            }
            // TODO RETURN AMOUNT TO OWNER
            self.pending_returns.insert(sender, 0);

            self.env().emit_event(Withdrawal {
                account: Some(sender),
                amount: amount,
            });
            true
        }



    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use std::convert::TryFrom;


        // We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let auction = Auction::default();
            assert_eq!(auction.highest_bid, 0);
            assert_eq!(auction.ended , false);
            assert_eq!(auction.pending_returns.len(), 0);
        }

        // We test if the ne constructor does its job.
        #[test]
        fn new_works() {
            // a positive starting price begins as the highest bid
            let auction = Auction::new(5);
            assert_eq!(auction.highest_bid, 5);
        }


        // we test that getters work
        #[test]
        fn getters_works() {
            let auction = Auction::default();
            assert_eq!(auction.highest_bid, auction.get_highest_bid());
            assert_eq!(auction.highest_bidder, auction.get_highest_bidder());
        }

        // we test that owner can end ballot
        #[test]
        fn end_works() {
            let mut auction = Auction::default();
            assert!(auction.end());
            assert!(auction.is_over());
        }

        #[test]
        fn bid_and_withdraw_works() {
            let mut auction = Auction::default();
            assert_eq!(auction.highest_bid, 0);
            assert!(!auction.withdraw());
            // make a bid works
            auction.bid(5);
            assert_eq!(auction.highest_bid, 5);
            // making a higher bid replaces previous bid and also adds to pending amounts
            auction.bid(6);
            assert_eq!(auction.highest_bid, 6);
            assert_eq!(auction.curr_pendding_amount(auction.get_highest_bidder()), 5);
            assert_eq!(auction.my_withdrawl_balance(), 5);

            //making a bid lower than the highest bid does nothing
            auction.bid(5);
            assert_eq!(auction.highest_bid, 6);
            assert_eq!(auction.curr_pendding_amount(auction.get_highest_bidder()), 5);
            //making a higher bid and then adding to the current balance of the pending amount
            auction.bid(7);
            assert_eq!(auction.highest_bid, 7);
            assert_eq!(auction.curr_pendding_amount(auction.get_highest_bidder()), 11);

            auction.withdraw();
            assert_eq!(auction.curr_pendding_amount(auction.get_highest_bidder()), 0);
        }

        //bidding after end
        #[test]
        fn bid_anfter_end() {
            let mut auction = Auction::default();
            assert_eq!(auction.highest_bid, 0);
            auction.bid(5);
            assert_eq!(auction.highest_bid, 5);
            auction.end();
            assert!(!auction.bid(6));
            
        }

    }
}
