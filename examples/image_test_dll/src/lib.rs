use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr, RString},
};
use hackers::hackrs::stable_abi::{
    HackersModule, HackersModule_Ref, StableDrawList, StableHaCK, StableHaCK_TO, StableHaCMetadata,
    StableUiBackend_TO,
};
use hackers::metadata::HaCKLoadType;
use hackers::sprites::{
    discover_sprite_folders, AnimationMode, DiscoveredFolder, ImageCategory, ImageLoader,
    ImageLoadingState, LoadingConfig, UnitAnimationMode,
};
use image::{DynamicImage, GenericImageView};
use std::collections::HashMap;
use std::time::Instant;

// We'll reimplement or mock the parts of image_test we need
// For now, let's just make it compilable and show the UI

pub struct ImageTestHaCK {
    metadata: StableHaCMetadata,
    enabled: bool,
    show_grid: bool,
    image_scale: i32,
    animation_speed: f32,
    paused: bool,
    current_frame: i32,
    show_animation_modes: bool,
    current_animation_mode: u8,

    // Path configuration
    search_paths: Vec<String>,
    discovered_folders: Vec<DiscoveredFolder>,
    selected_folder_index: i32,

    // State
    is_loading: bool,
    new_path_input: RString,
    loading_state: Option<ImageLoadingState>,
    image_list: Option<HashMap<String, DynamicImage>>,
    animation_start: Option<Instant>,

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
            is_loading: false,
            new_path_input: RString::new(),
            loading_state: None,
            image_list: None,
            animation_start: None,
            center_position: true,
            x_offset: 0.0,
            y_offset: 0.0,
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
        println!(
            "[ImageTest] update_loading called, is_loading={}",
            self.is_loading
        );

        if !self.is_loading {
            return;
        }

        let Some(loading_state) = &mut self.loading_state else {
            self.is_loading = false;
            return;
        };

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
    }

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        ui.text(RStr::from_str("Image Test Plugin"));
        ui.separator();

        ui.checkbox(RStr::from_str("Enabled"), &mut self.enabled);

        // === LOADING PROGRESS ===
        if self.is_loading {
            if let Some((progress, status)) = self.get_loading_progress() {
                ui.text_colored([1.0, 1.0, 0.0, 1.0], RStr::from_str("⚙ Loading Images..."));
                ui.text_colored([0.7, 0.7, 0.7, 1.0], RStr::from_str(&status));
                ui.text(RStr::from_str(&format!(
                    "Progress: {:.0}%",
                    progress * 100.0
                )));
                ui.separator();
            }
        } else if let Some(images) = &self.image_list {
            ui.text_colored(
                [0.5, 1.0, 0.5, 1.0],
                RStr::from_str(&format!("✓ {} images loaded", images.len())),
            );
            ui.separator();
        }

        if ui.button(RStr::from_str("Scan & Reload All")) {
            self.scan_and_load();
        }

        ui.separator();
        ui.text(RStr::from_str("Add New Path:"));
        ui.input_text(RStr::from_str("##PathInput"), &mut self.new_path_input);
        ui.same_line();
        if ui.button(RStr::from_str("Add")) {
            let path_str = self.new_path_input.to_string();
            if !path_str.is_empty() {
                self.search_paths.push(path_str);
                self.new_path_input = RString::new();
                self.scan_and_load();
            }
        }

        ui.text(RStr::from_str(
            format!("Discovered folders: {}", self.discovered_folders.len()).as_str(),
        ));

        if !self.discovered_folders.is_empty() {
            ui.separator();

            // Simple combo simulation since we don't have full combo support in StableUiBackend yet?
            // Wait, do we have combo/listbox? Checked stable_abi.rs, we do NOT have combo exposed yet.
            // We can use a simple index cycler or similar for now.

            let current_name = if let Some(f) = self.get_current_folder() {
                f.name.clone()
            } else {
                "None".to_string()
            };

            ui.text(RStr::from_str(
                format!("Selected: {}", current_name).as_str(),
            ));

            ui.same_line();
            if ui.button(RStr::from_str("Next")) {
                if !self.discovered_folders.is_empty() {
                    self.selected_folder_index =
                        (self.selected_folder_index + 1) % (self.discovered_folders.len() as i32);
                }
            }

            // Show details
            if let Some(folder) = self.get_current_folder() {
                ui.text(RStr::from_str(
                    format!("Frames: {}", folder.frame_count.unwrap_or(0)).as_str(),
                ));
                ui.text(RStr::from_str(
                    format!("Type: {:?}", folder.source_type).as_str(),
                ));

                if folder.is_modular() {
                    ui.text(RStr::from_str("Modular: Yes"));
                }

                // Animation Mode
                if let Some(am) = &folder.animation_mode {
                    ui.text(RStr::from_str(format!("Mode: {:?}", am).as_str()));
                }

                // Render some animation controls
                ui.separator();
                ui.checkbox(RStr::from_str("Paused"), &mut self.paused);
                ui.slider_float(RStr::from_str("Speed"), 0.1, 5.0, &mut self.animation_speed);

                ui.checkbox(
                    RStr::from_str("Test Anim Modes"),
                    &mut self.show_animation_modes,
                );
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
            }
        }

        ui.separator();
        ui.text(RStr::from_str("Draw Settings"));

        // Scale slider (using float since slider_int isn't in StableUiBackend)
        let mut scale_float = self.image_scale as f32;
        if ui.slider_float(
            RStr::from_str("Image Scale %"),
            1.0,
            200.0,
            &mut scale_float,
        ) {
            self.image_scale = scale_float as i32;
        }

        ui.checkbox(RStr::from_str("Center Position"), &mut self.center_position);
        if !self.center_position {
            ui.slider_float(
                RStr::from_str("X Offset"),
                -500.0,
                500.0,
                &mut self.x_offset,
            );
            ui.slider_float(
                RStr::from_str("Y Offset"),
                -500.0,
                500.0,
                &mut self.y_offset,
            );
        }
    }

    fn render_draw(
        &mut self,
        _ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>,
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
        // TODO: Get actual display size through StableUiBackend when exposed
        // For now, assume standard 1920x1080 display
        let screen_width = 1920.0;
        let screen_height = 1080.0;

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

        // Calculate current frame index
        let frame_index = if self.paused {
            self.current_frame.min(frames.len() as i32 - 1).max(0) as usize
        } else {
            // Time-based animation
            let elapsed = if let Some(start) = self.animation_start {
                start.elapsed().as_secs_f32() * self.animation_speed
            } else {
                0.0
            };

            // Simple FPS-based frame calculation
            let fps = 4.0; // Default FPS
            let total_frames = frames.len();
            let frame_time = elapsed * fps;
            (frame_time as usize) % total_frames
        };

        let (_, img) = frames[frame_index];

        // Downsample large images to prevent vertex overflow
        // ImGui has a 65k vertex limit (16-bit indices)
        // Each rectangle = 4 vertices, so max rectangles = 16,384
        // Worst case: 1 rectangle per pixel, so max = 128x128 pixels
        let max_dimension = 128; // Safe limit even with no strip compression
        let needs_downsample = img.width() > max_dimension || img.height() > max_dimension;

        let display_img;
        let img_to_draw = if needs_downsample {
            use image::imageops::FilterType;
            let scale_factor = (max_dimension as f32 / img.width().max(img.height()) as f32);
            let new_width = (img.width() as f32 * scale_factor) as u32;
            let new_height = (img.height() as f32 * scale_factor) as u32;
            display_img = img.resize(new_width, new_height, FilterType::Nearest);
            &display_img
        } else {
            img
        };

        // Draw the image using horizontal strips to reduce vertex count
        let img_width = img_to_draw.width() as f32;
        let img_height = img_to_draw.height() as f32;

        let x_base = center_x - (img_width * scale / 2.0);
        let y_base = center_y - (img_height * scale / 2.0);

        use image::GenericImageView;

        // Process row by row, merging adjacent pixels of same color into strips
        for row in 0..img_to_draw.height() {
            let mut current_strip_start: Option<(u32, [f32; 4])> = None;

            for col in 0..=img_to_draw.width() {
                let pixel_color = if col < img_to_draw.width() {
                    let pixel = img_to_draw.get_pixel(col, row);
                    if pixel[3] == 0 {
                        None // Transparent
                    } else {
                        Some([
                            pixel[0] as f32 / 255.0,
                            pixel[1] as f32 / 255.0,
                            pixel[2] as f32 / 255.0,
                            pixel[3] as f32 / 255.0,
                        ])
                    }
                } else {
                    None // End of row
                };

                match (&current_strip_start, pixel_color) {
                    (Some((start_col, strip_color)), Some(color)) if color == *strip_color => {
                        // Same color, continue strip
                    }
                    (Some((start_col, strip_color)), _) => {
                        // Different color or end of row, draw the strip
                        let x1 = x_base + (*start_col as f32 * scale);
                        let y1 = y_base + (row as f32 * scale);
                        let x2 = x_base + (col as f32 * scale);
                        let y2 = y1 + scale;

                        draw_bg.add_rect([x1, y1], [x2, y2], *strip_color, true);

                        // Start new strip if this pixel is not transparent
                        current_strip_start = pixel_color.map(|c| (col, c));
                    }
                    (None, Some(color)) => {
                        // Start new strip
                        current_strip_start = Some((col, color));
                    }
                    (None, None) => {
                        // Transparent pixel, no strip
                    }
                }
            }
        }
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
