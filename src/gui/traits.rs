use crate::gui::{
    Color, IoState, Key, MouseButton, StyleColor, TableFlags, TreeNodeFlags, Vec2, WinCondition,
};

#[cfg(feature = "ui-imgui")]
use crate::{gui::ImguiWindowFlags, impl_backends::imgui_backend::convert_condition};

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

pub struct StyleToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::ColorStackToken<'a>,
}

pub struct DisabledToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::DisabledToken<'a>,
}

pub struct ChildWindowToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::ChildWindowToken<'a>,
}

pub struct FontToken<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) token: imgui::FontStackToken<'a>,
}

pub struct ColorEdit3Builder<'a> {
    #[cfg(feature = "ui-imgui")]
    pub(crate) ui: &'a imgui::Ui,
    #[cfg(feature = "ui-imgui")]
    pub(crate) label: &'a str,
    #[cfg(feature = "ui-imgui")]
    pub(crate) color: &'a mut [f32; 3],
    #[cfg(feature = "ui-imgui")]
    pub(crate) inputs: bool,
    #[cfg(feature = "ui-imgui")]
    pub(crate) show_label: bool,
    #[cfg(feature = "ui-imgui")]
    pub(crate) tooltip: bool,
}

#[cfg(feature = "ui-imgui")]
impl<'a> ColorEdit3Builder<'a> {
    pub fn inputs(mut self, show: bool) -> Self {
        self.inputs = show;
        self
    }

    pub fn label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn tooltip(mut self, show: bool) -> Self {
        self.tooltip = show;
        self
    }

    pub fn build(self) -> bool {
        let mut builder = self.ui.color_edit3_config(self.label, self.color);

        if !self.inputs {
            builder = builder.inputs(false);
        }
        if !self.show_label {
            builder = builder.label(false);
        }
        if !self.tooltip {
            builder = builder.tooltip(false);
        }

        builder.build()
    }
}
#[cfg(feature = "ui-imgui")]
pub struct ColorEdit4Builder<'a> {
    pub(crate) ui: &'a imgui::Ui,
    pub(crate) label: &'a str,
    pub(crate) color: &'a mut [f32; 4],
}

#[cfg(feature = "ui-imgui")]
impl<'a> ColorEdit4Builder<'a> {
    pub fn build(self) -> bool {
        self.ui.color_edit4(self.label, self.color)
    }
}

#[cfg(feature = "ui-imgui")]
pub struct ChildWindowBuilder<'a> {
    pub(crate) ui: &'a imgui::Ui,
    pub(crate) id: &'a str,
    pub(crate) size: [f32; 2],
    pub(crate) border: bool,
}

#[cfg(feature = "ui-imgui")]
impl<'a> ChildWindowBuilder<'a> {
    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.size = size;
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn build<F: FnOnce()>(self, f: F) {
        let builder = self
            .ui
            .child_window(self.id)
            .size(self.size)
            .border(self.border);
        let _ = builder.build(f);
    }

    pub fn begin(self) -> Option<ChildWindowToken<'a>> {
        self.ui
            .child_window(self.id)
            .size(self.size)
            .border(self.border)
            .begin()
            .map(|token| ChildWindowToken { token })
    }
}

#[cfg(feature = "ui-imgui")]
pub struct TreeNodeBuilder<'a> {
    pub(crate) ui: &'a imgui::Ui,
    pub(crate) label: &'a str,
    pub(crate) flags: TreeNodeFlags,
}

#[cfg(feature = "ui-imgui")]
impl<'a> TreeNodeBuilder<'a> {
    pub fn default_open(mut self, open: bool) -> Self {
        self.flags.default_open = open;
        self
    }

    pub fn flags(mut self, flags: TreeNodeFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build<F: FnOnce()>(self, f: F) {
        let mut imgui_flags = imgui::TreeNodeFlags::empty();

        if self.flags.default_open {
            imgui_flags |= imgui::TreeNodeFlags::DEFAULT_OPEN;
        }
        if self.flags.open_on_arrow {
            imgui_flags |= imgui::TreeNodeFlags::OPEN_ON_ARROW;
        }
        if self.flags.open_on_double_click {
            imgui_flags |= imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK;
        }
        if self.flags.leaf {
            imgui_flags |= imgui::TreeNodeFlags::LEAF;
        }
        if self.flags.bullet {
            imgui_flags |= imgui::TreeNodeFlags::BULLET;
        }

        if let Some(_token) = self
            .ui
            .tree_node_config(self.label)
            .flags(imgui_flags)
            .push()
        {
            f();
        }
    }
}

#[cfg(feature = "ui-imgui")]
pub struct SelectableBuilder<'ui, 'label> {
    pub(crate) ui: &'ui imgui::Ui,
    pub(crate) label: &'label str,
    pub(crate) selected: bool,
    pub(crate) disabled: bool,
}

#[cfg(feature = "ui-imgui")]
impl<'ui, 'label> SelectableBuilder<'ui, 'label> {
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn build(self) -> bool {
        let mut builder = self.ui.selectable_config(self.label);

        if self.selected {
            builder = builder.selected(true);
        }
        if self.disabled {
            builder = builder.disabled(true);
        }

        builder.build()
    }
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

// ===== WindowBuilder =====

#[cfg(feature = "ui-imgui")]
pub struct WindowBuilder<'a> {
    pub(crate) ui: &'a imgui::Ui,
    pub(crate) name: &'a str,
    pub(crate) opened: Option<&'a mut bool>,
    pub(crate) position: Option<([f32; 2], WinCondition)>,
    pub(crate) size: Option<([f32; 2], WinCondition)>,
    pub(crate) bg_alpha: Option<f32>,
    pub(crate) flags: Option<imgui::WindowFlags>,
}

#[cfg(feature = "ui-imgui")]
impl<'a> WindowBuilder<'a> {
    pub fn bg_alpha(mut self, alpha: f32) -> Self {
        self.bg_alpha = Some(alpha);
        self
    }

    pub fn flags(mut self, flags: ImguiWindowFlags) -> Self {
        //need to convert hackers window flags to imgui window flags
        // bitflags! {
        //     /// Configuration flags for windows
        //     #[repr(transparent)]
        //     pub struct WindowFlags: u32 {
        //         /// Disable the title bar
        //         const NO_TITLE_BAR = sys::ImGuiWindowFlags_NoTitleBar;
        //         /// Disable resizing with the lower-right grip
        //         const NO_RESIZE = sys::ImGuiWindowFlags_NoResize;
        //         /// Disable moving the window
        //         const NO_MOVE = sys::ImGuiWindowFlags_NoMove;
        //         /// Disable scrollbars (scrolling is still possible with the mouse or programmatically)
        //         const NO_SCROLLBAR = sys::ImGuiWindowFlags_NoScrollbar;
        //         /// Disable vertical scrolling with the mouse wheel.
        //         ///
        //         /// On child window, the mouse wheel will be forwarded to the parent unless `NO_SCROLLBAR`
        //         /// is also set.
        //         const NO_SCROLL_WITH_MOUSE = sys::ImGuiWindowFlags_NoScrollWithMouse;
        //         /// Disable collapsing the window by double-clicking it
        //         const NO_COLLAPSE = sys::ImGuiWindowFlags_NoCollapse;
        //         /// Resize the window to its content on every frame
        //         const ALWAYS_AUTO_RESIZE = sys::ImGuiWindowFlags_AlwaysAutoResize;
        //         /// Disable drawing of background color and outside border
        //         const NO_BACKGROUND = sys::ImGuiWindowFlags_NoBackground;
        //         /// Never load/save settings
        //         const NO_SAVED_SETTINGS = sys::ImGuiWindowFlags_NoSavedSettings;
        //         /// Disable catching mouse input. Hovering test will pass through
        //         const NO_MOUSE_INPUTS = sys::ImGuiWindowFlags_NoMouseInputs;
        //         /// Show a menu bar
        //         const MENU_BAR = sys::ImGuiWindowFlags_MenuBar;
        //         /// Allow horizontal scrollbar to appear
        //         const HORIZONTAL_SCROLLBAR = sys::ImGuiWindowFlags_HorizontalScrollbar;
        //         /// Disable taking focus when transitioning from hidden to visible state
        //         const NO_FOCUS_ON_APPEARING = sys::ImGuiWindowFlags_NoFocusOnAppearing;
        //         /// Disable bringing window to front when taking focus (e.g. clicking it or
        //         /// programmatically giving it focus)
        //         const NO_BRING_TO_FRONT_ON_FOCUS = sys::ImGuiWindowFlags_NoBringToFrontOnFocus;
        //         /// Always show vertical scrollbar
        //         const ALWAYS_VERTICAL_SCROLLBAR = sys::ImGuiWindowFlags_AlwaysVerticalScrollbar;
        //         /// Always show horizontal scrollbar
        //         const ALWAYS_HORIZONTAL_SCROLLBAR = sys::ImGuiWindowFlags_AlwaysHorizontalScrollbar;
        //         /// Ensure child windows without border use `style.window_padding`
        //         const ALWAYS_USE_WINDOW_PADDING = sys::ImGuiWindowFlags_AlwaysUseWindowPadding;
        //         /// Disable gamepad/keyboard navigation within the window
        //         const NO_NAV_INPUTS = sys::ImGuiWindowFlags_NoNavInputs;
        //         /// No focusing toward this window with gamepad/keyboard navigation (e.g. skipped by
        //         /// CTRL+TAB)
        //         const NO_NAV_FOCUS = sys::ImGuiWindowFlags_NoNavFocus;
        //         /// Append '*' to title without affecting the ID, as a convenience
        //         const UNSAVED_DOCUMENT = sys::ImGuiWindowFlags_UnsavedDocument;
        //         /// Disable gamepad/keyboard navigation and focusing.
        //         ///
        //         /// Shorthand for `WindowFlags::NO_NAV_INPUTS | WindowFlags::NO_NAV_FOCUS`.
        //         const NO_NAV = sys::ImGuiWindowFlags_NoNav;
        //         /// Disable all window decorations.
        //         ///
        //         /// Shorthand for `WindowFlags::NO_TITLE_BAR | WindowFlags::NO_RESIZE |
        //         /// WindowFlags::NO_SCROLLBAR | WindowFlags::NO_COLLAPSE`.
        //         const NO_DECORATION = sys::ImGuiWindowFlags_NoDecoration;
        //         /// Don't handle input.
        //         ///
        //         /// Shorthand for `WindowFlags::NO_MOUSE_INPUTS | WindowFlags::NO_NAV_INPUTS |
        //         /// WindowFlags::NO_NAV_FOCUS`.
        //         const NO_INPUTS = sys::ImGuiWindowFlags_NoInputs;

        //         #[cfg(feature="docking")]
        //         const NO_DOCKING = sys::ImGuiWindowFlags_NoDocking;
        //     }
        // }
        //need to convert hackers window flags to imgui window flags through abstractions
        let mut imgui_flags = imgui::WindowFlags::empty();
        //map hackers window flags to imgui window flags to respect type safety

        if flags.0 & crate::gui::ImguiWindowFlags::NO_TITLE_BAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_TITLE_BAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_RESIZE.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_RESIZE;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_MOVE.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_MOVE;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_SCROLLBAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_SCROLLBAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_SCROLL_WITH_MOUSE.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_SCROLL_WITH_MOUSE;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_COLLAPSE.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_COLLAPSE;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::ALWAYS_AUTO_RESIZE.0 != 0 {
            imgui_flags |= imgui::WindowFlags::ALWAYS_AUTO_RESIZE;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_BACKGROUND.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_BACKGROUND;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_SAVED_SETTINGS.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_SAVED_SETTINGS;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_MOUSE_INPUTS.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_MOUSE_INPUTS;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::MENU_BAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::MENU_BAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::HORIZONTAL_SCROLLBAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::HORIZONTAL_SCROLLBAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_FOCUS_ON_APPEARING.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_FOCUS_ON_APPEARING;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_BRING_TO_FRONT_ON_FOCUS.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::ALWAYS_VERTICAL_SCROLLBAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::ALWAYS_VERTICAL_SCROLLBAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::ALWAYS_HORIZONTAL_SCROLLBAR.0 != 0 {
            imgui_flags |= imgui::WindowFlags::ALWAYS_HORIZONTAL_SCROLLBAR;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::ALWAYS_USE_WINDOW_PADDING.0 != 0 {
            imgui_flags |= imgui::WindowFlags::ALWAYS_USE_WINDOW_PADDING;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_NAV_INPUTS.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_NAV_INPUTS;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::NO_NAV_FOCUS.0 != 0 {
            imgui_flags |= imgui::WindowFlags::NO_NAV_FOCUS;
        }
        if flags.0 & crate::gui::ImguiWindowFlags::UNSAVED_DOCUMENT.0 != 0 {
            imgui_flags |= imgui::WindowFlags::UNSAVED_DOCUMENT;
        }

        self.flags = Some(imgui_flags);
        self
    }

    pub fn position(mut self, pos: [f32; 2], cond: WinCondition) -> Self {
        self.position = Some((pos, cond));
        self
    }

    pub fn size(mut self, size: [f32; 2], cond: WinCondition) -> Self {
        self.size = Some((size, cond));
        self
    }

    pub fn opened(mut self, opened: &'a mut bool) -> Self {
        self.opened = Some(opened);
        self
    }

    pub fn build<F: FnOnce()>(self, f: F) {
        let mut window = self.ui.window(self.name);

        if let Some(opened) = self.opened {
            window = window.opened(opened);
        }

        if let Some((pos, cond)) = self.position {
            window = window.position(pos, convert_condition(cond));
        }

        if let Some((size, cond)) = self.size {
            window = window.size(size, convert_condition(cond));
        }

        if let Some(alpha) = self.bg_alpha {
            window = window.bg_alpha(alpha);
        }

        if let Some(flags) = self.flags {
            window = window.flags(flags);
        }

        if let Some(_token) = window.begin() {
            f();
        }
    }
}

#[cfg(feature = "ui-imgui")]
pub struct InputTextBuilder<'a> {
    pub(crate) ui: &'a imgui::Ui,
    pub(crate) label: &'a str,
    pub(crate) buffer: &'a mut String,
    pub(crate) enter_returns_true: bool,
    pub(crate) read_only: bool,
}

#[cfg(feature = "ui-imgui")]
impl<'a> InputTextBuilder<'a> {
    pub fn enter_returns_true(mut self, value: bool) -> Self {
        self.enter_returns_true = value;
        self
    }

    pub fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }

    pub fn build(self) -> bool {
        let mut builder = self.ui.input_text(self.label, self.buffer);

        if self.enter_returns_true {
            builder = builder.enter_returns_true(true);
        }
        if self.read_only {
            builder = builder.read_only(true);
        }

        builder.build()
    }
}

// ===== UiBackend Trait =====

pub trait UiBackend {
    fn native_ui(&self) -> &imgui::Ui;
    // ===== Text =====
    fn text(&self, text: &str);
    fn text_colored(&self, color: [f32; 4], text: &str);
    fn text_wrapped(&self, text: &str);
    fn text_disabled(&self, text: &str);
    fn bullet_text(&self, text: &str);

    // ===== Layout =====
    fn same_line(&self);
    fn same_line_with_pos(&self, pos_x: f32);
    fn align_text_to_frame_padding(&self);
    fn separator(&self);
    fn spacing(&self);
    fn indent(&self);
    fn unindent(&self);
    fn new_line(&self);
    fn dummy(&self, size: Vec2);
    fn group(&self, f: &mut dyn FnMut());
    fn child_window<'a>(&'a self, id: &'a str) -> ChildWindowBuilder<'a>;
    fn tree_node_config<'a>(&'a self, label: &'a str) -> TreeNodeBuilder<'a>;

    // ===== Basic Widgets =====
    fn button(&self, label: &str) -> bool;
    fn small_button(&self, label: &str) -> bool;
    fn checkbox(&self, label: &str, value: &mut bool) -> bool;
    fn radio_button_bool(&self, label: &str, active: bool) -> bool;
    // fn radio_button<T>(&self, label: &str, value: &mut T, button_value: T) -> bool;
    fn radio_button_usize(&self, label: &str, value: &mut usize, test_value: usize) -> bool;
    fn list_box(
        &self,
        label: &str,
        current_item: &mut i32,
        items: &[&str],
        height_in_items: i32,
    ) -> bool;
    fn slider(&self, label: &str, min: f32, max: f32, value: &mut f32) -> bool;
    fn color_edit4(&self, label: &str, color: &mut [f32; 4]) -> bool;

    // ===== Disabled State =====
    fn disabled(&self, disabled: bool, f: &mut dyn FnMut());
    fn begin_disabled(&self, disabled: bool) -> DisabledToken<'_>;

    // ===== Input Widgets - Simple versions =====
    fn input_text_simple(&self, label: &str, buffer: &mut String) -> bool;
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

    fn input_scalar(&self, label: &str, value: &mut i32) -> bool;

    // ===== Slider Widgets =====
    fn slider_int(&self, label: &str, min: i32, max: i32, value: &mut i32) -> bool;
    fn slider_float(&self, label: &str, min: f32, max: f32, value: &mut f32) -> bool;

    // ===== Color Edit Widgets =====
    fn color_edit3(&self, label: &str, color: &mut [f32; 3]) -> bool;
    fn color_edit3_config<'a>(
        &'a self,
        label: &'a str,
        color: &'a mut [f32; 3],
    ) -> ColorEdit3Builder<'a>;

    fn selectable_config<'a>(&'a self, label: &'a str) -> SelectableBuilder<'a, 'a>;

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
    fn current_font_size(&self) -> f32;

    // ===== Drawing =====
    fn get_window_draw_list(&self) -> Box<dyn DrawList + '_>;
    fn get_background_draw_list(&self) -> Box<dyn DrawList + '_>;
    fn get_foreground_draw_list(&self) -> Box<dyn DrawList + '_>;

    // ===== Input =====
    fn is_key_pressed(&self, key: Key) -> bool;
    fn is_key_down(&self, key: Key) -> bool;
    fn is_mouse_clicked(&self, button: MouseButton) -> bool;
    fn is_mouse_down(&self, button: MouseButton) -> bool;
    fn get_mouse_pos(&self) -> Vec2;
    fn io(&self) -> IoState;
    fn keys_down(&self) -> &[bool];

    // ===== Misc =====
    fn calc_text_size(&self, text: &str) -> Vec2;
    fn push_style_color(&self, style: StyleColor, color: [f32; 4]) -> StyleToken<'_>;
    fn pop_style_color(&self);
    fn progress_bar(&self, fraction: f32, size: Vec2, overlay: Option<&str>);

    // ===== Fonts =====
    /// Push a font by its index in the font atlas. Returns a RAII token that pops the font when dropped.
    fn push_font_by_index(&self, index: usize) -> Option<FontToken<'_>>;
    /// Get the total number of fonts loaded in the font atlas.
    fn get_fonts_count(&self) -> usize;

    // ===== New Methods =====

    /// Create a window builder for more flexible window creation
    fn window<'a>(&'a self, name: &'a str) -> WindowBuilder<'a>;

    /// Create an input text builder that returns the builder for chaining
    fn input_text<'a>(&'a self, label: &'a str, buffer: &'a mut String) -> InputTextBuilder<'a>;

    /// Display a tooltip with the given text
    fn tooltip_text(&self, text: &str);

    /// Get current scroll position Y
    fn scroll_y(&self) -> f32;

    /// Get maximum scroll position Y
    fn scroll_max_y(&self) -> f32;

    /// Auto-scroll to current position
    fn set_scroll_here_y(&self);

    /// Get current frame count
    fn frame_count(&self) -> u32;

    /// Get clipboard text
    fn clipboard_text(&self) -> Option<String>;

    /// Set clipboard text
    fn set_clipboard_text(&self, text: &str);

    /// Get height of a single text line
    fn text_line_height(&self) -> f32;

    /// Begin table header row
    fn begin_table_header(&self);

    /// Get access to imgui style (unsafe because it returns raw imgui type)
    unsafe fn style(&self) -> &imgui::Style;

    /// Get content region available (width and height)
    fn content_region_avail(&self) -> Vec2;

    // ===== Texture Management =====

    /// Upload RGBA image data as a GPU texture and return a texture ID
    /// The texture must be freed with free_texture when no longer needed
    ///
    /// # Arguments
    /// * `data` - Raw RGBA pixel data (4 bytes per pixel: R, G, B, A)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    /// A texture ID that can be used with DrawList::add_image
    fn upload_texture(&self, data: &[u8], width: u32, height: u32) -> imgui::TextureId;

    /// Free a previously uploaded texture
    ///
    /// # Arguments
    /// * `texture_id` - The texture ID returned from upload_texture
    fn free_texture(&self, texture_id: imgui::TextureId);
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

    // ===== Texture Rendering =====

    /// Draw a textured rectangle using a GPU texture
    ///
    /// # Arguments
    /// * `texture_id` - Texture ID from UiBackend::upload_texture
    /// * `p_min` - Top-left corner in screen coordinates
    /// * `p_max` - Bottom-right corner in screen coordinates
    fn add_image(&mut self, texture_id: imgui::TextureId, p_min: Vec2, p_max: Vec2);

    /// Draw a textured rectangle with optional UV coordinates and tint color
    ///
    /// # Arguments
    /// * `texture_id` - Texture ID from UiBackend::upload_texture  
    /// * `p_min` - Top-left corner in screen coordinates
    /// * `p_max` - Bottom-right corner in screen coordinates
    /// * `uv_min` - Top-left UV coordinate (default: [0.0, 0.0])
    /// * `uv_max` - Bottom-right UV coordinate (default: [1.0, 1.0])
    /// * `col` - Tint color (default: white [1.0, 1.0, 1.0, 1.0])
    fn add_image_quad(
        &mut self,
        texture_id: imgui::TextureId,
        p_min: Vec2,
        p_max: Vec2,
        uv_min: Vec2,
        uv_max: Vec2,
        col: Color,
    );
}
