use crate::model::inventory::{Inventory, ItemsDatabase};
use raui::prelude::*;

pub fn inventory(context: WidgetContext) -> WidgetNode {
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
        .map(|(index, (_, count, item))| {
            let col = index as i32 % 5;
            let row = index as i32 / 5;
            make_widget!(inventory_item)
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
                        make_widget!(grid_paper)
                            .with_props(ContentBoxItemLayout {
                                anchors: Rect {
                                    left: 0.5,
                                    right: 0.5,
                                    top: 0.0,
                                    bottom: 1.0,
                                },
                                align: Vec2 { x: 0.5, y: 0.0 },
                                ..Default::default()
                            })
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
    let icon = props.read_cloned_or_default::<String>();
    let count = props.read_cloned_or_default::<usize>();

    make_widget!(paper)
        .key(key)
        .listed_slot(
            make_widget!(image_box)
                .key("icon")
                .with_props(ContentBoxItemLayout {
                    margin: 10.0.into(),
                    ..Default::default()
                })
                .with_props(ImageBoxProps::image(icon)),
        )
        .listed_slot(make_widget!(text_paper).with_props(TextPaperProps {
            text: count.to_string(),
            variant: "inventory-item-count".to_owned(),
            use_main_color: true,
            ..Default::default()
        }))
        .into()
}
