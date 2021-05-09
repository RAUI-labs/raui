use raui_core::{
    application::Application,
    interactive::{
        default_interactions_engine::{
            DefaultInteractionsEngine, DefaultInteractionsEngineResult, Interaction, PointerButton,
        },
        InteractionsEngine,
    },
    layout::CoordsMapping,
    widget::{
        component::interactive::navigation::{NavJump, NavScroll, NavSignal, NavTextChange},
        utils::Vec2,
    },
    Scalar,
};
use tetra::{
    input::{get_text_input, is_key_modifier_down, Key, KeyModifier, MouseButton},
    Context, Event,
};

#[derive(Debug)]
pub struct TetraInteractionsEngine {
    pub engine: DefaultInteractionsEngine,
    pub single_scroll_units: Vec2,
    pointer_position: Vec2,
}

impl Default for TetraInteractionsEngine {
    fn default() -> Self {
        Self::with_capacity(32, 32, 1024, 32, 32, 32, 32, 32)
    }
}

impl TetraInteractionsEngine {
    fn default_single_scroll_units() -> Vec2 {
        Vec2 { x: 10.0, y: 10.0 }
    }

    pub fn new() -> Self {
        Self {
            engine: Default::default(),
            single_scroll_units: Self::default_single_scroll_units(),
            pointer_position: Default::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        resize_listeners: usize,
        relative_layout_listeners: usize,
        interactions_queue: usize,
        containers: usize,
        buttons: usize,
        text_inputs: usize,
        scroll_views: usize,
        selected_chain: usize,
    ) -> Self {
        Self {
            engine: DefaultInteractionsEngine::with_capacity(
                resize_listeners,
                relative_layout_listeners,
                interactions_queue,
                containers,
                buttons,
                text_inputs,
                scroll_views,
                selected_chain,
            ),
            single_scroll_units: Self::default_single_scroll_units(),
            pointer_position: Default::default(),
        }
    }

    pub fn update(&mut self, context: &Context) {
        if self.engine.focused_text_input().is_some() {
            if let Some(text) = get_text_input(context) {
                for c in text.chars() {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::InsertCharacter(c),
                        )));
                }
            }
        }
    }

    pub fn event(&mut self, context: &Context, event: &Event, mapping: &CoordsMapping) {
        match event {
            Event::MouseMoved { position, .. } => {
                self.pointer_position = mapping.real_to_virtual_vec2(Vec2 {
                    x: position.x,
                    y: position.y,
                });
                self.engine
                    .interact(Interaction::PointerMove(self.pointer_position));
            }
            Event::MouseButtonPressed { button } => match button {
                MouseButton::Left => {
                    self.engine.interact(Interaction::PointerDown(
                        PointerButton::Trigger,
                        self.pointer_position,
                    ));
                }
                MouseButton::Right => {
                    self.engine.interact(Interaction::PointerDown(
                        PointerButton::Context,
                        self.pointer_position,
                    ));
                }
                _ => {}
            },
            Event::MouseButtonReleased { button } => match button {
                MouseButton::Left => {
                    self.engine.interact(Interaction::PointerUp(
                        PointerButton::Trigger,
                        self.pointer_position,
                    ));
                }
                MouseButton::Right => {
                    self.engine.interact(Interaction::PointerUp(
                        PointerButton::Context,
                        self.pointer_position,
                    ));
                }
                _ => {}
            },
            Event::MouseWheelMoved { amount } => {
                let value = Vec2 {
                    x: -self.single_scroll_units.x * amount.x as Scalar,
                    y: -self.single_scroll_units.y * amount.y as Scalar,
                };
                self.engine
                    .interact(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                        NavScroll::Units(value, true),
                    ))));
            }
            Event::KeyPressed { key } => {
                if self.engine.focused_text_input().is_some() {
                    match key {
                        Key::Left => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::MoveCursorLeft,
                                )))
                        }
                        Key::Right => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::MoveCursorRight,
                                )))
                        }
                        Key::Home => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::MoveCursorStart,
                                )))
                        }
                        Key::End => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::MoveCursorEnd,
                                )))
                        }
                        Key::Backspace => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::DeleteLeft,
                                )))
                        }
                        Key::Delete => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::DeleteRight,
                                )))
                        }
                        Key::Enter | Key::NumPadEnter => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::NewLine,
                                )))
                        }
                        Key::Escape => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::FocusTextInput(
                                    ().into(),
                                )));
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        Key::Up | Key::W => {
                            self.engine.interact(Interaction::Navigate(NavSignal::Up))
                        }
                        Key::Down | Key::S => {
                            self.engine.interact(Interaction::Navigate(NavSignal::Down))
                        }
                        Key::Left | Key::A => {
                            if is_key_modifier_down(context, KeyModifier::Shift) {
                                self.engine.interact(Interaction::Navigate(NavSignal::Prev));
                            } else {
                                self.engine.interact(Interaction::Navigate(NavSignal::Left));
                            }
                        }
                        Key::Right | Key::D => {
                            if is_key_modifier_down(context, KeyModifier::Shift) {
                                self.engine.interact(Interaction::Navigate(NavSignal::Next));
                            } else {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Right));
                            }
                        }
                        Key::Enter | Key::NumPadEnter | Key::Space => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::Accept(true)));
                        }
                        Key::Escape => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::Cancel(true)));
                        }
                        _ => {}
                    }
                }
            }
            Event::KeyReleased { key } => {
                if self.engine.focused_text_input().is_none() {
                    match key {
                        Key::Enter | Key::NumPadEnter | Key::Space => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::Accept(false)));
                        }
                        Key::Escape => {
                            self.engine
                                .interact(Interaction::Navigate(NavSignal::Cancel(false)));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for TetraInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}
