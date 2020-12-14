use ggez::{
    graphics::{self, Align, Font, Image, MeshBuilder, Scale, Text, TextFragment},
    Context,
};
use raui_core::{
    layout::Layout,
    renderer::Renderer,
    widget::{
        unit::{
            image::{ImageBoxImageScaling, ImageBoxMaterial},
            text::TextBoxAlignment,
            WidgetUnit,
        },
        utils::{lerp, Rect},
        WidgetId,
    },
    Scalar,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Error {
    CouldNotDrawImage(WidgetId),
    CouldNotBuildImageMesh(WidgetId),
    ImageResourceNotFound(WidgetId, String),
    WidgetHasNoLayout(WidgetId),
    UnsupportedImageMaterial(ImageBoxMaterial),
    UnsupportedWidget(WidgetUnit),
}

#[derive(Default)]
pub struct GgezResources {
    pub fonts: HashMap<String, Font>,
    pub images: HashMap<String, Image>,
}

pub struct GgezRenderer<'a> {
    context: &'a mut Context,
    resources: &'a mut GgezResources,
}

impl<'a> GgezRenderer<'a> {
    pub fn new(context: &'a mut Context, resources: &'a mut GgezResources) -> Self {
        Self { context, resources }
    }

    fn render_node(&mut self, unit: &WidgetUnit, layout: &Layout) -> Result<(), Error> {
        match unit {
            WidgetUnit::None => Ok(()),
            WidgetUnit::ContentBox(unit) => {
                let mut items = unit
                    .items
                    .iter()
                    .map(|item| (item.layout.depth, item))
                    .collect::<Vec<_>>();
                items.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());
                for (_, item) in items {
                    self.render_node(&item.slot, layout)?;
                }
                Ok(())
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    self.render_node(&item.slot, layout)?;
                }
                Ok(())
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    self.render_node(&item.slot, layout)?;
                }
                Ok(())
            }
            WidgetUnit::SizeBox(unit) => self.render_node(&unit.slot, layout),
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(color) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let rect = item.ui_space;
                        let mut builder = MeshBuilder::new();
                        builder.rectangle(
                            graphics::DrawMode::fill(),
                            graphics::Rect::new(rect.left, rect.top, rect.width(), rect.height()),
                            graphics::Color::new(color.r, color.g, color.b, color.a),
                        );
                        if let Ok(mesh) = builder.build(self.context) {
                            if graphics::draw(self.context, &mesh, graphics::DrawParam::default())
                                .is_ok()
                            {
                                Ok(())
                            } else {
                                Err(Error::CouldNotDrawImage(unit.id.to_owned()))
                            }
                        } else {
                            Err(Error::CouldNotBuildImageMesh(unit.id.to_owned()))
                        }
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
                ImageBoxMaterial::Image(image) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        if let Some(resource) = self.resources.images.get(&image.id) {
                            let rect = if let Some(aspect) = unit.content_keep_aspect_ratio {
                                let ox = item.ui_space.left;
                                let oy = item.ui_space.top;
                                let width = resource.width() as Scalar;
                                let height = resource.height() as Scalar;
                                if item.ui_space.width() >= item.ui_space.height() {
                                    if width >= height {
                                        let h = item.ui_space.height();
                                        let w = h * width / height;
                                        let o = lerp(
                                            0.0,
                                            item.ui_space.width() - w,
                                            aspect.horizontal_alignment,
                                        );
                                        Rect {
                                            left: o + ox,
                                            right: w + o + ox,
                                            top: oy,
                                            bottom: h + oy,
                                        }
                                    } else {
                                        let w = item.ui_space.width();
                                        let h = w * height / width;
                                        let o = lerp(
                                            0.0,
                                            item.ui_space.height() - h,
                                            aspect.vertical_alignment,
                                        );
                                        Rect {
                                            left: ox,
                                            right: w + ox,
                                            top: o + oy,
                                            bottom: h + o + oy,
                                        }
                                    }
                                } else {
                                    if width >= height {
                                        let w = item.ui_space.width();
                                        let h = w * height / width;
                                        let o = lerp(
                                            0.0,
                                            item.ui_space.height() - h,
                                            aspect.vertical_alignment,
                                        );
                                        Rect {
                                            left: ox,
                                            right: w + ox,
                                            top: o + oy,
                                            bottom: h + o + oy,
                                        }
                                    } else {
                                        let h = item.ui_space.height();
                                        let w = h * width / height;
                                        let o = lerp(
                                            0.0,
                                            item.ui_space.width() - w,
                                            aspect.horizontal_alignment,
                                        );
                                        Rect {
                                            left: o + ox,
                                            right: w + o + ox,
                                            top: oy,
                                            bottom: h + oy,
                                        }
                                    }
                                }
                            } else {
                                item.ui_space
                            };
                            let mut builder = MeshBuilder::new();
                            match image.scaling {
                                ImageBoxImageScaling::Strech => {
                                    let vertices = &[
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top],
                                            uv: [0.0, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top],
                                            uv: [1.0, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom],
                                            uv: [1.0, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom],
                                            uv: [0.0, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                    ];
                                    let indices = &[0, 1, 2, 2, 3, 0];
                                    builder.raw(vertices, indices, Some(resource.clone()));
                                }
                                ImageBoxImageScaling::Frame(v) => {
                                    let fx = v / resource.width() as Scalar;
                                    let fy = v / resource.height() as Scalar;
                                    let vertices = &[
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top],
                                            uv: [0.0, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.top],
                                            uv: [fx, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.top],
                                            uv: [1.0 - fx, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top],
                                            uv: [1.0, 0.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top + v],
                                            uv: [0.0, fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.top + v],
                                            uv: [fx, fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.top + v],
                                            uv: [1.0 - fx, fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top + v],
                                            uv: [1.0, fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom - v],
                                            uv: [0.0, 1.0 - fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.bottom - v],
                                            uv: [fx, 1.0 - fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.bottom - v],
                                            uv: [1.0 - fx, 1.0 - fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom - v],
                                            uv: [1.0, 1.0 - fy],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom],
                                            uv: [0.0, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.bottom],
                                            uv: [fx, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.bottom],
                                            uv: [1.0 - fx, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom],
                                            uv: [1.0, 1.0],
                                            color: [1.0, 1.0, 1.0, 1.0],
                                        },
                                    ];
                                    let indices = &[
                                        0, 1, 5, 5, 4, 0, 1, 2, 6, 6, 5, 1, 2, 3, 7, 7, 6, 2, 4, 5,
                                        9, 9, 8, 4, 5, 6, 10, 10, 9, 5, 6, 7, 11, 11, 10, 6, 8, 9,
                                        13, 13, 12, 8, 9, 10, 14, 14, 13, 9, 10, 11, 15, 15, 14,
                                        10,
                                    ];
                                    builder.raw(vertices, indices, Some(resource.clone()));
                                }
                            }
                            if let Ok(mesh) = builder.build(self.context) {
                                if graphics::draw(
                                    self.context,
                                    &mesh,
                                    graphics::DrawParam::default(),
                                )
                                .is_ok()
                                {
                                    Ok(())
                                } else {
                                    Err(Error::CouldNotDrawImage(unit.id.to_owned()))
                                }
                            } else {
                                Err(Error::CouldNotBuildImageMesh(unit.id.to_owned()))
                            }
                        } else {
                            Err(Error::ImageResourceNotFound(
                                unit.id.to_owned(),
                                image.id.to_owned(),
                            ))
                        }
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
                _ => Err(Error::UnsupportedImageMaterial(unit.material.clone())),
            },
            WidgetUnit::TextBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    if let Some(resource) = self.resources.fonts.get(&unit.font.name) {
                        let rect = item.ui_space;
                        let mut text = Text::new(TextFragment::new(unit.text.as_str()).color(
                            graphics::Color::new(
                                unit.color.r,
                                unit.color.g,
                                unit.color.b,
                                unit.color.a,
                            ),
                        ));
                        text.set_font(resource.clone(), Scale::uniform(unit.font.size));
                        text.set_bounds(
                            [rect.width(), rect.height()],
                            match unit.alignment {
                                TextBoxAlignment::Left => Align::Left,
                                TextBoxAlignment::Center => Align::Center,
                                TextBoxAlignment::Right => Align::Right,
                            },
                        );
                        // NOTE:
                        // this is a solution for a bug that when passing position to DrawParam,
                        // next item after text is positioned relative to this text offset.
                        graphics::queue_text(self.context, &text, [rect.left, rect.top], None);
                        if graphics::draw_queued_text(
                            self.context,
                            graphics::DrawParam::default(),
                            None,
                            graphics::FilterMode::Linear,
                        )
                        .is_ok()
                        {
                            Ok(())
                        } else {
                            Err(Error::CouldNotDrawImage(unit.id.to_owned()))
                        }
                    } else {
                        Err(Error::ImageResourceNotFound(
                            unit.id.to_owned(),
                            unit.font.name.to_owned(),
                        ))
                    }
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
        }
    }
}

impl<'a> Renderer<(), Error> for GgezRenderer<'a> {
    fn render(&mut self, tree: &WidgetUnit, layout: &Layout) -> Result<(), Error> {
        self.render_node(tree, layout)
    }
}
