use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr},
};
use hackers::hackrs::stable_abi::{
    HackersModule, HackersModule_Ref, StableHaCK, StableHaCK_TO, StableUiBackend_TO,
};

pub struct ExampleHaCK2 {
    counter: i32,
    enabled: bool,
}

impl ExampleHaCK2 {
    pub fn new() -> Self {
        ExampleHaCK2 {
            counter: 0,
            enabled: true,
        }
    }
}

impl StableHaCK for ExampleHaCK2 {
    fn name(&self) -> RStr<'_> {
        RStr::from_str("Example Plugin 2")
    }

    fn update(&mut self) {
        if self.enabled {
            self.counter += 1;
        }
    }

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        // Use a submenu to make it more distinct
        if ui.begin_menu(RStr::from_str("🎨 Plugin 2")) {
            ui.text(RStr::from_str("Advanced Plugin"));
            ui.separator();

            if ui.menu_item(RStr::from_str("Toggle Enabled")) {
                self.enabled = !self.enabled;
            }

            if ui.menu_item(RStr::from_str("Reset Counter")) {
                self.counter = 0;
            }

            ui.end_menu();
        }
    }

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        ui.text(RStr::from_str("🎨 Advanced Plugin 2"));
        ui.separator();

        let status = if self.enabled {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        };
        let status_str = format!("Status: {}", status);
        let status_rstr = RStr::from_str(&status_str);
        ui.text(status_rstr);

        ui.separator();

        let counter_str = format!("🔢 Counter: {}", self.counter);
        let counter_rstr = RStr::from_str(&counter_str);
        ui.text(counter_rstr);

        if ui.button(RStr::from_str("➕ Increment")) {
            if self.enabled {
                self.counter += 1;
            }
        }

        ui.same_line();

        if ui.button(RStr::from_str("🔄 Reset")) {
            self.counter = 0;
        }

        ui.separator();
        ui.text(RStr::from_str("🎨 Theme Color:"));

        let mut color = [0.2f32, 0.8f32, 0.4f32];
        if ui.color_edit3(RStr::from_str("Plugin Theme"), &mut color) {
            // Color changed
        }

        ui.separator();

        if !self.enabled {
            ui.text(RStr::from_str("⚠️ Plugin is disabled"));
        }
    }

    fn on_load(&mut self) {
        println!("🎨 Example Plugin 2 loaded!");
    }

    fn on_unload(&mut self) {
        println!("🎨 Example Plugin 2 unloaded!");
    }
}

/// The callback to create the module instance.
#[no_mangle]
pub extern "C" fn create_hack() -> StableHaCK_TO<'static, RBox<()>> {
    StableHaCK_TO::from_value(ExampleHaCK2::new(), TD_Opaque)
}

/// Export the root module.
#[export_root_module]
pub fn get_library() -> HackersModule_Ref {
    HackersModule { create_hack }.leak_into_prefix()
}
