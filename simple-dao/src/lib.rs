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
        vote: [u8; 1],
    }

    struct SimpleDao {
        // voters have a role and an id (identified by their place in registration)
        voters: storage::HashMap<AccountId, (RoleType, u32)>,
        // proposals have an id and are represented by a 32-byte description
        proposals: storage::HashMap<u32, [u8; 32]>,
        // vote indices map a (prop_id, voter_id) => vote_index
        vote_index: storage::HashMap<(u32, u32), u32>,
        // max vote index for particular proposal
        next_vote_index: storage::HashMap<u32, u32>,
        // vote arrays indexed by (prop_id, vote_index) => vote (256 options)
        votes: storage::HashMap<(u32, u32), [u8; 1]>,

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

        pub(external) fn vote(&mut self, prop_id: u32, vote: [u8; 1]) {
            if prop_id > self.proposals.len() { return; }
            // grab voter if already registered
            if let Some(voter) = self.voters.get(&env.caller()) {
                // grab existing or new vote vec
                match self.next_vote_index.get_mut(&(prop_id)) {
                    Some(next_index) => {
                        // if the voter has voted, change vote, otherwise create vote record
                        if let Some(vote_inx) = self.vote_index.get(&(prop_id, voter.1)) {
                            if let Some(v) = self.votes.get_mut(&(prop_id, *vote_inx)) {
                                *v = vote;
                            }
                        } else {
                            // add new vote at the next vote index
                            self.votes.insert((prop_id, *next_index), vote);
                            // map (prop_id, voter_id) => vote_index
                            self.vote_index.insert((prop_id, voter.1), *next_index);
                            // increment next index
                            *next_index += 1;
                        }
                    },
                    None => {
                        // map (prop_id, voter_id) to zero'th index, as the first voter for the proposal
                        self.vote_index.insert((prop_id, voter.1), 0);
                        // set the next vote index to 1
                        self.next_vote_index.insert(prop_id, 1);
                        // map (prop_id, vote_index) => vote for the first voter
                        self.votes.insert((prop_id, 0), vote);
                    },
                }

                // emit vote event
                env.emit(Vote {
                    voter: Some(env.caller()),
                    vote: vote,
                });
            }
        }

        pub(external) fn get_proposal(&self, prop_id: u32) -> ([u8; 32], [u32; 256]) {
            if prop_id > self.proposals.len() { return ([0x0; 32], [0; 256]); }
            // get proposal description
            let desc = match self.proposals.get(&prop_id) {
                Some(d) => *d,
                None => [0x0; 32],
            };

            let mut tally = [0; 256];
            if let Some(next_index) = self.next_vote_index.get(&prop_id) {
                for i in 0..*next_index {
                    if let Some(vote) = self.votes.get(&(prop_id, i)) {
                        tally[vote[0] as usize] += 1;
                    }
                }
            }

            // return values
            (desc, tally)
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
        contract.vote(0, [0]);

        let bob = AccountId::from([0x01; 32]);
        env::test::set_caller::<Types>(bob);
        contract.register();
        contract.vote(0, [0]);

        let charlie = AccountId::from([0x02; 32]);
        env::test::set_caller::<Types>(charlie);
        contract.register();
        contract.vote(0, [0]);
        assert_eq!(contract.get_voter_count(), 3);

        let result = contract.get_proposal(0);
        assert_eq!(result.1[0], 3);
        assert_eq!(result.0, [0x09; 32]);
    }

    #[test]
    fn should_let_one_change_a_vote() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);
        let mut contract = SimpleDao::deploy_mock();
        let descriptor = [0x09; 32];
        contract.create_proposal(descriptor);
        contract.vote(0, [1]);
        let mut result = contract.get_proposal(0);
        assert_eq!(result.1[1], 1);
        contract.vote(0, [0]);
        result = contract.get_proposal(0);
        assert_eq!(result.1[0], 1);
        assert_eq!(result.1[1], 0);
    }
}
