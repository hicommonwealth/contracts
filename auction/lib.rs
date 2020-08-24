#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;
//cargo +nightly test 
#[ink::contract(version = "0.1.0")]
mod auction {
    use ink_core::storage;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct Auction {
        beneficiary: storage::Value<AccountId>, // this is the account that created the auction
        highest_bidder: storage::Value<AccountId>, //highest bidder
        highest_bid: storage::Value<Balance>, // highest bid amount (Balance can only be positive)
        starting_price: storage::Value<Balance>, // starting bid amount (Balance can only be positive)
        ended: storage::Value<bool>, // is the auction over
        pending_returns: storage::HashMap<AccountId, Balance>, // Allowed withdrawals of previous bids
    }


    // events
    #[ink(event)]
    struct Created {
        #[ink(topic)]
        beneficiary: Option<AccountId>,
        #[ink(topic)]
        starting_bid: Balance,
    }
    

    #[ink(event)]
    struct New_Higher_Bid {
        #[ink(topic)]
        previous_highest_bidder: Option<AccountId>,
        #[ink(topic)]
        previous_highest_bid: Balance,
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }

    #[ink(event)]
    struct Failed_Bid_Lower_Than_Highest_Bid {
        #[ink(topic)]
        attempted_bidder: Option<AccountId>,
        #[ink(topic)]
        attempted_bid: Balance,
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }

    #[ink(event)]
    struct Failed_Bid_Lower_Than_Starting_Price {
        #[ink(topic)]
        attempted_bidder: Option<AccountId>,
        #[ink(topic)]
        attempted_bid: Balance,
        #[ink(topic)]
        starting_price: Balance,
    }

    #[ink(event)]
    struct Withdrawal {
        #[ink(topic)]
        account: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
    }


     #[ink(event)]
    struct Ended {
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }

    #[ink(event)]
    struct Already_Ended {
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }

     #[ink(event)]
    struct No_More_Bidding {
        #[ink(topic)]
        is_over: bool,
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }

     // events
    #[ink(event)]
    struct Not_Beneficiary {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        beneficiary: Option<AccountId>,
    }


    impl Auction {
        /// Constructor that initializes the starting_price value to the given `init_value`.
        #[ink(constructor)]
        fn new(&mut self, init_value: Balance) {
            self.beneficiary.set(self.env().caller());
            self.highest_bidder.set(self.env().caller());
            self.starting_price.set(init_value);
            self.highest_bid.set(0);
            self.ended.set(false);
            self.pending_returns.insert(self.env().caller(), 0);
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
        fn get_highest_bid(&self) -> Balance {
            *self.highest_bid
        }

        // Simply returns the current value of our highestBid.
        #[ink(message)]
        fn get_starting_price(&self) -> Balance {
            *self.starting_price
        }

        #[ink(message)]
        fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        //it gets the highest asking price so far
        #[ink(message)]
        fn get_current_asking_price(&self) -> Balance {
            if self.get_highest_bid() > self.get_starting_price() {
                return self.get_highest_bid()
            }
            self.get_starting_price()
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

        //getting sender's withdrawl balance
        #[ink(message)]
        fn my_withdrawl_balance(&self) -> Balance {
            *self.pending_returns.get(&self.env().caller()).unwrap_or(&0)
        }

         #[ink(message)]
        fn end(&mut self) -> bool {

            //checks that the caller is the beneficiary
            if self.beneficiary != self.env().caller() {
                self.env().emit_event(Not_Beneficiary {
                    sender: Some(self.env().caller()),
                    beneficiary: Some(*self.beneficiary),
                });
                return false
            }

             //making sure the contract was not already ended
            else if *self.ended {
                self.env().emit_event(Already_Ended {
                    highest_bidder: Some(self.get_highest_bidder()),
                    highest_bid: self.get_highest_bid(),
                });
                return false
            }
            self.ended.set(true);

            //give highest bid to beneficiary
            match self.env().transfer(*self.beneficiary, *self.highest_bid) {
                Ok(now) => (),
                Err(error) => {
                    //since the beneficiary does not recieve the money, the bid does not end
                    self.ended.set(false);
                    return false
                }
            };


            // emit event
            self.env().emit_event(Ended {
                highest_bidder: Some(self.get_highest_bidder()),
                highest_bid: self.get_highest_bid(),
            });
            true
        }

        //returns the pending amount of an account
         #[ink(message)]
        fn curr_withdrawl_amount(&self, id: AccountId) -> Balance {
            *self.pending_returns.get(&id).unwrap_or(&0)
        }


        #[ink(message)] 
        fn bid(&mut self) -> bool {
            //the amount transfered to the contract ie the bid amount
            let amount: Balance = self.env().transferred_balance();

            let previous_highest_bid = self.get_highest_bid();
            

            // if the bid is not higher than the starting price, then return false
            if amount <= self.get_starting_price() {
                //value is to low so allow the sender to collect the funds
                self.pending_returns.insert(self.env().caller(), amount.into());

                // emit event
                self.env().emit_event(Failed_Bid_Lower_Than_Starting_Price {
                    attempted_bidder: Some(self.env().caller()),
                    attempted_bid: amount,
                    starting_price: *self.starting_price,
                });
                return false
            }
            // if the bid is not higher than the current highest, then return false
            else if amount <= previous_highest_bid {
                //value is to low so allow the sender to collect the funds
                self.pending_returns.insert(self.env().caller(), amount.into());

                // emit event
                self.env().emit_event(Failed_Bid_Lower_Than_Highest_Bid {
                    attempted_bidder: Some(self.env().caller()),
                    attempted_bid: amount,
                    highest_bidder: Some(self.get_highest_bidder()),
                    highest_bid: self.get_highest_bid(),
                });
                return false
            }

            //if the bid is made after the voting closes, return false
            else if *self.ended {
                self.pending_returns.insert(self.env().caller(), amount.into());

                // emit event
                self.env().emit_event(No_More_Bidding {
                    is_over: *self.ended,
                    highest_bidder: Some(self.get_highest_bidder()),
                    highest_bid: self.get_highest_bid(),
                });
                return false
            }


            // Sending back the money by simply using a command is a security risk
            // because it could execute an untrusted contract.
            // It is safer to let the recipients withdraw their money themselves.
            // look up "reentracy attack"
            let previous_highest_bidder = self.get_highest_bidder();
            let curr_pending = self.curr_withdrawl_amount(previous_highest_bidder);
            // the curr_withdrawl_amount is here in case the highest bidder had other pending money
            self.pending_returns.insert(previous_highest_bidder, 
                                        previous_highest_bid + curr_pending);


            // change the highest bidder to the new highest bidder
            self.highest_bid.set(amount);
            self.highest_bidder.set(self.env().caller());

            // emit event
            self.env().emit_event(New_Higher_Bid {
                previous_highest_bidder: Some(previous_highest_bidder),
                previous_highest_bid: previous_highest_bid,
                highest_bidder: Some(self.env().caller()),
                highest_bid: amount,
            });

            true
        }

        #[ink(message)]
        fn withdraw(&mut self) -> bool {
            let sender = self.env().caller();
            let amount = self.curr_withdrawl_amount(sender);
            if amount == 0 {
                return false
            }
            
            //remove the balance
            self.pending_returns.insert(sender, 0);
            
            //return amount to owner
            match self.env().transfer(sender, amount) {
                Ok(now) => (),
                Err(error) => {
                    //Since the amount is not retunred readd the amount to the pending_returns
                    self.pending_returns.insert(sender, amount);
                    return false
                }
            };

            self.env().emit_event(Withdrawal {
                account: Some(sender),
                amount: amount,
            });
            true
        }

        // #[ink(message)]
        // fn get_current_time(&self) -> Timestamp {
        //     let time = match ink_core::env::block_timestamp::<T>() {
        //         Ok(now) => now,
        //         Err(error) => panic!("Error retrieving the time"),
        //     };
        //     time
        // }



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
            assert_eq!(auction.pending_returns.len(), 1);
            assert_eq!{auction.get_current_asking_price(), 0};
        }

        // We test if the ne constructor does its job.
        #[test]
        fn new_works() {
            // a positive starting price begins as the highest bid
            let auction = Auction::new(5);
            assert_eq!(auction.get_starting_price(), 5);
        }

        // we test that getters work
        #[test]
        fn getters_works() {
            let auction = Auction::default();
            assert_eq!(auction.highest_bid, auction.get_current_asking_price());
            assert_eq!(auction.highest_bidder, auction.get_highest_bidder());
        }

        // we test that owner can end ballot
        #[test]
        fn end_works() {
            let mut auction = Auction::default();
            assert!(auction.end());
            assert!(auction.is_over());
            assert!(!auction.end());
        }

        // #[test]
        // fn bid_and_withdraw_works() {
        //     let mut auction = Auction::default();
        //     assert_eq!(auction.highest_bid, 0);
        //     assert!(!auction.withdraw());
        //     // make a bid works
        //     auction.bid(5);
        //     assert_eq!(auction.highest_bid, 5);
        //     // making a higher bid replaces previous bid and also adds to pending amounts
        //     auction.bid(6);
        //     assert_eq!(auction.highest_bid, 6);
        //     assert_eq!(auction.curr_withdrawl_amount(auction.get_highest_bidder()), 5);
        //     assert_eq!(auction.my_withdrawl_balance(), 5);

        //     //making a bid lower than the highest bid does nothing
        //     auction.bid(5);
        //     assert_eq!(auction.highest_bid, 6);
        //     assert_eq!(auction.curr_withdrawl_amount(auction.get_highest_bidder()), 5);
        //     //making a higher bid and then adding to the current balance of the pending amount
        //     auction.bid(7);
        //     assert_eq!(auction.highest_bid, 7);
        //     assert_eq!(auction.curr_withdrawl_amount(auction.get_highest_bidder()), 11);

        //     auction.withdraw();
        //     assert_eq!(auction.curr_withdrawl_amount(auction.get_highest_bidder()), 0);
        // }

        // //bidding after end
        // #[test]
        // fn bid_anfter_end() {
        //     let mut auction = Auction::default();
        //     assert_eq!(auction.highest_bid, 0);
        //     auction.bid(5);
        //     assert_eq!(auction.highest_bid, 5);
        //     auction.end();
        //     assert!(!auction.bid(6));
            
        // }

    }
}
