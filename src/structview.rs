pub use hackers_derive::DeriveFieldInfo;

use imgui::{self, TreeNodeFlags, Ui};
use strum::{EnumIter, IntoEnumIterator, Display};
use std::mem;

#[derive(Debug, Clone)]
pub struct FieldMeta {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub type_name: &'static str,
    pub interpret: Option<String>,
}

pub trait FieldInfo: std::any::Any {
    fn get_field_info(&self) -> Vec<FieldMeta>;
}

// ============================================================================
// Trait Definitions
// ============================================================================

pub trait StructDisplayable: Send {
    fn draw_config_ui(&mut self, ui: &imgui::Ui);
    fn display(&mut self, ui: &imgui::Ui, value: &dyn FieldInfo);
    fn clone_box(&self) -> Box<dyn StructDisplayable>;
}

#[derive(Clone)]
pub struct StructViewerWrapper<T: FieldInfo> {
    pub viewer: StructViewer<T>,
}

impl<T: FieldInfo + 'static> StructViewerWrapper<T> {
    pub fn new() -> Self {
        Self {
            viewer: StructViewer::new(),
        }
    }
}

impl<T: FieldInfo + Send + 'static> StructDisplayable for StructViewerWrapper<T> {
    fn draw_config_ui(&mut self, ui: &imgui::Ui) {
        self.viewer.draw_config_ui(ui);
    }

    fn display(&mut self, ui: &imgui::Ui, value: &dyn FieldInfo) {
        // Downcast from trait object to concrete type
        if let Some(concrete) = (value as &dyn std::any::Any).downcast_ref::<T>() {
            self.viewer.display(ui, concrete);
        } else {
            ui.text_colored([1.0, 0.0, 0.0, 1.0], "Mismatched type for viewer");
        }
    }

    fn clone_box(&self) -> Box<dyn StructDisplayable> {  // Add this
        Box::new(StructViewerWrapper {
            viewer: self.viewer.clone(),
        })
    }
}

// ============================================================================
// Highlight Value Types
// ============================================================================

#[derive(Debug, Clone)]
enum HighlightValue {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    Ascii(Vec<u8>),
}

impl HighlightValue {
    pub fn matches_bytes(&self, bytes: &[u8]) -> bool {
        match self {
            HighlightValue::U8(val) => bytes.len() >= 1 && bytes[0] == *val,
            HighlightValue::U16(val) => bytes.len() >= 2 && u16::from_le_bytes([bytes[0], bytes[1]]) == *val,
            HighlightValue::U32(val) => bytes.len() >= 4 && u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) == *val,
            HighlightValue::I8(val) => bytes.len() >= 1 && (bytes[0] as i8) == *val,
            HighlightValue::I16(val) => bytes.len() >= 2 && i16::from_le_bytes([bytes[0], bytes[1]]) == *val,
            HighlightValue::I32(val) => bytes.len() >= 4 && i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) == *val,
            HighlightValue::Ascii(seq) => {
                // Read up to null terminator (0x00) or 32 bytes max
                let max_len = bytes.len().min(32);
                let null_pos = bytes[..max_len]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(max_len);
    
                let memory_str = &bytes[..null_pos];
                memory_str == seq.as_slice()
            }
        }
    }
    
    fn to_string(&self) -> String {
        match self {
            HighlightValue::U8(v)  => format!("u8: {}", v),
            HighlightValue::U16(v) => format!("u16: {}", v),
            HighlightValue::U32(v) => format!("u32: {}", v),
            HighlightValue::I8(v)  => format!("i8: {}", v),
            HighlightValue::I16(v) => format!("i16: {}", v),
            HighlightValue::I32(v) => format!("i32: {}", v),
            HighlightValue::Ascii(bytes) => {
                let s = bytes.iter()
                    .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                    .collect::<String>();
                format!("ascii: \"{}\"", s)
            }
        }
    }
}

// ============================================================================
// Display Modes
// ============================================================================

#[derive(PartialEq, Debug, Display, EnumIter, Copy, Clone)]
enum DisplayMode {
    Hex,
    Dec,
    Both,
}

#[derive(PartialEq, Copy, Clone)]
enum ByteDisplayMode {
    Hex,
    Dec,
    Ascii,
    HexAscii,
}

#[derive(Debug, Display, EnumIter, Copy, Clone, PartialEq)]
pub enum HighlightRuleType {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    Ascii,
}

// ============================================================================
// Highlight Rules
// ============================================================================

#[derive(Clone)]
struct HighlightRule {
    value: HighlightValue,
    color: [f32; 4],
    description: String,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Clone)]
struct StructViewerConfig {
    struct_name_color: [f32; 4],
    field_name_color:  [f32; 4],
    default_byte_color: [f32; 4],
    highlight_rules:   Vec<HighlightRule>,
    pub bytes_per_row: usize,
    byte_display_mode: ByteDisplayMode,
}

impl Default for StructViewerConfig {
    fn default() -> Self {
        Self {
            struct_name_color: [0.3, 0.7, 1.0, 1.0],
            field_name_color:  [0.8, 0.5, 0.2, 1.0],
            default_byte_color: [0.7, 0.7, 0.7, 1.0],
            highlight_rules:   Vec::new(),
            bytes_per_row:     16,
            byte_display_mode: ByteDisplayMode::Hex,
        }
    }
}

// ============================================================================
// Highlight Rule Manager (now instance-based)
// ============================================================================

#[derive(Clone)]
struct HighlightRuleManager {
    rules: Vec<HighlightRule>,
    // New-rule inputs:
    new_rule_color: [f32; 4],
    new_rule_description: String,
    new_rule_input_string: String,
    pub display_mode: DisplayMode,
    cached_parsed_value: Option<u32>,
    pub new_rule_type: HighlightRuleType,
}

impl Default for HighlightRuleManager {
    fn default() -> Self {
        HighlightRuleManager {
            rules: Vec::new(),
            new_rule_color: [1.0, 0.5, 0.0, 1.0],
            new_rule_description: String::new(),
            new_rule_input_string: String::new(),
            display_mode: DisplayMode::Both,
            cached_parsed_value: None,
            new_rule_type: HighlightRuleType::U8,
        }
    }
}

impl HighlightRuleManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw_ui(&mut self, ui: &Ui) {
        // --- Section: Display Mode Selection ---
        ui.text("Display Mode:");
    
        for mode in DisplayMode::iter() {
            ui.radio_button(
                &format!("{:#?}", mode),
                &mut self.display_mode,
                mode
            );
            ui.same_line();
        }
        ui.new_line();
    
        // --- Section: New Rule Inputs ---
        ui.separator();
        ui.text("Add New Filter Rule:");
    
        // Input for value (hex/dec/char)
        ui.text("Select Type:");
        for typ in HighlightRuleType::iter() {
            if ui.radio_button(
                &format!("{:#?}", typ),
                &mut self.new_rule_type,
                typ
            ) {
                // User picked a type
            }
            ui.same_line();
        }
        ui.new_line();
        ui.input_text("Value", &mut self.new_rule_input_string).build();
        ui.input_text("Description", &mut self.new_rule_description).build();
    
        // Color picker
        ui.color_edit4("Color", &mut self.new_rule_color);
        
        match self.new_rule_type {
            HighlightRuleType::Ascii => {
                // Only check if ASCII
                let is_all_ascii = self.new_rule_input_string
                    .chars()
                    .all(|c| c.is_ascii());
        
                if is_all_ascii {
                    self.cached_parsed_value = None;
                } else {
                    ui.text_colored([1.0, 0.2, 0.2, 1.0], "Invalid ASCII input!");
                }
            }
            _ => {
                // Number parsing (for U8/U16/U32 etc)
                let input = self.new_rule_input_string.trim();
                let sanitized_input = if input.to_lowercase().starts_with("0x") {
                    &input[2..]
                } else {
                    input
                };
        
                let parse_result = u32::from_str_radix(sanitized_input, 16)
                    .or_else(|_| sanitized_input.parse::<u32>());
        
                match parse_result {
                    Ok(parsed_value) => {
                        self.cached_parsed_value = Some(parsed_value);
                    }
                    Err(_) => {
                        self.cached_parsed_value = None;
                        if !input.is_empty() {
                            ui.text_colored([1.0, 0.2, 0.2, 1.0], "Invalid number input!");
                        }
                    }
                }
            }
        }
        
        // ASCII Preview
        if self.new_rule_type == HighlightRuleType::Ascii && !self.new_rule_input_string.is_empty() {
            ui.separator();
            ui.text("ASCII Preview:");
    
            let ascii_bytes: Vec<u8> = self.new_rule_input_string
                .chars()
                .filter(|c| c.is_ascii())
                .take(32)
                .map(|c| c as u8)
                .collect();
    
            // Show ASCII Characters
            let ascii_display = ascii_bytes.iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                })
                .collect::<String>();
    
            ui.text(format!("Chars: \"{}\"", ascii_display));
    
            // Show Hex Bytes
            let hex_display = ascii_bytes.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
    
            ui.text(format!("Hex: [{}]", hex_display));
        }
        
        // --- Button to Add Rule ---
        if ui.button("Add Rule") {
            if self.new_rule_type == HighlightRuleType::Ascii {
                let ascii_bytes: Vec<u8> = self.new_rule_input_string
                    .chars()
                    .filter(|c| c.is_ascii())
                    .take(32)
                    .map(|c| c as u8)
                    .collect();
            
                if !ascii_bytes.is_empty() {
                    self.rules.push(HighlightRule {
                        value: HighlightValue::Ascii(ascii_bytes),
                        color: self.new_rule_color,
                        description: self.new_rule_description.clone(),
                    });
            
                    self.new_rule_input_string.clear();
                    self.new_rule_description.clear();
                }
            } else {
                if let Some(parsed_value) = self.cached_parsed_value {
                    let highlight_value = match self.new_rule_type {
                        HighlightRuleType::U8 => HighlightValue::U8(parsed_value as u8),
                        HighlightRuleType::U16 => HighlightValue::U16(parsed_value as u16),
                        HighlightRuleType::U32 => HighlightValue::U32(parsed_value),
                        HighlightRuleType::I8 => HighlightValue::I8(parsed_value as i8),
                        HighlightRuleType::I16 => HighlightValue::I16(parsed_value as i16),
                        HighlightRuleType::I32 => HighlightValue::I32(parsed_value as i32),
                        _ => unreachable!(),
                    };
            
                    self.rules.push(HighlightRule {
                        value: highlight_value,
                        color: self.new_rule_color,
                        description: self.new_rule_description.clone(),
                    });
            
                    self.new_rule_input_string.clear();
                    self.new_rule_description.clear();
                }
            }
        }
    
        // --- Section: List Existing Rules ---
        ui.separator();
        ui.text("Current Filter Rules:");
    
        let mut remove_index: Option<usize> = None;
    
        for (i, rule) in self.rules.iter().enumerate() {
            ui.group(|| {
                ui.text(format!(
                    "{}: {} ({})",
                    i,
                    rule.value.to_string(),
                    rule.description
                ));

                ui.same_line();
                if ui.button(&format!("Remove##{}", i)) {
                    remove_index = Some(i);
                }
            });
        }
    
        if let Some(idx) = remove_index {
            self.rules.remove(idx);
        }
    }
    
    fn get_rules(&self) -> &Vec<HighlightRule> {
        &self.rules
    }
}

// ============================================================================
// Main StructViewer
// ============================================================================

pub struct StructViewer<T: FieldInfo> {
    config: StructViewerConfig,
    rule_manager: HighlightRuleManager,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FieldInfo> Clone for StructViewer<T> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rule_manager: self.rule_manager.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: FieldInfo> Default for StructViewer<T> {
    fn default() -> Self {
        StructViewer {
            config: StructViewerConfig::default(),
            rule_manager: HighlightRuleManager::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: FieldInfo> StructViewer<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn byte_display_mode(&mut self, ui: &Ui) {
        ui.text("Byte Display Mode:");
        if ui.radio_button("Hex", &mut self.config.byte_display_mode, ByteDisplayMode::Hex) {}
        ui.same_line();
        if ui.radio_button("Dec", &mut self.config.byte_display_mode, ByteDisplayMode::Dec) {}
        ui.same_line();
        if ui.radio_button("Type", &mut self.config.byte_display_mode, ByteDisplayMode::Ascii) {}
        ui.same_line();
        if ui.radio_button("Hex+ASCII", &mut self.config.byte_display_mode, ByteDisplayMode::HexAscii) {}
        
        ui.text("Bytes Per Row:");
        ui.same_line();
        ui.set_next_item_width(100.0);
        let mut bytes_per_row = self.config.bytes_per_row as i32;
        if ui.input_int("##bytes_per_row", &mut bytes_per_row).build() {
            // Clamp to 1-32 range
            if bytes_per_row < 1 { bytes_per_row = 1; }
            if bytes_per_row > 32 { bytes_per_row = 32; }
            self.config.bytes_per_row = bytes_per_row as usize;
        }
    }

    pub fn draw_config_ui(&mut self, ui: &Ui) {
        self.byte_display_mode(ui);
        
        ui.separator();
        
        if ui.collapsing_header("Highlight Rules", TreeNodeFlags::DEFAULT_OPEN) {
            self.rule_manager.draw_ui(ui);
            self.config.highlight_rules = self.rule_manager.get_rules().clone();
        }
    }

    pub fn display(&mut self, ui: &Ui, value: &T) {
        let type_name = std::any::type_name::<T>()
            .rsplit("::").next().unwrap_or("Unknown");
        ui.text_colored(self.config.struct_name_color, type_name);
        
        let ptr = value as *const T as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, mem::size_of::<T>()) };
        let fields = value.get_field_info();
        
        self.display_all_fields(ui, bytes, &fields);
    }
    
    pub fn display_all_fields(&self, ui: &imgui::Ui, bytes: &[u8], fields: &[FieldMeta]) {
        let bytes_per_row = self.config.bytes_per_row.max(1);
    
        // First pass: determine max hex width across all fields
        let mut max_hex_width = 0;
    
        for field in fields {
            let FieldMeta { offset, size, .. } = field;
    
            if *offset >= bytes.len() { continue; }
    
            let avail = bytes.len() - offset;
            let slice = &bytes[*offset..(*offset + size.min(&avail))];
    
            let mut i = 0;
            while i < slice.len() {
                let row_end = (i + bytes_per_row).min(slice.len());
                let row = &slice[i..row_end];
                let hex = row.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                max_hex_width = max_hex_width.max(hex.len());
                i = row_end;
            }
        }
    
        // Second pass: render with uniform hex width
        for field in fields {
            let FieldMeta { name, offset, size, type_name, interpret } = field;
    
            ui.tree_node_config(name)
                .flags(TreeNodeFlags::DEFAULT_OPEN)
                .build(|| {
                    ui.text_colored(self.config.field_name_color, format!(
                        "Offset: {:#06X}, Size: {} bytes, Type: {}",
                        offset, size, type_name,
                    ));
                    ui.spacing();
    
                    if *offset >= bytes.len() {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], "Offset out of range");
                        return;
                    }
    
                    let avail = bytes.len() - offset;
                    let slice = &bytes[*offset..(*offset + size.min(&avail))];
    
                    self.draw_memory_table(
                        ui,
                        slice,
                        bytes_per_row,
                        Some(type_name),
                        interpret.clone(),
                        &self.config.highlight_rules,
                        max_hex_width,
                    );
                });
        }
    }
    
    fn draw_memory_table(
        &self,
        ui: &imgui::Ui,
        memory: &[u8],
        bytes_per_row: usize,
        type_name: Option<&str>,
        interpret: Option<String>,
        highlight_rules: &[HighlightRule],
        max_hex_width: usize,
    ) {
        let mut rows = Vec::new();
        let mut max_val_width = 0;
        let mut i = 0;
    
        while i < memory.len() {
            let row_start = i;
            let row_end = (i + bytes_per_row).min(memory.len());
            let slice = &memory[row_start..row_end];
    
            let hex = slice
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
    
            let val = match self.config.byte_display_mode {
                ByteDisplayMode::Ascii => {
                    if let Some(ref interp) = interpret {
                        interp.clone()
                    } else {
                        slice
                            .iter()
                            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                            .collect::<String>()
                    }
                }
                _ => {
                    slice
                        .iter()
                        .map(|&b| format_byte(b, self.config.byte_display_mode))
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            };
    
            max_val_width = max_val_width.max(val.len());
    
            rows.push((i, slice.to_vec(), hex, val));
            i = row_end;
        }
    
        if let Some(_t) = ui.begin_table_with_flags(
            "MemoryView",
            3,
            imgui::TableFlags::BORDERS
                | imgui::TableFlags::ROW_BG
                | imgui::TableFlags::SIZING_FIXED_FIT
                | imgui::TableFlags::NO_SAVED_SETTINGS,
        ) {
            ui.table_setup_column("Addr");
            ui.table_setup_column("Hex");
            ui.table_setup_column(format!("{}", type_name.unwrap_or("Value")));
            ui.table_headers_row();
    
            let base_address = memory.as_ptr() as usize;
    
            for (offset, row_bytes, hex, val) in rows {
                ui.table_next_row();
    
                // Column 0: Address
                ui.table_set_column_index(0);
                ui.text(format!("{:08X}:", base_address + offset));
    
                // Determine color for this row
                let mut color = self.config.default_byte_color;
                for rule in highlight_rules {
                    if rule.value.matches_bytes(&row_bytes) {
                        color = rule.color;
                        break;
                    }
                }
    
                // Column 1: Hex
                ui.table_set_column_index(1);
                let style_hex = ui.push_style_color(imgui::StyleColor::Text, color);
                ui.text(format!("{:width$}", hex, width = max_hex_width));
                style_hex.pop();
    
                // Column 2: Value
                ui.table_set_column_index(2);
                let style_val = ui.push_style_color(imgui::StyleColor::Text, color);
                ui.text(val);
                style_val.pop();
            }
        }
    }
}

fn format_byte(byte: u8, mode: ByteDisplayMode) -> String {
    match mode {
        ByteDisplayMode::Hex => format!("{:02X}", byte),
        ByteDisplayMode::Dec => format!("{}", byte),
        ByteDisplayMode::Ascii => {
            if byte.is_ascii_graphic() || byte == b' ' {
                format!("{}", byte as char)
            } else {
                ".".to_string()
            }
        }
        ByteDisplayMode::HexAscii => {
            let ascii = if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                '.'
            };
            format!("{:02X}({})", byte, ascii)
        }
    }
}
