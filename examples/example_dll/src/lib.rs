use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr, RString},
};
use hackers::hackrs::stable_abi::{
    HackersModule, HackersModule_Ref, StableDrawList_TO, StableHaCK, StableHaCK_TO,
    StableHaCMetadata, StableUiBackend_TO,
};
use hackers::metadata::HaCKLoadType;

pub struct ExampleHaCK {
    metadata: StableHaCMetadata,
    counter: i32,
    enabled: bool,
}

impl ExampleHaCK {
    pub fn new() -> Self {
        ExampleHaCK {
            metadata: StableHaCMetadata {
                name: RString::from("Example DLL"),
                description: RString::from("Basic example plugin with counter functionality"),
                category: RString::from("Examples"),
                menu_weight: 1.0,
                window_weight: 1.0,
                draw_weight: 0.0,
                update_weight: 1.0,
                visible_in_gui: true,
                is_menu_enabled: true,
                is_window_enabled: false,
                is_render_enabled: true,
                is_update_enabled: true,
                window_pos: [50.0, 50.0],
                window_size: [300.0, 200.0],
                auto_resize_window: true,
                load_type: HaCKLoadType::Plugin,
            },
            counter: 0,
            enabled: true,
        }
    }
}

impl StableHaCK for ExampleHaCK {
    fn name(&self) -> RStr<'_> {
        RStr::from_str("Example DLL")
    }

    fn update(&mut self) {
        if self.enabled {
            // Update logic can go here if needed
        }
    }

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        if ui.begin_menu(RStr::from_str("Example DLL")) {
            if ui.menu_item(RStr::from_str("Toggle Enabled")) {
                self.enabled = !self.enabled;
            }

            if ui.menu_item(RStr::from_str("Reset Counter")) {
                self.counter = 0;
            }

            ui.end_menu();
        }

        ui.text(RStr::from_str("Example DLL Plugin"));
        ui.separator();

        let status = if self.enabled { "Enabled" } else { "Disabled" };
        let status_str = format!("Status: {}", status);
        let status_rstr = RStr::from_str(&status_str);
        ui.text(status_rstr);

        ui.separator();

        let counter_str = format!("Counter: {}", self.counter);
        let counter_rstr = RStr::from_str(&counter_str);
        ui.text(counter_rstr);

        if ui.button(RStr::from_str("Increment")) {
            self.counter += 1;
        }

        ui.same_line();

        if ui.button(RStr::from_str("Decrement")) {
            self.counter -= 1;
        }

        ui.separator();

        if !self.enabled {
            ui.text(RStr::from_str("Plugin is disabled"));
        }
    }

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        self.render_menu(ui);
    }

    fn on_load(&mut self) {
        println!("Example DLL on_load!");
    }

    fn on_unload(&mut self) {
        println!("Example DLL on_unload!");
    }

    fn metadata(&self) -> &StableHaCMetadata {
        &self.metadata
    }
    fn render_draw(
        &mut self,
        ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>,
        draw_fg: &mut StableDrawList_TO<'_, RBox<()>>,
        draw_bg: &mut StableDrawList_TO<'_, RBox<()>>,
    ) {
    }
}

/// The callback to create the module instance.
pub extern "C" fn create_hack() -> StableHaCK_TO<'static, RBox<()>> {
    StableHaCK_TO::from_value(ExampleHaCK::new(), TD_Opaque)
}

/// Export the root module.
#[export_root_module]
pub fn get_library() -> HackersModule_Ref {
    HackersModule { create_hack }.leak_into_prefix()
}
