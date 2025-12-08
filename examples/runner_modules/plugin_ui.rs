use hackers::{
    hackrs::HaCKS::HaCKS,
    metadata::{self, HaCKLoadType},
};
use std::path::PathBuf;

pub fn render_plugin_manager_inner(
    ui: &imgui::Ui,
    hacks: &mut HaCKS,
    unload_queue: &mut Vec<String>,
) {
    // List loaded plugins
    let mut internal_count = 0;
    ui.text(format!("Loaded HaCKS: {}", hacks.hacs.len()));
    ui.separator();
    if !hacks.hacs.is_empty() {
        for (i, (name, hack)) in hacks.hacs.iter().enumerate() {
            let load_type = hack.borrow().metadata().load_type.clone();
            let name_format = format!("{}. {}", i + 1, name);
            if load_type != hackers::metadata::HaCKLoadType::Plugin {
                internal_count += 1;
                ui.text_disabled(&name);
            } else {
                ui.text(&name);
                ui.same_line();
                // Unload button - queue it instead of immediate unload
                let button_id = format!("Unload##{}", name);
                if ui.small_button(&button_id) {
                    unload_queue.push(name.clone());
                }
            }
        }
    }
    ui.separator();

    ui.text(format!(
        "Loaded Plugins: {}",
        hacks.hacs.len() - internal_count
    ));

    ui.separator();

    ui.text("Available in plugins/:");
    ui.separator();
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

                    // Check if already loaded by comparing file names against loaded_libs registry
                    // This matches the actual loaded file on disk rather than fuzzy matching the module name
                    let current_file_name = path.file_name();
                    let already_loaded = hacks.loaded_libs.borrow().values().any(|lib| {
                        std::path::Path::new(&lib.path).file_name() == current_file_name
                    });

                    if already_loaded {
                        ui.text_disabled(format!("{}", filename));
                    } else {
                        ui.text(format!("{}", filename));
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
}

pub fn render_plugin_manager(ui: &imgui::Ui, hacks: &mut HaCKS, unload_queue: &mut Vec<String>) {
    ui.window("Plugin Manager")
        .size([400.0, 300.0], imgui::Condition::FirstUseEver)
        .build(|| {
            render_plugin_manager_inner(ui, hacks, unload_queue);
        });
}
