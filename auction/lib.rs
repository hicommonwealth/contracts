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
        created_time: storage::Value<Timestamp>, //time the auction was created
        end_time: storage::Value<Timestamp>, //time the auction is set to be allowed to end
    }


    // events
    #[ink(event)]
    struct Created {
        #[ink(topic)]
        beneficiary: Option<AccountId>,
        #[ink(topic)]
        starting_bid: Balance,
        #[ink(topic)]
        created_time: Timestamp,
        #[ink(topic)]
        end_time: Timestamp,
    }
    

    #[ink(event)]
    struct New_Highest_Bid {
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
        is_ended: bool,
        #[ink(topic)]
        highest_bidder: Option<AccountId>,
        #[ink(topic)]
        highest_bid: Balance,
    }


    #[ink(event)]
    struct Not_Beneficiary {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        beneficiary: Option<AccountId>,
    }


    impl Auction {
        /// Constructor that initializes the starting_price value to the given `init_value`
        /// and the amount of time (in milliseconds) until non-beneficiaries can bid using 'millisecs'
        #[ink(constructor)]
        fn new(&mut self, init_value: Balance, millisecs: u64) {
            self.beneficiary.set(self.env().caller());
            self.highest_bidder.set(self.env().caller());
            self.starting_price.set(init_value);
            self.highest_bid.set(0);
            self.ended.set(false);
            self.pending_returns.insert(self.env().caller(), 0);


            // Timestamps are in milliseconds
            let curr_time: Timestamp = self.env().block_timestamp();
            self.created_time.set(curr_time);
            self.end_time.set(curr_time.saturating_add(millisecs));

            // emit event
            self.env().emit_event(Created {
                beneficiary: Some(self.env().caller()),
                starting_bid: init_value,
                created_time: curr_time,
                end_time: curr_time.saturating_add(millisecs),
            });
        }


        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(0, 60)
        }

        /// returns the highestBid.
        fn get_highest_bid(&self) -> Balance {
            *self.highest_bid
        }

        /// returns the starting_price.
        #[ink(message)]
        fn get_starting_price(&self) -> Balance {
            *self.starting_price
        }

        /// returns the balance in the contract
        #[ink(message)]
        fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        /// returns the highest asking price so far i.e max(highest_bid, starting_price)
        #[ink(message)]
        fn get_current_asking_price(&self) -> Balance {
            if self.get_highest_bid() > self.get_starting_price() {
                return self.get_highest_bid()
            }
            self.get_starting_price()
        }

        /// returns the highestBidder.
        #[ink(message)]
        fn get_highest_bidder(&self) -> AccountId {
            *self.highest_bidder
        }

        /// returns the AccountId of the beneficiary
        #[ink(message)]
        fn get_beneficiary(&self) -> AccountId {
            * self.beneficiary
        }

        /// returns whether the auction ended
        #[ink(message)]
        fn is_ended(&self) -> bool {
            *self.ended
        }

        /// returns the sender's withdrawl balance
        #[ink(message)]
        fn my_withdrawl_balance(&self) -> Balance {
            *self.pending_returns.get(&self.env().caller()).unwrap_or(&0)
        }

        /// returns a given accounts withdraw balance  
         #[ink(message)]
        fn curr_withdrawl_amount(&self, id: AccountId) -> Balance {
            *self.pending_returns.get(&id).unwrap_or(&0)
        }

        /// returns whether a non-beneficiaries can end the auction
        #[ink(message)]
        fn time_end_allowed(&self) -> bool{
            self.env().block_timestamp() > *self.end_time
        }

        /// returns the current block timestamp
        #[ink(message)]
        fn get_time(&self) -> Timestamp{
            self.env().block_timestamp()
        }

        /// returns the block timestamp when the auction was created
        #[ink(message)]
        fn get_created_time(&self) -> Timestamp{
            *self.created_time
        }

        /// returns the block timestamp when the auction is allowed to be ended by non-beneficiaries
        #[ink(message)]
        fn get_end_time(&self) -> Timestamp{
            *self.end_time
        }

        /// returns the amount of time left in mileseconds until non-beneficiaries can end the auction
        #[ink(message)]
        fn get_time_left(&self) -> Timestamp{
            self.get_end_time().saturating_sub(self.get_time())
        }


        /// this function can be called to end the auction and returns a bool indicating whether the call was successful
        /// note that you can't end the auction more than once, the beneficiary can always end the auction and non-beneficiaries
        /// can end the auction after the end_time. Ending ends bidding but withdrawing is still allowed. The highest bid will
        /// be added to the beneficiary's withdraw balance
        #[ink(message)]
        fn end(&mut self) -> bool {
            
            //making sure the contract was not already ended
            if *self.ended {
                self.env().emit_event(Already_Ended {
                    highest_bidder: Some(self.get_highest_bidder()),
                    highest_bid: self.get_highest_bid(),
                });
                return false
            }
            //only allowed to end if you are the benificiary or the time is past end_time
            else if self.beneficiary != self.env().caller() && !self.time_end_allowed() {
                self.env().emit_event(Not_Beneficiary {
                    sender: Some(self.env().caller()),
                    beneficiary: Some(*self.beneficiary),
                });
                return false
            }


            self.ended.set(true);


            //add to the highest_bid to the beneficiary pending returns
            let beneficiary_curr_pending = self.curr_withdrawl_amount(self.get_beneficiary());
            self.pending_returns.insert(self.get_beneficiary(), 
                                        beneficiary_curr_pending + *self.highest_bid);


            // emit event
            self.env().emit_event(Ended {
                highest_bidder: Some(self.get_highest_bidder()),
                highest_bid: self.get_highest_bid(),
            });
            true
        }


        /// To call this funciton, money must be sent to the contract. Bids under the curent asking price are
        /// added to the senders withdraw balance. Bids higher than the asking price are locked in the contract
        /// until either the bid is trumped in which case the bid is returned or the bidding period is ended
        /// in which case the amount is transfered to the beificiary
        #[ink(message)] 
        fn bid(&mut self) -> bool {
            //the amount transfered to the contract ie the bid amount
            let amount: Balance = self.env().transferred_balance();
            let previous_highest_bid = self.get_highest_bid();
            let sender = self.env().caller();
            let sender_curr_pending = self.curr_withdrawl_amount(sender);

            
                        //if the bid is made after the voting closes, return false
            if *self.ended {
                self.pending_returns.insert(sender, amount + sender_curr_pending);
                // emit event
                self.env().emit_event(No_More_Bidding {
                    is_ended: *self.ended,
                    highest_bidder: Some(self.get_highest_bidder()),
                    highest_bid: self.get_highest_bid(),
                });
                return false
            }
            // if the bid is not higher than the starting price, then return false
            else if amount <= self.get_starting_price() {
                //value is to low so allow the sender to collect the funds
                self.pending_returns.insert(sender, amount + sender_curr_pending);

                // emit event
                self.env().emit_event(Failed_Bid_Lower_Than_Starting_Price {
                    attempted_bidder: Some(sender),
                    attempted_bid: amount,
                    starting_price: *self.starting_price,
                });
                return false
            }
            // if the bid is not higher than the current highest, then return false
            else if amount <= previous_highest_bid {
                //value is to low so allow the sender to collect the funds
                self.pending_returns.insert(sender, amount + sender_curr_pending);

                // emit event
                self.env().emit_event(Failed_Bid_Lower_Than_Highest_Bid {
                    attempted_bidder: Some(sender),
                    attempted_bid: amount,
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
            self.env().emit_event(New_Highest_Bid {
                previous_highest_bidder: Some(previous_highest_bidder),
                previous_highest_bid: previous_highest_bid,
                highest_bidder: Some(self.env().caller()),
                highest_bid: amount,
            });

            true
        }


        /// this function transfers all the sender's pending withdraw balance to the sender
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
                    //Since the amount is not returned re-add the amount to pending_returns
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
            let auction = Auction::new(5, 60);
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
            assert!(auction.is_ended());
            assert!(!auction.end());
        }

    }
}
