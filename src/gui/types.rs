use std::ops::{Add, Div, Index, Mul, Sub};

/// Generic color representation (RGBA, 0.0-1.0 range)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

/// 2D vector/position
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(arr: [f32; 2]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
        }
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Index<usize> for Vec2 {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of bounds: {}", index),
        }
    }
}

/// Window condition (when to apply position/size)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinCondition {
    Always,
    Once,
    FirstUseEver,
    Appearing,
}

/// Tree node condition (when to apply open state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeCondition {
    Always,
    Once,
    FirstUseEver,
    Appearing,
}

// Generic condition alias if needed, but prefer specific ones
pub enum Condition {
    Always,
    Once,
    FirstUseEver,
    Appearing,
}

/// Tree node flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNodeFlags {
    pub default_open: bool,
    pub open_on_arrow: bool,
    pub open_on_double_click: bool,
    pub leaf: bool,
    pub bullet: bool,
}

impl Default for TreeNodeFlags {
    fn default() -> Self {
        Self {
            default_open: false,
            open_on_arrow: false,
            open_on_double_click: true,
            leaf: false,
            bullet: false,
        }
    }
}

impl TreeNodeFlags {
    pub const EMPTY: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: false,
        leaf: false,
        bullet: false,
    };
    pub const DEFAULT_OPEN: Self = Self {
        default_open: true,
        open_on_arrow: false,
        open_on_double_click: false,
        leaf: false,
        bullet: false,
    };
    pub const DEFAULT_OPEN_ON_ARROW: Self = Self {
        default_open: false,
        open_on_arrow: true,
        open_on_double_click: false,
        leaf: false,
        bullet: false,
    };
    pub const DEFAULT_OPEN_ON_DOUBLE_CLICK: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: true,
        leaf: false,
        bullet: false,
    };
    pub const DEFAULT_OPEN_ON_BULLET: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: false,
        leaf: false,
        bullet: true,
    };
    pub const DEFAULT_OPEN_ON_LEAF: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: false,
        leaf: true,
        bullet: false,
    };
    pub const DEFAULT_OPEN_ON_LEAF_AND_BULLET: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: false,
        leaf: true,
        bullet: true,
    };
    pub const DEFAULT_OPEN_ON_LEAF_AND_BULLET_AND_ARROW: Self = Self {
        default_open: false,
        open_on_arrow: true,
        open_on_double_click: false,
        leaf: true,
        bullet: true,
    };
    pub const DEFAULT_OPEN_ON_LEAF_AND_BULLET_AND_DOUBLE_CLICK: Self = Self {
        default_open: false,
        open_on_arrow: false,
        open_on_double_click: true,
        leaf: true,
        bullet: true,
    };
    pub const DEFAULT_OPEN_ON_LEAF_AND_BULLET_AND_ARROW_AND_DOUBLE_CLICK: Self = Self {
        default_open: false,
        open_on_arrow: true,
        open_on_double_click: true,
        leaf: true,
        bullet: true,
    };
    pub const DEFAULT_OPEN_ON_LEAF_AND_BULLET_AND_ARROW_AND_DOUBLE_CLICK_AND_OPEN_ON_ARROW: Self =
        Self {
            default_open: true,
            open_on_arrow: true,
            open_on_double_click: true,
            leaf: true,
            bullet: true,
        };
}

impl std::ops::BitOr for TreeNodeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            default_open: self.default_open || rhs.default_open,
            open_on_arrow: self.open_on_arrow || rhs.open_on_arrow,
            open_on_double_click: self.open_on_double_click || rhs.open_on_double_click,
            leaf: self.leaf || rhs.leaf,
            bullet: self.bullet || rhs.bullet,
        }
    }
}

/// Table flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableFlags {
    pub borders: bool,
    pub row_bg: bool,
    pub resizable: bool,
    pub sizing_fixed_fit: bool,
}

impl TableFlags {
    pub const BORDERS: Self = Self {
        borders: true,
        row_bg: false,
        resizable: false,
        sizing_fixed_fit: false,
    };

    pub const ROW_BG: Self = Self {
        borders: false,
        row_bg: true,
        resizable: false,
        sizing_fixed_fit: false,
    };

    pub const RESIZABLE: Self = Self {
        borders: false,
        row_bg: false,
        resizable: true,
        sizing_fixed_fit: false,
    };

    pub const SIZING_FIXED_FIT: Self = Self {
        borders: false,
        row_bg: false,
        resizable: false,
        sizing_fixed_fit: true,
    };

    pub const NO_SAVED_SETTINGS: Self = Self {
        borders: false,
        row_bg: false,
        resizable: false,
        sizing_fixed_fit: false,
    };
}

impl Default for TableFlags {
    fn default() -> Self {
        Self {
            borders: false,
            row_bg: false,
            resizable: false,
            sizing_fixed_fit: false,
        }
    }
}

impl std::ops::BitOr for TableFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            borders: self.borders || rhs.borders,
            row_bg: self.row_bg || rhs.row_bg,
            resizable: self.resizable || rhs.resizable,
            sizing_fixed_fit: self.sizing_fixed_fit || rhs.sizing_fixed_fit,
        }
    }
}

/// Input key representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Numbers
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Keypad
    Keypad0,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8,
    Keypad9,
    // Special keys
    Space,
    Tab,
    Enter,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Escape,
    GraveAccent,
    // Punctuation
    Comma,
    Period,
    Slash,
    Semicolon,
    Apostrophe,
    LeftBracket,
    RightBracket,
    Backslash,
    Minus,
    Equal,
    // Modifiers
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    LeftSuper,
    RightSuper,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Extra1,
    Extra2,
}

/// IO State
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IoState {
    pub key_shift: bool,
    pub key_ctrl: bool,
    pub key_alt: bool,
    pub mouse_pos: Vec2,
    pub display_size: Vec2,
}

/// Style colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleColor {
    Text,
    TextDisabled,
    WindowBg,
    ChildBg,
    PopupBg,
    Border,
    BorderShadow,
    FrameBg,
    FrameBgHovered,
    FrameBgActive,
    TitleBg,
    TitleBgActive,
    TitleBgCollapsed,
    MenuBarBg,
    ScrollbarBg,
    ScrollbarGrab,
    ScrollbarGrabHovered,
    ScrollbarGrabActive,
    CheckMark,
    SliderGrab,
    SliderGrabActive,
    Button,
    ButtonHovered,
    ButtonActive,
    Header,
    HeaderHovered,
    HeaderActive,
    Separator,
    SeparatorHovered,
    SeparatorActive,
    ResizeGrip,
    ResizeGripHovered,
    ResizeGripActive,
    Tab,
    TabHovered,
    TabActive,
    TabUnfocused,
    TabUnfocusedActive,
    PlotLines,
    PlotLinesHovered,
    PlotHistogram,
    PlotHistogramHovered,
    TableHeaderBg,
    TableBorderStrong,
    TableBorderLight,
    TableRowBg,
    TableRowBgAlt,
    TextSelectedBg,
    DragDropTarget,
    NavHighlight,
    NavWindowingHighlight,
    NavWindowingDimBg,
    ModalWindowDimBg,
}
// ===== Flags =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImguiWindowFlags(pub u32);

impl ImguiWindowFlags {
    pub const NONE: Self = Self(0);
    pub const NO_TITLE_BAR: Self = Self(1 << 0);
    pub const NO_RESIZE: Self = Self(1 << 1);
    pub const NO_MOVE: Self = Self(1 << 2);
    pub const NO_SCROLLBAR: Self = Self(1 << 3);
    pub const NO_SCROLL_WITH_MOUSE: Self = Self(1 << 4);
    pub const NO_COLLAPSE: Self = Self(1 << 5);
    pub const ALWAYS_AUTO_RESIZE: Self = Self(1 << 6);
    pub const NO_BACKGROUND: Self = Self(1 << 7);
    pub const NO_SAVED_SETTINGS: Self = Self(1 << 8);
    pub const NO_MOUSE_INPUTS: Self = Self(1 << 9);
    pub const MENU_BAR: Self = Self(1 << 10);
    pub const HORIZONTAL_SCROLLBAR: Self = Self(1 << 11);
    pub const NO_FOCUS_ON_APPEARING: Self = Self(1 << 12);
    pub const NO_BRING_TO_FRONT_ON_FOCUS: Self = Self(1 << 13);
    pub const ALWAYS_VERTICAL_SCROLLBAR: Self = Self(1 << 14);
    pub const ALWAYS_HORIZONTAL_SCROLLBAR: Self = Self(1 << 15);
    pub const ALWAYS_USE_WINDOW_PADDING: Self = Self(1 << 16);
    pub const NO_NAV_INPUTS: Self = Self(1 << 18);
    pub const NO_NAV_FOCUS: Self = Self(1 << 19);
    pub const UNSAVED_DOCUMENT: Self = Self(1 << 20);
    pub const NO_DECORATION: Self = Self(1 << 0 | 1 << 1 | 1 << 3 | 1 << 5);
    pub const NO_INPUTS: Self = Self(1 << 9 | 1 << 18 | 1 << 19);
    pub const NAV_FLATTENED: Self = Self(1 << 23);
    pub const CHILD_WINDOW: Self = Self(1 << 24);
    pub const TOOLTIP: Self = Self(1 << 25);
    pub const POPUP: Self = Self(1 << 26);
    pub const MODAL: Self = Self(1 << 27);
    pub const CHILD_MENU: Self = Self(1 << 28);
}

impl std::ops::BitOr for ImguiWindowFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableColumnFlags(pub u32);

impl TableColumnFlags {
    pub const NONE: Self = Self(0);
    pub const DEFAULT_HIDE: Self = Self(1 << 0);
    pub const DEFAULT_SORT: Self = Self(1 << 1);
    pub const WIDTH_STRETCH: Self = Self(1 << 2);
    pub const WIDTH_FIXED: Self = Self(1 << 3);
    pub const NO_RESIZE: Self = Self(1 << 4);
    pub const NO_REORDER: Self = Self(1 << 5);
    pub const NO_HIDE: Self = Self(1 << 6);
    pub const NO_CLIP: Self = Self(1 << 7);
    pub const NO_SORT: Self = Self(1 << 8);
    pub const NO_SORT_ASCENDING: Self = Self(1 << 9);
    pub const NO_SORT_DESCENDING: Self = Self(1 << 10);
    pub const NO_HEADER_LABEL: Self = Self(1 << 11);
    pub const NO_HEADER_WIDTH: Self = Self(1 << 12);
    pub const PREFER_SORT_ASCENDING: Self = Self(1 << 13);
    pub const PREFER_SORT_DESCENDING: Self = Self(1 << 14);
    pub const INDENT_ENABLE: Self = Self(1 << 15);
    pub const INDENT_DISABLE: Self = Self(1 << 16);
    pub const IS_ENABLED: Self = Self(1 << 24);
    pub const IS_VISIBLE: Self = Self(1 << 25);
    pub const IS_SORTED: Self = Self(1 << 26);
    pub const IS_HOVERED: Self = Self(1 << 27);
}

impl std::ops::BitOr for TableColumnFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

pub struct TableColumnSetup {
    pub name: String,
    pub flags: TableColumnFlags,
    pub init_width_or_weight: f32,
    pub user_id: u32,
}

impl TableColumnSetup {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            flags: TableColumnFlags::NONE,
            init_width_or_weight: 0.0,
            user_id: 0,
        }
    }

    pub fn flags(mut self, flags: TableColumnFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn init_width_or_weight(mut self, width: f32) -> Self {
        self.init_width_or_weight = width;
        self
    }

    pub fn user_id(mut self, id: u32) -> Self {
        self.user_id = id;
        self
    }
}
