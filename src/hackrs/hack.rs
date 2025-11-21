use std::any::{Any, TypeId};
use serde::Deserialize;
use erased_serde::Serialize as ErasedSerialize;
use imgui::{Ui,DrawListMut};
use crate::{HaCKS, HaCMetadata, gui::hotkey_manager::HotkeyManager, metadata::HotkeyBinding};

#[allow(unused)]
pub trait HaCK: ErasedSerialize + Send + 'static {
    fn name(&self) -> &str;
    fn nac_type_id(&self) -> TypeId { TypeId::of::<Self>() }
    fn update(&mut self, hacs: &HaCKS) {}
    fn render_draw(&mut self,ui: &Ui,
        // fonts: Option<Fonts>,
        draw_fg: &mut DrawListMut,
        draw_bg: &mut DrawListMut
    ) {}
    fn render_menu(&mut self, ui: &Ui) {}
    fn render_window(&mut self, ui: &Ui) {}
    fn is_menu_enabled(&self) -> bool;
    fn is_window_enabled(&self) -> bool;
    fn is_render_enabled(&self) -> bool;
    fn is_update_enabled(&self) -> bool;
    fn window_weight(&self) -> f32;
    fn update_weight(&self) -> f32;
    fn draw_weight(&self) -> f32;
    fn menu_weight(&self) -> f32;
    fn metadata(&self) -> &HaCMetadata;
    fn metadata_mut(&mut self) -> &mut HaCMetadata;
    fn set_update_weight(&mut self, weight: f32);
    fn set_window_weight(&mut self, weight: f32);
    fn set_show_menu(&mut self, truth: bool) -> bool;
    fn set_show_window(&mut self, truth: bool) -> bool;
    fn set_menu_weight(&mut self, weight: f32);
    fn set_draw_weight(&mut self, weight: f32);
    fn set_render_enabled(&mut self, enabled: bool);
    fn set_update_enabled(&mut self, enabled: bool);
    fn render_draw_path(&self) -> Vec<&str> {vec![]}
    fn menu_path(&self) -> Vec<&str> { vec![self.name()] }
    fn on_unload(&mut self) { }
    fn on_load(&mut self) { }
    fn before_render(&mut self,ui: &Ui) {}
    fn init(&mut self) {}
    fn hotkey_bindings(&self) -> &[HotkeyBinding] {
        &self.metadata().hotkeys
    }
    fn on_hotkey(&mut self, hotkey_id: &str) {}
    fn post_load_init(&mut self) {}
    fn exit(&mut self) {}
    fn blocking_mode_with_kb(&mut self,if_bool:bool,else_bool:bool){}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_dependencies(&self) -> Vec<TypeId> { vec![] }
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
    fn to_json_data_only(&self) -> Result<serde_json::Value, serde_json::Error> {
        let mut json = self.to_json()?;
        if let Some(obj) = json.as_object_mut() {
            obj.remove("hac_data");
        }
        Ok(json)
    }

    
}

#[allow(unused)]
pub trait ModuleSettings: HaCK + Sized + for<'de> Deserialize<'de> {
    fn settings_key() -> &'static str;
}

use erased_serde::serialize_trait_object;
serialize_trait_object!(HaCK);
