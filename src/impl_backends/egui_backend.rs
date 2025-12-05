use crate::gui::{
    ChildWindowBuilder, ColorEdit3Builder, ComboBoxToken, DisabledToken, DrawList, FontToken,
    IoState, Key, MenuBarToken, MenuToken, MouseButton, SelectableBuilder, StyleColor, StyleToken,
    TableFlags, TableToken, TreeNodeBuilder, TreeNodeFlags, TreeNodeToken, UiBackend, Vec2,
    WindowOptions, WindowToken,
};
#[cfg(feature = "ui-egui")]
use ::egui;
use std::cell::RefCell;

#[cfg(feature = "ui-egui")]
pub struct EguiBackend<'a> {
    pub(crate) ctx: &'a egui::Context,
    pub(crate) ui: RefCell<&'a mut egui::Ui>,
}

#[cfg(feature = "ui-egui")]
impl<'a> EguiBackend<'a> {
    pub fn new(ctx: &'a egui::Context, ui: &'a mut egui::Ui) -> Self {
        Self {
            ctx,
            ui: RefCell::new(ui),
        }
    }
}

#[cfg(feature = "ui-egui")]
impl<'a> UiBackend for EguiBackend<'a> {
    fn native_ui(&self) -> &imgui::Ui {
        panic!("EguiBackend does not support native_ui (imgui)")
    }

    fn text(&self, text: &str) {
        self.ui.borrow_mut().label(text);
    }

    fn text_colored(&self, color: [f32; 4], text: &str) {
        use egui::Color32;
        let color = Color32::from_rgba_unmultiplied(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        self.ui.borrow_mut().colored_label(color, text);
    }

    fn text_wrapped(&self, text: &str) {
        self.ui.borrow_mut().label(text);
    }

    fn text_disabled(&self, text: &str) {
        self.ui
            .borrow_mut()
            .add_enabled(false, egui::Label::new(text));
    }

    fn bullet_text(&self, text: &str) {
        self.ui.borrow_mut().label(format!("• {}", text));
    }

    fn same_line(&self) {}
    fn same_line_with_pos(&self, _pos_x: f32) {}
    fn align_text_to_frame_padding(&self) {}
    fn separator(&self) {
        self.ui.borrow_mut().separator();
    }
    fn spacing(&self) {
        self.ui.borrow_mut().add_space(4.0);
    }
    fn indent(&self) {}
    fn unindent(&self) {}
    fn new_line(&self) {}
    fn dummy(&self, size: Vec2) {
        self.ui
            .borrow_mut()
            .allocate_space(egui::vec2(size.x, size.y));
    }

    fn group(&self, f: &mut dyn FnMut()) {
        f();
    }

    fn child_window<'b>(&'b self, _id: &'b str) -> ChildWindowBuilder<'b> {
        todo!("child_window not implemented for egui")
    }

    fn tree_node_config<'b>(&'b self, _label: &'b str) -> TreeNodeBuilder<'b> {
        todo!("tree_node_config not implemented for egui")
    }

    fn button(&self, label: &str) -> bool {
        self.ui.borrow_mut().button(label).clicked()
    }

    fn small_button(&self, label: &str) -> bool {
        self.ui.borrow_mut().small_button(label).clicked()
    }

    fn checkbox(&self, label: &str, value: &mut bool) -> bool {
        self.ui.borrow_mut().checkbox(value, label).changed()
    }

    fn radio_button_bool(&self, label: &str, active: bool) -> bool {
        self.ui.borrow_mut().radio(active, label).clicked()
    }

    fn radio_button_usize(&self, label: &str, value: &mut usize, test_value: usize) -> bool {
        let mut wrapper = *value == test_value;
        if self.ui.borrow_mut().radio(wrapper, label).clicked() {
            *value = test_value;
            return true;
        }
        false
    }

    fn list_box(
        &self,
        _label: &str,
        _current_item: &mut i32,
        _items: &[&str],
        _height_in_items: i32,
    ) -> bool {
        false
    }

    fn slider(&self, label: &str, min: f32, max: f32, value: &mut f32) -> bool {
        self.ui
            .borrow_mut()
            .add(egui::Slider::new(value, min..=max).text(label))
            .changed()
    }

    fn color_edit4(&self, label: &str, color: &mut [f32; 4]) -> bool {
        let mut c = egui::Color32::from_rgba_unmultiplied(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        let mut ui = self.ui.borrow_mut();
        if ui.color_edit_button_srgba(&mut c).changed() {
            color[0] = c.r() as f32 / 255.0;
            color[1] = c.g() as f32 / 255.0;
            color[2] = c.b() as f32 / 255.0;
            color[3] = c.a() as f32 / 255.0;
            ui.label(label);
            return true;
        }
        ui.label(label);
        false
    }

    fn disabled(&self, disabled: bool, f: &mut dyn FnMut()) {
        f();
    }

    fn begin_disabled(&self, _disabled: bool) -> DisabledToken<'_> {
        DisabledToken {
            #[cfg(feature = "ui-imgui")]
            token: unsafe { std::mem::zeroed() },
        }
    }

    fn input_text_simple(&self, label: &str, buffer: &mut String) -> bool {
        self.ui
            .borrow_mut()
            .horizontal(|ui| {
                ui.label(label);
                ui.text_edit_singleline(buffer).changed()
            })
            .inner
    }

    fn input_text_readonly(&self, label: &str, buffer: &mut String) -> bool {
        self.ui
            .borrow_mut()
            .horizontal(|ui| {
                ui.label(label);
                ui.add_enabled(false, egui::TextEdit::singleline(buffer))
                    .changed()
            })
            .inner
    }

    fn input_int(&self, label: &str, value: &mut i32) -> bool {
        self.ui
            .borrow_mut()
            .horizontal(|ui| {
                ui.label(label);
                ui.add(egui::DragValue::new(value)).changed()
            })
            .inner
    }

    fn input_int_with_step(
        &self,
        _label: &str,
        _value: &mut i32,
        _step: i32,
        _step_fast: i32,
    ) -> bool {
        false
    }
    fn input_float(&self, _label: &str, _value: &mut f32) -> bool {
        false
    }
    fn input_float_with_step(
        &self,
        _label: &str,
        _value: &mut f32,
        _step: f32,
        _step_fast: f32,
    ) -> bool {
        false
    }
    fn input_float2(&self, _label: &str, _values: &mut [f32; 2]) -> bool {
        false
    }
    fn input_scalar(&self, _label: &str, _value: &mut i32) -> bool {
        false
    }

    fn slider_int(&self, _label: &str, _min: i32, _max: i32, _value: &mut i32) -> bool {
        false
    }
    fn slider_float(&self, _label: &str, _min: f32, _max: f32, _value: &mut f32) -> bool {
        false
    }

    fn color_edit3(&self, _label: &str, _color: &mut [f32; 3]) -> bool {
        false
    }
    fn color_edit3_config<'b>(
        &'b self,
        _label: &'b str,
        _color: &'b mut [f32; 3],
    ) -> ColorEdit3Builder<'b> {
        todo!("color_edit3_config")
    }

    fn selectable_config<'b>(&'b self, _label: &'b str) -> SelectableBuilder<'_, 'b> {
        todo!("selectable_config")
    }

    fn begin_window_with_options(
        &self,
        title: &str,
        options: &WindowOptions,
    ) -> Option<WindowToken<'_>> {
        None
    }

    fn begin_window_simple(
        &self,
        title: &str,
        opened: &mut bool,
        _options: &WindowOptions,
    ) -> Option<WindowToken<'_>> {
        None
    }

    fn begin_child(&self, _id: &str, _size: Vec2, _border: bool, _ch_fn: &mut fn()) -> bool {
        false
    }
    fn end_child(&self) {}

    fn begin_menu_bar(&self) -> Option<MenuBarToken<'_>> {
        None
    }
    fn end_menu_bar(&self, _token: Option<MenuBarToken<'_>>) {}
    fn begin_menu(&self, _label: &str) -> Option<MenuToken<'_>> {
        None
    }
    fn end_menu(&self, _token: Option<MenuBarToken<'_>>) {}
    fn menu_item(&self, label: &str) -> bool {
        self.ui.borrow_mut().button(label).clicked()
    }

    fn collapsing_header(&self, label: &str, _flags: TreeNodeFlags) -> bool {
        self.ui.borrow_mut().label(format!("> {}", label));
        true
    }

    fn tree_node(&self, _label: &str) -> Option<TreeNodeToken<'_>> {
        None
    }
    fn tree_pop(&self) {}

    fn begin_table(
        &self,
        _id: &str,
        _columns: usize,
        _flags: TableFlags,
    ) -> Option<TableToken<'_>> {
        None
    }
    fn end_table(&self) {}
    fn table_next_row(&self) {}
    fn table_next_column(&self) {}
    fn table_setup_column(&self, _label: &str) {}
    fn table_headers_row(&self) {}

    fn columns(&self, _count: i32, _id: &str, _border: bool) {}
    fn next_column(&self) {}
    fn set_column_width(&self, _column_idx: i32, _width: f32) {}
    fn get_column_width(&self) -> f32 {
        0.0
    }

    fn begin_combo(&self, _label: &str, _preview: &str) -> Option<ComboBoxToken<'_>> {
        None
    }
    fn end_combo(&self) {}
    fn selectable<'b>(&self, _label: &'b str) -> SelectableBuilder<'_, 'b> {
        todo!()
    }

    fn open_popup(&self, _id: &str) {}
    fn begin_popup(&self, _id: &str) -> bool {
        false
    }
    fn end_popup(&self) {}

    fn push_id_str(&self, _id: &str) {}
    fn push_id_int(&self, _id: i32) {}
    fn push_id_usize(&self, _id: usize) {}
    fn pop_id(&self) {}

    fn set_item_default_focus(&self) {}
    fn is_item_hovered(&self) -> bool {
        false
    }
    fn is_item_active(&self) -> bool {
        false
    }
    fn is_item_clicked(&self, _button: MouseButton) -> bool {
        false
    }

    fn set_next_item_width(&self, _width: f32) {}
    fn set_cursor_pos(&self, _pos: Vec2) {}
    fn get_cursor_pos(&self) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn get_cursor_screen_pos(&self) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn set_cursor_screen_pos(&self, _pos: Vec2) {}

    fn get_window_pos(&self) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn get_window_size(&self) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn current_font_size(&self) -> f32 {
        14.0
    }

    fn get_window_draw_list(&self) -> Box<dyn crate::gui::DrawList + '_> {
        Box::new(EguiDrawList)
    }
    fn get_background_draw_list(&self) -> Box<dyn crate::gui::DrawList + '_> {
        Box::new(EguiDrawList)
    }
    fn get_foreground_draw_list(&self) -> Box<dyn crate::gui::DrawList + '_> {
        Box::new(EguiDrawList)
    }

    fn is_key_pressed(&self, _key: Key) -> bool {
        false
    }
    fn is_key_down(&self, _key: Key) -> bool {
        false
    }
    fn is_mouse_clicked(&self, _button: MouseButton) -> bool {
        false
    }
    fn is_mouse_down(&self, _button: MouseButton) -> bool {
        false
    }
    fn get_mouse_pos(&self) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn io(&self) -> IoState {
        IoState::default()
    }
    fn keys_down(&self) -> &[bool] {
        &[]
    }

    fn calc_text_size(&self, _text: &str) -> Vec2 {
        [0.0, 0.0].into()
    }
    fn push_style_color(&self, _style: StyleColor, _color: [f32; 4]) -> StyleToken<'_> {
        StyleToken {
            #[cfg(feature = "ui-imgui")]
            token: unsafe { std::mem::zeroed() },
        }
    }
    fn pop_style_color(&self) {}
    fn progress_bar(&self, fraction: f32, _size: Vec2, _overlay: Option<&str>) {
        self.ui.borrow_mut().add(egui::ProgressBar::new(fraction));
    }

    fn push_font_by_index(&self, _index: usize) -> Option<FontToken<'_>> {
        None
    }
    fn get_fonts_count(&self) -> usize {
        1
    }

    fn window<'b>(&'b self, _name: &'b str) -> crate::gui::WindowBuilder<'b> {
        todo!("window builder stub")
    }

    fn input_text<'b>(
        &'b self,
        _label: &'b str,
        _buffer: &'b mut String,
    ) -> crate::gui::InputTextBuilder<'b> {
        todo!("input text builder stub")
    }

    fn tooltip_text(&self, text: &str) {
        self.ui.borrow_mut().label(text).on_hover_text(text);
    }

    fn scroll_y(&self) -> f32 {
        0.0
    }
    fn scroll_max_y(&self) -> f32 {
        0.0
    }
    fn set_scroll_here_y(&self) {}
    fn frame_count(&self) -> u32 {
        0
    }
    fn clipboard_text(&self) -> Option<String> {
        None
    }
    fn set_clipboard_text(&self, _text: &str) {}
    fn text_line_height(&self) -> f32 {
        14.0
    }
    fn begin_table_header(&self) {}
    unsafe fn style(&self) -> &imgui::Style {
        panic!("No imgui style")
    }
    fn content_region_avail(&self) -> Vec2 {
        [100.0, 100.0].into()
    }
}

#[cfg(feature = "ui-egui")]
pub struct EguiDrawList;

#[cfg(feature = "ui-egui")]
impl DrawList for EguiDrawList {
    fn add_rect(&mut self, _p1: Vec2, _p2: Vec2, _color: crate::gui::Color, _filled: bool) {}
    fn add_text(&mut self, _pos: Vec2, _color: crate::gui::Color, _text: &str) {}
    fn add_line(&mut self, _p1: Vec2, _p2: Vec2, _color: crate::gui::Color, _thickness: f32) {}
    fn add_circle(
        &mut self,
        _center: Vec2,
        _radius: f32,
        _color: crate::gui::Color,
        _filled: bool,
    ) {
    }
    fn push_clip_rect(&mut self, _min: Vec2, _max: Vec2, _intersect_with_current: bool) {}
    fn pop_clip_rect(&mut self) {}
}
