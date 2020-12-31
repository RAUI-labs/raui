use raui_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeColor {
    Default,
    Primary,
    Secondary,
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    ContentOnly,
    Filled,
    Outline,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Filled
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedWidgetProps {
    #[serde(default)]
    pub color: ThemeColor,
    #[serde(default)]
    pub variant: ThemeVariant,
}
implement_props_data!(ThemedWidgetProps, "ThemedWidgetProps");

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorSet {
    #[serde(default)]
    pub main: Color,
    #[serde(default)]
    pub light: Color,
    #[serde(default)]
    pub dark: Color,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(default)]
    pub default: ThemeColorSet,
    #[serde(default)]
    pub primary: ThemeColorSet,
    #[serde(default)]
    pub secondary: ThemeColorSet,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorsBundle {
    #[serde(default)]
    pub main: ThemeColors,
    #[serde(default)]
    pub contrast: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemedImageMaterial {
    Color,
    Image(ImageBoxImage),
    Procedural(ImageBoxProcedural),
}

impl Default for ThemedImageMaterial {
    fn default() -> Self {
        Self::Color
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedTextMaterial {
    #[serde(default)]
    pub alignment: TextBoxAlignment,
    #[serde(default)]
    pub direction: TextBoxDirection,
    #[serde(default)]
    pub font: TextBoxFont,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedButtonMaterial {
    #[serde(default)]
    pub default: ThemedImageMaterial,
    #[serde(default)]
    pub selected: ThemedImageMaterial,
    #[serde(default)]
    pub trigger: ThemedImageMaterial,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedSwitchMaterial {
    #[serde(default)]
    pub on: ThemedImageMaterial,
    #[serde(default)]
    pub off: ThemedImageMaterial,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeProps {
    #[serde(default)]
    pub active_colors: ThemeColorsBundle,
    #[serde(default)]
    pub background_colors: ThemeColorsBundle,
    #[serde(default)]
    pub content_backgrounds: HashMap<String, ThemedImageMaterial>,
    #[serde(default)]
    pub button_backgrounds: HashMap<String, ThemedButtonMaterial>,
    #[serde(default)]
    pub icons_level_sizes: Vec<Scalar>,
    #[serde(default)]
    pub text_variants: HashMap<String, ThemedTextMaterial>,
    #[serde(default)]
    pub switch_variants: HashMap<String, ThemedSwitchMaterial>,
}
implement_props_data!(ThemeProps, "ThemeProps");

pub fn new_light_theme() -> ThemeProps {
    make_default_theme(
        color_from_rgba(241, 250, 238, 1.0),
        color_from_rgba(29, 53, 87, 1.0),
        color_from_rgba(230, 57, 70, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
    )
}

pub fn new_dark_theme() -> ThemeProps {
    make_default_theme(
        color_from_rgba(64, 64, 64, 1.0),
        color_from_rgba(255, 98, 86, 1.0),
        color_from_rgba(0, 196, 228, 1.0),
        color_from_rgba(4, 4, 4, 1.0),
    )
}

pub fn new_all_white_theme() -> ThemeProps {
    make_default_theme(
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
    )
}

pub fn make_default_theme(
    default: Color,
    primary: Color,
    secondary: Color,
    background: Color,
) -> ThemeProps {
    let background_primary = color_lerp(background, primary, 0.05);
    let background_secondary = color_lerp(background, secondary, 0.05);
    let mut content_backgrounds = HashMap::with_capacity(1);
    content_backgrounds.insert(String::new(), Default::default());
    let mut button_backgrounds = HashMap::with_capacity(1);
    button_backgrounds.insert(String::new(), Default::default());
    let mut text_variants = HashMap::with_capacity(1);
    text_variants.insert(
        String::new(),
        ThemedTextMaterial {
            font: TextBoxFont {
                size: 18.0,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let mut switch_variants = HashMap::with_capacity(4);
    switch_variants.insert(String::new(), ThemedSwitchMaterial::default());
    switch_variants.insert("checkbox".to_owned(), ThemedSwitchMaterial::default());
    switch_variants.insert("toggle".to_owned(), ThemedSwitchMaterial::default());
    switch_variants.insert("radio".to_owned(), ThemedSwitchMaterial::default());
    ThemeProps {
        active_colors: make_colors_bundle(
            make_color_set(default, 0.1, 0.2),
            make_color_set(primary, 0.1, 0.2),
            make_color_set(secondary, 0.1, 0.2),
        ),
        background_colors: make_colors_bundle(
            make_color_set(background, 0.1, 0.2),
            make_color_set(background_primary, 0.1, 0.2),
            make_color_set(background_secondary, 0.1, 0.2),
        ),
        content_backgrounds,
        button_backgrounds,
        icons_level_sizes: vec![18.0, 24.0, 32.0, 48.0, 64.0, 128.0, 256.0, 512.0, 1024.0],
        text_variants,
        switch_variants,
    }
}

pub fn color_from_rgba(r: u8, g: u8, b: u8, a: Scalar) -> Color {
    Color {
        r: r as Scalar / 255.0,
        g: g as Scalar / 255.0,
        b: b as Scalar / 255.0,
        a,
    }
}

pub fn make_colors_bundle(
    default: ThemeColorSet,
    primary: ThemeColorSet,
    secondary: ThemeColorSet,
) -> ThemeColorsBundle {
    let contrast = ThemeColors {
        default: ThemeColorSet {
            main: contrast_color(default.main),
            light: contrast_color(default.light),
            dark: contrast_color(default.dark),
        },
        primary: ThemeColorSet {
            main: contrast_color(primary.main),
            light: contrast_color(primary.light),
            dark: contrast_color(primary.dark),
        },
        secondary: ThemeColorSet {
            main: contrast_color(secondary.main),
            light: contrast_color(secondary.light),
            dark: contrast_color(secondary.dark),
        },
    };
    let main = ThemeColors {
        default,
        primary,
        secondary,
    };
    ThemeColorsBundle { main, contrast }
}

pub fn contrast_color(base_color: Color) -> Color {
    Color {
        r: 1.0 - base_color.r,
        g: 1.0 - base_color.g,
        b: 1.0 - base_color.b,
        a: base_color.a,
    }
}

pub fn make_color_set(base_color: Color, lighter: Scalar, darker: Scalar) -> ThemeColorSet {
    let main = base_color;
    let light = Color {
        r: lerp_clamped(main.r, 1.0, lighter),
        g: lerp_clamped(main.g, 1.0, lighter),
        b: lerp_clamped(main.b, 1.0, lighter),
        a: main.a,
    };
    let dark = Color {
        r: lerp_clamped(main.r, 0.0, darker),
        g: lerp_clamped(main.g, 0.0, darker),
        b: lerp_clamped(main.b, 0.0, darker),
        a: main.a,
    };
    ThemeColorSet { main, light, dark }
}

pub fn color_lerp(from: Color, to: Color, factor: Scalar) -> Color {
    Color {
        r: lerp_clamped(from.r, to.r, factor),
        g: lerp_clamped(from.g, to.g, factor),
        b: lerp_clamped(from.b, to.b, factor),
        a: lerp_clamped(from.a, to.a, factor),
    }
}
