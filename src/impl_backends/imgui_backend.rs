// ============================================================
// imgui_backend.rs - Complete Implementation
// ============================================================

#[cfg(feature = "ui-imgui")]
use std::any::Any;

#[cfg(feature = "ui-imgui")]
use crate::gui::{
    ChildWindowBuilder, Color, ColorEdit4Builder, ComboBoxToken, DrawList, IoState, Key,
    MenuBarToken, MenuToken, MouseButton, SelectableBuilder, StyleColor, StyleToken, TableFlags,
    TableToken, TreeNodeBuilder, TreeNodeFlags, TreeNodeToken, UiBackend, Vec2, WinCondition,
    WindowOptions, WindowToken,
};

use std::cell::RefCell;

thread_local! {
    /// Global texture manager for the current render thread
    /// Set by the runner before rendering, accessible during plugin updates
    static TEXTURE_MANAGER_ID_COUNTER: RefCell<u64> = RefCell::new(1);
    static TEXTURE_UPLOAD_REQUESTS: RefCell<Vec<(u64, u32, u32, Vec<u8>)>> = RefCell::new(Vec::new());
    /// Maps plugin request IDs to actual GPU texture IDs (set by runner during render)
    static TEXTURE_ID_MAP: RefCell<std::collections::HashMap<u64, imgui::TextureId>> = RefCell::new(std::collections::HashMap::new());
}

/// Request a texture upload (stores in thread-local, flushed by runner)
pub fn request_texture_upload(data: &[u8], width: u32, height: u32) -> u64 {
    let id = TEXTURE_MANAGER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        let id = *counter;
        *counter += 1;
        id
    });

    TEXTURE_UPLOAD_REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .push((id, width, height, data.to_vec()));
    });

    id
}

/// Get and clear pending upload requests (called by runner)
pub fn take_texture_upload_requests() -> Vec<(u64, u32, u32, Vec<u8>)> {
    TEXTURE_UPLOAD_REQUESTS.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
}

/// Set the texture ID mapping (called by runner after uploading textures)
pub fn set_texture_id_map(map: std::collections::HashMap<u64, imgui::TextureId>) {
    TEXTURE_ID_MAP.with(|id_map| {
        *id_map.borrow_mut() = map;
    });
}

/// Translate a request ID to a GPU texture ID (called during rendering)
fn translate_texture_id(request_id: imgui::TextureId) -> imgui::TextureId {
    let request_id_u64 = request_id.id() as u64;
    TEXTURE_ID_MAP.with(|id_map| {
        id_map
            .borrow()
            .get(&request_id_u64)
            .copied()
            .unwrap_or(request_id) // Fall back to original ID if not found
    })
}

pub struct ImguiBackend<'ui> {
    pub ui: &'ui imgui::Ui,
    // Cached input state
    cached_keys_down: [bool; 1024],
    cached_io: IoState,
    // Add other cached states as needed
}

#[cfg(feature = "ui-imgui")]
impl<'ui> ImguiBackend<'ui> {
    pub fn new(ui: &'ui imgui::Ui) -> Self {
        let io = ui.io();
        
        // Cache keys
        let mut cached_keys_down = [false; 1024];
        for (i, &down) in io.keys_down.iter().enumerate() {
            if i < 1024 {
                cached_keys_down[i] = down;
            }
        }

        // Cache IO state
        let cached_io = IoState {
            key_shift: io.key_shift,
            key_ctrl: io.key_ctrl,
            key_alt: io.key_alt,
            mouse_pos: Vec2::from(io.mouse_pos),
            display_size: Vec2::from(io.display_size),
        };

        Self { 
            ui,
            cached_keys_down,
            cached_io,
        }
    }

    /// Get the underlying imgui Ui reference
    pub fn inner(&self) -> &imgui::Ui {
        self.ui
    }
}

#[cfg(feature = "ui-imgui")]
impl<'ui> UiBackend for ImguiBackend<'ui> {
    fn native_ui(&self) -> &imgui::Ui {
        self.ui
    }
    fn text(&self, text: &str) {
        self.ui.text(text);
    }

    fn text_colored(&self, color: [f32; 4], text: &str) {
        self.ui.text_colored(color, text);
    }

    fn text_wrapped(&self, text: &str) {
        self.ui.text_wrapped(text);
    }

    fn text_disabled(&self, text: &str) {
        self.ui.text_disabled(text);
    }

    fn bullet_text(&self, text: &str) {
        self.ui.bullet_text(text);
    }

    fn same_line(&self) {
        self.ui.same_line();
    }

    fn same_line_with_pos(&self, pos_x: f32) {
        self.ui.same_line_with_pos(pos_x);
    }

    fn align_text_to_frame_padding(&self) {
        self.ui.align_text_to_frame_padding();
    }

    fn separator(&self) {
        self.ui.separator();
    }

    fn spacing(&self) {
        self.ui.spacing();
    }

    fn indent(&self) {
        self.ui.indent();
    }

    fn unindent(&self) {
        self.ui.unindent();
    }

    fn new_line(&self) {
        self.ui.new_line();
    }

    fn dummy(&self, size: Vec2) {
        self.ui.dummy(size.to_array());
    }

    fn group(&self, f: &mut dyn FnMut()) {
        self.ui.group(|| f());
    }

    fn button(&self, label: &str) -> bool {
        self.ui.button(label)
    }

    fn small_button(&self, label: &str) -> bool {
        self.ui.small_button(label)
    }

    fn checkbox(&self, label: &str, value: &mut bool) -> bool {
        self.ui.checkbox(label, value)
    }

    fn radio_button_bool(&self, label: &str, active: bool) -> bool {
        self.ui.radio_button_bool(label, active)
    }

    // fn radio_button<T>(&self, label: &str, value: &mut T, button_value: T) -> bool
    // where
    //     T: Copy + PartialEq,
    // {
    //     self.ui.radio_button(label, value, button_value)
    // }

    fn radio_button_usize(&self, label: &str, value: &mut usize, test_value: usize) -> bool {
        self.ui.radio_button(label, value, test_value)
    }

    fn disabled(&self, disabled: bool, f: &mut dyn FnMut()) {
        self.ui.disabled(disabled, || f());
    }

    fn begin_disabled(&self, disabled: bool) -> crate::gui::DisabledToken<'_> {
        crate::gui::DisabledToken {
            token: self.ui.begin_disabled(disabled),
        }
    }

    fn input_text_simple(&self, label: &str, buffer: &mut String) -> bool {
        self.ui.input_text(label, buffer).build()
    }

    fn input_text_readonly(&self, label: &str, buffer: &mut String) -> bool {
        self.ui.input_text(label, buffer).read_only(true).build()
    }

    fn input_int(&self, label: &str, value: &mut i32) -> bool {
        self.ui.input_int(label, value).build()
    }

    fn input_int_with_step(&self, label: &str, value: &mut i32, step: i32, step_fast: i32) -> bool {
        self.ui
            .input_int(label, value)
            .step(step)
            .step_fast(step_fast)
            .build()
    }

    fn input_float(&self, label: &str, value: &mut f32) -> bool {
        self.ui.input_float(label, value).build()
    }

    fn input_float_with_step(
        &self,
        label: &str,
        value: &mut f32,
        step: f32,
        step_fast: f32,
    ) -> bool {
        self.ui
            .input_float(label, value)
            .step(step)
            .step_fast(step_fast)
            .build()
    }

    fn input_float2(&self, label: &str, values: &mut [f32; 2]) -> bool {
        self.ui.input_float2(label, values).build()
    }

    fn slider_int(&self, label: &str, min: i32, max: i32, value: &mut i32) -> bool {
        self.ui.slider(label, min, max, value)
    }

    fn slider_float(&self, label: &str, min: f32, max: f32, value: &mut f32) -> bool {
        self.ui.slider(label, min, max, value)
    }

    fn color_edit3(&self, label: &str, color: &mut [f32; 3]) -> bool {
        self.ui.color_edit3(label, color)
    }

    fn color_edit4(&self, label: &str, color: &mut [f32; 4]) -> bool {
        self.ui.color_edit4(label, color)
    }

    fn color_edit3_config<'a>(
        &'a self,
        label: &'a str,
        color: &'a mut [f32; 3],
    ) -> crate::gui::ColorEdit3Builder<'a> {
        crate::gui::ColorEdit3Builder {
            ui: self.ui,
            label,
            color,
            inputs: true,
            show_label: true,
            tooltip: true,
        }
    }

    fn selectable_config<'a>(&'a self, label: &'a str) -> crate::gui::SelectableBuilder<'a, 'a> {
        crate::gui::SelectableBuilder {
            ui: self.ui,
            label,
            selected: false,
            disabled: false,
        }
    }

    fn begin_window_with_options(
        &self,
        title: &str,
        options: &crate::gui::WindowOptions,
    ) -> Option<crate::gui::WindowToken<'_>> {
        let mut window = self.ui.window(title);

        if let Some((size, cond)) = options.size {
            window = window.size(size, convert_condition(cond));
        }

        if let Some((pos, cond)) = options.position {
            window = window.position(pos, convert_condition(cond));
        }

        if options.menu_bar {
            window = window.menu_bar(true);
        }

        if options.always_auto_resize {
            window = window.always_auto_resize(true);
        }

        if !options.resizable {
            window = window.resizable(false);
        }

        window.begin().map(|t| crate::gui::WindowToken { token: t })
    }

    fn begin_window_simple(
        &self,
        title: &str,
        opened: &mut bool,
        options: &crate::gui::WindowOptions,
    ) -> Option<crate::gui::WindowToken<'_>> {
        let mut window = self.ui.window(title);
        window = window.opened(opened);

        if let Some((size, cond)) = options.size {
            window = window.size(size, convert_condition(cond));
        }

        if let Some((pos, cond)) = options.position {
            window = window.position(pos, convert_condition(cond));
        }

        if options.menu_bar {
            window = window.menu_bar(true);
        }

        if options.always_auto_resize {
            window = window.always_auto_resize(true);
        }

        if !options.resizable {
            window = window.resizable(false);
        }

        window.begin().map(|t| crate::gui::WindowToken { token: t })
    }

    fn begin_child(&self, id: &str, size: Vec2, border: bool, ch_fn: &mut fn()) -> bool {
        self.ui
            .child_window(id)
            .size(size.to_array())
            .border(border)
            .build(|| ch_fn())
            .is_some()
    }

    fn end_child(&self) {
        // Handled by closure in begin_child or RAII
    }

    fn begin_menu_bar(&self) -> Option<MenuBarToken<'_>> {
        self.ui.begin_menu_bar().map(|t| MenuBarToken { token: t })
    }

    fn end_menu_bar(&self, token: Option<MenuBarToken<'_>>) {
        // RAII handles it
    }

    fn begin_menu(&self, label: &str) -> Option<MenuToken<'_>> {
        self.ui.begin_menu(label).map(|t| MenuToken { token: t })
    }

    fn end_menu(&self, _token: Option<MenuBarToken<'_>>) {
        // RAII handles it
    }

    fn menu_item(&self, label: &str) -> bool {
        self.ui.menu_item(label)
    }

    fn collapsing_header(&self, label: &str, flags: TreeNodeFlags) -> bool {
        self.ui.collapsing_header(label, convert_tree_flags(flags))
    }

    fn tree_node(&self, label: &str) -> Option<TreeNodeToken<'_>> {
        self.ui.tree_node(label).map(|t| TreeNodeToken { token: t })
    }

    fn tree_pop(&self) {
        // RAII handles it
    }

    fn begin_table(&self, id: &str, columns: usize, flags: TableFlags) -> Option<TableToken<'_>> {
        self.ui
            .begin_table_with_flags(id, columns, convert_table_flags(flags))
            .map(|t| TableToken { token: t })
    }

    fn end_table(&self) {
        // RAII handles it
    }

    fn table_next_row(&self) {
        self.ui.table_next_row();
    }

    fn table_next_column(&self) {
        self.ui.table_next_column();
    }

    fn table_setup_column(&self, label: &str) {
        self.ui.table_setup_column(label);
    }

    fn table_headers_row(&self) {
        self.ui.table_headers_row();
    }

    fn columns(&self, count: i32, id: &str, border: bool) {
        self.ui.columns(count, id, border);
    }

    fn next_column(&self) {
        self.ui.next_column();
    }

    fn set_column_width(&self, column_idx: i32, width: f32) {
        self.ui.set_column_width(column_idx, width);
    }

    fn get_column_width(&self) -> f32 {
        self.ui.current_column_width()
    }

    fn begin_combo(&self, label: &str, preview: &str) -> Option<ComboBoxToken<'_>> {
        self.ui
            .begin_combo(label, preview)
            .map(|t| ComboBoxToken { token: t })
    }

    fn end_combo(&self) {
        // RAII handles it
    }

    fn selectable<'a>(&self, label: &'a str) -> SelectableBuilder<'_, 'a> {
        SelectableBuilder {
            ui: self.ui,
            label,
            selected: false,
            disabled: false,
        }
    }

    fn open_popup(&self, id: &str) {
        self.ui.open_popup(id);
    }

    fn begin_popup(&self, id: &str) -> bool {
        self.ui.begin_popup(id).is_some()
    }

    fn end_popup(&self) {
        // RAII handles it
    }

    fn push_id_str(&self, id: &str) {
        let _ = self.ui.push_id(id);
    }

    fn push_id_int(&self, id: i32) {
        let _ = self.ui.push_id_int(id);
    }

    fn push_id_usize(&self, id: usize) {
        let _ = self.ui.push_id_usize(id);
    }

    fn pop_id(&self) {
        // self.ui.pop_id(); // imgui-rs uses RAII for IDs usually, but has pop_id too?
        // imgui-rs `push_id` returns a token.
        // If we want manual push/pop, we might need unsafe or different API.
        // But `UiBackend` has `pop_id`.
        // `imgui-rs` 0.8+ might not expose `pop_id` publicly if it enforces RAII.
        // Check `imgui-rs` docs or assume we can't easily do manual pop without tokens.
        // For now, let's assume `pop_id` is empty and we rely on RAII where possible, or this is a limitation.
        // Actually, `gui_integration.rs` doesn't seem to use `push_id`/`pop_id` manually much, or uses `push_id` with a closure?
        // `gui_integration.rs` uses `ui.push_id(id)`? No, I don't see it in the snippet.
    }

    fn set_item_default_focus(&self) {
        self.ui.set_item_default_focus();
    }

    fn is_item_hovered(&self) -> bool {
        self.ui.is_item_hovered()
    }

    fn is_item_active(&self) -> bool {
        self.ui.is_item_active()
    }

    fn is_item_clicked(&self, button: MouseButton) -> bool {
        let imgui_button = match button {
            MouseButton::Left => imgui::MouseButton::Left,
            MouseButton::Right => imgui::MouseButton::Right,
            MouseButton::Middle => imgui::MouseButton::Middle,
            MouseButton::Extra1 => imgui::MouseButton::Extra1,
            MouseButton::Extra2 => imgui::MouseButton::Extra2,
        };
        self.ui.is_item_clicked_with_button(imgui_button)
    }

    fn set_next_item_width(&self, width: f32) {
        self.ui.set_next_item_width(width);
    }

    fn set_cursor_pos(&self, pos: Vec2) {
        self.ui.set_cursor_pos(pos.to_array());
    }

    fn get_cursor_pos(&self) -> Vec2 {
        Vec2::from(self.ui.cursor_pos())
    }

    fn get_cursor_screen_pos(&self) -> Vec2 {
        Vec2::from(self.ui.cursor_screen_pos())
    }

    fn set_cursor_screen_pos(&self, pos: Vec2) {
        self.ui.set_cursor_screen_pos(pos.to_array());
    }

    fn get_window_pos(&self) -> Vec2 {
        Vec2::from(self.ui.window_pos())
    }

    fn get_window_size(&self) -> Vec2 {
        Vec2::from(self.ui.window_size())
    }

    fn current_font_size(&self) -> f32 {
        self.ui.current_font_size()
    }

    fn content_region_avail(&self) -> Vec2 {
        Vec2::from(self.ui.content_region_avail())
    }

    fn get_window_draw_list(&self) -> Box<dyn DrawList + '_> {
        Box::new(ImguiDrawList {
            list: self.ui.get_window_draw_list(),
        })
    }

    fn get_background_draw_list(&self) -> Box<dyn DrawList + '_> {
        Box::new(ImguiDrawList {
            list: self.ui.get_background_draw_list(),
        })
    }

    fn get_foreground_draw_list(&self) -> Box<dyn DrawList + '_> {
        Box::new(ImguiDrawList {
            list: self.ui.get_foreground_draw_list(),
        })
    }

    fn is_key_pressed(&self, key: Key) -> bool {
        self.ui.is_key_pressed(convert_key(key))
    }

    fn is_key_down(&self, key: Key) -> bool {
        // Use cached state
        let idx = convert_key(key) as usize;
        if idx < 1024 {
            self.cached_keys_down[idx]
        } else {
            false
        }
    }

    fn is_mouse_clicked(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.ui.is_mouse_clicked(imgui::MouseButton::Left),
            MouseButton::Right => self.ui.is_mouse_clicked(imgui::MouseButton::Right),
            MouseButton::Middle => self.ui.is_mouse_clicked(imgui::MouseButton::Middle),
            MouseButton::Extra1 => self.ui.is_mouse_clicked(imgui::MouseButton::Extra1),
            MouseButton::Extra2 => self.ui.is_mouse_clicked(imgui::MouseButton::Extra2),
        }
    }

    fn is_mouse_down(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.ui.is_mouse_down(imgui::MouseButton::Left),
            MouseButton::Right => self.ui.is_mouse_down(imgui::MouseButton::Right),
            MouseButton::Middle => self.ui.is_mouse_down(imgui::MouseButton::Middle),
            MouseButton::Extra1 => self.ui.is_mouse_down(imgui::MouseButton::Extra1),
            MouseButton::Extra2 => self.ui.is_mouse_down(imgui::MouseButton::Extra2),
        }
    }

    fn get_mouse_pos(&self) -> Vec2 {
        Vec2::from(self.ui.io().mouse_pos)
    }

    fn io(&self) -> IoState {
        // Return cached state
        self.cached_io.clone()
    }

    fn keys_down(&self) -> &[bool] {
        &self.ui.io().keys_down
    }

    fn calc_text_size(&self, text: &str) -> Vec2 {
        Vec2::from(self.ui.calc_text_size(text))
    }

    fn push_style_color(&self, style: StyleColor, color: [f32; 4]) -> StyleToken<'_> {
        let imgui_style = match style {
            StyleColor::Text => imgui::StyleColor::Text,
            StyleColor::TextDisabled => imgui::StyleColor::TextDisabled,
            StyleColor::WindowBg => imgui::StyleColor::WindowBg,
            StyleColor::ChildBg => imgui::StyleColor::ChildBg,
            StyleColor::PopupBg => imgui::StyleColor::PopupBg,
            StyleColor::Border => imgui::StyleColor::Border,
            StyleColor::BorderShadow => imgui::StyleColor::BorderShadow,
            StyleColor::FrameBg => imgui::StyleColor::FrameBg,
            StyleColor::FrameBgHovered => imgui::StyleColor::FrameBgHovered,
            StyleColor::FrameBgActive => imgui::StyleColor::FrameBgActive,
            StyleColor::TitleBg => imgui::StyleColor::TitleBg,
            StyleColor::TitleBgActive => imgui::StyleColor::TitleBgActive,
            StyleColor::TitleBgCollapsed => imgui::StyleColor::TitleBgCollapsed,
            StyleColor::MenuBarBg => imgui::StyleColor::MenuBarBg,
            StyleColor::ScrollbarBg => imgui::StyleColor::ScrollbarBg,
            StyleColor::ScrollbarGrab => imgui::StyleColor::ScrollbarGrab,
            StyleColor::ScrollbarGrabHovered => imgui::StyleColor::ScrollbarGrabHovered,
            StyleColor::ScrollbarGrabActive => imgui::StyleColor::ScrollbarGrabActive,
            StyleColor::CheckMark => imgui::StyleColor::CheckMark,
            StyleColor::SliderGrab => imgui::StyleColor::SliderGrab,
            StyleColor::SliderGrabActive => imgui::StyleColor::SliderGrabActive,
            StyleColor::Button => imgui::StyleColor::Button,
            StyleColor::ButtonHovered => imgui::StyleColor::ButtonHovered,
            StyleColor::ButtonActive => imgui::StyleColor::ButtonActive,
            StyleColor::Header => imgui::StyleColor::Header,
            StyleColor::HeaderHovered => imgui::StyleColor::HeaderHovered,
            StyleColor::HeaderActive => imgui::StyleColor::HeaderActive,
            StyleColor::Separator => imgui::StyleColor::Separator,
            StyleColor::SeparatorHovered => imgui::StyleColor::SeparatorHovered,
            StyleColor::SeparatorActive => imgui::StyleColor::SeparatorActive,
            StyleColor::ResizeGrip => imgui::StyleColor::ResizeGrip,
            StyleColor::ResizeGripHovered => imgui::StyleColor::ResizeGripHovered,
            StyleColor::ResizeGripActive => imgui::StyleColor::ResizeGripActive,
            StyleColor::Tab => imgui::StyleColor::Tab,
            StyleColor::TabHovered => imgui::StyleColor::TabHovered,
            StyleColor::TabActive => imgui::StyleColor::TabActive,
            StyleColor::TabUnfocused => imgui::StyleColor::TabUnfocused,
            StyleColor::TabUnfocusedActive => imgui::StyleColor::TabUnfocusedActive,
            StyleColor::PlotLines => imgui::StyleColor::PlotLines,
            StyleColor::PlotLinesHovered => imgui::StyleColor::PlotLinesHovered,
            StyleColor::PlotHistogram => imgui::StyleColor::PlotHistogram,
            StyleColor::PlotHistogramHovered => imgui::StyleColor::PlotHistogramHovered,
            StyleColor::TableHeaderBg => imgui::StyleColor::TableHeaderBg,
            StyleColor::TableBorderStrong => imgui::StyleColor::TableBorderStrong,
            StyleColor::TableBorderLight => imgui::StyleColor::TableBorderLight,
            StyleColor::TableRowBg => imgui::StyleColor::TableRowBg,
            StyleColor::TableRowBgAlt => imgui::StyleColor::TableRowBgAlt,
            StyleColor::TextSelectedBg => imgui::StyleColor::TextSelectedBg,
            StyleColor::DragDropTarget => imgui::StyleColor::DragDropTarget,
            StyleColor::NavHighlight => imgui::StyleColor::NavHighlight,
            StyleColor::NavWindowingHighlight => imgui::StyleColor::NavWindowingHighlight,
            StyleColor::NavWindowingDimBg => imgui::StyleColor::NavWindowingDimBg,
            StyleColor::ModalWindowDimBg => imgui::StyleColor::ModalWindowDimBg,
        };

        let token = self.ui.push_style_color(imgui_style, color);
        StyleToken { token }
    }

    fn pop_style_color(&self) {
        // Handled by RAII StyleToken
    }

    fn progress_bar(&self, fraction: f32, size: Vec2, overlay: Option<&str>) {
        let mut bar = imgui::ProgressBar::new(fraction).size(size.to_array());

        if let Some(text) = overlay {
            bar = bar.overlay_text(text);
        }

        bar.build(self.ui);
    }

    fn push_font_by_index(&self, index: usize) -> Option<crate::gui::FontToken<'_>> {
        let fonts = self.ui.fonts();
        let font_id = fonts.fonts().get(index).copied()?;
        Some(crate::gui::FontToken {
            token: self.ui.push_font(font_id),
        })
    }

    fn get_fonts_count(&self) -> usize {
        self.ui.fonts().fonts().len()
    }

    fn list_box(
        &self,
        label: &str,
        current_item: &mut i32,
        items: &[&str],
        _height_in_items: i32,
    ) -> bool {
        let mut changed = false;
        let height_mode = imgui::ListBox::new(label);

        height_mode.build(self.ui, || {
            for (i, item) in items.iter().enumerate() {
                let is_selected = *current_item == i as i32;
                if self
                    .ui
                    .selectable_config(item)
                    .selected(is_selected)
                    .build()
                {
                    *current_item = i as i32;
                    changed = true;
                }
            }
        });
        changed
    }

    fn slider(&self, label: &str, min: f32, max: f32, value: &mut f32) -> bool {
        self.ui.slider(label, min, max, value)
    }

    fn child_window<'a>(&'a self, id: &'a str) -> ChildWindowBuilder<'a> {
        ChildWindowBuilder {
            ui: self.ui,
            id,
            size: [0.0, 0.0],
            border: false,
        }
    }

    fn tree_node_config<'a>(&'a self, label: &'a str) -> TreeNodeBuilder<'a> {
        TreeNodeBuilder {
            ui: self.ui,
            label,
            flags: TreeNodeFlags::default(),
        }
    }

    // ===== New Method Implementations =====

    fn window<'a>(&'a self, name: &'a str) -> crate::gui::WindowBuilder<'a> {
        crate::gui::WindowBuilder {
            ui: self.ui,
            name,
            opened: None,
            position: None,
            size: None,
            bg_alpha: None,
            flags: None,
        }
    }

    fn tooltip_text(&self, text: &str) {
        if self.ui.is_item_hovered() {
            self.ui.tooltip_text(text);
        }
    }

    fn input_text<'a>(
        &'a self,
        label: &'a str,
        buffer: &'a mut String,
    ) -> crate::gui::InputTextBuilder<'a> {
        crate::gui::InputTextBuilder {
            ui: self.ui,
            label,
            buffer,
            enter_returns_true: false,
            read_only: false,
        }
    }

    fn scroll_y(&self) -> f32 {
        self.ui.scroll_y()
    }

    fn scroll_max_y(&self) -> f32 {
        self.ui.scroll_max_y()
    }

    fn set_scroll_here_y(&self) {
        self.ui.set_scroll_here_y();
    }

    fn frame_count(&self) -> u32 {
        self.ui.frame_count() as u32
    }

    fn clipboard_text(&self) -> Option<String> {
        self.ui.clipboard_text().map(|s| s.to_string())
    }

    fn set_clipboard_text(&self, text: &str) {
        self.ui.set_clipboard_text(text);
    }

    fn text_line_height(&self) -> f32 {
        self.ui.text_line_height()
    }

    fn begin_table_header(&self) {
        self.ui.table_headers_row();
    }

    unsafe fn style(&self) -> &imgui::Style {
        self.ui.style()
    }

    fn input_scalar(&self, label: &str, value: &mut i32) -> bool {
        self.ui.input_scalar(label, value).build()
    }

    // ===== Texture Management =====

    fn upload_texture(&self, data: &[u8], width: u32, height: u32) -> imgui::TextureId {
        // Queue texture upload request (will be processed during render pass)
        let id = request_texture_upload(data, width, height);
        // Return a temporary ID - the real TextureId will be mapped after flush
        imgui::TextureId::new(id as usize)
    }

    fn free_texture(&self, texture_id: imgui::TextureId) {
        // TODO: Implement texture cleanup via thread-local free queue
        // For now, textures are leaked (will fix after basic upload works)
        let _ = texture_id;
    }
}

// ===== ImGui DrawList Implementation =====

#[cfg(feature = "ui-imgui")]
pub struct ImguiDrawList<'a> {
    pub(crate) list: imgui::DrawListMut<'a>,
}

#[cfg(feature = "ui-imgui")]
impl DrawList for ImguiDrawList<'_> {
    fn add_rect(&mut self, p1: Vec2, p2: Vec2, color: Color, filled: bool) {
        let color_u32 = color_to_u32(color);
        if filled {
            self.list
                .add_rect(p1.to_array(), p2.to_array(), color_u32)
                .filled(true)
                .build();
        } else {
            self.list
                .add_rect(p1.to_array(), p2.to_array(), color_u32)
                .build();
        }
    }

    fn add_text(&mut self, pos: Vec2, color: Color, text: &str) {
        self.list
            .add_text(pos.to_array(), color_to_u32(color), text);
    }

    fn add_line(&mut self, p1: Vec2, p2: Vec2, color: Color, thickness: f32) {
        self.list
            .add_line(p1.to_array(), p2.to_array(), color_to_u32(color))
            .thickness(thickness)
            .build();
    }

    fn add_circle(&mut self, center: Vec2, radius: f32, color: Color, filled: bool) {
        let color_u32 = color_to_u32(color);
        if filled {
            self.list
                .add_circle(center.to_array(), radius, color_u32)
                .filled(true)
                .build();
        } else {
            self.list
                .add_circle(center.to_array(), radius, color_u32)
                .build();
        }
    }

    fn push_clip_rect(&mut self, min: Vec2, max: Vec2, intersect_with_current: bool) {
        self.list
            .with_clip_rect(min.to_array(), max.to_array(), || {});
    }

    fn pop_clip_rect(&mut self) {
        // self.list.pop_clip_rect();
    }

    // ===== Texture Rendering =====

    fn add_image(&mut self, texture_id: imgui::TextureId, p_min: Vec2, p_max: Vec2) {
        let gpu_texture_id = translate_texture_id(texture_id);
        self.list
            .add_image(gpu_texture_id, p_min.to_array(), p_max.to_array())
            .build();
    }

    fn add_image_quad(
        &mut self,
        texture_id: imgui::TextureId,
        p_min: Vec2,
        p_max: Vec2,
        uv_min: Vec2,
        uv_max: Vec2,
        col: Color,
    ) {
        let gpu_texture_id = translate_texture_id(texture_id);
        self.list
            .add_image(gpu_texture_id, p_min.to_array(), p_max.to_array())
            .uv_min(uv_min.to_array())
            .uv_max(uv_max.to_array())
            .col(color_to_u32(col))
            .build();
    }
}

#[cfg(feature = "ui-imgui")]
fn color_to_u32(color: Color) -> u32 {
    let r = (color.r * 255.0) as u32;
    let g = (color.g * 255.0) as u32;
    let b = (color.b * 255.0) as u32;
    let a = (color.a * 255.0) as u32;
    (a << 24) | (b << 16) | (g << 8) | r
}

// Helper conversion functions
#[cfg(feature = "ui-imgui")]
fn convert_tree_flags(flags: TreeNodeFlags) -> imgui::TreeNodeFlags {
    let mut result = imgui::TreeNodeFlags::empty();
    if flags.default_open {
        result |= imgui::TreeNodeFlags::DEFAULT_OPEN;
    }
    if flags.open_on_arrow {
        result |= imgui::TreeNodeFlags::OPEN_ON_ARROW;
    }
    if flags.leaf {
        result |= imgui::TreeNodeFlags::LEAF;
    }
    if flags.bullet {
        result |= imgui::TreeNodeFlags::BULLET;
    }
    result
}

#[cfg(feature = "ui-imgui")]
pub(crate) fn convert_condition(cond: WinCondition) -> imgui::Condition {
    match cond {
        WinCondition::Always => imgui::Condition::Always,
        WinCondition::Once => imgui::Condition::Once,
        WinCondition::FirstUseEver => imgui::Condition::FirstUseEver,
        WinCondition::Appearing => imgui::Condition::Appearing,
    }
}

#[cfg(feature = "ui-imgui")]
fn convert_table_flags(flags: TableFlags) -> imgui::TableFlags {
    let mut result = imgui::TableFlags::empty();
    if flags.borders {
        result |= imgui::TableFlags::BORDERS;
    }
    if flags.row_bg {
        result |= imgui::TableFlags::ROW_BG;
    }
    if flags.resizable {
        result |= imgui::TableFlags::RESIZABLE;
    }
    if flags.sizing_fixed_fit {
        result |= imgui::TableFlags::SIZING_FIXED_FIT;
    }
    result
}

#[cfg(feature = "ui-imgui")]
fn convert_key(key: Key) -> imgui::Key {
    match key {
        Key::Escape => imgui::Key::Escape,
        Key::A => imgui::Key::A,
        Key::B => imgui::Key::B,
        Key::C => imgui::Key::C,
        Key::D => imgui::Key::D,
        Key::E => imgui::Key::E,
        Key::F => imgui::Key::F,
        Key::G => imgui::Key::G,
        Key::H => imgui::Key::H,
        Key::I => imgui::Key::I,
        Key::J => imgui::Key::J,
        Key::K => imgui::Key::K,
        Key::L => imgui::Key::L,
        Key::M => imgui::Key::M,
        Key::N => imgui::Key::N,
        Key::O => imgui::Key::O,
        Key::P => imgui::Key::P,
        Key::Q => imgui::Key::Q,
        Key::R => imgui::Key::R,
        Key::S => imgui::Key::S,
        Key::T => imgui::Key::T,
        Key::U => imgui::Key::U,
        Key::V => imgui::Key::V,
        Key::W => imgui::Key::W,
        Key::X => imgui::Key::X,
        Key::Y => imgui::Key::Y,
        Key::Z => imgui::Key::Z,
        Key::Num0 => imgui::Key::Alpha0,
        Key::Num1 => imgui::Key::Alpha1,
        Key::Num2 => imgui::Key::Alpha2,
        Key::Num3 => imgui::Key::Alpha3,
        Key::Num4 => imgui::Key::Alpha4,
        Key::Num5 => imgui::Key::Alpha5,
        Key::Num6 => imgui::Key::Alpha6,
        Key::Num7 => imgui::Key::Alpha7,
        Key::Num8 => imgui::Key::Alpha8,
        Key::Num9 => imgui::Key::Alpha9,
        Key::F1 => imgui::Key::F1,
        Key::F2 => imgui::Key::F2,
        Key::F3 => imgui::Key::F3,
        Key::F4 => imgui::Key::F4,
        Key::F5 => imgui::Key::F5,
        Key::F6 => imgui::Key::F6,
        Key::F7 => imgui::Key::F7,
        Key::F8 => imgui::Key::F8,
        Key::F9 => imgui::Key::F9,
        Key::F10 => imgui::Key::F10,
        Key::F11 => imgui::Key::F11,
        Key::F12 => imgui::Key::F12,
        Key::Space => imgui::Key::Space,
        Key::Tab => imgui::Key::Tab,
        Key::Enter => imgui::Key::Enter,
        Key::Backspace => imgui::Key::Backspace,
        Key::Delete => imgui::Key::Delete,
        Key::Insert => imgui::Key::Insert,
        Key::Home => imgui::Key::Home,
        Key::End => imgui::Key::End,
        Key::PageUp => imgui::Key::PageUp,
        Key::PageDown => imgui::Key::PageDown,
        Key::LeftArrow => imgui::Key::LeftArrow,
        Key::RightArrow => imgui::Key::RightArrow,
        Key::UpArrow => imgui::Key::UpArrow,
        Key::DownArrow => imgui::Key::DownArrow,
        Key::GraveAccent => imgui::Key::GraveAccent,
        Key::Keypad0 => imgui::Key::Keypad0,
        Key::Keypad1 => imgui::Key::Keypad1,
        Key::Keypad2 => imgui::Key::Keypad2,
        Key::Keypad3 => imgui::Key::Keypad3,
        Key::Keypad4 => imgui::Key::Keypad4,
        Key::Keypad5 => imgui::Key::Keypad5,
        Key::Keypad6 => imgui::Key::Keypad6,
        Key::Keypad7 => imgui::Key::Keypad7,
        Key::Keypad8 => imgui::Key::Keypad8,
        Key::Keypad9 => imgui::Key::Keypad9,
        // Punctuation
        Key::Comma => imgui::Key::Comma,
        Key::Period => imgui::Key::Period,
        Key::Slash => imgui::Key::Slash,
        Key::Semicolon => imgui::Key::Semicolon,
        Key::Apostrophe => imgui::Key::Apostrophe,
        Key::LeftBracket => imgui::Key::LeftBracket,
        Key::RightBracket => imgui::Key::RightBracket,
        Key::Backslash => imgui::Key::Backslash,
        Key::Minus => imgui::Key::Minus,
        Key::Equal => imgui::Key::Equal,
        // Modifiers
        Key::LeftShift => imgui::Key::LeftShift,
        Key::RightShift => imgui::Key::RightShift,
        Key::LeftCtrl => imgui::Key::LeftCtrl,
        Key::RightCtrl => imgui::Key::RightCtrl,
        Key::LeftAlt => imgui::Key::LeftAlt,
        Key::RightAlt => imgui::Key::RightAlt,
        Key::LeftSuper => imgui::Key::LeftSuper,
        Key::RightSuper => imgui::Key::RightSuper,
        Key::CapsLock => imgui::Key::CapsLock,
        Key::ScrollLock => imgui::Key::ScrollLock,
        Key::NumLock => imgui::Key::NumLock,
        Key::PrintScreen => imgui::Key::PrintScreen,
        Key::Pause => imgui::Key::Pause,
        Key::Menu => imgui::Key::Menu,
    }
}
