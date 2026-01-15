use crate::access::{AccessControl, AccessLevel};
use crate::gui::UiBackend;
use crate::gui::{DrawList, Vec2};
use crate::hackrs::HaCMetadata;
use crate::metadata::HaCKLoadType;
use crate::{Color, HaCK};
use abi_stable::library::RootModule;
use abi_stable::sabi_types::{RRef, VersionStrings};
use abi_stable::{
    declare_root_module_statics, package_version_strings, sabi_trait,
    std_types::{RBox, ROption, RStr, RString},
    StableAbi,
};
use erased_serde::Serialize as ErasedSerialize;
use lazy_static::lazy_static;
use serde::Serialize;
use serde::Serializer;
use std::any::TypeId;
use std::borrow::Cow;

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct StableWinCondition {
    pub pos: [f32; 2],
    pub cond: u32,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct StableWindowOptions {
    pub opened: ROption<bool>,
    pub size: ROption<StableWinCondition>,
    pub position: ROption<StableWinCondition>,
    pub menu_bar: bool,
    pub always_auto_resize: bool,
    pub resizable: bool,
}

/// Opaque texture handle for ABI-safe texture management
#[repr(C)]
#[derive(StableAbi, Clone, Copy)]
pub struct StableTextureId {
    // Just store the texture ID as a usize
    id: usize,
}

impl StableTextureId {
    pub fn new(id: imgui::TextureId) -> Self {
        Self { id: id.id() }
    }

    pub fn get(&self) -> imgui::TextureId {
        imgui::TextureId::new(self.id)
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

/// Stable version of HaCMetadata for ABI-safe transfer across DLL boundaries.
#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct StableHaCMetadata {
    pub name: RString,
    pub description: RString,
    pub category: RString,
    pub menu_weight: f32,
    pub window_weight: f32,
    pub draw_weight: f32,
    pub update_weight: f32,
    pub visible_in_gui: bool,
    pub is_menu_enabled: bool,
    pub is_window_enabled: bool,
    pub is_render_enabled: bool,
    pub is_update_enabled: bool,
    pub window_pos: [f32; 2],
    pub window_size: [f32; 2],
    pub auto_resize_window: bool,
    pub load_type: HaCKLoadType,
}

impl StableHaCMetadata {
    /// Convert from native HaCMetadata to StableHaCMetadata
    pub fn from_metadata(meta: &HaCMetadata) -> Self {
        Self {
            name: meta.name.to_string().into(),
            description: meta.description.to_string().into(),
            category: meta.category.to_string().into(),
            menu_weight: meta.menu_weight,
            window_weight: meta.window_weight,
            draw_weight: meta.draw_weight,
            update_weight: meta.update_weight,
            visible_in_gui: meta.visible_in_gui,
            is_menu_enabled: meta.is_menu_enabled,
            is_window_enabled: meta.is_window_enabled,
            is_render_enabled: meta.is_render_enabled,
            is_update_enabled: meta.is_update_enabled,
            window_pos: meta.window_pos,
            window_size: meta.window_size,
            auto_resize_window: meta.auto_resize_window,
            load_type: meta.load_type.clone(),
        }
    }

    /// Convert to native HaCMetadata
    pub fn to_metadata(&self) -> HaCMetadata {
        HaCMetadata {
            name: Cow::Owned(self.name.to_string()),
            description: Cow::Owned(self.description.to_string()),
            category: Cow::Owned(self.category.to_string()),
            hotkeys: Vec::new(), // Hotkeys not transferred via ABI for now
            menu_weight: self.menu_weight,
            window_weight: self.window_weight,
            draw_weight: self.draw_weight,
            update_weight: self.update_weight,
            visible_in_gui: self.visible_in_gui,
            is_menu_enabled: self.is_menu_enabled,
            is_window_enabled: self.is_window_enabled,
            is_render_enabled: self.is_render_enabled,
            is_update_enabled: self.is_update_enabled,
            auto_resize_window: self.auto_resize_window,
            window_pos: self.window_pos,
            window_size: self.window_size,
            access_control: AccessControl::new(AccessLevel::ReadWrite),
            load_type: self.load_type.clone(),
            undocked_from_menu: false,
        }
    }
}

/// Note: Removed Send/Sync bounds as UiBackend is not Sync.
#[sabi_trait]
pub trait StableUiBackend {
    // Basic Text
    fn text(&self, text: RStr<'_>);
    fn text_colored(&self, color: [f32; 4], text: RStr<'_>);

    // Layout
    fn separator(&self);
    fn same_line(&self);
    fn dummy(&self, width: f32, height: f32);
    fn get_display_size(&self) -> [f32; 2];

    // Basic Widgets
    fn button(&self, label: RStr<'_>) -> bool;
    fn checkbox(&self, label: RStr<'_>, value: &mut bool) -> bool;
    fn slider_float(&self, label: RStr<'_>, min: f32, max: f32, value: &mut f32) -> bool;
    fn color_edit3(&self, label: RStr<'_>, color: &mut [f32; 3]) -> bool;
    fn input_text(&self, label: RStr<'_>, value: &mut RString) -> bool;

    // Windows
    fn begin_window(&self, title: RStr<'_>, options: &StableWindowOptions) -> bool;
    fn end_window(&self);

    fn begin_menu_bar(&self) -> bool;
    fn end_menu_bar(&self);

    fn begin_menu(&self, label: RStr<'_>) -> bool;
    fn end_menu(&self);
    fn menu_item(&self, label: RStr<'_>) -> bool;

    // Expanded UI
    fn collapsing_header(&self, label: RStr<'_>) -> bool;
    fn begin_combo(&self, label: RStr<'_>, preview: RStr<'_>) -> bool;
    fn end_combo(&self);
    fn selectable(&self, label: RStr<'_>, selected: bool) -> bool;
    fn set_item_default_focus(&self);

    // Drawing
    fn get_window_draw_list(&self) -> StableDrawList_TO<'_, RBox<()>>;

    // Texture Management
    fn upload_texture(
        &self,
        data: abi_stable::std_types::RSlice<'_, u8>,
        width: u32,
        height: u32,
    ) -> StableTextureId;

    // Input
    fn is_key_down(&self, key: crate::gui::Key) -> bool;
}

#[sabi_trait]
pub trait StableDrawList {
    fn add_rect(&mut self, p1: [f32; 2], p2: [f32; 2], color: [f32; 4], filled: bool);
    fn add_text(&mut self, pos: [f32; 2], color: [f32; 4], text: RStr<'_>);
    fn add_line(&mut self, p1: [f32; 2], p2: [f32; 2], color: [f32; 4], thickness: f32);
    fn add_circle(&mut self, center: [f32; 2], radius: f32, color: [f32; 4], filled: bool);
    fn push_clip_rect(&mut self, min: [f32; 2], max: [f32; 2], intersect_with_current: bool);
    fn pop_clip_rect(&mut self);

    // Texture Rendering
    fn add_image(&mut self, texture_id: StableTextureId, p_min: [f32; 2], p_max: [f32; 2]);
}

#[sabi_trait]
pub trait StableHaCK: Send + Sync {
    fn name(&self) -> RStr<'_>;

    fn update(&mut self);

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>);
    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>);
    fn render_draw(
        &mut self,
        ui: &StableUiBackend_TO<'_, RRef<'_, ()>>,
        draw_fg: &mut StableDrawList_TO<'_, RBox<()>>,
        draw_bg: &mut StableDrawList_TO<'_, RBox<()>>,
    );
    fn metadata(&self) -> &StableHaCMetadata;

    // Persistence
    fn save_settings(&self) -> RString {
        RString::new()
    }
    fn load_settings(&mut self, _settings: RString) {}

    fn on_load(&mut self) {}
    fn on_unload(&mut self) {}
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct HackersModule {
    #[sabi(last_prefix_field)]
    pub create_hack: extern "C" fn() -> StableHaCK_TO<'static, RBox<()>>,
}

/// Implement RootModule for the generated Ref type.
impl RootModule for HackersModule_Ref {
    declare_root_module_statics! {HackersModule_Ref}

    const BASE_NAME: &'static str = "hackers_module";
    const NAME: &'static str = "hackers_module";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

///////////////////////////////////////////////////////////////////////////////
// IMPLS
///////////////////////////////////////////////////////////////////////////////

use crate::gui::{ComboBoxToken, MenuBarToken, MenuToken};
use std::cell::RefCell;

/// Wraps the host's `&dyn UiBackend` and exposes it as `StableUiBackend`.
pub struct StableUiBackendWrapper<'a> {
    pub backend: &'a dyn UiBackend,
    pub menu_tokens: RefCell<Vec<MenuToken<'a>>>,
    pub menu_bar_tokens: RefCell<Vec<MenuBarToken<'a>>>,
    pub combo_tokens: RefCell<Vec<ComboBoxToken<'a>>>,
}

impl<'a> StableUiBackend for StableUiBackendWrapper<'a> {
    fn text(&self, text: RStr<'_>) {
        self.backend.text(text.as_str());
    }

    fn text_colored(&self, color: [f32; 4], text: RStr<'_>) {
        self.backend.text_colored(color, text.as_str());
    }

    // ... (keep existing methods unchanged until begin_menu)

    fn separator(&self) {
        self.backend.separator();
    }

    fn same_line(&self) {
        self.backend.same_line();
    }

    fn dummy(&self, width: f32, height: f32) {
        self.backend.dummy(crate::gui::Vec2::new(width, height));
    }

    fn get_display_size(&self) -> [f32; 2] {
        self.backend.io().display_size.into()
    }

    fn button(&self, label: RStr<'_>) -> bool {
        self.backend.button(label.as_str())
    }

    fn checkbox(&self, label: RStr<'_>, value: &mut bool) -> bool {
        self.backend.checkbox(label.as_str(), value)
    }

    fn slider_float(&self, label: RStr<'_>, min: f32, max: f32, value: &mut f32) -> bool {
        self.backend.slider_float(label.as_str(), min, max, value)
    }

    fn color_edit3(&self, label: RStr<'_>, color: &mut [f32; 3]) -> bool {
        self.backend.color_edit3(label.as_str(), color)
    }

    fn input_text(&self, label: RStr<'_>, value: &mut RString) -> bool {
        let mut string_val = value.to_string();
        let changed = self
            .backend
            .input_text(label.as_str(), &mut string_val)
            .build();
        if changed {
            *value = RString::from(string_val);
        }
        changed
    }

    fn collapsing_header(&self, label: RStr<'_>) -> bool {
        self.backend
            .collapsing_header(label.as_str(), crate::gui::TreeNodeFlags::default())
    }

    fn begin_combo(&self, label: RStr<'_>, preview: RStr<'_>) -> bool {
        if let Some(token) = self.backend.begin_combo(label.as_str(), preview.as_str()) {
            self.combo_tokens.borrow_mut().push(token);
            true
        } else {
            false
        }
    }

    fn end_combo(&self) {
        let _ = self.combo_tokens.borrow_mut().pop();
    }

    fn selectable(&self, label: RStr<'_>, selected: bool) -> bool {
        self.backend
            .selectable(label.as_str())
            .selected(selected)
            .build()
    }

    fn set_item_default_focus(&self) {
        self.backend.set_item_default_focus();
    }

    fn begin_window(&self, title: RStr<'_>, options: &StableWindowOptions) -> bool {
        let mut opts = crate::gui::WindowOptions::default();

        opts.menu_bar = options.menu_bar;
        opts.always_auto_resize = options.always_auto_resize;
        opts.resizable = options.resizable;

        if let Some(opened) = options.opened.clone().into_option() {
            opts.opened = Some(opened);
        }

        if let Some(size) = options.size.clone().into_option() {
            let cond = match size.cond {
                1 => crate::gui::WinCondition::Once,
                2 => crate::gui::WinCondition::FirstUseEver,
                3 => crate::gui::WinCondition::Appearing,
                _ => crate::gui::WinCondition::Always,
            };
            opts.size = Some((size.pos, cond));
        }

        if let Some(pos) = options.position.clone().into_option() {
            let cond = match pos.cond {
                1 => crate::gui::WinCondition::Once,
                2 => crate::gui::WinCondition::FirstUseEver,
                3 => crate::gui::WinCondition::Appearing,
                _ => crate::gui::WinCondition::Always,
            };
            opts.position = Some((pos.pos, cond));
        }

        self.backend
            .begin_window_with_options(title.as_str(), &opts)
            .is_some()
    }

    fn end_window(&self) {
        // No-op
    }

    fn begin_menu_bar(&self) -> bool {
        if let Some(token) = self.backend.begin_menu_bar() {
            self.menu_bar_tokens.borrow_mut().push(token);
            true
        } else {
            false
        }
    }

    fn end_menu_bar(&self) {
        self.menu_bar_tokens.borrow_mut().pop();
    }

    fn begin_menu(&self, label: RStr<'_>) -> bool {
        if let Some(token) = self.backend.begin_menu(label.as_str()) {
            self.menu_tokens.borrow_mut().push(token);
            true
        } else {
            false
        }
    }

    fn end_menu(&self) {
        self.menu_tokens.borrow_mut().pop();
    }

    fn menu_item(&self, label: RStr<'_>) -> bool {
        self.backend.menu_item(label.as_str())
    }

    fn get_window_draw_list(&self) -> StableDrawList_TO<'_, RBox<()>> {
        let draw_list = self.backend.get_window_draw_list();
        let wrapper = StableDrawListWrapper { inner: draw_list };
        StableDrawList_TO::from_value(wrapper, abi_stable::sabi_trait::TD_Opaque)
    }

    fn upload_texture(
        &self,
        data: abi_stable::std_types::RSlice<'_, u8>,
        width: u32,
        height: u32,
    ) -> StableTextureId {
        let texture_id = self.backend.upload_texture(data.as_slice(), width, height);
        StableTextureId::new(texture_id)
    }

    fn is_key_down(&self, key: crate::gui::Key) -> bool {
        self.backend.is_key_down(key)
    }
}

pub struct StableDrawListWrapper<'a> {
    pub inner: Box<dyn DrawList + 'a>,
}

impl<'a> StableDrawList for StableDrawListWrapper<'a> {
    fn add_rect(&mut self, p1: [f32; 2], p2: [f32; 2], color: [f32; 4], filled: bool) {
        self.inner.add_rect(
            Vec2::from(p1),
            Vec2::from(p2),
            Color::rgba(color[0], color[1], color[2], color[3]),
            filled,
        );
    }

    fn add_text(&mut self, pos: [f32; 2], color: [f32; 4], text: RStr<'_>) {
        self.inner.add_text(
            Vec2::from(pos),
            Color::rgba(color[0], color[1], color[2], color[3]),
            text.as_str(),
        );
    }

    fn add_line(&mut self, p1: [f32; 2], p2: [f32; 2], color: [f32; 4], thickness: f32) {
        self.inner.add_line(
            Vec2::from(p1),
            Vec2::from(p2),
            Color::rgba(color[0], color[1], color[2], color[3]),
            thickness,
        );
    }

    fn add_circle(&mut self, center: [f32; 2], radius: f32, color: [f32; 4], filled: bool) {
        self.inner.add_circle(
            Vec2::from(center),
            radius,
            Color::rgba(color[0], color[1], color[2], color[3]),
            filled,
        );
    }

    fn push_clip_rect(&mut self, min: [f32; 2], max: [f32; 2], intersect_with_current: bool) {
        self.inner
            .push_clip_rect(Vec2::from(min), Vec2::from(max), intersect_with_current);
    }

    fn pop_clip_rect(&mut self) {
        self.inner.pop_clip_rect();
    }

    fn add_image(&mut self, texture_id: StableTextureId, p_min: [f32; 2], p_max: [f32; 2]) {
        let tex_id = texture_id.get();
        self.inner
            .add_image(tex_id, Vec2::from(p_min), Vec2::from(p_max));
    }
}

/// Wraps a DLL-provided `StableHaCK` and adapts it to the host `HaCK` trait.
pub struct ForeignHaCK {
    pub inner: StableHaCK_TO<'static, RBox<()>>,
    pub metadata: HaCMetadata,
}

impl Serialize for ForeignHaCK {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as null or empty struct for now
        serializer.serialize_none()
    }
}

impl ForeignHaCK {
    /// Create a new ForeignHaCK from a StableHaCK instance.
    /// Extracts metadata from the plugin during creation.
    pub fn new(inner: StableHaCK_TO<'static, RBox<()>>) -> Self {
        let metadata = inner.metadata().to_metadata();
        Self { inner, metadata }
    }
}

// Helper to forward DrawList calls from a Box to a mutable reference
struct ForwardingDrawList<'a> {
    inner: &'a mut dyn crate::gui::DrawList,
}

impl<'a> crate::gui::DrawList for ForwardingDrawList<'a> {
    fn add_rect(
        &mut self,
        p1: crate::gui::Vec2,
        p2: crate::gui::Vec2,
        color: crate::gui::Color,
        filled: bool,
    ) {
        self.inner.add_rect(p1, p2, color, filled);
    }

    fn add_text(&mut self, pos: crate::gui::Vec2, color: crate::gui::Color, text: &str) {
        self.inner.add_text(pos, color, text);
    }
    fn add_line(
        &mut self,
        p1: crate::gui::Vec2,
        p2: crate::gui::Vec2,
        color: crate::gui::Color,
        thickness: f32,
    ) {
        self.inner.add_line(p1, p2, color, thickness);
    }
    fn add_circle(
        &mut self,
        center: crate::gui::Vec2,
        radius: f32,
        color: crate::gui::Color,
        filled: bool,
    ) {
        self.inner.add_circle(center, radius, color, filled);
    }
    fn push_clip_rect(
        &mut self,
        min: crate::gui::Vec2,
        max: crate::gui::Vec2,
        intersect_with_current: bool,
    ) {
        self.inner.push_clip_rect(min, max, intersect_with_current);
    }
    fn pop_clip_rect(&mut self) {
        self.inner.pop_clip_rect();
    }

    fn add_image(
        &mut self,
        texture_id: imgui::TextureId,
        p_min: crate::gui::Vec2,
        p_max: crate::gui::Vec2,
    ) {
        self.inner.add_image(texture_id, p_min, p_max);
    }

    fn add_image_quad(
        &mut self,
        texture_id: imgui::TextureId,
        p_min: crate::gui::Vec2,
        p_max: crate::gui::Vec2,
        uv_min: crate::gui::Vec2,
        uv_max: crate::gui::Vec2,
        col: crate::gui::Color,
    ) {
        self.inner
            .add_image_quad(texture_id, p_min, p_max, uv_min, uv_max, col);
    }
}

impl HaCK for ForeignHaCK {
    fn name(&self) -> &str {
        // Cache the name as a String since we can't return a temporary
        // For now, return a static string that represents it's foreign
        // TODO: Store name in ForeignHaCK struct during creation
        self.inner.name().as_str()
    }

    fn init(&mut self) {
        self.inner.on_load();
    }

    fn on_unload(&mut self) {
        self.inner.on_unload();
    }

    fn render_menu(&mut self, ui: &dyn UiBackend) {
        let wrapper = StableUiBackendWrapper {
            backend: ui,
            menu_tokens: RefCell::new(Vec::new()),
            menu_bar_tokens: RefCell::new(Vec::new()),
            combo_tokens: RefCell::new(Vec::new()),
        };
        let wrapper_ref = StableUiBackend_TO::from_ptr(&wrapper, abi_stable::sabi_trait::TD_Opaque);
        self.inner.render_menu(&wrapper_ref);
    }

    fn render_window(&mut self, ui: &dyn UiBackend) {
        let wrapper = StableUiBackendWrapper {
            backend: ui,
            menu_tokens: RefCell::new(Vec::new()),
            menu_bar_tokens: RefCell::new(Vec::new()),
            combo_tokens: RefCell::new(Vec::new()),
        };
        let wrapper_ref = StableUiBackend_TO::from_ptr(&wrapper, abi_stable::sabi_trait::TD_Opaque);
        self.inner.render_window(&wrapper_ref);
    }

    fn render_draw(
        &mut self,
        ui: &dyn UiBackend,
        draw_fg: &mut dyn crate::gui::DrawList,
        draw_bg: &mut dyn crate::gui::DrawList,
    ) {
        let wrapper = StableUiBackendWrapper {
            backend: ui,
            menu_tokens: RefCell::new(Vec::new()),
            menu_bar_tokens: RefCell::new(Vec::new()),
            combo_tokens: RefCell::new(Vec::new()),
        };
        let wrapper_ref = StableUiBackend_TO::from_ptr(&wrapper, abi_stable::sabi_trait::TD_Opaque);

        // Wrap the draw lists
        let draw_fg_wrapper = StableDrawListWrapper {
            inner: Box::new(ForwardingDrawList { inner: draw_fg }),
        };
        let mut draw_fg_ref =
            StableDrawList_TO::from_value(draw_fg_wrapper, abi_stable::sabi_trait::TD_Opaque);

        let draw_bg_wrapper = StableDrawListWrapper {
            inner: Box::new(ForwardingDrawList { inner: draw_bg }),
        };
        let mut draw_bg_ref =
            StableDrawList_TO::from_value(draw_bg_wrapper, abi_stable::sabi_trait::TD_Opaque);

        self.inner
            .render_draw(&wrapper_ref, &mut draw_fg_ref, &mut draw_bg_ref);
    }

    fn update(&mut self, _hacs: &crate::hackrs::HaCKS::HaCKS) {
        self.inner.update();
    }

    fn nac_type_id(&self) -> TypeId {
        TypeId::of::<ForeignHaCK>()
    }
    fn update_weight(&self) -> f32 {
        self.metadata.update_weight
    }
    fn window_weight(&self) -> f32 {
        self.metadata.window_weight
    }
    fn draw_weight(&self) -> f32 {
        self.metadata.draw_weight
    }
    fn menu_weight(&self) -> f32 {
        self.metadata.menu_weight
    }
    fn metadata(&self) -> &HaCMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut HaCMetadata {
        &mut self.metadata
    }
    fn is_menu_enabled(&self) -> bool {
        self.metadata.is_menu_enabled
    }
    fn is_window_enabled(&self) -> bool {
        self.metadata.is_window_enabled
    }
    fn is_render_enabled(&self) -> bool {
        self.metadata.is_render_enabled
    }
    fn is_update_enabled(&self) -> bool {
        self.metadata.is_update_enabled
    }

    fn set_update_weight(&mut self, val: f32) {
        self.metadata.update_weight = val;
    }
    fn set_window_weight(&mut self, val: f32) {
        self.metadata.window_weight = val;
    }
    fn set_show_menu(&mut self, val: bool) -> bool {
        self.metadata.is_menu_enabled = val;
        val
    }
    fn set_show_window(&mut self, val: bool) -> bool {
        self.metadata.is_window_enabled = val;
        val
    }
    fn set_menu_weight(&mut self, val: f32) {
        self.metadata.menu_weight = val;
    }
    fn set_draw_weight(&mut self, val: f32) {
        self.metadata.draw_weight = val;
    }
    fn set_render_enabled(&mut self, val: bool) {
        self.metadata.is_render_enabled = val;
    }
    fn set_update_enabled(&mut self, val: bool) {
        self.metadata.is_update_enabled = val;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let s = self.inner.save_settings();
        if s.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        serde_json::from_str(s.as_str())
    }

    fn apply_settings(&mut self, settings: serde_json::Value) {
        let json_str = serde_json::to_string(&settings).unwrap_or_default();
        use abi_stable::std_types::RString;
        self.inner.load_settings(RString::from(json_str));
    }
}
