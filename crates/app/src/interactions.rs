use glutin::event::{
    ElementState, Event, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use raui_core::{
    application::Application,
    interactive::{
        InteractionsEngine,
        default_interactions_engine::{
            DefaultInteractionsEngine, DefaultInteractionsEngineResult, Interaction, PointerButton,
        },
    },
    layout::CoordsMapping,
    widget::{
        component::interactive::navigation::{NavJump, NavScroll, NavSignal, NavTextChange},
        utils::Vec2,
    },
};

#[derive(Debug)]
pub struct AppInteractionsEngine {
    pub engine: DefaultInteractionsEngine,
    pub single_scroll_units: Vec2,
    pointer_position: Vec2,
    modifiers: ModifiersState,
}

impl Default for AppInteractionsEngine {
    fn default() -> Self {
        Self::with_capacity(32, 32, 1024, 32, 32, 32, 32, 32, 32)
    }
}

impl AppInteractionsEngine {
    fn default_single_scroll_units() -> Vec2 {
        Vec2 { x: 10.0, y: 10.0 }
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
        tracking: usize,
        selected_chain: usize,
    ) -> Self {
        let mut engine = DefaultInteractionsEngine::with_capacity(
            resize_listeners,
            relative_layout_listeners,
            interactions_queue,
            containers,
            buttons,
            text_inputs,
            scroll_views,
            tracking,
            selected_chain,
        );
        engine.deselect_when_no_button_found = true;
        Self {
            engine,
            single_scroll_units: Self::default_single_scroll_units(),
            pointer_position: Default::default(),
            modifiers: Default::default(),
        }
    }

    pub fn event(&mut self, event: &Event<()>, mapping: &CoordsMapping) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.modifiers = *modifiers;
                }
                WindowEvent::ReceivedCharacter(character) => {
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::TextChange(
                            NavTextChange::InsertCharacter(*character),
                        )));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.pointer_position = mapping.real_to_virtual_vec2(
                        Vec2 {
                            x: position.x as _,
                            y: position.y as _,
                        },
                        false,
                    );
                    self.engine
                        .interact(Interaction::PointerMove(self.pointer_position));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let value = match delta {
                        MouseScrollDelta::LineDelta(x, y) => Vec2 {
                            x: -self.single_scroll_units.x * *x,
                            y: -self.single_scroll_units.y * *y,
                        },
                        MouseScrollDelta::PixelDelta(delta) => Vec2 {
                            x: -delta.x as _,
                            y: -delta.y as _,
                        },
                    };
                    self.engine
                        .interact(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                            NavScroll::Units(value, true),
                        ))));
                }
                WindowEvent::MouseInput { state, button, .. } => match state {
                    ElementState::Pressed => match button {
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
                    ElementState::Released => match button {
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
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(key) = input.virtual_keycode {
                            if self.engine.focused_text_input().is_some() {
                                match key {
                                    VirtualKeyCode::Left => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::MoveCursorLeft),
                                        ))
                                    }
                                    VirtualKeyCode::Right => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::MoveCursorRight),
                                        ))
                                    }
                                    VirtualKeyCode::Home => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::MoveCursorStart),
                                        ))
                                    }
                                    VirtualKeyCode::End => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::MoveCursorEnd),
                                        ))
                                    }
                                    VirtualKeyCode::Back => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::DeleteLeft),
                                        ))
                                    }
                                    VirtualKeyCode::Delete => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::DeleteRight),
                                        ))
                                    }
                                    VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::TextChange(NavTextChange::NewLine),
                                        ))
                                    }
                                    VirtualKeyCode::Escape => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::FocusTextInput(().into()),
                                        ));
                                    }
                                    _ => {}
                                }
                            } else {
                                match key {
                                    VirtualKeyCode::Up => {
                                        self.engine.interact(Interaction::Navigate(NavSignal::Up))
                                    }
                                    VirtualKeyCode::Down => {
                                        self.engine.interact(Interaction::Navigate(NavSignal::Down))
                                    }
                                    VirtualKeyCode::Left => {
                                        if self.modifiers.shift() {
                                            self.engine
                                                .interact(Interaction::Navigate(NavSignal::Prev));
                                        } else {
                                            self.engine
                                                .interact(Interaction::Navigate(NavSignal::Left));
                                        }
                                    }
                                    VirtualKeyCode::Right => {
                                        if self.modifiers.shift() {
                                            self.engine
                                                .interact(Interaction::Navigate(NavSignal::Next));
                                        } else {
                                            self.engine
                                                .interact(Interaction::Navigate(NavSignal::Right));
                                        }
                                    }
                                    VirtualKeyCode::Return
                                    | VirtualKeyCode::NumpadEnter
                                    | VirtualKeyCode::Space => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::Accept(true),
                                        ));
                                    }
                                    VirtualKeyCode::Escape | VirtualKeyCode::Back => {
                                        self.engine.interact(Interaction::Navigate(
                                            NavSignal::Cancel(true),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if input.state == ElementState::Released
                        && let Some(key) = input.virtual_keycode
                        && self.engine.focused_text_input().is_none()
                    {
                        match key {
                            VirtualKeyCode::Return
                            | VirtualKeyCode::NumpadEnter
                            | VirtualKeyCode::Space => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Accept(false)));
                            }
                            VirtualKeyCode::Escape | VirtualKeyCode::Back => {
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
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for AppInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}
