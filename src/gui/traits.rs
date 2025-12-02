use crate::gui::{
    Color, IoState, Key, MouseButton, StyleColor, TableFlags, TreeNodeFlags, Vec2, WinCondition,
};

// ===== RAII Tokens =====

pub struct WindowToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::WindowToken<'a>,
}

pub struct TableToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::TableToken<'a>,
}

pub struct TreeNodeToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::TreeNodeToken<'a>,
}

pub struct MenuBarToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::MenuBarToken<'a>,
}

pub struct MenuToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::MenuToken<'a>,
}

pub struct ComboBoxToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::ComboBoxToken<'a>,
}

pub struct SelectableBuilder<'ui, 'label> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) ui: &'ui imgui::Ui,
    #[cfg(feature = "ui-imgui")]
    pub(crate) label: &'label str,
    #[cfg(feature = "ui-imgui")]
    pub(crate) selected: bool,
}

#[cfg(feature = "ui-imgui")]
impl<'ui, 'label> SelectableBuilder<'ui, 'label> {
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn build(self) -> bool {
        self.ui
            .selectable_config(self.label)
            .selected(self.selected)
            .build()
    }
}

pub struct StyleToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::ColorStackToken<'a>,
}

// ===== Window Options =====

#[derive(Default)]
pub struct WindowOptions {
    pub opened: Option<bool>,
    pub size: Option<([f32; 2], WinCondition)>,
    pub position: Option<([f32; 2], WinCondition)>,
    pub menu_bar: bool,
    pub always_auto_resize: bool,
    pub resizable: bool,
}

impl WindowOptions {
    pub fn new() -> Self {
        Self {
            opened: None,
            size: None,
            position: None,
            menu_bar: false,
            always_auto_resize: false,
            resizable: true,
        }
    }

    pub fn with_opened(mut self, opened: bool) -> Self {
        self.opened = Some(opened);
        self
    }

    pub fn with_size(mut self, size: [f32; 2], cond: WinCondition) -> Self {
        self.size = Some((size, cond));
        self
    }

    pub fn with_position(mut self, pos: [f32; 2], cond: WinCondition) -> Self {
        self.position = Some((pos, cond));
        self
    }

    pub fn with_menu_bar(mut self, enabled: bool) -> Self {
        self.menu_bar = enabled;
        self
    }

    pub fn with_always_auto_resize(mut self, enabled: bool) -> Self {
        self.always_auto_resize = enabled;
        self
    }

    pub fn with_resizable(mut self, enabled: bool) -> Self {
        self.resizable = enabled;
        self
    }
}

// ===== UiBackend Trait =====

pub trait UiBackend {
    // ===== Text =====
    fn text(&self, text: &str);
    fn text_colored(&self, color: Color, text: &str);
    fn text_wrapped(&self, text: &str);
    fn text_disabled(&self, text: &str);

    // ===== Layout =====
    fn same_line(&self);
    fn separator(&self);
    fn spacing(&self);
    fn indent(&self);
    fn unindent(&self);
    fn new_line(&self);
    fn dummy(&self, size: Vec2);
    fn group(&self, f: &mut dyn FnMut());

    // ===== Basic Widgets =====
    fn button(&self, label: &str) -> bool;
    fn small_button(&self, label: &str) -> bool;
    fn checkbox(&self, label: &str, value: &mut bool) -> bool;
    fn radio_button(&self, label: &str, active: bool) -> bool;

    // ===== Input Widgets - Simple versions =====
    fn input_text(&self, label: &str, buffer: &mut String) -> bool;
    fn input_text_readonly(&self, label: &str, buffer: &mut String) -> bool;
    fn input_int(&self, label: &str, value: &mut i32) -> bool;
    fn input_int_with_step(&self, label: &str, value: &mut i32, step: i32, step_fast: i32) -> bool;
    fn input_float(&self, label: &str, value: &mut f32) -> bool;
    fn input_float_with_step(
        &self,
        label: &str,
        value: &mut f32,
        step: f32,
        step_fast: f32,
    ) -> bool;
    fn input_float2(&self, label: &str, values: &mut [f32; 2]) -> bool;

    // ===== Windows =====
    fn begin_window_with_options(
        &self,
        title: &str,
        options: &WindowOptions,
    ) -> Option<WindowToken<'_>>;
    fn begin_window_simple(
        &self,
        title: &str,
        opened: &mut bool,
        options: &WindowOptions,
    ) -> Option<WindowToken<'_>>;
    // fn end_window(&self); // RAII handles this
    fn begin_child(&self, id: &str, size: Vec2, border: bool, ch_fn: &mut fn()) -> bool;
    fn end_child(&self);

    // ===== Menus =====
    fn begin_menu_bar(&self) -> Option<MenuBarToken<'_>>;
    fn end_menu_bar(&self, token: Option<MenuBarToken<'_>>);
    fn begin_menu(&self, label: &str) -> Option<MenuToken<'_>>;
    fn end_menu(&self, token: Option<MenuBarToken<'_>>);
    fn menu_item(&self, label: &str) -> bool;

    // ===== Trees & Headers =====
    fn collapsing_header(&self, label: &str, flags: TreeNodeFlags) -> bool;
    fn tree_node(&self, label: &str) -> Option<TreeNodeToken<'_>>;
    fn tree_pop(&self);

    // ===== Tables =====
    fn begin_table(&self, id: &str, columns: usize, flags: TableFlags) -> Option<TableToken<'_>>;
    fn end_table(&self);
    fn table_next_row(&self);
    fn table_next_column(&self);
    fn table_setup_column(&self, label: &str);
    fn table_headers_row(&self);

    // ===== Columns (Legacy) =====
    fn columns(&self, count: i32, id: &str, border: bool);
    fn next_column(&self);
    fn set_column_width(&self, column_idx: i32, width: f32);
    fn get_column_width(&self) -> f32;

    // ===== Combos =====
    fn begin_combo(&self, label: &str, preview: &str) -> Option<ComboBoxToken<'_>>;
    fn end_combo(&self);
    fn selectable<'a>(&self, label: &'a str) -> SelectableBuilder<'_, 'a>;

    // ===== Popups =====
    fn open_popup(&self, id: &str);
    fn begin_popup(&self, id: &str) -> bool;
    fn end_popup(&self);

    // ===== ID Stack =====
    fn push_id_str(&self, id: &str);
    fn push_id_int(&self, id: i32);
    fn push_id_usize(&self, id: usize);
    fn pop_id(&self);

    // ===== Item State =====
    fn set_item_default_focus(&self);
    fn is_item_hovered(&self) -> bool;
    fn is_item_active(&self) -> bool;
    fn is_item_clicked(&self, button: MouseButton) -> bool;

    // ===== Positioning =====
    fn set_next_item_width(&self, width: f32);
    fn set_cursor_pos(&self, pos: Vec2);
    fn get_cursor_pos(&self) -> Vec2;
    fn get_cursor_screen_pos(&self) -> Vec2;
    fn set_cursor_screen_pos(&self, pos: Vec2);

    // ===== Window Info =====
    fn get_window_pos(&self) -> Vec2;
    fn get_window_size(&self) -> Vec2;
    fn get_content_region_avail(&self) -> Vec2;
    fn current_font_size(&self) -> f32;

    // ===== Drawing =====
    fn get_window_draw_list(&self) -> Box<dyn DrawList + '_>;
    fn get_background_draw_list(&self) -> Box<dyn DrawList + '_>;
    fn get_foreground_draw_list(&self) -> Box<dyn DrawList + '_>;

    // ===== Input =====
    fn is_key_pressed(&self, key: Key) -> bool;
    fn is_key_down(&self, key: Key) -> bool;
    fn is_mouse_clicked(&self, button: MouseButton) -> bool;
    fn get_mouse_pos(&self) -> Vec2;
    fn io(&self) -> IoState;
    fn keys_down(&self) -> &[bool];

    // ===== Misc =====
    fn calc_text_size(&self, text: &str) -> Vec2;
    fn push_style_color(&self, style: StyleColor, color: Color) -> StyleToken;
    fn pop_style_color(&self);
    fn progress_bar(&self, fraction: f32, size: Vec2, overlay: Option<&str>);
}

// ===== DrawList Trait =====

pub trait DrawList {
    fn add_rect(&mut self, p1: Vec2, p2: Vec2, color: Color, filled: bool);
    fn add_text(&mut self, pos: Vec2, color: Color, text: &str);
    fn add_line(&mut self, p1: Vec2, p2: Vec2, color: Color, thickness: f32);
    fn add_circle(&mut self, center: Vec2, radius: f32, color: Color, filled: bool);

    // Manual clipping - dyn-compatible alternative to with_clip_rect
    fn push_clip_rect(&mut self, min: Vec2, max: Vec2, intersect_with_current: bool);
    fn pop_clip_rect(&mut self);
}
