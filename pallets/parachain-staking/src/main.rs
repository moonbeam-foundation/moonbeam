//! Simple binary to let users poke at the staking state machine directly without
//! the overhead of a blockchain service.
//!
//! My end-game is to compile this to wasm and let users eplore it through the browser.


use parachain_staking::mock::{
	two_collators_four_nominators,
    Origin,
    Stake,
    AccountId, Balance,
    Test,
};
use sp_io::TestExternalities;
// use sp_runtime::{traits::Zero, DispatchError};

/// A state machine that the user will poke at when running the cli. Maybe there is a better
/// name for this thing.
pub struct StateMachine {
    /// The test externalities that this state amchine will use
    ext: TestExternalities,
}

impl StateMachine {
    /// Create a new Parachain Staking state machine with two colaltors and four nominators.
    /// TODO could use more of a builder pattern here.
    pub fn new() -> Self {
        Self {
            ext: two_collators_four_nominators(),
        }
    }

    /// Make a new nomination
    pub fn nominate(&mut self, nominator: AccountId, collator: AccountId, amount: Balance) -> bool {
        self.ext.execute_with(|| {
            // TODO should I actually make an outer call and dispatch it here?
            match Stake::nominate(Origin::signed(nominator), collator, amount) {
                Ok(_) => true,
                Err(_e) => {
                    println!("It failed");
                    false
                }
            }
        })
    }

    /// See who all the current nominators are
    /// Although we need a mutable reference to call `execute_with` this function should not
    /// do any mutations
    /// TODO is there a bette rthing than `execute_with`?
    pub fn get_nominators(&mut self) -> Vec<AccountId> {
        self.ext.execute_with(|| {
            parachain_staking::NominatorState::<Test>::iter()
            .map(|(x, _)| x)
            .collect()
        })
    }
}
fn main() {
    println!("Welcome to the parachain staking test util.");

    // Create a State Machine to use as long as this program is running
    let mut machine = StateMachine::new();
    
    // Print current nominators
    println!("Nominators are: {:?}", machine.get_nominators());

    // Nominate new
    let success = machine.nominate(7, 1, 100);
    println!("Did the nominate call succeed? {:?}", success);

    // Print current nominators
    println!("Nominators are: {:?}", machine.get_nominators());

    println!("Leaving staking test util");
}