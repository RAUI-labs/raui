use crate::Vertex;
use fontdue::Font;
use image::EncodableLayout;
use raui_core::widget::{
    unit::{image::ImageBoxMaterial, portal::PortalBoxSlot, WidgetUnit},
    utils::{Rect, Vec2},
};
use raui_tesselate_renderer::TesselateResourceProvider;
use serde::{Deserialize, Serialize};
use spitfire_glow::{
    graphics::{Graphics, Texture},
    renderer::GlowTextureFormat,
};
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct AssetAtlasRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize)]
struct AssetAtlas {
    image: PathBuf,
    regions: HashMap<String, AssetAtlasRegion>,
}

pub(crate) struct AssetTexture {
    pub(crate) texture: Texture,
    /// {name: uvs}
    regions: HashMap<String, Rect>,
    frames_left: usize,
}

pub(crate) struct AssetFont {
    hash: usize,
    frames_left: usize,
}

pub(crate) struct AssetsManager {
    pub frames_alive: usize,
    pub(crate) root_path: PathBuf,
    pub(crate) textures: HashMap<String, AssetTexture>,
    pub(crate) font_map: HashMap<String, AssetFont>,
    pub(crate) fonts: Vec<Font>,
}

impl Default for AssetsManager {
    fn default() -> Self {
        Self::new("./", 1024)
    }
}

impl AssetsManager {
    fn new(root_path: impl Into<PathBuf>, frames_alive: usize) -> Self {
        Self {
            frames_alive,
            root_path: root_path.into(),
            textures: Default::default(),
            font_map: Default::default(),
            fonts: Default::default(),
        }
    }

    pub(crate) fn maintain(&mut self) {
        let to_remove = self
            .textures
            .iter_mut()
            .filter_map(|(id, texture)| {
                if texture.frames_left > 0 {
                    texture.frames_left -= 1;
                    None
                } else {
                    Some(id.to_owned())
                }
            })
            .collect::<Vec<_>>();
        for id in to_remove {
            self.textures.remove(&id);
        }

        let to_remove = self
            .font_map
            .iter_mut()
            .filter_map(|(id, font)| {
                if font.frames_left > 0 {
                    font.frames_left -= 1;
                    None
                } else {
                    Some(id.to_owned())
                }
            })
            .collect::<Vec<_>>();
        for id in to_remove {
            let hash = self.font_map.remove(&id).unwrap().hash;
            if let Some(index) = self.fonts.iter().position(|font| font.file_hash() == hash) {
                self.fonts.swap_remove(index);
            }
        }
    }

    pub(crate) fn load(&mut self, node: &WidgetUnit, graphics: &Graphics<Vertex>) {
        match node {
            WidgetUnit::None => {}
            WidgetUnit::AreaBox(node) => {
                self.load(&node.slot, graphics);
            }
            WidgetUnit::PortalBox(node) => match &*node.slot {
                PortalBoxSlot::Slot(node) => {
                    self.load(node, graphics);
                }
                PortalBoxSlot::ContentItem(node) => {
                    self.load(&node.slot, graphics);
                }
                PortalBoxSlot::FlexItem(node) => {
                    self.load(&node.slot, graphics);
                }
                PortalBoxSlot::GridItem(node) => {
                    self.load(&node.slot, graphics);
                }
            },
            WidgetUnit::ContentBox(node) => {
                for item in &node.items {
                    self.load(&item.slot, graphics);
                }
            }
            WidgetUnit::FlexBox(node) => {
                for item in &node.items {
                    self.load(&item.slot, graphics);
                }
            }
            WidgetUnit::GridBox(node) => {
                for item in &node.items {
                    self.load(&item.slot, graphics);
                }
            }
            WidgetUnit::SizeBox(node) => {
                self.load(&node.slot, graphics);
            }
            WidgetUnit::ImageBox(node) => {
                if let ImageBoxMaterial::Image(image) = &node.material {
                    let id = Self::parse_image_id(&image.id).0;
                    if let Some(texture) = self.textures.get_mut(id) {
                        texture.frames_left = self.frames_alive;
                    } else {
                        let mut path = self.root_path.join(id);
                        match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
                            "toml" => {
                                let content = match std::fs::read_to_string(&path) {
                                    Ok(content) => content,
                                    _ => {
                                        eprintln!("Could not load image atlas file: {:?}", path);
                                        return;
                                    }
                                };
                                let atlas = match toml::from_str::<AssetAtlas>(&content) {
                                    Ok(atlas) => atlas,
                                    _ => {
                                        eprintln!("Could not parse image atlas file: {:?}", path);
                                        return;
                                    }
                                };
                                path.pop();
                                let path = path.join(atlas.image);
                                let image = match image::open(&path) {
                                    Ok(image) => image.to_rgba8(),
                                    _ => {
                                        eprintln!("Could not load image file: {:?}", path);
                                        return;
                                    }
                                };
                                let texture = match graphics.texture(
                                    image.width(),
                                    image.height(),
                                    1,
                                    GlowTextureFormat::Rgba,
                                    image.as_bytes(),
                                ) {
                                    Ok(texture) => texture,
                                    _ => return,
                                };
                                let regions = atlas
                                    .regions
                                    .into_iter()
                                    .map(|(name, region)| {
                                        let left = region.x as f32 / image.width() as f32;
                                        let right =
                                            (region.x + region.width) as f32 / image.width() as f32;
                                        let top = region.y as f32 / image.height() as f32;
                                        let bottom = (region.y + region.height) as f32
                                            / image.height() as f32;
                                        (
                                            name,
                                            Rect {
                                                left,
                                                right,
                                                top,
                                                bottom,
                                            },
                                        )
                                    })
                                    .collect();
                                self.textures.insert(
                                    id.to_owned(),
                                    AssetTexture {
                                        texture,
                                        regions,
                                        frames_left: self.frames_alive,
                                    },
                                );
                            }
                            _ => {
                                let image = match image::open(&path) {
                                    Ok(image) => image.to_rgba8(),
                                    _ => {
                                        eprintln!("Could not load image file: {:?}", path);
                                        return;
                                    }
                                };
                                let texture = match graphics.texture(
                                    image.width(),
                                    image.height(),
                                    1,
                                    GlowTextureFormat::Rgba,
                                    image.as_bytes(),
                                ) {
                                    Ok(texture) => texture,
                                    _ => return,
                                };
                                self.textures.insert(
                                    id.to_owned(),
                                    AssetTexture {
                                        texture,
                                        regions: Default::default(),
                                        frames_left: self.frames_alive,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            WidgetUnit::TextBox(node) => {
                if let Some(font) = self.font_map.get_mut(&node.font.name) {
                    font.frames_left = self.frames_alive;
                } else {
                    let path = self.root_path.join(&node.font.name);
                    let content = match std::fs::read(&path) {
                        Ok(content) => content,
                        _ => {
                            eprintln!("Could not load font file: {:?}", path);
                            return;
                        }
                    };
                    let font = match Font::from_bytes(content, Default::default()) {
                        Ok(font) => font,
                        _ => return,
                    };
                    self.font_map.insert(
                        node.font.name.to_owned(),
                        AssetFont {
                            hash: font.file_hash(),
                            frames_left: self.frames_alive,
                        },
                    );
                    self.fonts.push(font);
                }
            }
        }
    }

    pub(crate) fn parse_image_id(id: &str) -> (&str, Option<&str>) {
        match id.find('@') {
            Some(index) => (&id[..index], Some(&id[(index + 1)..])),
            None => (id, None),
        }
    }
}

impl TesselateResourceProvider for AssetsManager {
    fn image_id_and_uv_and_size_by_atlas_id(&self, id: &str) -> Option<(String, Rect, Vec2)> {
        let (id, region) = Self::parse_image_id(id);
        let texture = self.textures.get(id)?;
        let uvs = region
            .and_then(|region| texture.regions.get(region))
            .copied()
            .unwrap_or(Rect {
                left: 0.0,
                right: 1.0,
                top: 0.0,
                bottom: 1.0,
            });
        let size = Vec2 {
            x: texture.texture.width() as _,
            y: texture.texture.height() as _,
        };
        Some((id.to_owned(), uvs, size))
    }

    fn fonts(&self) -> &[Font] {
        &self.fonts
    }

    fn font_index_by_id(&self, id: &str) -> Option<usize> {
        let hash = self.font_map.get(id)?.hash;
        self.fonts.iter().position(|font| font.file_hash() == hash)
    }
}
