use raui_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;

const DEFAULT_BACKGROUND_MIXING_FACTOR: Scalar = 0.1;
const DEFAULT_VARIANT_MIXING_FACTOR: Scalar = 0.2;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeColor {
    #[default]
    Default,
    Primary,
    Secondary,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeColorVariant {
    #[default]
    Main,
    Light,
    Dark,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    ContentOnly,
    #[default]
    Filled,
    Outline,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct ThemedWidgetProps {
    #[serde(default)]
    pub color: ThemeColor,
    #[serde(default)]
    pub color_variant: ThemeColorVariant,
    #[serde(default)]
    pub variant: ThemeVariant,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorSet {
    #[serde(default)]
    pub main: Color,
    #[serde(default)]
    pub light: Color,
    #[serde(default)]
    pub dark: Color,
}

impl ThemeColorSet {
    pub fn get(&self, variant: ThemeColorVariant) -> Color {
        match variant {
            ThemeColorVariant::Main => self.main,
            ThemeColorVariant::Light => self.light,
            ThemeColorVariant::Dark => self.dark,
        }
    }

    pub fn get_themed(&self, themed: &ThemedWidgetProps) -> Color {
        self.get(themed.color_variant)
    }
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

impl ThemeColors {
    pub fn get(&self, color: ThemeColor, variant: ThemeColorVariant) -> Color {
        match color {
            ThemeColor::Default => self.default.get(variant),
            ThemeColor::Primary => self.primary.get(variant),
            ThemeColor::Secondary => self.secondary.get(variant),
        }
    }

    pub fn get_themed(&self, themed: &ThemedWidgetProps) -> Color {
        self.get(themed.color, themed.color_variant)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorsBundle {
    #[serde(default)]
    pub main: ThemeColors,
    #[serde(default)]
    pub contrast: ThemeColors,
}

impl ThemeColorsBundle {
    pub fn get(&self, use_main: bool, color: ThemeColor, variant: ThemeColorVariant) -> Color {
        if use_main {
            self.main.get(color, variant)
        } else {
            self.contrast.get(color, variant)
        }
    }

    pub fn get_themed(&self, use_main: bool, themed: &ThemedWidgetProps) -> Color {
        self.get(use_main, themed.color, themed.color_variant)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum ThemedImageMaterial {
    #[default]
    Color,
    Image(ImageBoxImage),
    Procedural(ImageBoxProcedural),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedTextMaterial {
    #[serde(default)]
    pub horizontal_align: TextBoxHorizontalAlign,
    #[serde(default)]
    pub vertical_align: TextBoxVerticalAlign,
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
pub struct ThemedSliderMaterial {
    #[serde(default)]
    pub background: ThemedImageMaterial,
    #[serde(default)]
    pub filling: ThemedImageMaterial,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct ThemeProps {
    #[serde(default)]
    pub active_colors: ThemeColorsBundle,
    #[serde(default)]
    pub background_colors: ThemeColorsBundle,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub content_backgrounds: HashMap<String, ThemedImageMaterial>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub button_backgrounds: HashMap<String, ThemedButtonMaterial>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub icons_level_sizes: Vec<Scalar>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub text_variants: HashMap<String, ThemedTextMaterial>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub switch_variants: HashMap<String, ThemedSwitchMaterial>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub slider_variants: HashMap<String, ThemedSliderMaterial>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub modal_shadow_variants: HashMap<String, Color>,
}

pub fn new_light_theme() -> ThemeProps {
    new_light_theme_parameterized(
        DEFAULT_BACKGROUND_MIXING_FACTOR,
        DEFAULT_VARIANT_MIXING_FACTOR,
    )
}

pub fn new_light_theme_parameterized(
    background_mixing_factor: Scalar,
    variant_mixing_factor: Scalar,
) -> ThemeProps {
    new_default_theme_parameterized(
        color_from_rgba(241, 250, 238, 1.0),
        color_from_rgba(29, 53, 87, 1.0),
        color_from_rgba(230, 57, 70, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
        background_mixing_factor,
        variant_mixing_factor,
    )
}

pub fn new_dark_theme() -> ThemeProps {
    new_dark_theme_parameterized(
        DEFAULT_BACKGROUND_MIXING_FACTOR,
        DEFAULT_VARIANT_MIXING_FACTOR,
    )
}

pub fn new_dark_theme_parameterized(
    background_mixing_factor: Scalar,
    variant_mixing_factor: Scalar,
) -> ThemeProps {
    new_default_theme_parameterized(
        color_from_rgba(64, 64, 64, 1.0),
        color_from_rgba(255, 98, 86, 1.0),
        color_from_rgba(0, 196, 228, 1.0),
        color_from_rgba(32, 32, 32, 1.0),
        background_mixing_factor,
        variant_mixing_factor,
    )
}

pub fn new_all_white_theme() -> ThemeProps {
    new_default_theme(
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
        color_from_rgba(255, 255, 255, 1.0),
    )
}

pub fn new_default_theme(
    default: Color,
    primary: Color,
    secondary: Color,
    background: Color,
) -> ThemeProps {
    new_default_theme_parameterized(
        default,
        primary,
        secondary,
        background,
        DEFAULT_BACKGROUND_MIXING_FACTOR,
        DEFAULT_VARIANT_MIXING_FACTOR,
    )
}

pub fn new_default_theme_parameterized(
    default: Color,
    primary: Color,
    secondary: Color,
    background: Color,
    background_mixing_factor: Scalar,
    variant_mixing_factor: Scalar,
) -> ThemeProps {
    let background_primary = color_lerp(background, primary, background_mixing_factor);
    let background_secondary = color_lerp(background, secondary, background_mixing_factor);
    let mut background_modal = fluid_polarize_color(background);
    background_modal.a = 0.75;
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
    let mut slider_variants = HashMap::with_capacity(1);
    let mut modal_shadow_variants = HashMap::with_capacity(1);
    slider_variants.insert(String::default(), ThemedSliderMaterial::default());
    modal_shadow_variants.insert(String::new(), background_modal);
    ThemeProps {
        active_colors: make_colors_bundle(
            make_color_set(default, variant_mixing_factor, variant_mixing_factor),
            make_color_set(primary, variant_mixing_factor, variant_mixing_factor),
            make_color_set(secondary, variant_mixing_factor, variant_mixing_factor),
        ),
        background_colors: make_colors_bundle(
            make_color_set(background, variant_mixing_factor, variant_mixing_factor),
            make_color_set(
                background_primary,
                variant_mixing_factor,
                variant_mixing_factor,
            ),
            make_color_set(
                background_secondary,
                variant_mixing_factor,
                variant_mixing_factor,
            ),
        ),
        content_backgrounds,
        button_backgrounds,
        icons_level_sizes: vec![18.0, 24.0, 32.0, 48.0, 64.0, 128.0, 256.0, 512.0, 1024.0],
        text_variants,
        switch_variants,
        slider_variants,
        modal_shadow_variants,
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

pub fn fluid_polarize(v: Scalar) -> Scalar {
    (v - 0.5 * PI).sin() * 0.5 + 0.5
}

pub fn fluid_polarize_color(color: Color) -> Color {
    Color {
        r: fluid_polarize(color.r),
        g: fluid_polarize(color.g),
        b: fluid_polarize(color.b),
        a: color.a,
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
