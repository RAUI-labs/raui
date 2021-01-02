use crate::{resources::GgezResources, Error};
use ggez::{
    graphics::{self, Align, MeshBuilder, Scale, Text, TextFragment},
    Context,
};
use raui_core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::{
        unit::{
            image::{ImageBoxImageScaling, ImageBoxMaterial},
            text::TextBoxAlignment,
            WidgetUnit,
        },
        utils::{lerp, Rect, Transform, Vec2},
    },
    Scalar,
};

pub struct GgezRenderer<'a> {
    context: &'a mut Context,
    resources: &'a mut GgezResources,
}

impl<'a> GgezRenderer<'a> {
    pub fn new(context: &'a mut Context, resources: &'a mut GgezResources) -> Self {
        Self { context, resources }
    }

    fn transform_rect(rect: Rect, transform: &Transform) -> (Vec2, Scalar, Vec2, Rect) {
        let offset = Vec2 {
            x: lerp(rect.left, rect.right, transform.pivot.x),
            y: lerp(rect.top, rect.bottom, transform.pivot.y),
        };
        let rect = Rect {
            left: rect.left - offset.x,
            right: rect.right - offset.x,
            top: rect.top - offset.y,
            bottom: rect.bottom - offset.y,
        };
        (offset, transform.rotation, transform.scale, rect)
    }

    fn render_node(
        &mut self,
        unit: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<(), Error> {
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
                    self.render_node(&item.slot, mapping, layout)?;
                }
                Ok(())
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    self.render_node(&item.slot, mapping, layout)?;
                }
                Ok(())
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    self.render_node(&item.slot, mapping, layout)?;
                }
                Ok(())
            }
            WidgetUnit::SizeBox(unit) => self.render_node(&unit.slot, mapping, layout),
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(image) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let scale = mapping.scale();
                        let color = [image.color.r, image.color.g, image.color.b, image.color.a];
                        let rect = mapping.virtual_to_real_rect(item.ui_space);
                        let (offset, rotation, scaling, rect) =
                            Self::transform_rect(rect, &unit.transform);
                        let mut builder = MeshBuilder::new();
                        match image.scaling {
                            ImageBoxImageScaling::Strech => {
                                let vertices = &[
                                    graphics::Vertex {
                                        pos: [rect.left, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                ];
                                let indices = &[0, 1, 2, 2, 3, 0];
                                builder.raw(vertices, indices, None);
                            }
                            ImageBoxImageScaling::Frame(v, only) => {
                                let v = v * scale;
                                let vertices = &[
                                    graphics::Vertex {
                                        pos: [rect.left, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left + v, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right - v, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.top],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left, rect.top + v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left + v, rect.top + v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right - v, rect.top + v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.top + v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left, rect.bottom - v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left + v, rect.bottom - v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right - v, rect.bottom - v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.bottom - v],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.left + v, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right - v, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                    graphics::Vertex {
                                        pos: [rect.right, rect.bottom],
                                        uv: [0.0, 0.0],
                                        color,
                                    },
                                ];
                                if only {
                                    let indices = &[
                                        0, 1, 5, 5, 4, 0, 1, 2, 6, 6, 5, 1, 2, 3, 7, 7, 6, 2, 4, 5,
                                        9, 9, 8, 4, 6, 7, 11, 11, 10, 6, 8, 9, 13, 13, 12, 8, 9,
                                        10, 14, 14, 13, 9, 10, 11, 15, 15, 14, 10,
                                    ];
                                    builder.raw(vertices, indices, None);
                                } else {
                                    let indices = &[
                                        0, 1, 5, 5, 4, 0, 1, 2, 6, 6, 5, 1, 2, 3, 7, 7, 6, 2, 4, 5,
                                        9, 9, 8, 4, 5, 6, 10, 10, 9, 5, 6, 7, 11, 11, 10, 6, 8, 9,
                                        13, 13, 12, 8, 9, 10, 14, 14, 13, 9, 10, 11, 15, 15, 14,
                                        10,
                                    ];
                                    builder.raw(vertices, indices, None);
                                }
                            }
                        }
                        if let Ok(mesh) = builder.build(self.context) {
                            let params = graphics::DrawParam::default()
                                .rotation(rotation)
                                .scale([scaling.x, scaling.y])
                                .dest([offset.x, offset.y]);
                            if graphics::draw(self.context, &mesh, params).is_ok() {
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
                            let scale = mapping.scale();
                            let color = [image.tint.r, image.tint.g, image.tint.b, image.tint.a];
                            let source = image.source_rect.unwrap_or(Rect {
                                left: 0.0,
                                right: 1.0,
                                top: 0.0,
                                bottom: 1.0,
                            });
                            let sfx = source.left;
                            let stx = source.right;
                            let sfy = source.top;
                            let sty = source.bottom;
                            let rect = if let Some(aspect) = unit.content_keep_aspect_ratio {
                                let ox = item.ui_space.left;
                                let oy = item.ui_space.top;
                                let rw = resource.width() as Scalar;
                                let rh = resource.height() as Scalar;
                                let iw = item.ui_space.width();
                                let ih = item.ui_space.height();
                                let ra = rw / rh;
                                let ia = iw / ih;
                                let scale = if ra >= ia { iw / rw } else { ih / rh };
                                let w = rw * scale;
                                let h = rh * scale;
                                let ow = lerp(0.0, iw - w, aspect.horizontal_alignment);
                                let oh = lerp(0.0, ih - h, aspect.vertical_alignment);
                                Rect {
                                    left: ox + ow,
                                    right: ox + ow + w,
                                    top: oy + oh,
                                    bottom: oy + oh + h,
                                }
                            } else {
                                item.ui_space
                            };
                            let rect = mapping.virtual_to_real_rect(rect);
                            let (offset, rotation, scaling, rect) =
                                Self::transform_rect(rect, &unit.transform);
                            let mut builder = MeshBuilder::new();
                            match image.scaling {
                                ImageBoxImageScaling::Strech => {
                                    let vertices = &[
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                    ];
                                    let indices = &[0, 1, 2, 2, 3, 0];
                                    builder.raw(vertices, indices, Some(resource.clone()));
                                }
                                ImageBoxImageScaling::Frame(v, only) => {
                                    let fx = v / resource.width() as Scalar;
                                    let fy = v / resource.height() as Scalar;
                                    let v = v * scale;
                                    let vertices = &[
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.top],
                                            uv: [lerp(sfx, stx, fx), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.top],
                                            uv: [lerp(sfx, stx, 1.0 - fx), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, 0.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.top + v],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.top + v],
                                            uv: [lerp(sfx, stx, fx), lerp(sfy, sty, fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.top + v],
                                            uv: [lerp(sfx, stx, 1.0 - fx), lerp(sfy, sty, fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.top + v],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom - v],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, 1.0 - fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.bottom - v],
                                            uv: [lerp(sfx, stx, fx), lerp(sfy, sty, 1.0 - fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.bottom - v],
                                            uv: [
                                                lerp(sfx, stx, 1.0 - fx),
                                                lerp(sfy, sty, 1.0 - fy),
                                            ],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom - v],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, 1.0 - fy)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left, rect.bottom],
                                            uv: [lerp(sfx, stx, 0.0), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.left + v, rect.bottom],
                                            uv: [lerp(sfx, stx, fx), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right - v, rect.bottom],
                                            uv: [lerp(sfx, stx, 1.0 - fx), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                        graphics::Vertex {
                                            pos: [rect.right, rect.bottom],
                                            uv: [lerp(sfx, stx, 1.0), lerp(sfy, sty, 1.0)],
                                            color,
                                        },
                                    ];
                                    if only {
                                        let indices = &[
                                            0, 1, 5, 5, 4, 0, 1, 2, 6, 6, 5, 1, 2, 3, 7, 7, 6, 2,
                                            4, 5, 9, 9, 8, 4, 6, 7, 11, 11, 10, 6, 8, 9, 13, 13,
                                            12, 8, 9, 10, 14, 14, 13, 9, 10, 11, 15, 15, 14, 10,
                                        ];
                                        builder.raw(vertices, indices, Some(resource.clone()));
                                    } else {
                                        let indices = &[
                                            0, 1, 5, 5, 4, 0, 1, 2, 6, 6, 5, 1, 2, 3, 7, 7, 6, 2,
                                            4, 5, 9, 9, 8, 4, 5, 6, 10, 10, 9, 5, 6, 7, 11, 11, 10,
                                            6, 8, 9, 13, 13, 12, 8, 9, 10, 14, 14, 13, 9, 10, 11,
                                            15, 15, 14, 10,
                                        ];
                                        builder.raw(vertices, indices, Some(resource.clone()));
                                    }
                                }
                            }
                            if let Ok(mesh) = builder.build(self.context) {
                                let params = graphics::DrawParam::default()
                                    .rotation(rotation)
                                    .scale([scaling.x, scaling.y])
                                    .dest([offset.x, offset.y]);
                                if graphics::draw(self.context, &mesh, params).is_ok() {
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
                        let rect = mapping.virtual_to_real_rect(item.ui_space);
                        let (offset, rotation, scaling, rect) =
                            Self::transform_rect(rect, &unit.transform);
                        let mut text = Text::new(TextFragment::new(unit.text.as_str()).color(
                            graphics::Color::new(
                                unit.color.r,
                                unit.color.g,
                                unit.color.b,
                                unit.color.a,
                            ),
                        ));
                        text.set_font(*resource, Scale::uniform(unit.font.size * mapping.scale()));
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
                        let params = graphics::DrawParam::default()
                            .rotation(rotation)
                            .scale([scaling.x, scaling.y])
                            .dest([offset.x, offset.y]);
                        if graphics::draw_queued_text(
                            self.context,
                            params,
                            None,
                            graphics::FilterMode::Linear,
                        )
                        .is_ok()
                        {
                            // NOTE: yeah, we have to pop transforms after text rendering bc
                            // otherwise they just apply tot he next drawable somehow.
                            graphics::pop_transform(self.context);
                            if graphics::apply_transformations(self.context).is_ok() {
                                Ok(())
                            } else {
                                Err(Error::CouldNotDrawImage(unit.id.to_owned()))
                            }
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
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<(), Error> {
        self.render_node(tree, mapping, &layout)
    }
}
