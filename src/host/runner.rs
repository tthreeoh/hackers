use crate::gui::Key;
use crate::host::{plugin_manager as plugin_ui, ui as host_ui};
use crate::hotkey_manager::{Hotkey, HotkeyManager};
use crate::impl_backends::imgui_backend::ImguiBackend;
use crate::{HaCKS, UiBackend};
use imgui::Ui;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use std::time::Duration;

use crate::HackSettings;

#[derive(HackSettings)]
#[hack_settings(crate_name = "crate")]
pub struct RunnerHost {
    pub hacks: HaCKS,
    pub app_hotkeys: HotkeyManager,
    pub show_these: Show,
    pub unload_queue: Vec<String>,
    pub background_image: Option<imgui::TextureId>,
    #[setting]
    pub clear_color: [f32; 4],
    #[setting]
    pub bg_image_path_input: String,
    pub queued_bg_image: Option<String>,
    #[setting]
    pub bg_mode: BackgroundMode,
    #[setting]
    pub bg_image_size: [u32; 2],
    pub reload_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum BackgroundMode {
    #[default]
    Stretch,
    Fit,
    Center,
}

#[derive(Debug, Clone)]
pub struct Show {
    //Host
    pub system_menu: bool,
    pub log_viewer: bool,
    pub host_window: bool,
    pub plugin_manager: bool,
    pub metadata_editor: bool,
    pub debug_window: bool,
    //HaCKS
    pub draw_overlays: bool,
    pub windowed_groups: bool,
    pub undocked_menus: bool,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            //Host
            system_menu: true,
            log_viewer: false,
            host_window: false,
            plugin_manager: false,
            metadata_editor: false,
            debug_window: false,
            //HaCKS
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
        let mut host = Self {
            hacks: Self::load_hacks(),
            app_hotkeys: Self::set_hotkeys(),
            show_these: Show::default(),
            unload_queue: Vec::new(),
            background_image: None,
            clear_color: [0.1, 0.1, 0.1, 0.5],
            bg_image_path_input: r"assets\backgrounds\GB.png".to_string(), // Default suggestion
            queued_bg_image: None,
            bg_mode: BackgroundMode::Fit,
            bg_image_size: [0, 0],
            reload_requested: false,
        };

        // Attempt to load settings on startup
        host.perform_settings_reload();
        host
    }

    pub fn with_internal_modules<F>(mut self, creator: F) -> Self
    where
        F: Fn() -> Vec<Rc<RefCell<dyn crate::HaCK>>>,
    {
        self.hacks.reset_all_modules(creator);
        println!("Reloading plugins after internal module reset...");
        match self.hacks.load_plugins_from_folder("./plugins") {
            Ok(count) => println!("Reloaded {} plugin(s)", count),
            Err(e) => eprintln!("Failed to reload plugins: {}", e),
        }

        // Re-apply settings to newly loaded modules
        self.perform_settings_reload();

        self
    }

    pub fn save_settings(&self) {
        let extra_settings = self.get_settings();

        let res = self.hacks.save_to_file(
            "runner_settings.json",
            |modules| {
                let mut map = HashMap::new();
                for (key, module) in modules {
                    let module_ref = module.borrow();
                    if let Ok(json) = module_ref.to_json() {
                        map.insert(key.clone(), json);
                    }
                }
                map
            },
            Some(&extra_settings),
        );

        if let Err(e) = res {
            eprintln!("Failed to save settings: {}", e);
        } else {
            println!("Settings saved to runner_settings.json");
        }
    }

    pub fn perform_settings_reload(&mut self) {
        let path = "runner_settings.json";
        match HaCKS::load_settings_map_from_file(path) {
            Ok(settings_map) => {
                // Apply to plugins
                for (_id, module_rc) in &self.hacks.hacs {
                    let mut module = module_rc.borrow_mut();
                    let name = module.name().to_string();

                    if let Some(json_val) = settings_map.get(&name) {
                        module.apply_settings(json_val.clone());
                    }
                }

                // Apply to host
                self.apply_settings(&settings_map);
                println!("Settings applied to running modules and host.");
            }
            Err(e) => {
                eprintln!("Failed to read/parse settings file '{}': {}", path, e);
            }
        }
    }

    pub fn load_settings(&mut self) {
        self.reload_requested = true;
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

        if let Some(texture_id) = self.background_image {
            let bg_draw_list = ui.get_background_draw_list();
            let [display_w, display_h] = ui.io().display_size;

            let (p_min, p_max) = match self.bg_mode {
                BackgroundMode::Stretch => ([0.0, 0.0], [display_w, display_h]),
                BackgroundMode::Fit => {
                    if self.bg_image_size[0] == 0 || self.bg_image_size[1] == 0 {
                        ([0.0, 0.0], [display_w, display_h])
                    } else {
                        let img_w = self.bg_image_size[0] as f32;
                        let img_h = self.bg_image_size[1] as f32;
                        let scale_x = display_w / img_w;
                        let scale_y = display_h / img_h;
                        let scale = scale_x.min(scale_y);

                        let w = img_w * scale;
                        let h = img_h * scale;
                        let x = (display_w - w) * 0.5;
                        let y = (display_h - h) * 0.5;
                        ([x, y], [x + w, y + h])
                    }
                }
                BackgroundMode::Center => {
                    if self.bg_image_size[0] == 0 || self.bg_image_size[1] == 0 {
                        ([0.0, 0.0], [display_w, display_h])
                    } else {
                        let img_w = self.bg_image_size[0] as f32;
                        let img_h = self.bg_image_size[1] as f32;
                        let x = (display_w - img_w) * 0.5;
                        let y = (display_h - img_h) * 0.5;
                        ([x, y], [x + img_w, y + img_h])
                    }
                }
            };

            bg_draw_list.add_image(texture_id, p_min, p_max).build();
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
                    ui.separator();
                    if ui.menu_item("Save Settings") {
                        self.save_settings();
                    }
                    if ui.menu_item("Load Settings") {
                        self.load_settings();
                    }
                    ui.separator();
                    if let Some(_token) = ui.begin_menu("Background") {
                        // Color Picker
                        ui.color_edit4("Clear Color", &mut self.clear_color);

                        // Image Loader
                        ui.input_text("Image Path", &mut self.bg_image_path_input)
                            .build();
                        if ui.button("Load Image") {
                            self.queued_bg_image = Some(self.bg_image_path_input.clone());
                        }
                        ui.same_line();
                        if ui.button("Clear Image") {
                            self.background_image = None;
                        }

                        ui.separator();
                        ui.text("Background Fit Mode:");
                        if ui.radio_button_bool("Stretch", self.bg_mode == BackgroundMode::Stretch)
                        {
                            self.bg_mode = BackgroundMode::Stretch;
                        };
                        ui.same_line();
                        if ui.radio_button_bool("Fit", self.bg_mode == BackgroundMode::Fit) {
                            self.bg_mode = BackgroundMode::Fit;
                        };
                        ui.same_line();
                        if ui.radio_button_bool("Center", self.bg_mode == BackgroundMode::Center) {
                            self.bg_mode = BackgroundMode::Center;
                        };
                    }
                }
                if let Some(_token) = ui.begin_menu("HaCKs") {
                    self.hacks.render_menu(&backend);
                }
                if let Some(_token) = ui.begin_menu("Plugin Manager") {
                    plugin_ui::render_plugin_manager_inner(
                        ui,
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
