use crate::runner_modules::{host_ui, plugin_ui};
use hackers::gui::Key;
use hackers::hotkey_manager::{Hotkey, HotkeyManager};
use hackers::impl_backends::imgui_backend::ImguiBackend;
use hackers::{HaCKS, UiBackend};
use imgui::Ui;
use std::fmt::Display;
use std::iter::Enumerate;
use std::time::Duration;

pub struct RunnerHost {
    pub hacks: HaCKS,
    pub app_hotkeys: HotkeyManager,
    pub show_these: Show,
    pub unload_queue: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct Show {
    pub system_menu: bool,
    pub log_viewer: bool,
    pub host_window: bool,
    pub plugin_manager: bool,
    pub metadata_editor: bool,
    pub debug_window: bool,
    pub draw_overlays: bool,
    pub windowed_groups: bool,
    pub undocked_menus: bool,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            system_menu: true,
            log_viewer: false,
            host_window: true,
            plugin_manager: false,
            metadata_editor: true,
            debug_window: true,
            draw_overlays: true,
            windowed_groups: true,
            undocked_menus: true,
        }
    }
}

impl Show {
    pub fn iter(&mut self) -> impl Iterator<Item = (&'static str, &mut bool)> {
        [
            ("System Menu", &mut self.system_menu),
            ("Log Viewer", &mut self.log_viewer),
            ("Host Window", &mut self.host_window),
            ("Plugin Manager", &mut self.plugin_manager),
            ("Metadata Editor", &mut self.metadata_editor),
            ("Debug Window", &mut self.debug_window),
            ("Draw Overlays", &mut self.draw_overlays),
            ("Windowed Groups", &mut self.windowed_groups),
            ("Undocked Menus", &mut self.undocked_menus),
        ]
        .into_iter()
    }
}

impl Display for Show {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl RunnerHost {
    pub fn load_hacks() -> HaCKS {
        let mut hacks = HaCKS::new();

        // Load plugins from the plugins folder
        println!("Loading plugins from ./plugins folder...");
        match hacks.load_plugins_from_folder("./plugins") {
            Ok(count) => println!("Loaded {} plugin(s)", count),
            Err(e) => eprintln!("Failed to load plugins: {}", e),
        }
        hacks
    }

    pub fn set_hotkeys() -> HotkeyManager {
        let mut app_hotkeys = HotkeyManager::new();
        app_hotkeys.register(
            "toggle_menu",
            Hotkey::new(Key::GraveAccent).with_ctrl(),
            Duration::from_millis(200),
        );
        app_hotkeys.register(
            "exit",
            Hotkey::new(Key::F4).with_ctrl(),
            Duration::from_millis(200),
        );
        app_hotkeys
    }

    pub fn new() -> Self {
        Self {
            hacks: Self::load_hacks(),
            app_hotkeys: Self::set_hotkeys(),
            show_these: Show::default(),
            unload_queue: Vec::new(),
        }
    }

    pub fn with_internal_modules<F>(mut self, creator: F) -> Self
    where
        F: Fn() -> Vec<std::rc::Rc<std::cell::RefCell<dyn hackers::HaCK>>>,
    {
        self.hacks.reset_all_modules(creator);
        // Re-load plugins after reset because reset clears everything
        // Note: In oxd, reset_all_modules is usually called first for internal mods.
        // But here we might want to keep plugins?
        // Actually reset_all_modules clears EVERYTHING including init_data.
        // So we should re-load plugins if we want them.
        // However, generic usage implies this sets the BASE state.

        println!("Reloading plugins after internal module reset...");
        match self.hacks.load_plugins_from_folder("./plugins") {
            Ok(count) => println!("Reloaded {} plugin(s)", count),
            Err(e) => eprintln!("Failed to reload plugins: {}", e),
        }

        self
    }

    fn show_or_hide(truth: bool) -> String {
        if truth {
            return "Hide".to_string();
        }
        "Show".to_string()
    }

    pub fn render_ui(&mut self, ui: &Ui) {
        // Create backend wrapper once
        let backend = ImguiBackend::new(ui);

        // Handle Hotkeys
        if self.app_hotkeys.is_triggered("toggle_menu", &backend) {
            self.show_these.system_menu = !self.show_these.system_menu;
        }

        // Render Menu Bar
        if self.show_these.system_menu {
            if let Some(_menu_bar) = ui.begin_main_menu_bar() {
                // Host Menu
                if let Some(_token) = ui.begin_menu("Hackers Host") {
                    //functional menu items for this in that
                    for (key, value) in self.show_these.iter() {
                        if ui.menu_item(format!("{} {}", Self::show_or_hide(*value), key)) {
                            *value = !*value;
                        }
                    }

                    if ui.menu_item("Exit") {
                        // Request exit
                    }
                }
                if let Some(_token) = ui.begin_menu("HaCKs") {
                    self.hacks.render_menu(&backend);
                }

                if let Some(_token) = ui.begin_menu("Plugin Manager") {
                    plugin_ui::render_plugin_manager_inner(
                        &ui,
                        &mut self.hacks,
                        &mut self.unload_queue,
                    );
                }
                if let Some(_token) = ui.begin_menu("About") {
                    host_ui::about(&ui, &mut self.hacks);
                }
            }
        }

        // Render Host Windows
        if self.show_these.host_window {
            host_ui::render_host_window(ui, &mut self.hacks);
        }
        if self.show_these.plugin_manager {
            plugin_ui::render_plugin_manager(ui, &mut self.hacks, &mut self.unload_queue);
        }

        // Render Module Windows
        if self.show_these.windowed_groups {
            self.hacks.render_window(&backend);
        }
        if self.show_these.undocked_menus {
            self.hacks.render_windowed_groups(&backend);
        }

        // Render Draw Overlays
        if self.show_these.draw_overlays {
            let mut fg = backend.get_foreground_draw_list();
            let mut bg = backend.get_background_draw_list();
            self.hacks.render_draw(&backend, fg.as_mut(), bg.as_mut());
        }

        // Render Metadata Editors
        if self.show_these.metadata_editor {
            self.hacks.metadata_window = true.into();
            self.hacks.render_metadata_editor_windows(&backend); // This is where the metadata editor window is rendered
        }

        if self.show_these.debug_window {
            self.hacks.render_debug_window(&backend);
        }

        // Update
        self.hacks.update();
        self.hacks.process_events();

        // Process Unload Queue
        self.process_unload_queue();
    }

    pub fn process_unload_queue(&mut self) {
        if !self.unload_queue.is_empty() {
            for name in self.unload_queue.drain(..) {
                println!("Unloading plugin '{}'", name);
                if let Err(e) = self.hacks.unload_dynamic(&name) {
                    eprintln!("Failed to unload plugin: {}", e);
                }
            }
        }
    }
}
