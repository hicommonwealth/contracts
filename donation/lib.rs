#![cfg_attr(not(feature = "std"), no_std)]
//cargo +nightly test 
use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod donation {
    use ink_core::storage;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct Donation {
        beneficiary: storage::Value<AccountId>, // this is the account that is asking for donations 
        largest_donor: storage::Value<AccountId>, // largest donor
        largest_total_donation: storage::Value<Balance>, // largest combined donated amount
        total_raised: storage::Value<Balance>, //total riased
        pending_collection: storage::Value<Balance>, //the amound of donations the benificiary has yet to collect
        donations: storage::HashMap<AccountId, Balance>, //map of donors to total amount donated
    }

    // events
    #[ink(event)]
    struct Created {
        #[ink(topic)]
        beneficiary: Option<AccountId>,
    }


    #[ink(event)]
    struct New_Largest_Donor {
        #[ink(topic)]
        prev_largest_donor: Option<AccountId>,
        #[ink(topic)]
        prev_largest_total_donation: Balance,
        #[ink(topic)]
        largest_donor: Option<AccountId>,
        #[ink(topic)]
        largest_total_donation: Balance,
    }


    #[ink(event)]
    struct New_Donation {
        #[ink(topic)]
        donor: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
    }
                
  
    #[ink(event)]
    struct Current_Funds_Withdrew{
        #[ink(topic)]
        beneficiary: Option<AccountId>,
        #[ink(topic)]
        amount_withdrew: Balance,
    }

    #[ink(event)]
    struct Not_Authorised_to_Withdraw{
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        beneficiary: Option<AccountId>,
        #[ink(topic)]
        amount_attempted: Balance,
    }

    impl Donation {
        // Constructor that initializes the `beneficiary` value to the given `id`.
        #[ink(constructor)]
        fn new(&mut self, id: AccountId) {
            self.beneficiary.set(self.env().caller());
            self.largest_donor.set(self.env().caller());
            self.largest_total_donation.set(0);
            self.total_raised.set(0);
            self.pending_collection.set(0);
            self.donations.insert(self.env().caller(), 0);

             // emit event
            self.env().emit_event(Created {
                beneficiary: Some(id),
            });
        }

        // Constructors can delegate to other constructors.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(self.env().caller())
        }

        #[ink(message)]
        fn get_beneficiary(&self) -> AccountId {
            *self.beneficiary
        }

        #[ink(message)]
        fn get_largest_donor(&self) -> AccountId {
            *self.largest_donor
        }


        #[ink(message)]
        fn get_largest_total_donation(&self) -> Balance {
            *self.largest_total_donation
        }

        #[ink(message)]
        fn get_total_raised(&self) -> Balance {
            *self.total_raised
        }

         #[ink(message)]
        fn get_pending_collection(&self) -> Balance {
            *self.pending_collection
        }

        #[ink(message)]
        fn get_my_total_donations(&self) -> Balance {
            *self.donations.get(&self.env().caller()).unwrap_or(&0)
        }

        #[ink(message)]
        fn get_accounts_total_donations(&self, id: AccountId) -> Balance {
            *self.donations.get(&id).unwrap_or(&0)
        }

        #[ink(message)] 
        fn make_dontation(&mut self) -> () {
            //the amount transfered to the contract ie the bid amount
            let sender = self.env().caller();
            let amount: Balance = self.env().transferred_balance();

            //emit event
            self.env().emit_event(New_Donation {
                donor: Some(sender),
                amount: amount,
            });

            //update total_raised
            self.total_raised.set(self.get_total_raised() + amount);

            //update pending_collection
            self.pending_collection.set(self.get_pending_collection() + amount);

            //update donations
            let sender_prev_total_donations = self.get_accounts_total_donations(sender);
            self.donations.insert(sender, 
                                  sender_prev_total_donations + amount);

            //update largest_donor and largest_total_donation
            let sender_total_donations = self.get_accounts_total_donations(sender);

            let prev_largest_donor = self.get_largest_donor();
            let prev_largest_total_donations = self.get_accounts_total_donations(prev_largest_donor);

            //if the sender is already the largest donor just update the largest donation amount
            if sender == prev_largest_donor {
                self.largest_total_donation.set(sender_total_donations);
            }
            // if the bid is not larger than the starting price, then return false
            else if sender_total_donations > prev_largest_total_donations {
                self.largest_donor.set(sender);
                self.largest_total_donation.set(sender_total_donations);

                self.env().emit_event(New_Largest_Donor {
                    prev_largest_donor: Some(prev_largest_donor),
                    prev_largest_total_donation: prev_largest_total_donations,
                    largest_donor: Some(sender),
                    largest_total_donation: sender_total_donations,
                });

            }
            
        }

        #[ink(message)]
        fn collect_pending_amount(&mut self) -> bool {
            let sender = self.env().caller();
            let curr_pending_collections = self.get_pending_collection();

            //checks that the caller is the beneficiary
            if self.beneficiary != sender {
                //emit event
                self.env().emit_event(Current_Funds_Withdrew {
                    sender: Some(sender),
                    beneficiary: Some(self.get_beneficiary()),
                    amount_withdrew: curr_pending_collections,
                });
                return false
            }

            self.pending_collection.set(0);

            //give largest bid to beneficiary
            match self.env().transfer(self.get_beneficiary(), curr_pending_collections) {
                Ok(now) => (),
                Err(error) => {
                    //since the beneficiary does not recieve the money, the bid does not end
                    self.pending_collection.set(curr_pending_collections);
                    return false
                }
            };

            self.env().emit_event(Current_Funds_Withdrew {
                beneficiary: Some(self.get_beneficiary()),
                amount_withdrew: curr_pending_collections,
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

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let donation = Donation::default();
            assert_eq!(donation.beneficiary, donation.get_beneficiary());
            assert_eq!(donation.largest_donor, donation.get_largest_donor());
            assert_eq!(donation.largest_total_donation, 0);
            assert_eq!(donation.pending_collection, 0);
            assert_eq!(donation.donations.len(), 1);
        }

        

    }
}
