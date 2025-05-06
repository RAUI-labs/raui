use crate::model::inventory::{Inventory, ItemsDatabase};
use raui::{
    core::{
        make_widget, pre_hooks,
        widget::{
            WidgetIdMetaParams,
            component::{
                containers::{
                    content_box::content_box,
                    grid_box::GridBoxProps,
                    size_box::{SizeBoxProps, size_box},
                },
                image_box::{ImageBoxProps, image_box},
                interactive::{
                    button::{ButtonNotifyMessage, ButtonNotifyProps},
                    navigation::NavItemActive,
                },
            },
            context::WidgetContext,
            node::WidgetNode,
            unit::{
                content::ContentBoxItemLayout,
                grid::GridBoxItemLayout,
                image::{ImageBoxImage, ImageBoxMaterial},
                size::SizeBoxSizeValue,
            },
            utils::{Color, IntRect, Rect, Vec2},
        },
    },
    material::component::{
        containers::{
            grid_paper::nav_grid_paper,
            window_paper::{WindowPaperProps, window_paper},
        },
        interactive::button_paper::button_paper,
        text_paper::{TextPaperProps, text_paper},
    },
};

#[pre_hooks(use_inventory)]
pub fn inventory(mut context: WidgetContext) -> WidgetNode {
    let inventory = context
        .view_models
        .view_model(Inventory::VIEW_MODEL)
        .unwrap()
        .read::<Inventory>()
        .unwrap();
    let database = context
        .view_models
        .view_model(ItemsDatabase::VIEW_MODEL)
        .unwrap()
        .read::<ItemsDatabase>()
        .unwrap();
    let items = inventory
        .owned(&database)
        .enumerate()
        .map(|(index, (id, count, item))| {
            let col = index as i32 % 5;
            let row = index as i32 / 5;
            make_widget!(inventory_item)
                .key(format!("{}?item={}", index, id))
                .with_props(ButtonNotifyProps(context.id.to_owned().into()))
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: col,
                        right: col + 1,
                        top: row,
                        bottom: row + 1,
                    },
                    margin: 6.0.into(),
                    ..Default::default()
                })
                .with_props(count)
                .with_props(item.icon.to_owned())
        });

    make_widget!(window_paper)
        .key("inventory")
        .with_props(WindowPaperProps {
            bar_margin: 20.0.into(),
            bar_height: Some(80.0),
            content_margin: 40.0.into(),
            ..Default::default()
        })
        .named_slot(
            "bar",
            make_widget!(text_paper)
                .key("title")
                .with_props(TextPaperProps {
                    text: "INVENTORY".to_owned(),
                    variant: "title".to_owned(),
                    use_main_color: true,
                    ..Default::default()
                }),
        )
        .named_slot(
            "content",
            make_widget!(content_box).listed_slot(
                make_widget!(size_box)
                    .with_props(ContentBoxItemLayout {
                        anchors: Rect {
                            left: 0.5,
                            right: 0.5,
                            top: 0.5,
                            bottom: 0.5,
                        },
                        align: Vec2 { x: 0.5, y: 0.5 },
                        ..Default::default()
                    })
                    .with_props(SizeBoxProps {
                        width: SizeBoxSizeValue::Exact(400.0),
                        height: SizeBoxSizeValue::Exact(400.0),
                        ..Default::default()
                    })
                    .named_slot(
                        "content",
                        make_widget!(nav_grid_paper)
                            .with_props(GridBoxProps {
                                cols: 5,
                                rows: 5,
                                ..Default::default()
                            })
                            .listed_slots(items),
                    ),
            ),
        )
        .into()
}

fn inventory_item(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let notify = props.read_cloned::<ButtonNotifyProps>().ok();
    let icon = props.read_cloned_or_default::<String>();
    let count = props.read_cloned_or_default::<usize>();

    make_widget!(button_paper)
        .key(key)
        .with_props(NavItemActive)
        .maybe_with_props(notify)
        .named_slot(
            "content",
            make_widget!(content_box)
                .key(key)
                .listed_slot(
                    make_widget!(image_box)
                        .key("icon")
                        .with_props(ContentBoxItemLayout {
                            margin: 16.0.into(),
                            ..Default::default()
                        })
                        .with_props(ImageBoxProps {
                            material: ImageBoxMaterial::Image(ImageBoxImage {
                                id: icon,
                                tint: Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                },
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(text_paper)
                        .with_props(ContentBoxItemLayout {
                            margin: 2.0.into(),
                            ..Default::default()
                        })
                        .with_props(TextPaperProps {
                            text: count.to_string(),
                            variant: "inventory-item-count".to_owned(),
                            ..Default::default()
                        }),
                ),
        )
        .into()
}

fn use_inventory(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        context
            .view_models
            .bindings(Inventory::VIEW_MODEL, Inventory::OWNED)
            .unwrap()
            .bind(context.id.to_owned());
    });

    context.life_cycle.change(|mut context| {
        let mut inventory = context
            .view_models
            .view_model_mut(Inventory::VIEW_MODEL)
            .unwrap()
            .write::<Inventory>()
            .unwrap();

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if let Some(id) = WidgetIdMetaParams::new(msg.sender.meta()).find_value("item") {
                    if msg.trigger_start() {
                        inventory.remove(id, 1);
                    }
                }
            }
        }
    });
}
