use ggez::{
    input::{keyboard::KeyCode, mouse},
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
    widget::{component::interactive::button::TextChange, utils::Vec2},
    Scalar,
};

#[derive(Default)]
pub struct GgezInteractionsEngine {
    pub engine: DefaultInteractionsEngine,
    pointer_position: (Scalar, Scalar),
    trigger_button: bool,
    trigger_context: bool,
}

impl GgezInteractionsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(buttons: usize, interactions_queue: usize) -> Self {
        Self {
            engine: DefaultInteractionsEngine::with_capacity(buttons, interactions_queue),
            ..Default::default()
        }
    }

    pub fn update(&mut self, ctx: &mut Context, mapping: &CoordsMapping) {
        let mouse_pos = mouse::position(ctx);
        if (mouse_pos.x - self.pointer_position.0).abs() > 1.0e-6
            || (mouse_pos.y - self.pointer_position.1).abs() > 1.0e-6
        {
            let Vec2 { x, y } = mapping.real_to_virtual_vec2(Vec2 {
                x: mouse_pos.x,
                y: mouse_pos.y,
            });
            self.engine.interact(Interaction::PointerMove(x, y));
            self.pointer_position = (mouse_pos.x, mouse_pos.y);
        }
        let mouse_trigger = mouse::button_pressed(ctx, mouse::MouseButton::Left);
        let mouse_context = mouse::button_pressed(ctx, mouse::MouseButton::Right);
        if self.trigger_button != mouse_trigger {
            if mouse_trigger {
                self.engine.interact(Interaction::PointerDown(
                    PointerButton::Trigger,
                    mouse_pos.x,
                    mouse_pos.y,
                ));
            } else {
                self.engine.interact(Interaction::PointerUp(
                    PointerButton::Trigger,
                    mouse_pos.x,
                    mouse_pos.y,
                ));
            }
            self.trigger_button = mouse_trigger;
        }
        if self.trigger_context != mouse_context {
            if mouse_context {
                self.engine.interact(Interaction::PointerDown(
                    PointerButton::Context,
                    mouse_pos.x,
                    mouse_pos.y,
                ));
            } else {
                self.engine.interact(Interaction::PointerUp(
                    PointerButton::Context,
                    mouse_pos.x,
                    mouse_pos.y,
                ));
            }
            self.trigger_context = mouse_context;
        }
    }

    pub fn text_input_event(&mut self, character: char) {
        self.engine
            .interact(Interaction::TextChange(TextChange::InsertCharacter(
                character,
            )));
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Left => self
                .engine
                .interact(Interaction::TextChange(TextChange::MoveCursorLeft)),
            KeyCode::Right => self
                .engine
                .interact(Interaction::TextChange(TextChange::MoveCursorRight)),
            KeyCode::Home => self
                .engine
                .interact(Interaction::TextChange(TextChange::MoveCursorStart)),
            KeyCode::End => self
                .engine
                .interact(Interaction::TextChange(TextChange::MoveCursorEnd)),
            KeyCode::Back => self
                .engine
                .interact(Interaction::TextChange(TextChange::DeleteLeft)),
            KeyCode::Delete => self
                .engine
                .interact(Interaction::TextChange(TextChange::DeleteRight)),
            KeyCode::Return | KeyCode::NumpadEnter => self
                .engine
                .interact(Interaction::TextChange(TextChange::NewLine)),
            _ => {}
        }
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for GgezInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}
