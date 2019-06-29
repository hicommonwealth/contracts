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
        pub(external) fn register(&mut self) {
            if self.voters.get(&env.caller()).is_none() {
                self.voters.insert(env.caller(), RoleType::Default);
            }
        }

        pub(external) fn vote(&mut self, proposal: u32, vote: bool) {
            let vote_hook = if vote {
                &mut self.yes_votes
            } else {
                &mut self.no_votes
            };

            if let Some(_) = self.voters.get(&env.caller()) {
                let votes = match vote_hook.get(&proposal) {
                    Some(ct) => *ct as u32,
                    None => 0,
                };

                vote_hook.insert(proposal, votes + 1);
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
}
