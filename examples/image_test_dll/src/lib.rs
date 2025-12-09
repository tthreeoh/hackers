use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TD_Opaque,
    sabi_types::RRef,
    std_types::{RBox, RStr, RString},
};
use hackers::hackrs::stable_abi::{
    HackersModule, HackersModule_Ref, StableHaCK, StableHaCK_TO, StableHaCMetadata,
    StableUiBackend_TO,
};
use hackers::metadata::HaCKLoadType;
use hackers::sprites::animation_utils::{calculate_frame_index, AnimationConfig};
use hackers::sprites::{
    discover_sprite_folders, DiscoveredFolder, ImageCategory, ImageLoader, ImageLoadingState,
    LoadingConfig, UnitAnimationMode,
};
use image::DynamicImage;
use std::collections::HashMap;
use std::time::Instant;

// --- Helper Structs ---
#[derive(Clone, Debug)]
pub struct AnimationSettings {
    pub fps: f32,
    pub speed: f32,
}

// We'll reimplement or mock the parts of image_test we need
// For now, let's just make it compilable and show the UI

pub struct ImageTestHaCK {
    metadata: StableHaCMetadata,
    enabled: bool,
    show_grid: bool,
    image_scale: i32,
    animation_speed: f32,
    // paused field is already present at line 30, so we just add the new ones nearby
    paused: bool,

    // New config fields
    use_config_settings: bool,
    temp_fps: Option<f32>,
    temp_speed: Option<f32>,
    override_fps: f32,

    current_frame: i32,
    show_animation_modes: bool,
    current_animation_mode: u8,

    // Path configuration
    search_paths: Vec<String>,
    discovered_folders: Vec<DiscoveredFolder>,
    selected_folder_index: i32,
    selected_image_index: i32,

    // State
    is_loading: bool,
    new_path_input: RString,
    loading_state: Option<ImageLoadingState>,
    image_list: Option<HashMap<String, DynamicImage>>,
    animation_start: Option<Instant>,
    /// Maps image keys to uploaded GPU texture IDs
    texture_cache: HashMap<String, hackers::hackrs::stable_abi::StableTextureId>,

    // UI State
    show_search_paths: bool,
    show_discovered_folders: bool,
    show_images: bool,
    show_config: bool,

    // Center pos
    center_position: bool,
    x_offset: f32,
    y_offset: f32,
}

impl ImageTestHaCK {
    pub fn new() -> Self {
        ImageTestHaCK {
            metadata: StableHaCMetadata {
                name: RString::from("Image/Sprite Test"),
                description: RString::from("Port of image_test.rs for sprite system testing"),
                category: RString::from("Debug"),
                menu_weight: 1.0,
                window_weight: 1.0,
                draw_weight: 1.0,
                update_weight: 1.0,
                visible_in_gui: true,
                is_menu_enabled: true,
                is_window_enabled: true,
                is_render_enabled: true,
                is_update_enabled: true,
                window_pos: [50.0, 50.0],
                window_size: [600.0, 700.0],
                auto_resize_window: false,
                load_type: HaCKLoadType::Plugin,
            },
            enabled: true,
            show_grid: false,
            image_scale: 50,
            animation_speed: 1.0,
            paused: false,
            use_config_settings: true,
            temp_fps: None,
            temp_speed: None,
            override_fps: 15.0,
            current_frame: 0,
            show_animation_modes: false,
            current_animation_mode: 1, // Stand
            search_paths: vec![
                "assets/images/monsters".to_string(),
                "assets/images/ui".to_string(),
                "assets/images".to_string(),
                "data/global/monsters".to_string(), // Common D2 path
            ],
            discovered_folders: Vec::new(),
            selected_folder_index: -1,
            selected_image_index: 0,
            is_loading: false,
            new_path_input: RString::new(),
            loading_state: None,
            image_list: None,
            animation_start: None,
            texture_cache: HashMap::new(),
            center_position: true,
            x_offset: 0.0,
            y_offset: 0.0,

            show_search_paths: false,
            show_discovered_folders: true,
            show_images: false,
            show_config: true,
        }
    }

    fn scan_and_load(&mut self) {
        // Discover all folders
        self.discovered_folders = discover_sprite_folders(&self.search_paths);

        // Create loading configuration
        let config = LoadingConfig {
            max_frame_time_ms: 16, // ~60 FPS budget
            batch_size: 2,         // Images to load per frame (reduced for visible progress)
        };

        // Initialize progressive loading
        self.loading_state = Some(ImageLoadingState::new(
            self.discovered_folders.clone(),
            self.search_paths.clone(),
            config,
        ));

        // Start with embedded images immediately
        let embedded = ImageLoader::load_category(ImageCategory::Monsters);
        self.image_list = Some(embedded);

        self.is_loading = true;

        // Reset animation timer
        self.animation_start = Some(Instant::now());
        self.current_frame = 0;

        // Auto-select first if available
        if !self.discovered_folders.is_empty() && self.selected_folder_index == -1 {
            self.selected_folder_index = 0;
        }
    }

    fn update_loading(&mut self) {
        if !self.is_loading {
            return;
        }

        let Some(loading_state) = &mut self.loading_state else {
            self.is_loading = false;
            return;
        };

        println!("[ImageTest.update_loading] is_loading={}", self.is_loading);
        // Load a chunk of images this frame
        let has_more = loading_state.load_chunk(None);

        // Update our image list with newly loaded images
        if let Some(existing_images) = &mut self.image_list {
            let loaded_images = loading_state.get_images();
            for (key, img) in loaded_images {
                if !existing_images.contains_key(key) {
                    existing_images.insert(key.clone(), img.clone());
                }
            }
        }

        if !has_more {
            println!("[ImageTest.update_loading] Loading complete");

            // Loading complete!
            if let Some(loading_state) = self.loading_state.take() {
                self.image_list = Some(loading_state.into_images());
            }
            self.is_loading = false;
        }
    }

    fn get_loading_progress(&self) -> Option<(f32, String)> {
        if let Some(loading_state) = &self.loading_state {
            let progress = loading_state.progress();
            let status = loading_state.status_message();
            Some((progress, status))
        } else {
            None
        }
    }

    fn get_current_folder(&self) -> Option<&DiscoveredFolder> {
        if self.selected_folder_index >= 0
            && (self.selected_folder_index as usize) < self.discovered_folders.len()
        {
            Some(&self.discovered_folders[self.selected_folder_index as usize])
        } else {
            None
        }
    }
    // Helper for simulated headers
    fn get_current_settings(&self) -> AnimationSettings {
        if !self.use_config_settings {
            // Use manual overrides
            return AnimationSettings {
                fps: self.temp_fps.unwrap_or(self.override_fps),
                speed: self.temp_speed.unwrap_or(1.0),
            };
        }

        // Use config settings
        let folder = match self.get_current_folder() {
            Some(f) => f,
            None => {
                return AnimationSettings {
                    fps: self.override_fps,
                    speed: 1.0,
                }
            }
        };

        // Check for mode-specific overrides
        if self.show_animation_modes {
            if let Some(mode) = UnitAnimationMode::from_u8(self.current_animation_mode) {
                let mode_name = format!("{:?}", mode).to_lowercase();

                if let Some(overrides) = &folder.mode_overrides {
                    let mode_config = match mode_name.as_str() {
                        "walk" => &overrides.walk,
                        "run" => &overrides.run,
                        "attack1" => &overrides.attack1,
                        "attack2" => &overrides.attack2,
                        "cast" => &overrides.cast,
                        "death" => &overrides.death,
                        "stand" => &overrides.stand,
                        _ => &None,
                    };

                    if let Some(config) = mode_config {
                        return AnimationSettings {
                            fps: config.fps.or(folder.fps).unwrap_or(self.override_fps),
                            speed: config.speed.or(folder.speed).unwrap_or(1.0),
                        };
                    }
                }
            }
        }

        // Use folder-level config
        AnimationSettings {
            fps: folder.fps.unwrap_or(self.override_fps),
            speed: folder.speed.unwrap_or(1.0),
        }
    }

    fn get_effective_fps(&self) -> f32 {
        self.get_current_settings().fps
    }

    fn reset_to_config(&mut self) {
        self.temp_fps = None;
        self.temp_speed = None;
        self.use_config_settings = true;
    }

    fn draw_header(ui: &StableUiBackend_TO<'_, RRef<'_, ()>>, label: &str, state: &mut bool) {
        let icon = if *state { "▼" } else { "▶" };
        let button_label = format!("{} {}", icon, label);
        if ui.button(RStr::from_str(&button_label)) {
            *state = !*state;
        }
    }

    // Helper for indentation
    fn draw_indent(ui: &StableUiBackend_TO<'_, RRef<'_, ()>>) {
        ui.dummy(20.0, 0.0);
        ui.same_line();
    }
}

impl StableHaCK for ImageTestHaCK {
    fn name(&self) -> RStr<'_> {
        self.metadata.name.as_rstr()
    }

    fn update(&mut self) {
        self.update_loading();
    }

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        if ui.begin_menu(RStr::from_str("Image Test")) {
            ui.checkbox(RStr::from_str("Enabled"), &mut self.enabled);
            if ui.menu_item(RStr::from_str("Scan Folders")) {
                self.scan_and_load();
            }
            ui.end_menu();
        }
        ui.text(RStr::from_str("Image Test Plugin"));
        ui.separator();

        ui.checkbox(RStr::from_str("Enabled"), &mut self.enabled);
        ui.separator();

        // --- Search Paths Section ---
        Self::draw_header(ui, "Search Paths", &mut self.show_search_paths);
        if self.show_search_paths {
            Self::draw_indent(ui);
            ui.text(RStr::from_str("configured paths:"));
            for path in &self.search_paths {
                Self::draw_indent(ui); // double indent
                ui.text(RStr::from_str(&format!("- {}", path)));
            }

            Self::draw_indent(ui);
            ui.text(RStr::from_str("Add new path:"));
            Self::draw_indent(ui);
            ui.input_text(RStr::from_str("##new_path"), &mut self.new_path_input);
            ui.same_line();
            if ui.button(RStr::from_str("Add")) {
                let path_str = self.new_path_input.to_string();
                if !path_str.is_empty() {
                    self.search_paths.push(path_str);
                    self.new_path_input = RString::from("");
                    // Trigger rescan
                    self.scan_and_load();
                }
            }
        }

        ui.separator();

        // --- Discovered Folders Section ---
        Self::draw_header(ui, "Discovered Folders", &mut self.show_discovered_folders);
        if self.show_discovered_folders {
            if self.discovered_folders.is_empty() {
                Self::draw_indent(ui);
                ui.text(RStr::from_str("No folders found. Check search paths."));
            } else {
                for (idx, folder) in self.discovered_folders.iter().enumerate() {
                    Self::draw_indent(ui);

                    // Mark selected item visually (simple prefix for now)
                    let prefix = if idx == self.selected_folder_index as usize {
                        "> "
                    } else {
                        "  "
                    };
                    let label = format!("{}{}", prefix, folder.name);

                    if ui.button(RStr::from_str(&label)) {
                        self.selected_folder_index = idx as i32;
                    }
                }
            }
        }

        ui.separator();

        // --- Selected Folder Details ---
        // --- Selected Folder Details ---
        let current_folder_idx = self.selected_folder_index;
        if current_folder_idx >= 0 && (current_folder_idx as usize) < self.discovered_folders.len()
        {
            let folder = &self.discovered_folders[current_folder_idx as usize];

            let total_images = folder
                .files
                .as_ref()
                .map_or(folder.frame_count.unwrap_or(0), |f| f.len());

            ui.text(RStr::from_str(&format!(
                "Selected: {} ({} images)",
                folder.name, total_images
            )));

            // Previous/Next buttons for images
            if ui.button(RStr::from_str("Prev Image")) {
                self.selected_image_index -= 1;
                if self.selected_image_index < 0 {
                    self.selected_image_index = (total_images as i32) - 1;
                }
            }
            ui.same_line();
            if ui.button(RStr::from_str("Next Image")) {
                if total_images > 0 {
                    self.selected_image_index =
                        (self.selected_image_index + 1) % (total_images as i32);
                }
            }

            ui.text(RStr::from_str(&format!(
                "Image Index: {}",
                self.selected_image_index
            )));

            // Animation Controls for selected folder
            if let Some(am) = &folder.animation_mode {
                ui.text(RStr::from_str(format!("Mode: {:?}", am).as_str()));
            }

            ui.separator();
            ui.text(RStr::from_str("Animation Controls"));
            Self::draw_indent(ui);

            // Paused
            ui.checkbox(RStr::from_str("Paused"), &mut self.paused);

            // Config vs Override
            ui.checkbox(
                RStr::from_str("Use Config Settings"),
                &mut self.use_config_settings,
            );
            ui.same_line();
            if ui.button(RStr::from_str("Reset")) {
                self.reset_to_config();
            }

            if self.use_config_settings {
                let settings = self.get_current_settings();
                ui.text_colored(
                    [0.5, 1.0, 0.5, 1.0],
                    RStr::from_str(&format!("FPS: {:.1} (Config)", settings.fps)),
                );
                ui.text_colored(
                    [0.5, 1.0, 0.5, 1.0],
                    RStr::from_str(&format!("Speed: {:.1}x (Config)", settings.speed)),
                );
            } else {
                ui.text_colored([1.0, 1.0, 0.5, 1.0], RStr::from_str("Using overrides"));

                // FPS Override
                let current_fps = self.temp_fps.unwrap_or(self.override_fps);
                let mut temp_fps = current_fps;
                if ui.slider_float(RStr::from_str("FPS Override"), 1.0, 60.0, &mut temp_fps) {
                    self.temp_fps = Some(temp_fps);
                }

                // Speed Override
                let current_speed = self.temp_speed.unwrap_or(1.0);
                let mut temp_speed = current_speed;
                if ui.slider_float(
                    RStr::from_str("Speed Multiplier"),
                    0.1,
                    5.0,
                    &mut temp_speed,
                ) {
                    self.temp_speed = Some(temp_speed);
                }
            }

            if ui.button(RStr::from_str("Test Anim Modes")) {
                self.show_animation_modes = !self.show_animation_modes;
            }

            if self.show_animation_modes {
                // Simple mode cycler
                let mode_name = format!(
                    "Current Mode: {:?}",
                    UnitAnimationMode::from_u8(self.current_animation_mode)
                        .unwrap_or(UnitAnimationMode::Stand)
                );
                ui.text(RStr::from_str(&mode_name));
                if ui.button(RStr::from_str("Next Mode")) {
                    self.current_animation_mode = (self.current_animation_mode + 1) % 16;
                }
            }
        } else {
            ui.text(RStr::from_str("No folder selected"));
        }

        ui.separator();

        // --- Config Section ---
        Self::draw_header(ui, "Configuration", &mut self.show_config);
        if self.show_config {
            Self::draw_indent(ui);
            ui.checkbox(RStr::from_str("Center Position"), &mut self.center_position);
            if !self.center_position {
                Self::draw_indent(ui);
                ui.slider_float(
                    RStr::from_str("X Offset"),
                    -500.0,
                    500.0,
                    &mut self.x_offset,
                );
                Self::draw_indent(ui);
                ui.slider_float(
                    RStr::from_str("Y Offset"),
                    -500.0,
                    500.0,
                    &mut self.y_offset,
                );
            }

            Self::draw_indent(ui);
            // Scale slider
            let mut scale_float = self.image_scale as f32;
            if ui.slider_float(
                RStr::from_str("Image Scale %"),
                1.0,
                200.0,
                &mut scale_float,
            ) {
                self.image_scale = scale_float as i32;
            }
        }
    }

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        self.render_menu(ui);
    }

    fn render_draw(
        &mut self,
        ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>,
        _draw_fg: &mut hackers::hackrs::stable_abi::StableDrawList_TO<
            '_,
            abi_stable::std_types::RBox<()>,
        >,
        draw_bg: &mut hackers::hackrs::stable_abi::StableDrawList_TO<
            '_,
            abi_stable::std_types::RBox<()>,
        >,
    ) {
        if !self.enabled {
            return;
        }

        // Upload textures for any newly loaded images
        if let Some(images) = &self.image_list {
            for (key, img) in images.iter() {
                if !self.texture_cache.contains_key(key) {
                    // Convert image to RGBA8
                    let rgba = img.to_rgba8();
                    let width = rgba.width();
                    let height = rgba.height();
                    let data = rgba.as_raw();

                    // Upload texture and store ID
                    use abi_stable::std_types::RSlice;
                    let texture_id = ui.upload_texture(RSlice::from_slice(data), width, height);
                    self.texture_cache.insert(key.clone(), texture_id);
                }
            }
        }

        // Get images and current folder
        let Some(images) = &self.image_list else {
            return;
        };

        if images.is_empty() || self.discovered_folders.is_empty() {
            return;
        }

        let Some(current_folder) = self.get_current_folder() else {
            return;
        };

        // Calculate center position
        let display_size = ui.get_display_size();
        let screen_width = display_size[0];
        let screen_height = display_size[1];

        let center_x = if self.center_position {
            screen_width / 2.0
        } else {
            screen_width / 2.0 + self.x_offset
        };
        let center_y = if self.center_position {
            screen_height / 2.0
        } else {
            screen_height / 2.0 + self.y_offset
        };

        let scale = self.image_scale as f32 / 100.0;

        // Get frames for current folder
        let folder_name = &current_folder.name;
        let mut frames: Vec<_> = images
            .iter()
            .filter(|(k, _)| {
                k.contains(&format!("/{}/", folder_name))
                    || k.starts_with(&format!("{}/", folder_name))
            })
            .collect();

        // Sort by frame number
        frames.sort_by_key(|(k, _)| {
            k.split('/')
                .last()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(usize::MAX)
        });

        if frames.is_empty() {
            return;
        }

        // Calculate current frame
        let elapsed = if let Some(start) = self.animation_start {
            if self.paused {
                // If paused, we effectively hold time.
                // For simplicity in this stateless view, we just use 0.0 or rely on current_frame
                0.0
            } else {
                let duration = Instant::now().duration_since(start);
                duration.as_secs_f32()
            }
        } else {
            0.0
        };

        let settings = self.get_current_settings();
        let frame_index = if self.paused {
            // Manual control or paused state
            self.current_frame as usize % frames.len()
        } else {
            // Automatic animation
            let config = AnimationConfig::new(
                folder_name.clone(),
                folder_name.clone(),
                frames.len(),
                settings.fps,
            );

            if self.show_animation_modes {
                if let Some(mode) = UnitAnimationMode::from_u8(self.current_animation_mode) {
                    calculate_frame_index(&mode, &config, elapsed * settings.speed)
                } else {
                    config.get_frame_index(elapsed * settings.speed)
                }
            } else {
                config.get_frame_index(elapsed * settings.speed)
            }
        };

        // Update stored frame for UI reflection
        self.current_frame = frame_index as i32;

        // Ensure safe index
        let safe_index = frame_index % frames.len();
        let (_name, img) = frames[safe_index];

        // Get the uploaded texture ID
        let Some(texture_id) = self.texture_cache.get(_name) else {
            return; // Texture not uploaded yet
        };

        // Draw the image
        // Draw the image

        let img_width = img.width() as f32 * scale;
        let img_height = img.height() as f32 * scale;

        let x1 = center_x - (img_width / 2.0);
        let y1 = center_y - (img_height / 2.0);
        let x2 = x1 + img_width;
        let y2 = y1 + img_height;

        draw_bg.add_image(texture_id.clone(), [x1, y1].into(), [x2, y2].into());

        // Draw frame info overlay
        let info = format!("Frame {}/{} ({})", safe_index + 1, frames.len(), _name);
        // Simple text shadow/outline for visibility
        draw_bg.add_text(
            [x1 + 1.0, y1 - 24.0].into(),
            [0.0, 0.0, 0.0, 1.0],
            RStr::from_str(&info),
        );
        draw_bg.add_text(
            [x1, y1 - 25.0].into(),
            [1.0, 1.0, 1.0, 1.0],
            RStr::from_str(&info),
        );
    }

    fn on_load(&mut self) {
        println!("Image Test Plugin loaded!");
        self.scan_and_load();
    }

    fn on_unload(&mut self) {
        println!("Image Test Plugin unloaded!");
    }

    fn metadata(&self) -> &StableHaCMetadata {
        &self.metadata
    }
}

/// The callback to create the module instance.
pub extern "C" fn create_hack() -> StableHaCK_TO<'static, RBox<()>> {
    StableHaCK_TO::from_value(ImageTestHaCK::new(), TD_Opaque)
}

/// Export the root module.
#[export_root_module]
pub fn get_library() -> HackersModule_Ref {
    HackersModule { create_hack }.leak_into_prefix()
}
