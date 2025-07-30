//! Interactivity traits

pub mod default_interactions_engine;

use crate::application::Application;

pub trait InteractionsEngine<R, E> {
    fn perform_interactions(&mut self, app: &mut Application) -> Result<R, E>;
}

impl InteractionsEngine<(), ()> for () {
    fn perform_interactions(&mut self, _: &mut Application) -> Result<(), ()> {
        Ok(())
    }
}
