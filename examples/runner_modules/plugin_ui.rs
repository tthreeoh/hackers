use hackers::hackrs::HaCKS::HaCKS;
use hackers::HaCK;
use std::path::PathBuf;

pub fn render_plugin_manager(ui: &imgui::Ui, hacks: &mut HaCKS, unload_queue: &mut Vec<usize>) {
    ui.window("Plugin Manager")
        .size([400.0, 300.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text(format!("Loaded Plugins: {}", hacks.dynamic_modules.len()));
            ui.separator();

            // List loaded plugins
            if !hacks.dynamic_modules.is_empty() {
                ui.text("Loaded:");
                for (i, module_rc) in hacks.dynamic_modules.iter().enumerate() {
                    let module = module_rc.borrow();
                    let hack: &dyn HaCK = &*module;
                    let name = hack.name();

                    ui.text(format!("{}. {}", i + 1, name));
                    ui.same_line();

                    // Unload button - queue it instead of immediate unload
                    let button_id = format!("Unload##{}", i);
                    if ui.small_button(&button_id) {
                        unload_queue.push(i);
                    }
                }
                ui.separator();
            }

            ui.text("Available in plugins/:");
            let plugins_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins");
            if plugins_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");

                            // Check if already loaded
                            let already_loaded = hacks
                                .loaded_libs
                                .iter()
                                .any(|lib| lib.path == path.to_string_lossy().to_string());

                            if already_loaded {
                                ui.text_disabled(format!("✓ {}", filename));
                            } else {
                                ui.text(format!("○ {}", filename));
                                ui.same_line();
                                let button_id = format!("Load##{}", filename);
                                if ui.small_button(&button_id) {
                                    if let Err(e) = hacks.load_dynamic(&path) {
                                        eprintln!("Failed to load {}: {}", filename, e);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                ui.text_disabled("(plugins folder not found)");
            }
            ui.separator();

            // Load new plugin button
            if ui.button("Reload Plugins Folder") {
                let mut plugins_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                plugins_dir.push("plugins");
                if let Err(e) = hacks.load_plugins_from_folder(&plugins_dir) {
                    eprintln!("Failed to reload plugins: {}", e);
                }
            }
        });
}
