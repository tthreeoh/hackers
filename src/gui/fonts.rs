use crate::gui::UiBackend;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::sizes::INPUT_WIDTH;

// Keep the original TextSize enum for backward compatibility
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum TextSize {
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge,
}

impl Default for TextSize {
    fn default() -> Self {
        Self::Small
    }
}

impl TextSize {
    pub const ALL: [Self; 5] = [
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::XLarge,
        Self::XXLarge,
    ];

    pub fn get_font_size(self) -> f32 {
        match self {
            Self::Small => 14.0,
            Self::Medium => 18.0,
            Self::Large => 24.0,
            Self::XLarge => 32.0,
            Self::XXLarge => 40.0,
        }
    }

    pub fn as_str(self) -> Cow<'static, str> {
        Cow::Owned(format!("{}", self.get_font_size() as i32))
    }

    pub fn index(self) -> usize {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
            Self::XLarge => 3,
            Self::XXLarge => 4,
        }
    }

    pub fn from_index(i: usize) -> Option<Self> {
        Self::ALL.get(i).copied()
    }

    pub fn get_font_scale(self) -> f32 {
        self.get_font_size() / TextSize::Small.get_font_size()
    }

    pub fn input_width(self) -> f32 {
        INPUT_WIDTH * self.get_font_scale()
    }

    // New method to convert to FontSpec using a specific family
    pub fn to_font_spec(self, family_index: usize) -> FontSpec {
        FontSpec {
            family_index,
            size: self.get_font_size(),
        }
    }

    // Convenience method that uses the default family (index 0)
    pub fn to_default_font_spec(self) -> FontSpec {
        self.to_font_spec(0)
    }

    // Legacy method for backward compatibility with old font system
    // This assumes fonts are still loaded at indices 1-5 (after default font at 0)
    pub fn push_font(self, ui: &dyn UiBackend) -> Option<crate::gui::FontToken<'_>> {
        let font_index = self.index() + 1; // +1 because index 0 is default font
        ui.push_font_by_index(font_index)
    }
}

// Legacy IconSize enum - keep if you still use it
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconSize {
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge,
}

impl IconSize {
    pub const ALL: [Self; 5] = [
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::XLarge,
        Self::XXLarge,
    ];

    pub fn get_font_size(self) -> f32 {
        match self {
            Self::Small => 14.0,
            Self::Medium => 18.0,
            Self::Large => 24.0,
            Self::XLarge => 32.0,
            Self::XXLarge => 40.0,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
            Self::XLarge => 3,
            Self::XXLarge => 4,
        }
    }

    pub fn to_font_spec(self, family_index: usize) -> FontSpec {
        FontSpec {
            family_index,
            size: self.get_font_size(),
        }
    }
}

/// Font specification using actual sizes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FontSpec {
    pub family_index: usize,
    pub size: f32,
}

impl FontSpec {
    pub fn new(family_index: usize, size: f32) -> Self {
        Self { family_index, size }
    }

    pub fn push_font<'a>(
        self,
        ui: &'a dyn UiBackend,
        registry: &FontRegistry,
    ) -> Option<crate::gui::FontToken<'a>> {
        let font_index = registry.get_font_index(self.family_index, self.size)?;
        ui.push_font_by_index(font_index)
    }

    pub fn get_scale(self, base_size: f32) -> f32 {
        self.size / base_size
    }

    pub fn input_width(self, base_size: f32) -> f32 {
        INPUT_WIDTH * self.get_scale(base_size)
    }

    pub fn with_size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn with_family(self, family_index: usize) -> Self {
        Self {
            family_index,
            ..self
        }
    }
}

impl Default for FontSpec {
    fn default() -> Self {
        Self {
            family_index: 0,
            size: 14.0,
        }
    }
}

// Automatic conversion from TextSize to FontSpec (uses default family)
impl From<TextSize> for FontSpec {
    fn from(text_size: TextSize) -> Self {
        text_size.to_default_font_spec()
    }
}

// Automatic conversion from IconSize to FontSpec
impl From<IconSize> for FontSpec {
    fn from(icon_size: IconSize) -> Self {
        icon_size.to_font_spec(1) // Assuming icons are family index 1
    }
}

/// Configuration for a font family
#[derive(Debug, Clone)]
pub struct FontFamilyConfig {
    pub name: String,
    pub data: Vec<u8>,
    pub sizes: Vec<f32>,
    pub glyph_ranges: Option<Vec<u32>>,
}

impl FontFamilyConfig {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            sizes: Vec::new(),
            glyph_ranges: None,
        }
    }

    pub fn with_sizes(mut self, sizes: Vec<f32>) -> Self {
        self.sizes = sizes;
        self
    }

    pub fn with_glyph_range(mut self, min: u32, max: u32) -> Self {
        self.glyph_ranges = Some(vec![min, max, 0]);
        self
    }

    pub fn with_glyph_ranges(mut self, ranges: Vec<u32>) -> Self {
        self.glyph_ranges = Some(ranges);
        self
    }
}

/// Builder for font configuration
#[derive(Debug, Clone)]
pub struct FontConfigBuilder {
    base_size: f32,
    families: Vec<FontFamilyConfig>,
    default_family_index: usize,
    glyph_range_storage: Vec<Vec<u32>>,
}

impl FontConfigBuilder {
    pub fn new(base_size: f32) -> Self {
        Self {
            base_size,
            families: Vec::new(),
            default_family_index: 0,
            glyph_range_storage: Vec::new(),
        }
    }

    pub fn add_family(mut self, config: FontFamilyConfig) -> Self {
        self.families.push(config);
        self
    }

    pub fn set_default_family(mut self, index: usize) -> Self {
        self.default_family_index = index;
        self
    }

    pub fn build(mut self, ctx: &mut Context) -> FontRegistry {
        let mut registry = FontRegistry {
            families: Vec::new(),
            base_size: self.base_size,
        };

        let font_atlas = ctx.fonts();

        // Add default font using the first family
        if let Some(default_family) = self.families.get(self.default_family_index) {
            font_atlas.add_font(&[FontSource::TtfData {
                data: &default_family.data,
                size_pixels: self.base_size,
                config: None,
            }]);
        }

        // Pre-store all glyph ranges and leak them to get 'static lifetime
        let mut leaked_ranges: Vec<&'static [u32]> = Vec::new();
        for family in &self.families {
            if let Some(ranges) = &family.glyph_ranges {
                let boxed = ranges.clone().into_boxed_slice();
                let leaked: &'static [u32] = Box::leak(boxed);
                leaked_ranges.push(leaked);
            }
        }

        // Track the starting index for each family
        let mut current_index = 1;
        let mut glyph_range_idx = 0;

        for family_config in self.families {
            let start_index = current_index;
            let mut size_map = Vec::new();

            let has_glyph_ranges = family_config.glyph_ranges.is_some();
            let current_glyph_ranges = if has_glyph_ranges {
                let ranges = leaked_ranges[glyph_range_idx];
                glyph_range_idx += 1;
                Some(ranges)
            } else {
                None
            };

            for &size in &family_config.sizes {
                let config = current_glyph_ranges.map(|ranges| FontConfig {
                    glyph_ranges: FontGlyphRanges::from_slice(ranges),
                    ..Default::default()
                });

                font_atlas.add_font(&[FontSource::TtfData {
                    data: &family_config.data,
                    size_pixels: size,
                    config,
                }]);

                size_map.push((size, current_index));
                current_index += 1;
            }

            registry.families.push(RegisteredFontFamily {
                name: family_config.name,
                base_index: start_index,
                size_map,
            });
        }

        registry
    }
}

/// Runtime registry of loaded fonts
#[derive(Debug, Clone)]
pub struct FontRegistry {
    families: Vec<RegisteredFontFamily>,
    base_size: f32,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self {
            families: Vec::new(),
            base_size: 14.0,
        }
    }
}

#[derive(Debug, Clone)]
struct RegisteredFontFamily {
    name: String,
    base_index: usize,
    size_map: Vec<(f32, usize)>,
}

impl FontRegistry {
    pub fn get_family_by_name(&self, name: &str) -> Option<usize> {
        self.families.iter().position(|f| f.name == name)
    }

    pub fn get_font_index(&self, family_index: usize, size: f32) -> Option<usize> {
        self.families.get(family_index).and_then(|family| {
            if let Some((_, idx)) = family.size_map.iter().find(|(s, _)| *s == size) {
                return Some(*idx);
            }

            family
                .size_map
                .iter()
                .min_by(|(a, _), (b, _)| {
                    let diff_a = (size - a).abs();
                    let diff_b = (size - b).abs();
                    diff_a.partial_cmp(&diff_b).unwrap()
                })
                .map(|(_, idx)| *idx)
        })
    }

    pub fn get_available_sizes(&self, family_index: usize) -> Option<Vec<f32>> {
        self.families
            .get(family_index)
            .map(|f| f.size_map.iter().map(|(size, _)| *size).collect())
    }

    pub fn family_count(&self) -> usize {
        self.families.len()
    }

    pub fn get_family_name(&self, index: usize) -> Option<&str> {
        self.families.get(index).map(|f| f.name.as_str())
    }

    pub fn get_all_families(&self) -> Vec<&str> {
        self.families.iter().map(|f| f.name.as_str()).collect()
    }
}

/// Helper for rendering text with a specific font
pub fn render_with_font(
    ui: &dyn UiBackend,
    registry: &FontRegistry,
    font_spec: FontSpec,
    f: impl FnOnce(),
) {
    let _font = font_spec.push_font(ui, registry);
    f();
}

// Utility functions
pub fn render_color_input(
    ui: &dyn UiBackend,
    ui_label: &str,
    show: &mut bool,
    color: &mut [f32; 3],
    color_label: &str,
) {
    if ui_label.is_empty() {
        return;
    }
    let scale = ui.current_font_size() / 14.0;
    ui.checkbox(ui_label, show);
    ui.same_line();
    ui.color_edit3_config(color_label, color)
        .inputs(false)
        .label(false)
        .tooltip(false)
        .build();
}

pub fn render_clamped_input_int(
    ui: &dyn UiBackend,
    ui_label: &str,
    value: &mut i32,
    min: i32,
    max: i32,
) {
    if ui_label.is_empty() {
        return;
    }
    let scale = ui.current_font_size() / 14.0;
    let last_value = *value;
    ui.set_next_item_width(INPUT_WIDTH * scale);
    ui.input_int(ui_label, value);
    if *value > max || *value < min {
        *value = last_value;
    }
}

pub fn render_clamped_input_f32(
    ui: &dyn UiBackend,
    ui_label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    step: f32,
) {
    if ui_label.is_empty() {
        return;
    }

    let scale = ui.current_font_size() / 14.0;
    let last_value = *value;
    ui.set_next_item_width(INPUT_WIDTH * scale);
    ui.input_float_with_step(ui_label, value, step, step);
    if *value > max || *value < min {
        *value = last_value;
    }
}

pub fn render_font_size_selector(
    ui: &dyn UiBackend,
    registry: &FontRegistry,
    current_spec: &mut FontSpec,
    label_id: &str,
) {
    let scale = ui.current_font_size() / 14.0;
    let input_width = INPUT_WIDTH * scale;

    if let Some(available_sizes) = registry.get_available_sizes(current_spec.family_index) {
        ui.set_next_item_width(input_width);

        let preview = format!("{:.0}", current_spec.size);
        if let Some(_token) = ui.begin_combo(label_id, &preview) {
            for &size in &available_sizes {
                let size_label = format!("{:.0}", size);
                if ui
                    .selectable_config(&size_label)
                    .selected((current_spec.size - size).abs() < 0.01)
                    .build()
                {
                    current_spec.size = size;
                }
            }
        }
        ui.same_line();
        ui.text("Font Size");
    }
}

pub fn render_font_family_selector(
    ui: &dyn UiBackend,
    registry: &FontRegistry,
    current_spec: &mut FontSpec,
    label_id: &str,
) {
    let scale = ui.current_font_size() / 14.0;
    let input_width = INPUT_WIDTH * scale;

    let families = registry.get_all_families();
    let preview = registry
        .get_family_name(current_spec.family_index)
        .unwrap_or("Unknown");

    ui.set_next_item_width(input_width);
    if let Some(_token) = ui.begin_combo(label_id, preview) {
        for (idx, &family_name) in families.iter().enumerate() {
            if ui
                .selectable_config(family_name)
                .selected(current_spec.family_index == idx)
                .build()
            {
                current_spec.family_index = idx;

                if let Some(sizes) = registry.get_available_sizes(idx) {
                    if let Some(&first_size) = sizes.first() {
                        current_spec.size = first_size;
                    }
                }
            }
        }
    }
    ui.same_line();
    ui.text("Font Family");
}

// Legacy function for backward compatibility - works with TextSize
pub fn render_font_size_input(ui: &dyn UiBackend, current_size: &mut TextSize, label_id: &str) {
    let scale = ui.current_font_size() / 14.0;
    let input_width = INPUT_WIDTH * scale;

    ui.set_next_item_width(input_width);
    if let Some(_token) = ui.begin_combo(label_id, &current_size.as_str()) {
        for size in TextSize::ALL {
            if ui
                .selectable_config(&size.as_str())
                .selected(*current_size == size)
                .build()
            {
                *current_size = size;
            }
        }
    }
    ui.same_line();
    ui.text("Text Size");
}
