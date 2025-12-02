use std::ops::{Add, Div, Mul, Sub};

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
