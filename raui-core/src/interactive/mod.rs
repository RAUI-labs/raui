pub mod default_interactions_engine;

use crate::application::Application;

pub trait InteractionsEngine<R, E> {
    fn perform_interactions(&mut self, app: &Application) -> Result<R, E>;
}

impl InteractionsEngine<(), ()> for () {
    fn perform_interactions(&mut self, _: &Application) -> Result<(), ()> {
        Ok(())
    }
}
