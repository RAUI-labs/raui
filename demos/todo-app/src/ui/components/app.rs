use super::{app_bar::app_bar, tasks_list::tasks_list};
use crate::model::{AppState, ThemeMode};
use raui::prelude::*;

fn new_theme(theme: ThemeMode) -> ThemeProps {
    let mut theme = match theme {
        ThemeMode::Light => new_light_theme(),
        ThemeMode::Dark => new_dark_theme(),
    };
    theme.text_variants.insert(
        "title".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "resources/fonts/Roboto/Roboto-Black.ttf".to_owned(),
                size: 24.0,
            },
            vertical_align: TextBoxVerticalAlign::Middle,
            ..Default::default()
        },
    );
    theme.text_variants.insert(
        "input".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "resources/fonts/Roboto/Roboto-Regular.ttf".to_owned(),
                size: 24.0,
            },
            ..Default::default()
        },
    );
    theme.text_variants.insert(
        "tooltip".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "resources/fonts/Roboto/Roboto-BoldItalic.ttf".to_owned(),
                size: 18.0,
            },
            ..Default::default()
        },
    );
    theme.text_variants.insert(
        "button".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "resources/fonts/Roboto/Roboto-Bold.ttf".to_owned(),
                size: 24.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Middle,
            ..Default::default()
        },
    );
    theme.switch_variants.insert(
        "checkbox".to_owned(),
        ThemedSwitchMaterial {
            on: ThemedImageMaterial::Image(ImageBoxImage {
                id: "resources/icons/check-box-on.png".to_owned(),
                ..Default::default()
            }),
            off: ThemedImageMaterial::Image(ImageBoxImage {
                id: "resources/icons/check-box-off.png".to_owned(),
                ..Default::default()
            }),
        },
    );
    theme
}

fn use_app(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        context
            .view_models
            .bindings(AppState::VIEW_MODEL, AppState::PROP_THEME)
            .unwrap()
            .bind(context.id.to_owned());
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
pub fn app(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key, view_models, ..
    } = context;
    let app_state = view_models
        .view_model::<AppState>(AppState::VIEW_MODEL)
        .unwrap();
    let idref = WidgetRef::default();

    make_widget!(paper)
        .key(key)
        .idref(idref.clone())
        .with_shared_props(PortalsContainer(idref.clone()))
        .with_shared_props(new_theme(app_state.theme()))
        .listed_slot(
            make_widget!(wrap_box)
                .key("wrap")
                .with_props(WrapBoxProps {
                    margin: 32.0.into(),
                    fill: true,
                })
                .named_slot(
                    "content",
                    make_widget!(vertical_box)
                        .key("list")
                        .with_props(VerticalBoxProps {
                            separation: 10.0,
                            ..Default::default()
                        })
                        .listed_slot(make_widget!(app_bar).key("app-bar").with_props(
                            FlexBoxItemLayout {
                                grow: 0.0,
                                shrink: 0.0,
                                ..Default::default()
                            },
                        ))
                        .listed_slot(make_widget!(tasks_list).key("tasks-list")),
                ),
        )
        .into()
}
