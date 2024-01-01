//! Theme-able RAUI components

pub mod component;
pub mod theme;

use raui_core::{application::Application, widget::FnWidget};

pub fn setup(app: &mut Application) {
    app.register_props::<component::containers::context_paper::ContextPaperProps>(
        "ContextPaperProps",
    );
    app.register_props::<component::containers::modal_paper::ModalPaperProps>("ModalPaperProps");
    app.register_props::<component::containers::paper::PaperProps>("PaperProps");
    app.register_props::<component::containers::paper::PaperContentLayoutProps>(
        "PaperContentLayoutProps",
    );
    app.register_props::<component::containers::tooltip_paper::TooltipPaperProps>(
        "TooltipPaperProps",
    );
    app.register_props::<component::containers::scroll_paper::SideScrollbarsPaperProps>(
        "SideScrollbarsPaperProps",
    );
    app.register_props::<component::containers::window_paper::WindowPaperProps>("WindowPaperProps");
    app.register_props::<component::icon_paper::IconPaperProps>("IconPaperProps");
    app.register_props::<component::interactive::button_paper::ButtonPaperOverrideStyle>(
        "ButtonPaperOverrideStyle",
    );
    app.register_props::<component::interactive::text_field_paper::TextFieldPaperProps>(
        "TextFieldPaperProps",
    );
    app.register_props::<component::switch_paper::SwitchPaperProps>("SwitchPaperProps");
    app.register_props::<component::text_paper::TextPaperProps>("TextPaperProps");
    app.register_props::<theme::ThemedWidgetProps>("ThemedWidgetProps");
    app.register_props::<theme::ThemeProps>("ThemeProps");

    app.register_component(
        "context_paper",
        FnWidget::pointer(component::containers::context_paper::context_paper),
    );
    app.register_component(
        "nav_flex_paper",
        FnWidget::pointer(component::containers::flex_paper::nav_flex_paper),
    );
    app.register_component(
        "flex_paper",
        FnWidget::pointer(component::containers::flex_paper::flex_paper),
    );
    app.register_component(
        "nav_grid_paper",
        FnWidget::pointer(component::containers::grid_paper::nav_grid_paper),
    );
    app.register_component(
        "grid_paper",
        FnWidget::pointer(component::containers::grid_paper::grid_paper),
    );
    app.register_component(
        "nav_horizontal_paper",
        FnWidget::pointer(component::containers::horizontal_paper::nav_horizontal_paper),
    );
    app.register_component(
        "horizontal_paper",
        FnWidget::pointer(component::containers::horizontal_paper::horizontal_paper),
    );
    app.register_component(
        "modal_paper",
        FnWidget::pointer(component::containers::modal_paper::modal_paper),
    );
    app.register_component(
        "paper",
        FnWidget::pointer(component::containers::paper::paper),
    );
    app.register_component(
        "scroll_paper",
        FnWidget::pointer(component::containers::scroll_paper::scroll_paper),
    );
    app.register_component(
        "scroll_paper_side_scrollbars",
        FnWidget::pointer(component::containers::scroll_paper::scroll_paper_side_scrollbars),
    );
    app.register_component(
        "text_tooltip_paper",
        FnWidget::pointer(component::containers::text_tooltip_paper::text_tooltip_paper),
    );
    app.register_component(
        "tooltip_paper",
        FnWidget::pointer(component::containers::tooltip_paper::tooltip_paper),
    );
    app.register_component(
        "nav_vertical_paper",
        FnWidget::pointer(component::containers::vertical_paper::nav_vertical_paper),
    );
    app.register_component(
        "vertical_paper",
        FnWidget::pointer(component::containers::vertical_paper::vertical_paper),
    );
    app.register_component(
        "window_paper",
        FnWidget::pointer(component::containers::window_paper::window_paper),
    );
    app.register_component(
        "window_title_controls_paper",
        FnWidget::pointer(component::containers::window_paper::window_title_controls_paper),
    );
    app.register_component(
        "wrap_paper",
        FnWidget::pointer(component::containers::wrap_paper::wrap_paper),
    );
    app.register_component(
        "icon_paper",
        FnWidget::pointer(component::icon_paper::icon_paper),
    );
    app.register_component(
        "button_paper",
        FnWidget::pointer(component::interactive::button_paper::button_paper),
    );
    app.register_component(
        "icon_button_paper",
        FnWidget::pointer(component::interactive::icon_button_paper::icon_button_paper),
    );
    app.register_component(
        "slider_paper",
        FnWidget::pointer(component::interactive::slider_paper::slider_paper),
    );
    app.register_component(
        "numeric_slider_paper",
        FnWidget::pointer(component::interactive::slider_paper::numeric_slider_paper),
    );
    app.register_component(
        "switch_button_paper",
        FnWidget::pointer(component::interactive::switch_button_paper::switch_button_paper),
    );
    app.register_component(
        "text_button_paper",
        FnWidget::pointer(component::interactive::text_button_paper::text_button_paper),
    );
    app.register_component(
        "text_field_paper",
        FnWidget::pointer(component::interactive::text_field_paper::text_field_paper),
    );
    app.register_component(
        "switch_paper",
        FnWidget::pointer(component::switch_paper::switch_paper),
    );
    app.register_component(
        "text_paper",
        FnWidget::pointer(component::text_paper::text_paper),
    );
}

pub mod prelude {
    pub use crate::{
        component::{
            containers::{
                context_paper::*, flex_paper::*, grid_paper::*, horizontal_paper::*,
                modal_paper::*, paper::*, scroll_paper::*, text_tooltip_paper::*, tooltip_paper::*,
                vertical_paper::*, window_paper::*, wrap_paper::*,
            },
            icon_paper::*,
            interactive::{
                button_paper::*, icon_button_paper::*, slider_paper::*, switch_button_paper::*,
                text_button_paper::*, text_field_paper::*,
            },
            switch_paper::*,
            text_paper::*,
        },
        theme::*,
    };
}
