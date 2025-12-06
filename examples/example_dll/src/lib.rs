use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::*,
    sabi_types::RRef,
    std_types::{RBox, ROption, RStr},
};
use hackers::hackrs::stable_abi::{
    HackersModule, HackersModule_Ref, StableHaCK, StableHaCK_TO, StableUiBackend_TO,
    StableWindowOptions,
};

#[derive(Clone, Debug)]
struct ExampleHaCK {
    counter: i32,
}

impl StableHaCK for ExampleHaCK {
    fn name(&self) -> RStr<'_> {
        "Example DLL".into()
    }

    fn update(&mut self) {
        // Update logic
    }

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>) {
        if ui.begin_menu("Example DLL".into()) {
            if ui.menu_item("Reset Counter".into()) {
                self.counter = 0;
            }
            ui.end_menu();
        }
    }

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, RRef<'_, ()>>) {
        let opts = StableWindowOptions {
            opened: ROption::RNone,
            size: ROption::RNone,
            position: ROption::RNone,
            menu_bar: false,
            always_auto_resize: true,
            resizable: true,
        };

        if ui.begin_window("Example DLL Window".into(), &opts) {
            let text = format!("Counter: {}", self.counter);
            ui.text(RStr::from_str(&text));
            if ui.button("Increment".into()) {
                self.counter += 1;
            }
            ui.same_line();
            if ui.button("Decrement".into()) {
                self.counter -= 1;
            }
        }
        ui.end_window();
    }
}

/// The callback to create the module instance.
#[no_mangle]
pub extern "C" fn create_hack() -> StableHaCK_TO<'static, RBox<()>> {
    StableHaCK_TO::from_value(ExampleHaCK { counter: 0 }, TD_Opaque)
}

/// Export the root module.
#[export_root_module]
pub fn get_library() -> HackersModule_Ref {
    HackersModule { create_hack }.leak_into_prefix()
}
