use imgui::Ui;
use std::fmt::Display;

/// Trait for types that can be iterated as enums
pub trait IntoEnumIterator: Sized {
    type Iterator: Iterator<Item = Self>;
    fn iter() -> Self::Iterator;
}

/// Generic enum dropdown with automatic width calculation
pub fn enum_dropdown<T>(ui: &Ui, label: &str, current: &mut T) -> bool
where
    T: IntoEnumIterator + Display + PartialEq + Copy + 'static,
{
    let mut changed = false;
    let padding = 10.0;
    
    // Calculate width based on longest variant
    let widest_text = T::iter()
        .map(|variant| {
            format!(
                "{} ({})",
                variant,
                T::iter().position(|x| x == variant).unwrap_or(0)
            )
        })
        .max_by_key(|text| ui.calc_text_size(text)[0] as i32)
        .unwrap_or_default();
    
    let input_width = ui.calc_text_size(&widest_text)[0] + padding;
    let current_idx = T::iter().position(|x| x == *current).unwrap_or(0);
    let display_text = format!("{} ({})", current, current_idx);
    
    ui.set_next_item_width(input_width);
    if let Some(_combo) = ui.begin_combo(label, &display_text) {
        for (idx, variant) in T::iter().enumerate() {
            let selected = variant == *current;
            let variant_text = format!("{} ({})", variant, idx);
            if ui
                .selectable_config(&variant_text)
                .selected(selected)
                .build()
            {
                *current = variant;
                changed = true;
            }
        }
    }
    
    changed
}

/// Enum dropdown with custom item filtering
pub fn enum_dropdown_with_iterator<T>(
    ui: &Ui, 
    label: &str, 
    current: &mut T, 
    items: &[T]
) -> bool
where
    T: IntoEnumIterator + Display + PartialEq + Copy + 'static,
{
    if items.is_empty() {
        return false;
    }
    
    // Ensure current is valid
    if !items.contains(current) {
        *current = items[0];
        return true;
    }
    
    let get_enum_idx = |item: &T| -> usize { 
        T::iter().position(|variant| &variant == item).unwrap_or(0) 
    };
    let format_item = |variant: &T| format!("{} ({})", variant, get_enum_idx(variant));
    
    let mut changed = false;
    let padding = 10.0;
    let widest_text = items
        .iter()
        .map(|v| format_item(v))
        .max_by_key(|text| ui.calc_text_size(text)[0] as i32)
        .unwrap_or_default();
    
    let input_width = ui.calc_text_size(&widest_text)[0] + padding;
    let display_text = format_item(current);
    
    ui.set_next_item_width(input_width);
    if let Some(_combo) = ui.begin_combo(label, &display_text) {
        for variant in items {
            let selected = variant == current;
            let variant_text = format_item(variant);
            if ui
                .selectable_config(&variant_text)
                .selected(selected)
                .build()
            {
                *current = *variant;
                changed = true;
            }
        }
    }
    
    changed
}