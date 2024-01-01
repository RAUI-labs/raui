use raui_immediate::*;

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        core::*,
        core::{containers::*, interactive::*},
        material::*,
        material::{containers::*, interactive::*},
    };
}

macro_rules! impl_list_components {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name<R>(
                props: impl Into<raui_core::props::Props>,
                f: impl FnMut() -> R,
            ) -> R {
                use raui_core::prelude::*;
                #[allow(unused_imports)]
                use raui_material::prelude::*;
                crate::list_component(make_widget!($name), props, f)
            }
        )+
    };
}

macro_rules! impl_content_components {
    ($content:literal : $($name:ident),+ $(,)?) => {
        $(
            pub fn $name<R>(
                props: impl Into<raui_core::props::Props>,
                f: impl FnMut() -> R,
            ) -> R {
                use raui_core::prelude::*;
                #[allow(unused_imports)]
                use raui_material::prelude::*;
                crate::content_component(make_widget!($name), $content, props, f)
            }
        )+
    };
}

macro_rules! impl_components {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name(
                props: impl Into<raui_core::props::Props>,
            ) {
                use raui_core::prelude::*;
                #[allow(unused_imports)]
                use raui_material::prelude::*;
                crate::component(make_widget!($name), props)
            }
        )+
    };
}

pub mod core {
    pub use raui_core::widget::{
        component::{image_box::ImageBoxProps, space_box::SpaceBoxProps, text_box::TextBoxProps},
        unit::{
            content::{ContentBoxItem, ContentBoxItemLayout},
            flex::{FlexBoxItem, FlexBoxItemLayout},
            grid::{GridBoxItem, GridBoxItemLayout},
            image::{
                ImageBoxAspectRatio, ImageBoxColor, ImageBoxFrame, ImageBoxImage,
                ImageBoxImageScaling, ImageBoxMaterial, ImageBoxProcedural, ImageBoxSizeValue,
            },
            size::SizeBoxSizeValue,
            text::{
                TextBoxDirection, TextBoxFont, TextBoxHorizontalAlign, TextBoxSizeValue,
                TextBoxVerticalAlign,
            },
        },
        utils::*,
    };

    impl_components! {
        image_box,
        text_box,
        space_box,
    }

    pub mod containers {
        pub use raui_core::widget::component::containers::{
            anchor_box::{AnchorNotifyProps, AnchorProps, PivotBoxProps},
            content_box::ContentBoxProps,
            context_box::ContextBoxProps,
            flex_box::FlexBoxProps,
            grid_box::GridBoxProps,
            hidden_box::HiddenBoxProps,
            horizontal_box::HorizontalBoxProps,
            portal_box::PortalsContainer,
            scroll_box::ScrollBoxOwner,
            size_box::SizeBoxProps,
            switch_box::SwitchBoxProps,
            tabs_box::{TabPlateProps, TabsBoxProps, TabsBoxTabsLocation},
            variant_box::VariantBoxProps,
            vertical_box::VerticalBoxProps,
            wrap_box::WrapBoxProps,
        };

        impl_content_components! {
            "content":
            anchor_box,
            pivot_box,
            context_box,
            portals_context_box,
            hidden_box,
            portal_box,
            size_box,
            tooltip_box,
            portals_tooltip_box,
            wrap_box,
        }

        impl_list_components! {
            content_box,
            nav_content_box,
            flex_box,
            nav_flex_box,
            grid_box,
            nav_grid_box,
            horizontal_box,
            nav_horizontal_box,
            nav_scroll_box,
            nav_scroll_box_content,
            nav_scroll_box_side_scrollbars,
            switch_box,
            nav_switch_box,
            nav_tabs_box,
            variant_box,
            nav_vertical_box,
            vertical_box,
        }
    }

    pub mod interactive {
        use raui_core::{
            make_widget, props::Props, widget::component::interactive::slider_view::SliderViewProxy,
        };
        use raui_immediate::{begin, end, push, use_state};
        use std::str::FromStr;

        pub use raui_core::widget::component::interactive::{
            button::{ButtonNotifyProps, ButtonProps},
            input_field::{
                TextInput, TextInputControlNotifyProps, TextInputMode, TextInputNotifyProps,
                TextInputProps, TextInputState,
            },
            navigation::{
                NavContainerActive, NavDirection, NavItemActive, NavJump, NavJumpActive,
                NavJumpLooped, NavJumpMapProps, NavJumpMode, NavScroll, NavTextChange,
                NavTrackingActive, NavType,
            },
            scroll_view::{ScrollViewNotifyProps, ScrollViewRange, ScrollViewState},
        };

        #[derive(Debug, Default, Copy, Clone)]
        pub struct ImmediateButton {
            pub state: ButtonProps,
            pub prev: ButtonProps,
        }

        impl ImmediateButton {
            pub fn select_start(&self) -> bool {
                !self.prev.selected && self.state.selected
            }

            pub fn select_stop(&self) -> bool {
                self.prev.selected && !self.state.selected
            }

            pub fn select_changed(&self) -> bool {
                self.prev.selected != self.state.selected
            }

            pub fn trigger_start(&self) -> bool {
                !self.prev.trigger && self.state.trigger
            }

            pub fn trigger_stop(&self) -> bool {
                self.prev.trigger && !self.state.trigger
            }

            pub fn trigger_changed(&self) -> bool {
                self.prev.trigger != self.state.trigger
            }

            pub fn context_start(&self) -> bool {
                !self.prev.context && self.state.context
            }

            pub fn context_stop(&self) -> bool {
                self.prev.context && !self.state.context
            }

            pub fn context_changed(&self) -> bool {
                self.prev.context != self.state.context
            }
        }

        impl_content_components! {
            "content":
            navigation_barrier,
        }

        pub fn button(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateButton),
        ) -> ImmediateButton {
            use crate::internal::*;
            use raui_core::prelude::*;
            let state = use_state(ImmediateButton::default);
            let result = state.read().unwrap().to_owned();
            begin();
            f(result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_button)
                    .with_props(ImmediateButtonProps { state: Some(state) })
                    .merge_props(props.into())
                    .named_slot("content", node),
            );
            result
        }

        pub fn text_input<T: ToString + FromStr + Send + Sync>(
            value: &T,
            props: impl Into<Props>,
            mut f: impl FnMut(&str, TextInputState),
        ) -> (Option<T>, TextInputState) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_string());
            let props = props.into();
            let TextInputProps { allow_new_line, .. } = props.read_cloned_or_default();
            let text_state = use_state(TextInputState::default);
            let text_result = text_state.read().unwrap().to_owned();
            if !text_result.focused {
                *content.write().unwrap() = value.to_string();
            }
            let result = content.read().unwrap().to_string();
            begin();
            f(&result, text_result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_text_input)
                    .with_props(ImmediateTextInputProps {
                        state: Some(text_state),
                    })
                    .merge_props(props)
                    .with_props(TextInputProps {
                        allow_new_line,
                        text: Some(content.into()),
                    })
                    .named_slot("content", node),
            );
            (result.parse().ok(), text_result)
        }

        pub fn input_field<T: ToString + FromStr + Send + Sync>(
            value: &T,
            props: impl Into<Props>,
            mut f: impl FnMut(&str, TextInputState, ImmediateButton),
        ) -> (Option<T>, TextInputState, ImmediateButton) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_string());
            let props = props.into();
            let TextInputProps { allow_new_line, .. } = props.read_cloned_or_default();
            let text_state = use_state(TextInputState::default);
            let text_result = text_state.read().unwrap().to_owned();
            let button_state = use_state(ImmediateButton::default);
            let button_result = button_state.read().unwrap().to_owned();
            if !text_result.focused {
                *content.write().unwrap() = value.to_string();
            }
            let result = content.read().unwrap().to_string();
            begin();
            f(&result, text_result, button_result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_input_field)
                    .with_props(ImmediateTextInputProps {
                        state: Some(text_state),
                    })
                    .with_props(ImmediateButtonProps {
                        state: Some(button_state),
                    })
                    .merge_props(props)
                    .with_props(TextInputProps {
                        allow_new_line,
                        text: Some(content.into()),
                    })
                    .named_slot("content", node),
            );
            (result.parse().ok(), text_result, button_result)
        }

        pub fn slider_view<T: SliderViewProxy + Clone + 'static>(
            value: T,
            props: impl Into<Props>,
            mut f: impl FnMut(&T, ImmediateButton),
        ) -> (T, ImmediateButton) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_owned());
            let props = props.into();
            let SliderViewProps {
                from,
                to,
                direction,
                ..
            } = props.read_cloned_or_default();
            let button_state = use_state(ImmediateButton::default);
            let button_result = button_state.read().unwrap().to_owned();
            let result = content.read().unwrap().to_owned();
            begin();
            f(&result, button_result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_slider_view)
                    .with_props(ImmediateButtonProps {
                        state: Some(button_state),
                    })
                    .merge_props(props)
                    .with_props(SliderViewProps {
                        input: Some(content.into()),
                        from,
                        to,
                        direction,
                    })
                    .named_slot("content", node),
            );
            (result, button_result)
        }
    }
}

pub mod material {
    pub use raui_material::component::{
        icon_paper::{IconImage, IconPaperProps},
        switch_paper::SwitchPaperProps,
        text_paper::TextPaperProps,
    };

    impl_components! {
        icon_paper,
        switch_paper,
        text_paper,
    }

    pub mod containers {
        pub use raui_material::component::containers::{
            context_paper::ContextPaperProps,
            modal_paper::ModalPaperProps,
            paper::{PaperContentLayoutProps, PaperProps},
            scroll_paper::SideScrollbarsPaperProps,
            tooltip_paper::TooltipPaperProps,
        };

        impl_list_components! {
            context_paper,
            flex_paper,
            nav_flex_paper,
            grid_paper,
            nav_grid_paper,
            horizontal_paper,
            nav_horizontal_paper,
            modal_paper,
            paper,
            scroll_paper,
            scroll_paper_side_scrollbars,
            text_tooltip_paper,
            tooltip_paper,
            vertical_paper,
            nav_vertical_paper,
            wrap_paper,
        }
    }

    pub mod interactive {
        use crate::core::interactive::ImmediateButton;
        use raui_core::{
            props::Props,
            widget::component::interactive::{
                input_field::TextInputState, slider_view::SliderViewProxy,
            },
        };
        use raui_immediate::{begin, end, push, use_state};
        use std::str::FromStr;

        pub use raui_material::component::interactive::{
            button_paper::ButtonPaperOverrideStyle, text_field_paper::TextFieldPaperProps,
        };

        pub fn button_paper(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateButton),
        ) -> ImmediateButton {
            use crate::internal::*;
            use raui_core::prelude::*;
            let state = use_state(ImmediateButton::default);
            let result = state.read().unwrap().to_owned();
            begin();
            f(result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_button_paper)
                    .with_props(ImmediateButtonProps { state: Some(state) })
                    .merge_props(props.into())
                    .named_slot("content", node),
            );
            result
        }

        pub fn icon_button_paper(props: impl Into<Props>) -> ImmediateButton {
            use crate::internal::*;
            use raui_core::prelude::*;
            let state = use_state(ImmediateButton::default);
            let result = state.read().unwrap().to_owned();
            push(
                make_widget!(immediate_icon_button_paper)
                    .with_props(ImmediateButtonProps { state: Some(state) })
                    .merge_props(props.into()),
            );
            result
        }

        pub fn switch_button_paper(props: impl Into<Props>) -> ImmediateButton {
            use crate::internal::*;
            use raui_core::prelude::*;
            let state = use_state(ImmediateButton::default);
            let result = state.read().unwrap().to_owned();
            push(
                make_widget!(immediate_switch_button_paper)
                    .with_props(ImmediateButtonProps { state: Some(state) })
                    .merge_props(props.into()),
            );
            result
        }

        pub fn text_button_paper(props: impl Into<Props>) -> ImmediateButton {
            use crate::internal::*;
            use raui_core::prelude::*;
            let state = use_state(ImmediateButton::default);
            let result = state.read().unwrap().to_owned();
            push(
                make_widget!(immediate_text_button_paper)
                    .with_props(ImmediateButtonProps { state: Some(state) })
                    .merge_props(props.into()),
            );
            result
        }

        pub fn text_field_paper<T: ToString + FromStr + Send + Sync>(
            value: &T,
            props: impl Into<Props>,
        ) -> (Option<T>, TextInputState, ImmediateButton) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_string());
            let props = props.into();
            let TextInputProps { allow_new_line, .. } =
                props.read_cloned_or_default::<TextInputProps>();
            let text_state = use_state(TextInputState::default);
            let text_result = text_state.read().unwrap().to_owned();
            let button_state = use_state(ImmediateButton::default);
            let button_result = button_state.read().unwrap().to_owned();
            if !text_result.focused {
                *content.write().unwrap() = value.to_string();
            }
            let result = content.read().unwrap().to_string();
            push(
                make_widget!(immediate_text_field_paper)
                    .with_props(ImmediateTextInputProps {
                        state: Some(text_state),
                    })
                    .with_props(ImmediateButtonProps {
                        state: Some(button_state),
                    })
                    .merge_props(props)
                    .with_props(TextInputProps {
                        allow_new_line,
                        text: Some(content.into()),
                    }),
            );
            (result.parse().ok(), text_result, button_result)
        }

        pub fn slider_paper<T: SliderViewProxy + Clone + 'static>(
            value: T,
            props: impl Into<Props>,
            mut f: impl FnMut(&T, ImmediateButton),
        ) -> (T, ImmediateButton) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_owned());
            let props = props.into();
            let SliderViewProps {
                from,
                to,
                direction,
                ..
            } = props.read_cloned_or_default();
            let button_state = use_state(ImmediateButton::default);
            let button_result = button_state.read().unwrap().to_owned();
            let result = content.read().unwrap().to_owned();
            begin();
            f(&result, button_result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_slider_paper)
                    .with_props(ImmediateButtonProps {
                        state: Some(button_state),
                    })
                    .merge_props(props)
                    .with_props(SliderViewProps {
                        input: Some(content.into()),
                        from,
                        to,
                        direction,
                    })
                    .named_slot("content", node),
            );
            (result, button_result)
        }

        pub fn numeric_slider_paper<T: SliderViewProxy + Clone + 'static>(
            value: T,
            props: impl Into<Props>,
        ) -> (T, ImmediateButton) {
            use crate::internal::*;
            use raui_core::prelude::*;
            let content = use_state(|| value.to_owned());
            let props = props.into();
            let SliderViewProps {
                from,
                to,
                direction,
                ..
            } = props.read_cloned_or_default();
            let button_state = use_state(ImmediateButton::default);
            let button_result = button_state.read().unwrap().to_owned();
            let result = content.read().unwrap().to_owned();
            push(
                make_widget!(immediate_numeric_slider_paper)
                    .with_props(ImmediateButtonProps {
                        state: Some(button_state),
                    })
                    .merge_props(props)
                    .with_props(SliderViewProps {
                        input: Some(content.into()),
                        from,
                        to,
                        direction,
                    }),
            );
            (result, button_result)
        }
    }
}

mod internal {
    use super::core::interactive::ImmediateButton;
    use raui_core::prelude::*;
    use raui_material::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(PropsData, Default, Clone, Serialize, Deserialize)]
    #[props_data(raui_core::props::PropsData)]
    pub struct ImmediateButtonProps {
        #[serde(default, skip)]
        pub state: Option<ManagedLazy<ImmediateButton>>,
    }

    impl std::fmt::Debug for ImmediateButtonProps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ImmediateButtonProps")
                .field(
                    "state",
                    &self
                        .state
                        .as_ref()
                        .and_then(|state| state.read())
                        .map(|state| *state),
                )
                .finish()
        }
    }

    #[derive(PropsData, Default, Clone, Serialize, Deserialize)]
    #[props_data(raui_core::props::PropsData)]
    pub struct ImmediateTextInputProps {
        #[serde(default, skip)]
        pub state: Option<ManagedLazy<TextInputState>>,
    }

    impl std::fmt::Debug for ImmediateTextInputProps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ImmediateTextInputProps")
                .finish_non_exhaustive()
        }
    }

    fn use_immediate_button(ctx: &mut WidgetContext) {
        ctx.props.write(ButtonNotifyProps(ctx.id.to_owned().into()));

        if let Ok(props) = ctx.props.read::<ImmediateButtonProps>() {
            let state = props.state.as_ref().unwrap();
            let mut state = state.write().unwrap();
            state.prev = state.state;
        }

        ctx.life_cycle.change(|ctx| {
            if let Ok(props) = ctx.props.read::<ImmediateButtonProps>() {
                if let Some(state) = props.state.as_ref() {
                    if let Some(mut state) = state.write() {
                        for msg in ctx.messenger.messages {
                            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                                state.state = msg.state;
                            }
                        }
                    }
                }
            }
        });
    }

    fn use_immediate_text_input(ctx: &mut WidgetContext) {
        if let Ok(data) = ctx.state.read_cloned::<TextInputState>() {
            if let Ok(props) = ctx.props.read::<ImmediateTextInputProps>() {
                let state = props.state.as_ref().unwrap();
                let mut state = state.write().unwrap();
                *state = data;
            }
        }
    }

    #[pre_hooks(use_immediate_button)]
    pub(crate) fn immediate_button(mut ctx: WidgetContext) -> WidgetNode {
        button(ctx)
    }

    #[pre_hooks(use_immediate_text_input)]
    pub(crate) fn immediate_text_input(mut ctx: WidgetContext) -> WidgetNode {
        text_input(ctx)
    }

    #[pre_hooks(use_immediate_text_input, use_immediate_button)]
    pub(crate) fn immediate_input_field(mut ctx: WidgetContext) -> WidgetNode {
        input_field(ctx)
    }

    #[pre_hooks(use_immediate_button)]
    pub(crate) fn immediate_slider_view(mut ctx: WidgetContext) -> WidgetNode {
        slider_view(ctx)
    }

    pub(crate) fn immediate_button_paper(ctx: WidgetContext) -> WidgetNode {
        button_paper_impl(make_widget!(immediate_button), ctx)
    }

    pub(crate) fn immediate_icon_button_paper(ctx: WidgetContext) -> WidgetNode {
        icon_button_paper_impl(make_widget!(immediate_button_paper), ctx)
    }

    pub(crate) fn immediate_switch_button_paper(ctx: WidgetContext) -> WidgetNode {
        switch_button_paper_impl(make_widget!(immediate_button_paper), ctx)
    }

    pub(crate) fn immediate_text_button_paper(ctx: WidgetContext) -> WidgetNode {
        text_button_paper_impl(make_widget!(immediate_button_paper), ctx)
    }

    pub(crate) fn immediate_text_field_paper(ctx: WidgetContext) -> WidgetNode {
        text_field_paper_impl(make_widget!(immediate_input_field), ctx)
    }

    pub(crate) fn immediate_slider_paper(ctx: WidgetContext) -> WidgetNode {
        slider_paper_impl(make_widget!(immediate_slider_view), ctx)
    }

    pub(crate) fn immediate_numeric_slider_paper(ctx: WidgetContext) -> WidgetNode {
        numeric_slider_paper_impl(make_widget!(immediate_slider_paper), ctx)
    }
}
