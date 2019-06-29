#![cfg_attr(not(any(test, feature = "std")), no_std)]
use parity_codec::*;
use ink_core::{
    env::DefaultSrmlTypes,
    memory::format,
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

    struct SimpleDao {
        voters: storage::HashMap<AccountId, RoleType>,
        yes_votes: storage::HashMap<u32, u32>,
        no_votes: storage::HashMap<u32, u32>
    }

    impl Deploy for SimpleDao {
        fn deploy(&mut self) {
            self.voters.insert(env.caller(), RoleType::Admin);
        }
    }

    impl SimpleDao {
        pub(external) fn add_voter(&mut self) {
            if self.voters.get(&env.caller()).is_none() {
                self.voters.insert(env.caller(), RoleType::Default);
            }
        }

        pub(external) fn vote(&mut self, proposal: u32, vote: bool) {
            if let Some(_) = self.voters.get(&env.caller()) {
                if vote {
                    let yes_votes = match self.yes_votes.get(&proposal) {
                        Some(ct) => *ct as u32,
                        None => 0,
                    };

                    self.yes_votes.insert(proposal, yes_votes + 1);
                } else {
                    let no_votes = match self.no_votes.get(&proposal) {
                        Some(ct) => *ct as u32,
                        None => 0,
                    };

                    self.no_votes.insert(proposal, no_votes + 1);
                }
            }
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
        let mut contract = SimpleDao::deploy_mock();
        assert_eq!(contract.get_voter_count(), 1);
    }
}
