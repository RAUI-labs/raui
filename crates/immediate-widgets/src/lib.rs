use raui_immediate::*;

macro_rules! impl_imports {
    () => {
        #[allow(unused_imports)]
        use raui_core::widget::component::{
            containers::{
                anchor_box::*, area_box::*, content_box::*, context_box::*, flex_box::*,
                float_box::*, grid_box::*, hidden_box::*, horizontal_box::*, portal_box::*,
                responsive_box::*, scroll_box::*, size_box::*, switch_box::*, tabs_box::*,
                tooltip_box::*, variant_box::*, vertical_box::*, wrap_box::*,
            },
            interactive::{
                button::*, float_view::*, input_field::*, navigation::*, options_view::*,
                scroll_view::*, slider_view::*,
            },
        };
        #[allow(unused_imports)]
        use raui_core::widget::{
            component::{image_box::*, space_box::*, text_box::*},
            none_widget,
        };
        #[allow(unused_imports)]
        use raui_material::component::{
            containers::{
                context_paper::*, flex_paper::*, grid_paper::*, horizontal_paper::*,
                modal_paper::*, paper::*, scroll_paper::*, text_tooltip_paper::*, tooltip_paper::*,
                vertical_paper::*, window_paper::*, wrap_paper::*,
            },
            interactive::{
                button_paper::*, icon_button_paper::*, slider_paper::*, switch_button_paper::*,
                text_button_paper::*, text_field_paper::*,
            },
        };
        #[allow(unused_imports)]
        use raui_material::component::{icon_paper::*, switch_paper::*, text_paper::*};
    };
}

macro_rules! impl_slot_components {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name<R>(
                props: impl Into<raui_core::props::Props>,
                f: impl FnMut() -> R,
            ) -> R {
                impl_imports!();
                crate::slot_component(raui_core::make_widget!($name), props, f)
            }
        )+
    };
}

macro_rules! impl_list_components {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name<R>(
                props: impl Into<raui_core::props::Props>,
                f: impl FnMut() -> R,
            ) -> R {
                impl_imports!();
                crate::list_component(raui_core::make_widget!($name), props, f)
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
                impl_imports!();
                crate::content_component(raui_core::make_widget!($name), $content, props, f)
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
                impl_imports!();
                crate::component(raui_core::make_widget!($name), props)
            }
        )+
    };
}

pub mod core {
    impl_components! {
        image_box,
        nav_scroll_box_side_scrollbars,
        none_widget,
        space_box,
        text_box,
    }

    pub mod containers {
        impl_content_components! {
            "content":
            anchor_box,
            hidden_box,
            nav_scroll_box_content,
            pivot_box,
            portal_box,
            responsive_props_box,
            size_box,
            wrap_box,
        }

        impl_slot_components! {
            context_box,
            nav_scroll_box,
            portals_context_box,
            portals_tooltip_box,
            tooltip_box,
            variant_box,
        }

        impl_list_components! {
            content_box,
            flex_box,
            grid_box,
            horizontal_box,
            nav_content_box,
            nav_flex_box,
            nav_grid_box,
            nav_horizontal_box,
            nav_switch_box,
            nav_tabs_box,
            nav_vertical_box,
            responsive_box,
            switch_box,
            vertical_box,
        }
    }

    pub mod interactive {
        use raui_core::{
            make_widget,
            props::Props,
            widget::{
                component::interactive::{
                    button::ButtonProps,
                    input_field::{TextInputProps, TextInputState},
                    navigation::NavTrackingProps,
                    options_view::{OptionsViewProps, OptionsViewProxy},
                    slider_view::{SliderViewProps, SliderViewProxy},
                },
                utils::Vec2,
            },
        };
        use raui_immediate::{begin, end, pop, push, use_state};
        use std::str::FromStr;

        #[derive(Debug, Default, Copy, Clone)]
        pub struct ImmediateTracking {
            pub state: NavTrackingProps,
            pub prev: NavTrackingProps,
        }

        impl ImmediateTracking {
            pub fn pointer_delta_factor(&self) -> Vec2 {
                Vec2 {
                    x: self.state.factor.x - self.prev.factor.x,
                    y: self.state.factor.y - self.prev.factor.y,
                }
            }

            pub fn pointer_delta_unscaled(&self) -> Vec2 {
                Vec2 {
                    x: self.state.unscaled.x - self.prev.unscaled.x,
                    y: self.state.unscaled.y - self.prev.unscaled.y,
                }
            }

            pub fn pointer_delta_ui_space(&self) -> Vec2 {
                Vec2 {
                    x: self.state.ui_space.x - self.prev.ui_space.x,
                    y: self.state.ui_space.y - self.prev.ui_space.y,
                }
            }

            pub fn pointer_moved(&self) -> bool {
                (self.state.factor.x - self.prev.factor.x)
                    + (self.state.factor.y - self.prev.factor.y)
                    > 1.0e-6
            }
        }

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

        pub fn tracking(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateTracking),
        ) -> ImmediateTracking {
            use crate::internal::*;
            let state = use_state(ImmediateTracking::default);
            let result = state.read().unwrap().to_owned();
            begin();
            f(result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_tracking)
                    .with_props(ImmediateTrackingProps { state: Some(state) })
                    .merge_props(props.into())
                    .named_slot("content", node),
            );
            result
        }

        pub fn self_tracking(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateTracking),
        ) -> ImmediateTracking {
            use crate::internal::*;
            let state = use_state(ImmediateTracking::default);
            let result = state.read().unwrap().to_owned();
            begin();
            f(result);
            let node = end().pop().unwrap_or_default();
            push(
                make_widget!(immediate_self_tracking)
                    .with_props(ImmediateTrackingProps { state: Some(state) })
                    .merge_props(props.into())
                    .named_slot("content", node),
            );
            result
        }

        pub fn button(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateButton),
        ) -> ImmediateButton {
            use crate::internal::*;
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

        pub fn options_view<T: OptionsViewProxy + Clone + 'static>(
            value: T,
            props: impl Into<Props>,
            mut f_items: impl FnMut(&T),
            mut f_content: impl FnMut(),
        ) -> T {
            let content = use_state(|| value.to_owned());
            let props = props.into();
            let result = content.read().unwrap().to_owned();
            begin();
            f_items(&result);
            let nodes = end();
            begin();
            f_content();
            let node = pop();
            push(
                make_widget!(raui_core::widget::component::interactive::options_view::options_view)
                    .merge_props(props)
                    .with_props(OptionsViewProps {
                        input: Some(content.into()),
                    })
                    .named_slot("content", node)
                    .listed_slots(nodes),
            );
            result
        }
    }
}

pub mod material {
    impl_components! {
        icon_paper,
        scroll_paper_side_scrollbars,
        switch_paper,
        text_paper,
    }

    pub mod containers {
        impl_slot_components! {
            context_paper,
            scroll_paper,
            tooltip_paper,
        }

        impl_content_components! {
            "content":
            modal_paper,
            text_tooltip_paper,
            wrap_paper,
        }

        impl_list_components! {
            flex_paper,
            grid_paper,
            horizontal_paper,
            nav_flex_paper,
            nav_grid_paper,
            nav_horizontal_paper,
            nav_vertical_paper,
            paper,
            vertical_paper,
        }
    }

    pub mod interactive {
        use crate::core::interactive::ImmediateButton;
        use raui_core::{
            make_widget,
            props::Props,
            widget::component::interactive::{
                input_field::{TextInputProps, TextInputState},
                slider_view::{SliderViewProps, SliderViewProxy},
            },
        };
        use raui_immediate::{begin, end, push, use_state};
        use std::str::FromStr;

        pub fn button_paper(
            props: impl Into<Props>,
            mut f: impl FnMut(ImmediateButton),
        ) -> ImmediateButton {
            use crate::internal::*;
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
    use crate::core::interactive::{ImmediateButton, ImmediateTracking};
    use raui_core::{
        ManagedLazy, Prefab, PropsData, make_widget, pre_hooks,
        widget::{
            component::interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, button},
                input_field::{TextInputState, input_field, text_input},
                navigation::{
                    NavTrackingNotifyMessage, NavTrackingNotifyProps, self_tracking, tracking,
                    use_nav_tracking_self,
                },
                slider_view::slider_view,
            },
            context::WidgetContext,
            node::WidgetNode,
        },
    };
    use raui_material::component::interactive::{
        button_paper::button_paper_impl,
        icon_button_paper::icon_button_paper_impl,
        slider_paper::{numeric_slider_paper_impl, slider_paper_impl},
        switch_button_paper::switch_button_paper_impl,
        text_button_paper::text_button_paper_impl,
        text_field_paper::text_field_paper_impl,
    };
    use serde::{Deserialize, Serialize};

    #[derive(PropsData, Default, Clone, Serialize, Deserialize)]
    #[props_data(raui_core::props::PropsData)]
    #[prefab(raui_core::Prefab)]
    pub struct ImmediateTrackingProps {
        #[serde(default, skip)]
        pub state: Option<ManagedLazy<ImmediateTracking>>,
    }

    impl std::fmt::Debug for ImmediateTrackingProps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ImmediateTrackingProps")
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
    #[prefab(raui_core::Prefab)]
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

    fn use_immediate_tracking(ctx: &mut WidgetContext) {
        ctx.props
            .write(NavTrackingNotifyProps(ctx.id.to_owned().into()));

        if let Ok(props) = ctx.props.read::<ImmediateTrackingProps>() {
            let state = props.state.as_ref().unwrap();
            let mut state = state.write().unwrap();
            state.prev = state.state;
        }

        ctx.life_cycle.change(|ctx| {
            if let Ok(props) = ctx.props.read::<ImmediateTrackingProps>()
                && let Some(state) = props.state.as_ref()
                && let Some(mut state) = state.write()
            {
                for msg in ctx.messenger.messages {
                    if let Some(msg) = msg.as_any().downcast_ref::<NavTrackingNotifyMessage>() {
                        state.state = msg.state;
                    }
                }
            }
        });
    }

    fn use_immediate_button(ctx: &mut WidgetContext) {
        ctx.props.write(ButtonNotifyProps(ctx.id.to_owned().into()));

        if let Ok(props) = ctx.props.read::<ImmediateButtonProps>() {
            let state = props.state.as_ref().unwrap();
            let mut state = state.write().unwrap();
            state.prev = state.state;
        }

        ctx.life_cycle.change(|ctx| {
            if let Ok(props) = ctx.props.read::<ImmediateButtonProps>()
                && let Some(state) = props.state.as_ref()
                && let Some(mut state) = state.write()
            {
                for msg in ctx.messenger.messages {
                    if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                        state.state = msg.state;
                    }
                }
            }
        });
    }

    fn use_immediate_text_input(ctx: &mut WidgetContext) {
        if let Ok(data) = ctx.state.read_cloned::<TextInputState>()
            && let Ok(props) = ctx.props.read::<ImmediateTextInputProps>()
        {
            let state = props.state.as_ref().unwrap();
            let mut state = state.write().unwrap();
            *state = data;
        }
    }

    #[pre_hooks(use_immediate_tracking)]
    pub(crate) fn immediate_tracking(mut ctx: WidgetContext) -> WidgetNode {
        tracking(ctx)
    }

    #[pre_hooks(use_immediate_tracking, use_nav_tracking_self)]
    pub(crate) fn immediate_self_tracking(mut ctx: WidgetContext) -> WidgetNode {
        self_tracking(ctx)
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
