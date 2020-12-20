use ggez::graphics::{Font, Image};
use std::collections::HashMap;

#[derive(Default)]
pub struct GgezResources {
    pub fonts: HashMap<String, Font>,
    pub images: HashMap<String, Image>,
}
