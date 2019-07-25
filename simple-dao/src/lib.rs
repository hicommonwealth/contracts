#![cfg_attr(not(any(test, feature = "std")), no_std)]
use parity_codec::*;

use ink_core::{
    env::DefaultSrmlTypes,
    storage,
};
use ink_lang::contract;

/// Role types
#[derive(Encode, Decode, Clone)]
pub enum RoleType {
    Default,
    Admin,
}



contract! {
    #![env = DefaultSrmlTypes]

    event Vote {
        voter: Option<AccountId>,
        vote: bool,
    }

    struct SimpleDao {
        // voters have a role and an id (identified by their place in registration)
        voters: storage::HashMap<AccountId, (RoleType, u32)>,
        // proposals have an id and are represented by a 32-byte description
        proposals: storage::HashMap<u32, [u8; 32]>,
        // votes map a proposal id and a voter id to a binary vote
        vote_index: storage::HashMap<(u32, u32), u32>,
        // vote arrays
        votes: storage::HashMap<u32, storage::Vec<bool>>,

    }

    impl Deploy for SimpleDao {
        fn deploy(&mut self) {
            self.voters.insert(env.caller(), (RoleType::Admin, self.voters.len()));
        }
    }

    impl SimpleDao {
        pub(external) fn register(&mut self) {
            if self.voters.get(&env.caller()).is_none() {
                self.voters.insert(env.caller(), (RoleType::Default, self.voters.len()));
            }
        }

        pub(external) fn create_proposal(&mut self, descriptor: [u8; 32]) {
            let new_prop_id = self.proposals.len();
            self.proposals.insert(new_prop_id, descriptor);

        }

        pub(external) fn vote(&mut self, prop_id: u32, vote: bool) {
            if prop_id > self.proposals.len() { return; }
            // grab voter if already registered
            if let Some(voter) = self.voters.get(&env.caller()) {
                // grab existing or new vote vec
                match self.votes.get_mut(&prop_id) {
                    Some(votes) => {
                        // if the voter has voted, change vote, otherwise create vote record
                        if let Some(vote_inx) = self.vote_index.get(&(prop_id, voter.1)) {
                            votes[*vote_inx] = vote;
                        } else {
                            self.vote_index.insert((prop_id, voter.1), votes.len());
                            votes.push(vote);
                        }

                        // emit vote event
                        env.emit(Vote {
                            voter: Some(env.caller()),
                            vote: vote,
                        });
                    },
                    None => {},
                }
            }
        }

        pub(external) fn get_proposal(&self, prop_id: u32) -> ([u8; 32], u32, u32) {
            if prop_id > self.proposals.len() { return ([0x0; 32], 0, 0); }
            // get proposal description
            let desc = match self.proposals.get(&prop_id) {
                Some(d) => *d,
                None => [0x0; 32],
            };

            let (yes_votes, no_votes) = match self.votes.get(&prop_id) {
                Some(votes) => {
                    let mut yes_votes = 0;
                    let mut no_votes = 0;
                    for v in votes.iter() {
                        if *v { yes_votes += 1 }
                        else { no_votes += 1 };
                    }
                    (yes_votes, no_votes)
                },
                None => (0, 0),
            };

            // return values
            (desc, yes_votes, no_votes)
        }

        pub(external) fn get_voter_count(&self) -> u32 {
            self.voters.len()
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn should_have_one_voter_on_deploy() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let contract = SimpleDao::deploy_mock();
        assert_eq!(contract.get_voter_count(), 1);
    }

    #[test]
    fn should_register_voters() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = SimpleDao::deploy_mock();

        let bob = AccountId::from([0x01; 32]);
        env::test::set_caller::<Types>(bob);
        contract.register();

        let charlie = AccountId::from([0x02; 32]);
        env::test::set_caller::<Types>(charlie);
        contract.register();
        assert_eq!(contract.get_voter_count(), 3);
    }

    #[test]
    fn should_create_and_vote_on_a_proposal() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = SimpleDao::deploy_mock();
        let descriptor = [0x09; 32];
        contract.create_proposal(descriptor);
        contract.vote(0, true);

        let bob = AccountId::from([0x01; 32]);
        env::test::set_caller::<Types>(bob);
        contract.register();
        contract.vote(0, false);

        let charlie = AccountId::from([0x02; 32]);
        env::test::set_caller::<Types>(charlie);
        contract.register();
        contract.vote(0, false);
        assert_eq!(contract.get_voter_count(), 3);
    }
}
