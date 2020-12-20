pub mod default_interactions_engine;

use crate::application::Application;

pub trait InteractionsEngine<E> {
    fn perform_interactions(&mut self, app: &Application) -> Result<(), E>;
}

impl InteractionsEngine<()> for () {
    fn perform_interactions(&mut self, _: &Application) -> Result<(), ()> {
        Ok(())
    }
}
