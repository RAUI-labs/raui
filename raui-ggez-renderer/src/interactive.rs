use ggez::{
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse,
    },
    Context,
};
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
        component::interactive::navigation::{NavSignal, NavTextChange},
        utils::Vec2,
    },
};

#[derive(Debug, Default)]
pub struct GgezInteractionsEngine {
    pub engine: DefaultInteractionsEngine,
    pointer_position: Vec2,
    trigger_button: bool,
    trigger_context: bool,
}

impl GgezInteractionsEngine {
    pub fn new() -> Self {
        Self::default()
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
            ..Default::default()
        }
    }

    pub fn update(&mut self, ctx: &mut Context, mapping: &CoordsMapping) {
        let mouse_pos = mouse::position(ctx);
        let mouse_pos = mapping.real_to_virtual_vec2(Vec2 {
            x: mouse_pos.x,
            y: mouse_pos.y,
        });
        if (mouse_pos.x - self.pointer_position.x).abs() > 1.0e-6
            || (mouse_pos.y - self.pointer_position.y).abs() > 1.0e-6
        {
            self.engine.interact(Interaction::PointerMove(mouse_pos));
            self.pointer_position = mouse_pos;
        }
        let mouse_trigger = mouse::button_pressed(ctx, mouse::MouseButton::Left);
        let mouse_context = mouse::button_pressed(ctx, mouse::MouseButton::Right);
        if self.trigger_button != mouse_trigger {
            if mouse_trigger {
                self.engine
                    .interact(Interaction::PointerDown(PointerButton::Trigger, mouse_pos));
            } else {
                self.engine
                    .interact(Interaction::PointerUp(PointerButton::Trigger, mouse_pos));
            }
            self.trigger_button = mouse_trigger;
        }
        if self.trigger_context != mouse_context {
            if mouse_context {
                self.engine
                    .interact(Interaction::PointerDown(PointerButton::Context, mouse_pos));
            } else {
                self.engine
                    .interact(Interaction::PointerUp(PointerButton::Context, mouse_pos));
            }
            self.trigger_context = mouse_context;
        }
    }

    pub fn text_input_event(&mut self, character: char) {
        if self.engine.focused_text_input().is_some() {
            self.engine
                .interact(Interaction::Navigate(NavSignal::TextChange(
                    NavTextChange::InsertCharacter(character),
                )));
        }
    }

    pub fn key_down_event(&mut self, keycode: KeyCode, keymods: KeyMods) {
        if self.engine.focused_text_input().is_some() {
            match keycode {
                KeyCode::Left => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::MoveCursorLeft,
                        )))
                }
                KeyCode::Right => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::MoveCursorRight,
                        )))
                }
                KeyCode::Home => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::MoveCursorStart,
                        )))
                }
                KeyCode::End => self
                    .engine
                    .interact(Interaction::Navigate(NavSignal::TextChange(
                        NavTextChange::MoveCursorEnd,
                    ))),
                KeyCode::Back => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::DeleteLeft,
                        )))
                }
                KeyCode::Delete => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::DeleteRight,
                        )))
                }
                KeyCode::Return | KeyCode::NumpadEnter => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::NewLine,
                        )))
                }
                KeyCode::Escape => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::FocusTextInput(().into())));
                }
                _ => {}
            }
        } else {
            match keycode {
                KeyCode::Up | KeyCode::W => {
                    self.engine.interact(Interaction::Navigate(NavSignal::Up))
                }
                KeyCode::Down | KeyCode::S => {
                    self.engine.interact(Interaction::Navigate(NavSignal::Down))
                }
                KeyCode::Left | KeyCode::A => {
                    if keymods.contains(KeyMods::SHIFT) {
                        self.engine.interact(Interaction::Navigate(NavSignal::Prev));
                    } else {
                        self.engine.interact(Interaction::Navigate(NavSignal::Left));
                    }
                }
                KeyCode::Right | KeyCode::D => {
                    if keymods.contains(KeyMods::SHIFT) {
                        self.engine.interact(Interaction::Navigate(NavSignal::Next));
                    } else {
                        self.engine
                            .interact(Interaction::Navigate(NavSignal::Right));
                    }
                }
                KeyCode::Return | KeyCode::NumpadEnter | KeyCode::Space => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::Accept(true)));
                }
                KeyCode::Escape => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::Cancel(true)));
                }
                _ => {}
            }
        }
    }

    pub fn key_up_event(&mut self, keycode: KeyCode) {
        if self.engine.focused_text_input().is_some() {
            return;
        }
        match keycode {
            KeyCode::Return | KeyCode::NumpadEnter | KeyCode::Space => {
                self.engine
                    .interact(Interaction::Navigate(NavSignal::Accept(false)));
            }
            KeyCode::Escape => {
                self.engine
                    .interact(Interaction::Navigate(NavSignal::Cancel(false)));
            }
            _ => {}
        }
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for GgezInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}
