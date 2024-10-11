use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

const DATA: &str = "data";
const INDEX: &str = "index";

struct AppData {
    index: Managed<ViewModelValue<usize>>,
}

fn use_app(ctx: &mut WidgetContext) {
    ctx.life_cycle.mount(|mut ctx| {
        ctx.view_models
            .bindings(DATA, INDEX)
            .unwrap()
            .bind(ctx.id.to_owned());
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    let idref = WidgetRef::default();
    let mut app_data = ctx
        .view_models
        .view_model_mut(DATA)
        .unwrap()
        .write::<AppData>()
        .unwrap();

    // We use content box marked with portals container as root to provide space
    // for option views to anchor thier content into.
    make_widget!(content_box)
        .idref(idref.clone())
        .with_shared_props(PortalsContainer(idref.into()))
        .listed_slot(
            // Options view is basically a button that toggles its content anchored
            // to itself. You can think of dropdown/context menus, but actually it
            // can present any user widgets, not only in a list - content widget can
            // be anything that takes listed slots and layouts them in some fashion.
            make_widget!(options_view)
                .with_props(ContentBoxItemLayout {
                    anchors: 0.25.into(),
                    margin: Rect {
                        left: -150.0,
                        right: -150.0,
                        top: -30.0,
                        bottom: -30.0,
                    },
                    ..Default::default()
                })
                // Here we provide options view index source, which tells which option
                // has to be shown.
                .with_props(OptionsViewProps {
                    input: Some(app_data.index.lazy().into()),
                })
                .with_props(NavItemActive)
                // Here we tell how to anchor content relatively to options box button.
                .with_props(PivotBoxProps {
                    pivot: [0.0, 1.0].into(),
                    align: 0.0.into(),
                })
                // Additionally we might want to provide size of the content.
                .with_props(SizeBoxProps {
                    width: SizeBoxSizeValue::Exact(300.0),
                    height: SizeBoxSizeValue::Exact(400.0),
                    ..Default::default()
                })
                // Here we provide content widget. Preferably without existing children,
                // because options will be appended, not replacing old children.
                // Lists are obvious choice but you could also put slots into a grid,
                // or even freeform content box to for example make a map with city
                // icons to select!
                .named_slot(
                    "content",
                    // Since this list will be injected into portal container, which is
                    // content box, we can make that list kept in bounds of the container.
                    make_widget!(vertical_box).with_props(ContentBoxItemLayout {
                        keep_in_bounds: true.into(),
                        ..Default::default()
                    }),
                )
                // And last but not least, we provide items as listed slots.
                // Each provided widget will be wrapped in button that will notify
                // options view about selected option.
                .listed_slot(
                    make_widget!(option)
                        .with_props("Hello".to_owned())
                        .with_props(NavItemActive),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("World".to_owned())
                        .with_props(NavItemActive),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("this".to_owned())
                        .with_props(NavItemActive),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("is".to_owned())
                        .with_props(NavItemActive),
                )
                .listed_slot(
                    make_widget!(option)
                        .with_props("dropdown".to_owned())
                        .with_props(NavItemActive),
                ),
        )
        .into()
}

fn option(ctx: WidgetContext) -> WidgetNode {
    // Since options are wrapped in buttons, we can read their button state and use it.
    let ButtonProps {
        selected, trigger, ..
    } = ctx.props.read_cloned_or_default();
    let color = if trigger {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    } else if selected {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    };
    let text = ctx.props.read_cloned_or_default::<String>();
    // We can also read options view mode property to render our option widget
    // diferently, depending if option is shown as selected or as content item.
    let text = match ctx.props.read_cloned_or_default::<OptionsViewMode>() {
        OptionsViewMode::Selected => format!("> {}", text),
        OptionsViewMode::Option => format!("# {}", text),
    };

    make_widget!(content_box)
        .listed_slot(make_widget!(image_box).with_props(ImageBoxProps::colored(color)))
        .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
            text,
            font: TextBoxFont {
                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                size: 32.0,
            },
            horizontal_align: TextBoxHorizontalAlign::Center,
            vertical_align: TextBoxVerticalAlign::Middle,
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            ..Default::default()
        }))
        .into()
}

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            DATA,
            ViewModel::produce(|properties| AppData {
                index: Managed::new(ViewModelValue::new(
                    Default::default(),
                    properties.notifier(INDEX),
                )),
            }),
        );

    App::new(AppConfig::default().title("Options View")).run(app);
}
