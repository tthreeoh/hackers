use crate::access::{AccessControl, AccessLevel};
use crate::gui::UiBackend;
use crate::hackrs::HaCMetadata;
use crate::HaCK;
use abi_stable::library::RootModule;
use abi_stable::sabi_types::{RRef, VersionStrings};
use abi_stable::{
    declare_root_module_statics, package_version_strings, sabi_trait,
    std_types::{RBox, ROption, RStr},
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

/// Stable UI Backend trait.
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

    // Basic Widgets
    fn button(&self, label: RStr<'_>) -> bool;
    fn checkbox(&self, label: RStr<'_>, value: &mut bool) -> bool;
    fn slider_float(&self, label: RStr<'_>, min: f32, max: f32, value: &mut f32) -> bool;
    fn color_edit3(&self, label: RStr<'_>, color: &mut [f32; 3]) -> bool;

    // Windows
    fn begin_window(&self, title: RStr<'_>, options: &StableWindowOptions) -> bool;
    fn end_window(&self);

    fn begin_menu_bar(&self) -> bool;
    fn end_menu_bar(&self);

    fn begin_menu(&self, label: RStr<'_>) -> bool;
    fn end_menu(&self);
    fn menu_item(&self, label: RStr<'_>) -> bool;
}

#[sabi_trait]
pub trait StableHaCK: Send + Sync {
    fn name(&self) -> RStr<'_>;

    fn update(&mut self);

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>);
    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>);

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

/// Wraps the host's `&dyn UiBackend` and exposes it as `StableUiBackend`.
pub struct StableUiBackendWrapper<'a> {
    pub backend: &'a dyn UiBackend,
}

impl<'a> StableUiBackend for StableUiBackendWrapper<'a> {
    fn text(&self, text: RStr<'_>) {
        self.backend.text(text.as_str());
    }

    fn text_colored(&self, color: [f32; 4], text: RStr<'_>) {
        self.backend.text_colored(color, text.as_str());
    }

    fn separator(&self) {
        self.backend.separator();
    }

    fn same_line(&self) {
        self.backend.same_line();
    }

    fn dummy(&self, width: f32, height: f32) {
        self.backend.dummy(crate::gui::Vec2::new(width, height));
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
        self.backend.begin_menu_bar().is_some()
    }
    fn end_menu_bar(&self) {}
    fn begin_menu(&self, label: RStr<'_>) -> bool {
        self.backend.begin_menu(label.as_str()).is_some()
    }
    fn end_menu(&self) {}
    fn menu_item(&self, label: RStr<'_>) -> bool {
        self.backend.menu_item(label.as_str())
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

lazy_static! {
    static ref FOREIGN_META: HaCMetadata = HaCMetadata {
        name: Cow::Borrowed("ForeignModule"),
        description: Cow::Borrowed("External DLL Module"),
        category: Cow::Borrowed("External"),
        hotkeys: vec![],
        menu_weight: 0.0,
        window_weight: 0.0,
        draw_weight: 0.0,
        update_weight: 0.0,
        visible_in_gui: true,
        is_menu_enabled: true,
        is_window_enabled: true,
        is_render_enabled: false,
        is_update_enabled: true,
        auto_resize_window: true,
        window_pos: [0.0, 0.0],
        window_size: [0.0, 0.0],
        access_control: AccessControl::new(AccessLevel::ReadWrite),
    };
}

impl HaCK for ForeignHaCK {
    fn name(&self) -> &str {
        "ForeignModule" // TODO: Cache name from inner
    }

    fn init(&mut self) {
        self.inner.on_load();
    }

    fn on_unload(&mut self) {
        self.inner.on_unload();
    }

    fn render_menu(&mut self, ui: &dyn UiBackend) {
        let wrapper = StableUiBackendWrapper { backend: ui };
        let wrapper_ref = StableUiBackend_TO::from_ptr(&wrapper, abi_stable::sabi_trait::TD_Opaque);
        self.inner.render_menu(&wrapper_ref);
    }

    fn render_window(&mut self, ui: &dyn UiBackend) {
        let wrapper = StableUiBackendWrapper { backend: ui };
        let wrapper_ref = StableUiBackend_TO::from_ptr(&wrapper, abi_stable::sabi_trait::TD_Opaque);
        self.inner.render_window(&wrapper_ref);
    }

    fn update(&mut self, _hacs: &crate::hackrs::HaCKS::HaCKS) {
        self.inner.update();
    }

    fn nac_type_id(&self) -> TypeId {
        TypeId::of::<ForeignHaCK>()
    }
    fn update_weight(&self) -> f32 {
        0.0
    }
    fn window_weight(&self) -> f32 {
        0.0
    }
    fn draw_weight(&self) -> f32 {
        0.0
    }
    fn menu_weight(&self) -> f32 {
        0.0
    }
    fn metadata(&self) -> &HaCMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut HaCMetadata {
        &mut self.metadata
    }
    fn is_menu_enabled(&self) -> bool {
        true
    }
    fn is_window_enabled(&self) -> bool {
        true
    }
    fn is_render_enabled(&self) -> bool {
        false
    }
    fn is_update_enabled(&self) -> bool {
        true
    }

    fn set_update_weight(&mut self, _: f32) {}
    fn set_window_weight(&mut self, _: f32) {}
    fn set_show_menu(&mut self, _: bool) -> bool {
        false
    }
    fn set_show_window(&mut self, _: bool) -> bool {
        false
    }
    fn set_menu_weight(&mut self, _: f32) {}
    fn set_draw_weight(&mut self, _: f32) {}
    fn set_render_enabled(&mut self, _: bool) {}
    fn set_update_enabled(&mut self, _: bool) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        Ok(serde_json::Value::Null)
    }
}
