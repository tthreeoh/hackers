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

pub mod map;
pub mod player;
pub mod player_types;
pub mod projectile;

use crate::map::{LevelMap, LevelTile, TileType};
use crate::player::{ControlMode, Direction, Player, PlayerState, SpriteBody};
use crate::player_types::JumpStyle;
use crate::projectile::{Projectile, ProjectileType};

#[derive(Clone, Debug)]
pub struct PreviewState {
    pub selected_folder_index: i32,
    pub animation_mode: u8, // UnitAnimationMode
    pub direction: u8,      // Direction enum u8
    pub zoom: f32,
    pub pan: [f32; 2],
    pub is_playing: bool,
    pub current_frame_override: Option<usize>, // If paused/scrubbing
    pub manual_progress: f32,
    pub background_color: [f32; 4],
    pub show_grid: bool,
    pub show_collision_box: bool,
    pub speed_override: f32,
    pub fps_override: f32,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            selected_folder_index: -1,
            animation_mode: 1, // Stand
            direction: 0,      // Down
            zoom: 1.0,
            pan: [0.0, 0.0],
            is_playing: true,
            current_frame_override: None,
            manual_progress: 0.0,
            background_color: [0.2, 0.2, 0.2, 1.0],
            show_grid: true,
            show_collision_box: false,
            speed_override: 1.0,
            fps_override: 15.0,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ImageTestConfig {
    gravity: f32,
    jump_force: f32,
    friction: f32,
    max_run_speed: f32,
    control_mode: ControlMode,
    body: SpriteBody,
    center_position: bool,
    x_offset: f32,
    y_offset: f32,
    show_collision_debug: bool,
    image_scale: i32,
    wall_slide_speed: f32,
    wall_jump_force: [f32; 2],
    jump_cut_off: f32,
    jump_style: JumpStyle,
    max_charge_time: f32,
    charge_bar_sprite: String,
    #[serde(default)]
    charge_bar_scale: f32,
}

// Local definitions moved to modules

// --- Helper Structs ---
#[derive(Clone, Debug)]
pub struct AnimationSettings {
    pub fps: f32,
    pub speed: f32,
}

pub struct RpgClientHaCK {
    metadata: StableHaCMetadata,
    enabled: bool,
    paused: bool,

    image_scale: i32,

    // Config
    search_paths: Vec<String>,

    // State
    discovered_folders: Vec<DiscoveredFolder>,

    // Split States
    // Split States
    enable_preview: bool,
    preview: PreviewState,
    player_sprite_path: Option<String>,
    game_camera_zoom: f32,

    // Loading State
    is_loading: bool,
    new_path_input: RString, // Input buffer for new path
    loading_state: Option<ImageLoadingState>,

    // Runtime
    image_list: Option<HashMap<String, DynamicImage>>,
    animation_start: Option<Instant>,
    current_frame: i32,

    // Animation Config Overrides
    override_fps: f32,
    temp_fps: Option<f32>,
    temp_speed: Option<f32>,
    use_config_settings: bool,

    // Animation Mode Testing
    show_animation_modes: bool,
    current_animation_mode: u8,
    last_animation_mode: u8,

    // Texture Cache (path -> texture_id)
    texture_cache: HashMap<
        String,
        abi_stable::std_types::ROption<hackers::hackrs::stable_abi::StableTextureId>,
    >,

    // Display options
    show_search_paths: bool,
    show_discovered_folders: bool,
    _show_images: bool,
    // config
    show_config: bool,
    show_debug_info: bool,

    center_position: bool,
    x_offset: f32,
    y_offset: f32,
    show_collision_debug: bool,

    // Player Entity
    player: Player,
    projectiles: Vec<Projectile>,
    charge_bar_sprite: String,
    charge_bar_scale: f32,

    // Level Data
    level_data: LevelMap,

    // Debug
    frame_count: u64,

    // Manual Player Debug
    manual_player_anim: bool,
    manual_anim_mode: u8,
    manual_facing: u8,

    // Rolling Projectile Config
    debug_rolling_gravity: f32,
    debug_rolling_vel: [f32; 2],
    debug_rolling_scale: f32,
    debug_rolling_size: [f32; 2],
    debug_rolling_wrap: bool,
    debug_rolling_offset_y: f32,

    // Bullet Projectile Config
    debug_bullet_gravity: f32,
    debug_bullet_vel: [f32; 2],
    debug_bullet_scale: f32,
    debug_bullet_size: [f32; 2],
    debug_bullet_wrap: bool,
    debug_bullet_offset_y: f32,

    // Boom Config
    debug_boom_sprite: String,
    debug_boom_scale: f32,
    debug_boom_duration: f32,
}

impl RpgClientHaCK {
    pub fn new() -> Self {
        let mut level = LevelMap::default();
        level.parse();

        Self {
            metadata: StableHaCMetadata {
                name: "RPG Client".into(),
                description: "Test for image loading and sprite animation".into(),
                category: "Debug".into(),
                menu_weight: 1.0,
                window_weight: 1.0,
                draw_weight: 1.0,
                update_weight: 1.0,
                visible_in_gui: true,
                is_menu_enabled: true,
                is_render_enabled: true,
                is_update_enabled: true,
                is_window_enabled: true,
                window_pos: [100.0, 100.0],
                window_size: [800.0, 600.0],
                auto_resize_window: false,
                load_type: HaCKLoadType::Plugin,
            },
            enabled: true,
            paused: false,
            image_scale: 100, // Standard scale (100%)
            search_paths: vec![
                "examples/assets".to_string(),
                "assets/images".to_string(),
                "assets/images/test".to_string(),
                "assets/images/monster_ui".to_string(),
                "assets/images/ui".to_string(),
                "assets/images/missiles".to_string(),
            ],
            discovered_folders: Vec::new(),
            enable_preview: false,
            preview: PreviewState::default(),
            player_sprite_path: None,
            game_camera_zoom: 1.0,

            is_loading: false,
            new_path_input: RString::from(""),
            loading_state: None,
            image_list: None,
            animation_start: None,
            current_frame: 0,

            // Animation config
            override_fps: 10.0,
            temp_fps: None,
            temp_speed: None,
            use_config_settings: true,

            center_position: true,
            x_offset: 0.0,
            y_offset: 0.0,
            show_collision_debug: true,

            // Display
            show_search_paths: false,
            show_discovered_folders: true,
            _show_images: false,
            show_config: true,
            show_debug_info: false,

            // Debug
            show_animation_modes: false,
            current_animation_mode: 0,
            last_animation_mode: 0,

            texture_cache: HashMap::new(),

            player: Player::default(),
            projectiles: Vec::new(),
            charge_bar_sprite: "monster_ui/world_healthbar".to_string(), // Default
            charge_bar_scale: 1.0,

            level_data: level,
            frame_count: 0,

            manual_player_anim: false,
            manual_anim_mode: 1, // Stand
            manual_facing: 0,    // Down

            // Rolling Default
            debug_rolling_gravity: 800.0,
            debug_rolling_vel: [500.0, -200.0],
            debug_rolling_scale: 1.0,
            debug_rolling_size: [30.0, 30.0],
            debug_rolling_wrap: false,
            debug_rolling_offset_y: 0.0,

            // Bullet Default
            debug_bullet_gravity: 0.0, // Bullets fly straight by default
            debug_bullet_vel: [1000.0, 0.0],
            debug_bullet_scale: 1.0,
            debug_bullet_size: [30.0, 30.0],
            debug_bullet_wrap: false,
            debug_bullet_offset_y: 0.0,

            // Boom Default
            debug_boom_sprite: "hurt".into(), // Default to "hurt" or whatever exist
            debug_boom_scale: 2.0,
            debug_boom_duration: 0.5,
        }
    }

    fn scan_and_load(&mut self) {
        println!("Scanning folders...");
        self.discovered_folders = discover_sprite_folders(&self.search_paths);
        println!("Discovered {} folders:", self.discovered_folders.len());
        for f in &self.discovered_folders {
            println!(
                " - {} (Frames: {:?}, Layout: {:?})",
                f.name, f.frame_count, f.layout
            );
        }

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
        if !self.discovered_folders.is_empty() && self.preview.selected_folder_index == -1 {
            self.preview.selected_folder_index = 0;
        }
    }

    // Helper for indentation - MUST stay as associated function or update call sites
    fn draw_indent(ui: &StableUiBackend_TO<'_, RRef<'_, ()>>) {
        ui.dummy(20.0, 0.0);
        ui.same_line();
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

    fn get_current_folder(&self) -> Option<&DiscoveredFolder> {
        if self.preview.selected_folder_index >= 0
            && (self.preview.selected_folder_index as usize) < self.discovered_folders.len()
        {
            Some(&self.discovered_folders[self.preview.selected_folder_index as usize])
        } else {
            None
        }
    }
    // Helper for simulated headers
    fn get_current_settings(&self) -> AnimationSettings {
        if !self.use_config_settings {
            // Use manual overrides
            return AnimationSettings {
                fps: self.preview.fps_override,
                speed: self.preview.speed_override,
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
        // In preview, we always check the preview's animation mode
        if let Some(mode) = UnitAnimationMode::from_u8(self.preview.animation_mode) {
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

        // Use folder-level config
        AnimationSettings {
            fps: folder.fps.unwrap_or(self.override_fps),
            speed: folder.speed.unwrap_or(1.0),
        }
    }

    fn reset_to_config(&mut self) {
        self.preview.fps_override = 15.0;
        self.preview.speed_override = 1.0;
        self.preview.zoom = 1.0;
        self.preview.pan = [0.0, 0.0];
        self.use_config_settings = true;
    }

    fn draw_header(ui: &StableUiBackend_TO<'_, RRef<'_, ()>>, label: &str, state: &mut bool) {
        let icon = if *state { "▼" } else { "▶" };
        let button_label = format!("{} {}", icon, label);
        if ui.button(RStr::from_str(&button_label)) {
            *state = !*state;
        }
    }

    // --- Animation State Logic (Moved) ---

    // Helper to map PlayerState + Direction to UnityAnimationMode
    fn get_target_animation_mode(&self) -> abi_stable::std_types::ROption<UnitAnimationMode> {
        use abi_stable::std_types::ROption;

        // If testing modes manually, don't override
        if self.show_animation_modes {
            return ROption::RNone;
        }

        let mode = match self.player.state {
            PlayerState::Idle => UnitAnimationMode::Stand,
            PlayerState::Walk => UnitAnimationMode::Walk,
            PlayerState::Run => UnitAnimationMode::Run,
            PlayerState::Jump => UnitAnimationMode::Jump,
            PlayerState::Fall => UnitAnimationMode::Fall,
            PlayerState::WallSlide => UnitAnimationMode::Fall,
            PlayerState::Afk => UnitAnimationMode::Stand,
        };

        ROption::RSome(mode)
    }

    fn update_player(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        if !self.enabled {
            return;
        }
        let dt = 0.016;

        let mut input = crate::player::InputState::default();
        if ui.is_key_down(hackers::gui::Key::W) || ui.is_key_down(hackers::gui::Key::UpArrow) {
            input.move_direction[1] -= 1.0;
        }
        if ui.is_key_down(hackers::gui::Key::S) || ui.is_key_down(hackers::gui::Key::DownArrow) {
            input.move_direction[1] += 1.0;
        }
        if ui.is_key_down(hackers::gui::Key::A) || ui.is_key_down(hackers::gui::Key::LeftArrow) {
            input.move_direction[0] -= 1.0;
        }
        if ui.is_key_down(hackers::gui::Key::D) || ui.is_key_down(hackers::gui::Key::RightArrow) {
            input.move_direction[0] += 1.0;
        }
        input.jump_held = ui.is_key_down(hackers::gui::Key::Space);
        input.run_held = ui.is_key_down(hackers::gui::Key::LeftShift)
            || ui.is_key_down(hackers::gui::Key::RightShift);

        self.player.update(&input, &self.level_data, dt);

        // Apply Animation State
        if let abi_stable::std_types::ROption::RSome(mode) = self.get_target_animation_mode() {
            let new_mode = mode as u8;
            if new_mode != self.current_animation_mode {
                self.last_animation_mode = self.current_animation_mode;
                self.current_animation_mode = new_mode;

                // Handle Transitions
                // Handle Transitions
                if let Some(player_sprite_name) = &self.player_sprite_path {
                    // Find folder name match
                    if let Some(folder) = self
                        .discovered_folders
                        .iter()
                        .find(|f| &f.name == player_sprite_name)
                    {
                        let _target_mode = self.get_target_animation_mode();
                        // Note: self.current_animation_mode is already up to date here

                        // Find the full path to the sprite folder
                        let mut folder_path = std::path::PathBuf::from(&folder.name);
                        for search_path in &self.search_paths {
                            let p = std::path::Path::new(search_path).join(&folder.name);
                            if p.exists() {
                                folder_path = p;
                                break;
                            }
                        }

                        // Check for transition config
                        if let (Some(from), Some(to)) = (
                            UnitAnimationMode::from_u8(self.last_animation_mode),
                            UnitAnimationMode::from_u8(self.current_animation_mode),
                        ) {
                            let from_name = format!("{:?}", from).to_lowercase();
                            let to_name = format!("{:?}", to).to_lowercase();
                            let transition_key = format!("{}->{}", from_name, to_name);

                            // Check if transitions exist in config
                            if let Some(cfg) =
                                hackers::sprites::ImageLoader::load_sprite_config(&folder_path)
                            {
                                if let Some(transitions) = &cfg.transitions {
                                    if let Some(trans) = transitions.get(&transition_key) {
                                        // Found a transition!
                                        if let Some(skip) = trans.skip_frames {
                                            // Calculate time offset to skip frames
                                            // We need effective FPS
                                            let fps =
                                                cfg.get_fps_for_mode(&to_name).unwrap_or(10.0);
                                            if fps > 0.0 {
                                                let time_skip = skip as f32 / fps;
                                                // Adjust animation start time back into the PAST
                                                let new_start = std::time::Instant::now()
                                                    - std::time::Duration::from_secs_f32(time_skip);
                                                self.animation_start = Some(new_start);
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Reset animation on normal mode change
                self.animation_start = Some(std::time::Instant::now());
            }
        }
    }

    fn update_projectiles(&mut self, dt: f32, map_width: f32) {
        let mut new_spawns = Vec::new();
        for p in &mut self.projectiles {
            if let Some(pos) = p.update(&self.level_data, dt) {
                // Spawn BOOM
                // Create a boom projectile
                let mut boom = Projectile::new(
                    pos,
                    [0.0, 0.0],
                    ProjectileType::Boom,
                    self.debug_boom_sprite.clone(),
                    self.debug_boom_scale,
                    [1.0, 1.0], // Size irrelevant for boom usually
                    false,      // No wrap for boom
                );
                boom.lifetime = self.debug_boom_duration;
                new_spawns.push(boom);
            }
        }
        self.projectiles.append(&mut new_spawns);
        self.projectiles.retain(|p| p.active);
    }
}

impl StableHaCK for RpgClientHaCK {
    fn name(&self) -> RStr<'_> {
        self.metadata.name.as_rstr()
    }

    fn metadata(&self) -> &StableHaCMetadata {
        &self.metadata
    }

    fn save_settings(&self) -> RString {
        let config = ImageTestConfig {
            gravity: self.player.gravity,
            jump_force: self.player.jump_force,
            friction: self.player.friction,
            max_run_speed: self.player.max_run_speed,
            control_mode: self.player.control_mode.clone(),
            body: self.player.body.clone(),
            center_position: self.center_position,
            x_offset: self.x_offset,
            y_offset: self.y_offset,
            show_collision_debug: self.show_collision_debug,
            image_scale: self.image_scale,
            wall_slide_speed: self.player.wall_slide_speed,
            wall_jump_force: self.player.wall_jump_force,
            jump_cut_off: self.player.jump_cut_off,
            jump_style: self.player.jump_style,
            max_charge_time: self.player.max_charge_time,
            charge_bar_sprite: self.charge_bar_sprite.clone(),
            charge_bar_scale: self.charge_bar_scale,
        };

        match serde_json::to_string(&config) {
            Ok(s) => RString::from(s),
            Err(e) => {
                eprintln!("Failed to serialize ImageTestHaCK settings: {}", e);
                RString::new()
            }
        }
    }

    fn load_settings(&mut self, settings: RString) {
        if settings.is_empty() {
            return;
        }

        match serde_json::from_str::<ImageTestConfig>(settings.as_str()) {
            Ok(loaded) => {
                self.player.gravity = loaded.gravity;
                self.player.jump_force = loaded.jump_force;
                self.player.friction = loaded.friction;
                self.player.max_run_speed = loaded.max_run_speed;
                self.player.control_mode = loaded.control_mode;
                self.player.body = loaded.body;
                self.center_position = loaded.center_position;
                self.x_offset = loaded.x_offset;
                self.y_offset = loaded.y_offset;
                self.show_collision_debug = loaded.show_collision_debug;
                self.image_scale = loaded.image_scale;
                self.player.wall_slide_speed = loaded.wall_slide_speed;
                self.player.wall_jump_force = loaded.wall_jump_force;
                self.player.jump_cut_off = loaded.jump_cut_off;
                self.player.jump_style = loaded.jump_style;
                self.player.max_charge_time = loaded.max_charge_time;
                self.charge_bar_sprite = loaded.charge_bar_sprite;
                self.charge_bar_scale = if loaded.charge_bar_scale == 0.0 {
                    1.0
                } else {
                    loaded.charge_bar_scale
                };
            }
            Err(e) => eprintln!("Failed to deserialize ImageTestHaCK settings: {}", e),
        }
    }

    fn update(&mut self) {
        self.update_loading();
    }

    fn render_menu(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        if ui.menu_item(RStr::from_str("Scan Folders")) {
            self.scan_and_load();
        }
        ui.text(RStr::from_str("Image Test Plugin"));
        ui.separator();

        ui.checkbox(RStr::from_str("Enabled"), &mut self.enabled);
        ui.separator();

        // --- 1. Sprite Preview Panel ---
        // Default to open
        if ui.collapsing_header(RStr::from_str("1. Sprite Preview")) {
            Self::draw_indent(ui);

            // Folder Selector
            let current_idx = self.preview.selected_folder_index;
            let preview_name =
                if current_idx >= 0 && (current_idx as usize) < self.discovered_folders.len() {
                    self.discovered_folders[current_idx as usize].name.clone()
                } else {
                    "None".to_string()
                };

            if ui.begin_combo(RStr::from_str("Sprite"), RStr::from_str(&preview_name)) {
                for (i, folder) in self.discovered_folders.iter().enumerate() {
                    let is_selected = i as i32 == current_idx;
                    if ui.selectable(RStr::from_str(&folder.name), is_selected) {
                        self.preview.selected_folder_index = i as i32;
                    }
                    if is_selected {
                        ui.set_item_default_focus();
                    }
                }
                ui.end_combo();
            }

            // Visual Controls
            ui.text(RStr::from_str("Visuals"));
            ui.slider_float(RStr::from_str("Zoom"), 0.1, 5.0, &mut self.preview.zoom);
            if ui.button(RStr::from_str("Reset Pan/Zoom")) {
                self.preview.zoom = 1.0;
                self.preview.pan = [0.0, 0.0];
            }
            // Background color could be added here if we had a color picker

            // Animation Controls
            ui.separator();
            ui.text(RStr::from_str("Animation"));

            // Mode Selector
            let current_mode = UnitAnimationMode::from_u8(self.preview.animation_mode)
                .unwrap_or(UnitAnimationMode::Stand);
            let mode_name = format!("{:?}", current_mode);
            if ui.begin_combo(RStr::from_str("Mode"), RStr::from_str(&mode_name)) {
                // Loop through common modes (approximate list or 0..16)
                for i in 0..18 {
                    if let Some(m) = UnitAnimationMode::from_u8(i as u8) {
                        let name = format!("{:?}", m);
                        if ui.selectable(
                            RStr::from_str(&name),
                            self.preview.animation_mode == i as u8,
                        ) {
                            self.preview.animation_mode = i as u8;
                        }
                    }
                }
                ui.end_combo();
            }

            // Direction Selector
            let dir_names = [
                "Down",
                "DownLeft",
                "Left",
                "UpLeft",
                "Up",
                "UpRight",
                "Right",
                "DownRight",
            ];
            let current_dir_name = dir_names
                .get(self.preview.direction as usize)
                .unwrap_or(&"Unknown");
            if ui.begin_combo(
                RStr::from_str("Direction"),
                RStr::from_str(current_dir_name),
            ) {
                for (i, name) in dir_names.iter().enumerate() {
                    if ui.selectable(RStr::from_str(name), self.preview.direction == i as u8) {
                        self.preview.direction = i as u8;
                    }
                }
                ui.end_combo();
            }

            // Playback
            ui.checkbox(RStr::from_str("Playing"), &mut self.preview.is_playing);

            // Speed / FPS
            ui.slider_float(
                RStr::from_str("Speed (x)"),
                0.1,
                5.0,
                &mut self.preview.speed_override,
            );
            ui.slider_float(
                RStr::from_str("FPS Override"),
                1.0,
                60.0,
                &mut self.preview.fps_override,
            );

            ui.checkbox(
                RStr::from_str("Show Collision Box"),
                &mut self.preview.show_collision_box,
            );
        }

        // --- 2. Player Settings Panel ---
        if ui.collapsing_header(RStr::from_str("2. Player Settings")) {
            Self::draw_indent(ui);

            // Sprite Selector
            let current_player_sprite = self
                .player_sprite_path
                .clone()
                .unwrap_or_else(|| "None".to_string());
            if ui.begin_combo(
                RStr::from_str("Player Sprite"),
                RStr::from_str(&current_player_sprite),
            ) {
                for folder in &self.discovered_folders {
                    let is_selected = Some(&folder.name) == self.player_sprite_path.as_ref();
                    if ui.selectable(RStr::from_str(&folder.name), is_selected) {
                        self.player_sprite_path = Some(folder.name.clone());
                    }
                }
                ui.end_combo();
            }

            // Sprite Scale (Legacy)
            let mut scale_f = self.image_scale as f32 / 100.0;
            if ui.slider_float(RStr::from_str("Sprite Scale"), 0.1, 5.0, &mut scale_f) {
                self.image_scale = (scale_f * 100.0) as i32;
            }

            if ui.button(RStr::from_str("Sync from Preview")) {
                if self.preview.selected_folder_index >= 0 {
                    let idx = self.preview.selected_folder_index as usize;
                    if idx < self.discovered_folders.len() {
                        self.player_sprite_path = Some(self.discovered_folders[idx].name.clone());
                    }
                }
            }

            ui.separator();
            ui.text(RStr::from_str("Animation Override"));
            ui.checkbox(
                RStr::from_str("Manual Control"),
                &mut self.manual_player_anim,
            );

            if self.manual_player_anim {
                // Mode Selector
                let current_mode = UnitAnimationMode::from_u8(self.manual_anim_mode)
                    .unwrap_or(UnitAnimationMode::Stand);
                let mode_name = format!("{:?}", current_mode);
                if ui.begin_combo(RStr::from_str("Override Mode"), RStr::from_str(&mode_name)) {
                    for i in 0..18 {
                        if let Some(m) = UnitAnimationMode::from_u8(i as u8) {
                            let name = format!("{:?}", m);
                            if ui
                                .selectable(RStr::from_str(&name), self.manual_anim_mode == i as u8)
                            {
                                self.manual_anim_mode = i as u8;
                            }
                        }
                    }
                    ui.end_combo();
                }

                // Direction Selector
                let dir_names = [
                    "Down",
                    "DownLeft",
                    "Left",
                    "UpLeft",
                    "Up",
                    "UpRight",
                    "Right",
                    "DownRight",
                ];
                let current_dir_name = dir_names
                    .get(self.manual_facing as usize)
                    .unwrap_or(&"Unknown");
                if ui.begin_combo(
                    RStr::from_str("Override Dir"),
                    RStr::from_str(current_dir_name),
                ) {
                    for (i, name) in dir_names.iter().enumerate() {
                        if ui.selectable(RStr::from_str(name), self.manual_facing == i as u8) {
                            self.manual_facing = i as u8;
                        }
                    }
                    ui.end_combo();
                }
            }

            // Game Camera
            ui.separator();
            ui.text(RStr::from_str("Game Camera"));
            ui.slider_float(
                RStr::from_str("Camera Zoom"),
                0.25,
                4.0,
                &mut self.game_camera_zoom,
            );
            if ui.button(RStr::from_str("Reset Camera")) {
                self.game_camera_zoom = 1.0;
            }

            // Physics & Control
            ui.separator();
            ui.text(RStr::from_str("Physics & Control"));

            // Control Mode
            if ui.button(RStr::from_str(
                if matches!(self.player.control_mode, ControlMode::TopDown) {
                    "[ Top Down ]"
                } else {
                    "  Top Down  "
                },
            )) {
                self.player.control_mode = ControlMode::TopDown;
            }
            ui.same_line();
            if ui.button(RStr::from_str(
                if matches!(self.player.control_mode, ControlMode::SideScroll) {
                    "[ Side Scroll ]"
                } else {
                    "  Side Scroll  "
                },
            )) {
                // Reset physics/state when switching
                self.player.velocity = [0.0, 0.0];
                self.player.is_grounded = false;
                self.player.control_mode = ControlMode::SideScroll;
            }

            ui.separator();
            ui.text(RStr::from_str("Movement Physics"));
            ui.slider_float(
                RStr::from_str("Gravity"),
                0.0,
                2000.0,
                &mut self.player.gravity,
            );
            ui.slider_float(
                RStr::from_str("Friction"),
                0.0,
                1.0,
                &mut self.player.friction,
            );
            ui.slider_float(
                RStr::from_str("Max Run Speed"),
                0.0,
                1000.0,
                &mut self.player.max_run_speed,
            );

            ui.separator();
            ui.text(RStr::from_str("Wall Mechanics"));
            ui.slider_float(
                RStr::from_str("Wall Slide Speed"),
                0.0,
                500.0,
                &mut self.player.wall_slide_speed,
            );
            ui.slider_float(
                RStr::from_str("Wall Jump X"),
                0.0,
                1000.0,
                &mut self.player.wall_jump_force[0],
            );
            ui.slider_float(
                RStr::from_str("Wall Jump Y"),
                0.0,
                1000.0,
                &mut self.player.wall_jump_force[1],
            );

            ui.separator();
            ui.text(RStr::from_str("Body & Collision"));
            ui.slider_float(
                RStr::from_str("Width"),
                1.0,
                256.0,
                &mut self.player.body.width,
            );
            ui.slider_float(
                RStr::from_str("Height"),
                1.0,
                256.0,
                &mut self.player.body.height,
            );
            ui.slider_float(
                RStr::from_str("Collision Offset X"),
                -100.0,
                100.0,
                &mut self.player.body.collision_offset[0],
            );
            ui.slider_float(
                RStr::from_str("Collision Offset Y"),
                -100.0,
                100.0,
                &mut self.player.body.collision_offset[1],
            );
            ui.checkbox(
                RStr::from_str("Show Collision Box"),
                &mut self.show_collision_debug,
            );

            ui.separator();
            ui.text(RStr::from_str("Sprite Alignment"));
            ui.checkbox(RStr::from_str("Center Position"), &mut self.center_position);
            ui.slider_float(
                RStr::from_str("Sprite Offset X"),
                -100.0,
                100.0,
                &mut self.x_offset,
            );
            ui.slider_float(
                RStr::from_str("Sprite Offset Y"),
                -100.0,
                100.0,
                &mut self.y_offset,
            );

            ui.separator();
            ui.text(RStr::from_str("Jump Settings"));
            ui.slider_float(
                RStr::from_str("Jump Force"),
                1.0,
                2000.0,
                &mut self.player.jump_force,
            );
            ui.slider_float(
                RStr::from_str("Jump Cutoff"),
                0.0,
                1.0,
                &mut self.player.jump_cut_off,
            );
            ui.slider_float(
                RStr::from_str("Max Charge Time"),
                0.0,
                2.0,
                &mut self.player.max_charge_time,
            );

            ui.text(RStr::from_str("Charge Bar Settings"));
            let current_charge_sprite = if self.charge_bar_sprite.is_empty() {
                "None".to_string()
            } else {
                self.charge_bar_sprite.clone()
            };
            if ui.begin_combo(
                RStr::from_str("Charge Bar Sprite"),
                RStr::from_str(&current_charge_sprite),
            ) {
                if ui.selectable(RStr::from_str("None"), self.charge_bar_sprite.is_empty()) {
                    self.charge_bar_sprite = "".to_string();
                }
                for folder in &self.discovered_folders {
                    let is_selected = folder.name == self.charge_bar_sprite;
                    if ui.selectable(RStr::from_str(&folder.name), is_selected) {
                        self.charge_bar_sprite = folder.name.clone();
                    }
                }
                ui.end_combo();
            }
            ui.slider_float(
                RStr::from_str("Charge Bar Scale"),
                0.1,
                5.0,
                &mut self.charge_bar_scale,
            );

            // Jump Style Selector
            use crate::player_types::JumpStyle;
            let current_jump_style = match self.player.jump_style {
                JumpStyle::Normal => "Normal",
                JumpStyle::Charge => "Charge",
            };

            if ui.begin_combo(
                RStr::from_str("Jump Style"),
                RStr::from_str(current_jump_style),
            ) {
                if ui.selectable(
                    RStr::from_str("Normal"),
                    matches!(self.player.jump_style, JumpStyle::Normal),
                ) {
                    self.player.jump_style = JumpStyle::Normal;
                }
                if ui.selectable(
                    RStr::from_str("Charge"),
                    matches!(self.player.jump_style, JumpStyle::Charge),
                ) {
                    self.player.jump_style = JumpStyle::Charge;
                }
                ui.end_combo();
            }
        } // End Player Settings

        ui.separator();

        // --- 3. Configuration ---
        // --- 3. Configuration ---
        if ui.collapsing_header(RStr::from_str("3. Configuration")) {
            Self::draw_indent(ui);

            // Search Paths
            ui.text(RStr::from_str("Configured Paths:"));
            for path in &self.search_paths {
                Self::draw_indent(ui);
                ui.text(RStr::from_str(&format!("- {}", path)));
            }

            ui.dummy(0.0, 5.0);
            ui.text(RStr::from_str("Add new path:"));
            ui.input_text(RStr::from_str("##new_path"), &mut self.new_path_input);
            ui.same_line();
            if ui.button(RStr::from_str("Add")) {
                let path_str = self.new_path_input.to_string();
                if !path_str.is_empty() {
                    self.search_paths.push(path_str);
                    self.new_path_input = RString::from("");
                    self.scan_and_load();
                }
            }

            ui.separator();
            ui.checkbox(RStr::from_str("Show Debug Info"), &mut self.show_debug_info);

            ui.same_line();
            if ui.button(RStr::from_str("Reload All")) {
                self.scan_and_load();
            }
        }

        ui.separator();

        // --- 4. Projectiles (Debug) ---
        if ui.collapsing_header(RStr::from_str("4. Projectiles")) {
            Self::draw_indent(ui);
            ui.text(RStr::from_str(&format!(
                "Active Projectiles: {}",
                self.projectiles.len()
            )));

            ui.separator();
            ui.text(RStr::from_str("Rolling Config"));
            ui.slider_float(
                RStr::from_str("Roll Gravity"),
                -2000.0,
                2000.0,
                &mut self.debug_rolling_gravity,
            );
            ui.slider_float(
                RStr::from_str("Roll Vel X"),
                -2000.0,
                2000.0,
                &mut self.debug_rolling_vel[0],
            );
            ui.slider_float(
                RStr::from_str("Roll Vel Y"),
                -2000.0,
                2000.0,
                &mut self.debug_rolling_vel[1],
            );
            ui.slider_float(
                RStr::from_str("Roll Scale"),
                0.1,
                5.0,
                &mut self.debug_rolling_scale,
            );
            ui.slider_float(
                RStr::from_str("Roll Width"),
                1.0,
                200.0,
                &mut self.debug_rolling_size[0],
            );
            ui.slider_float(
                RStr::from_str("Roll Height"),
                1.0,
                200.0,
                &mut self.debug_rolling_size[1],
            );
            ui.slider_float(
                RStr::from_str("Roll Offset Y"),
                -200.0,
                200.0,
                &mut self.debug_rolling_offset_y,
            );
            ui.checkbox(RStr::from_str("Roll Wrap"), &mut self.debug_rolling_wrap);

            ui.separator();
            ui.text(RStr::from_str("Bullet Config"));
            ui.slider_float(
                RStr::from_str("Bull Gravity"),
                -2000.0,
                2000.0,
                &mut self.debug_bullet_gravity,
            );
            ui.slider_float(
                RStr::from_str("Bull Vel X"),
                -2000.0,
                2000.0,
                &mut self.debug_bullet_vel[0],
            );
            ui.slider_float(
                RStr::from_str("Bull Vel Y"),
                -2000.0,
                2000.0,
                &mut self.debug_bullet_vel[1],
            );
            ui.slider_float(
                RStr::from_str("Bull Scale"),
                0.1,
                5.0,
                &mut self.debug_bullet_scale,
            );
            ui.slider_float(
                RStr::from_str("Bull Width"),
                1.0,
                200.0,
                &mut self.debug_bullet_size[0],
            );
            ui.slider_float(
                RStr::from_str("Bull Height"),
                1.0,
                200.0,
                &mut self.debug_bullet_size[1],
            );
            ui.slider_float(
                RStr::from_str("Bull Offset Y"),
                -200.0,
                200.0,
                &mut self.debug_bullet_offset_y,
            );
            ui.checkbox(RStr::from_str("Bull Wrap"), &mut self.debug_bullet_wrap);

            ui.separator();
            ui.text(RStr::from_str("Boom Config"));
            let mut boom_sprite = RString::from(self.debug_boom_sprite.clone());
            if ui.input_text(RStr::from_str("Boom Sprite"), &mut boom_sprite) {
                self.debug_boom_sprite = boom_sprite.into();
            }
            ui.slider_float(
                RStr::from_str("Boom Scale"),
                0.1,
                5.0,
                &mut self.debug_boom_scale,
            );
            ui.slider_float(
                RStr::from_str("Boom Duration"),
                0.1,
                5.0,
                &mut self.debug_boom_duration,
            );

            ui.separator();
            ui.text(RStr::from_str("Actions"));

            if ui.button(RStr::from_str("Spawn Rolling")) {
                let dir_mul = if self.player.facing_direction == Direction::Left {
                    -1.0
                } else {
                    1.0
                };
                let vel = [
                    self.debug_rolling_vel[0] * dir_mul,
                    self.debug_rolling_vel[1],
                ];

                let mut p = Projectile::new(
                    [
                        self.player.pos[0],
                        self.player.pos[1] + self.debug_rolling_offset_y,
                    ],
                    vel,
                    ProjectileType::Rolling,
                    "FrozenOrb".into(),
                    self.debug_rolling_scale,
                    self.debug_rolling_size,
                    self.debug_rolling_wrap,
                );
                p.sim.gravity = self.debug_rolling_gravity;
                self.projectiles.push(p);

                eprintln!("Spawned Rolling Projectile via UI");
            }
            ui.same_line();
            if ui.button(RStr::from_str("Spawn Bullet")) {
                let dir_mul = if self.player.facing_direction == Direction::Left {
                    -1.0
                } else {
                    1.0
                };
                // Bullet velocity logic
                let vel = [self.debug_bullet_vel[0] * dir_mul, self.debug_bullet_vel[1]];

                let mut p = Projectile::new(
                    [
                        self.player.pos[0],
                        self.player.pos[1] + self.debug_bullet_offset_y,
                    ],
                    vel,
                    ProjectileType::Bullet,
                    "FrozenOrb".into(),
                    self.debug_bullet_scale,
                    self.debug_bullet_size,
                    self.debug_bullet_wrap,
                );
                p.sim.gravity = self.debug_bullet_gravity;
                self.projectiles.push(p);
                eprintln!("Spawned Bullet Projectile via UI");
            }

            if ui.button(RStr::from_str("Clear All")) {
                self.projectiles.clear();
            }
        }
    }

    // --- Animation State Logic ---

    // Helper to map PlayerState + Direction to UnityAnimationMode

    fn render_window(&mut self, ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>) {
        self.render_menu(ui);
    }

    fn render_draw(
        &mut self,
        ui: &StableUiBackend_TO<'_, abi_stable::sabi_types::RRef<'_, ()>>,
        draw_fg: &mut hackers::hackrs::stable_abi::StableDrawList_TO<
            '_,
            abi_stable::std_types::RBox<()>,
        >,
        draw_bg: &mut hackers::hackrs::stable_abi::StableDrawList_TO<
            '_,
            abi_stable::std_types::RBox<()>,
        >,
    ) {
        self.frame_count = self.frame_count.wrapping_add(1);

        if !self.enabled {
            return;
        }

        self.update_player(ui);

        // Update Projectiles
        // Update Projectiles
        // Update Projectiles
        self.update_projectiles(0.016, self.level_data.width);

        // Debug Spawning
        if ui.is_key_down(hackers::gui::Key::Num1) {
            // Rolling
            let dir_mul = if self.player.facing_direction == Direction::Left {
                -1.0
            } else {
                1.0
            };
            let vel = [
                self.debug_rolling_vel[0] * dir_mul,
                self.debug_rolling_vel[1],
            ];

            if self.frame_count % 10 == 0 {
                eprintln!("Spawned Rolling Projectile via Key 1");
                let mut p = Projectile::new(
                    [
                        self.player.pos[0],
                        self.player.pos[1] + self.debug_rolling_offset_y,
                    ],
                    vel,
                    ProjectileType::Rolling,
                    "FrozenOrb".into(),
                    self.debug_rolling_scale,
                    self.debug_rolling_size,
                    self.debug_rolling_wrap,
                );
                p.sim.gravity = self.debug_rolling_gravity;
                self.projectiles.push(p);
            }
        }
        if ui.is_key_down(hackers::gui::Key::Num2) {
            // Bullet
            let dir_mul = if self.player.facing_direction == Direction::Left {
                -1.0
            } else {
                1.0
            };
            let vel = [self.debug_bullet_vel[0] * dir_mul, self.debug_bullet_vel[1]];

            if self.frame_count % 10 == 0 {
                eprintln!("Spawned Bullet Projectile via Key 2");
                let mut p = Projectile::new(
                    [
                        self.player.pos[0],
                        self.player.pos[1] + self.debug_bullet_offset_y,
                    ],
                    vel,
                    ProjectileType::Bullet,
                    "FrozenOrb".into(),
                    self.debug_bullet_scale,
                    self.debug_bullet_size,
                    self.debug_bullet_wrap,
                );
                p.sim.gravity = self.debug_bullet_gravity;
                self.projectiles.push(p);
            }
        }

        // --- 1. Texture Uploading ---
        if let Some(images) = &self.image_list {
            for (key, img) in images.iter() {
                if !self.texture_cache.contains_key(key) {
                    let rgba = img.to_rgba8();
                    // Upload texture and store ID
                    use abi_stable::std_types::RSlice;
                    let texture_id = ui.upload_texture(
                        RSlice::from_slice(rgba.as_raw()),
                        rgba.width(),
                        rgba.height(),
                    );
                    self.texture_cache.insert(
                        key.clone(),
                        abi_stable::std_types::ROption::RSome(texture_id),
                    );
                }
            }
        }

        // --- 2. Camera & World Setup ---
        let zoom = self.game_camera_zoom;
        let camera_offset = [
            (self.metadata.window_size[0] / 2.0) - (self.player.pos[0] * zoom),
            (self.metadata.window_size[1] / 2.0) - (self.player.pos[1] * zoom),
        ];

        let to_screen = |pos: [f32; 2]| -> [f32; 2] {
            [
                (pos[0] * zoom) + camera_offset[0],
                (pos[1] * zoom) + camera_offset[1],
            ]
        };

        // --- 3. Render Level ---
        for tile in &self.level_data.tiles {
            let color = match tile.tile_type {
                TileType::Wall => [0.5, 0.5, 0.5, 1.0],
                TileType::Floor => [0.3, 0.3, 0.3, 1.0],
                _ => [0.0, 0.0, 0.0, 0.0],
            };

            if color[3] > 0.0 {
                let p1 = to_screen(tile.position);
                let p2 = to_screen([
                    tile.position[0] + tile.size[0],
                    tile.position[1] + tile.size[1],
                ]);

                draw_bg.add_rect(p1, p2, color, true);
                draw_bg.add_rect(p1, p2, [0.0, 0.0, 0.0, 1.0], false); // Border
            }
        }

        // --- 4. Render Player ---
        // Auto-select "player" if not set
        if self.player_sprite_path.is_none() {
            if self.discovered_folders.iter().any(|f| f.name == "player") {
                self.player_sprite_path = Some("player".to_string());
            } else if let Some(first) = self.discovered_folders.first() {
                self.player_sprite_path = Some(first.name.clone());
            }
        }

        if let Some(player_sprite) = self.player_sprite_path.as_ref() {
            // Determine animation mode
            let anim_mode = self.get_target_animation_mode();
            let mut mode_str = if let abi_stable::std_types::ROption::RSome(m) = anim_mode {
                format!("{:?}", m).to_lowercase()
            } else {
                "idle".to_string()
            };

            // Common mappings (Folder Name vs Mode Name)
            // "stand" is often "idle" in assets
            if mode_str == "stand" {
                mode_str = "idle".to_string();
            }
            if mode_str == "death" {
                mode_str = "hurt".to_string(); // Approximate mapping
            }

            // Direction logic
            let dir_idx = if self.manual_player_anim {
                self.manual_facing
            } else {
                self.player.facing_direction.clone() as u8
            };

            // Mapping based on user report: Up=Right, Down=Up, Right=Down
            // Observed Sheet Order likely: Up, Left, Down, Right (0, 1, 2, 3)
            let dir_offset = match dir_idx {
                0 | 1 | 7 => 2, // Down -> 2
                2 => 1,         // Left -> 1
                6 => 3,         // Right -> 3
                3 | 4 | 5 => 0, // Up -> 0
                _ => 2,
            };

            let dir_row_str = match dir_offset {
                0 => "down",
                1 => "left",
                2 => "right",
                _ => "up",
            };

            // Mapping for shared assets (e.g. Fall using Jump sprites)
            let search_mode_str = if mode_str == "fall" {
                "jump".to_string()
            } else if mode_str == "death" {
                "hurt".to_string() // Often 'death' is 'natural_death' or 'hurt' in some packs
            } else {
                mode_str.clone()
            };

            // Find folder
            if let Some(folder) = self
                .discovered_folders
                .iter()
                .find(|f| f.name == *player_sprite)
            {
                // Find matching frames
                if let Some(images) = &self.image_list {
                    // Try to find specific frames for current state.
                    // Priority 1: Strict match (Folder + Mode + Direction String)
                    let mut frames: Vec<_> = images
                        .keys()
                        .filter(|k| k.starts_with(&folder.name))
                        .filter(|k| k.contains(&search_mode_str))
                        .filter(|k| k.contains(dir_row_str))
                        .collect();

                    let mut is_multi_direction_sheet = false;

                    if frames.is_empty() {
                        // Priority 2: Mode match only (e.g. "idle_00.png" containing all dirs)
                        // Fallback to "idle" if specific mode not found

                        let try_mode = if !images.keys().any(|k| k.contains(&search_mode_str))
                            && search_mode_str != "idle"
                        {
                            if search_mode_str == "jump" {
                                "idle".to_string()
                            } else {
                                "idle".to_string()
                            }
                        } else {
                            search_mode_str.clone()
                        };

                        frames = images
                            .keys()
                            .filter(|k| k.starts_with(&folder.name))
                            .filter(|k| k.contains(&try_mode))
                            .collect();

                        // If we fell back to a mode-only list, and it likely contains multiple directions, flag it.
                        // We assume it's multi-direction if it hasn't been filtered by direction string.
                        is_multi_direction_sheet = true;
                    }

                    // Debug log once per second-ish
                    if self.frame_count % 300 == 0 {
                        // log::info! ...
                    }

                    // Sort frames by trailing number in filename
                    frames.sort_by_key(|k| {
                        let filename = k.split('/').last().unwrap_or("");
                        let stem = std::path::Path::new(filename)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(filename);

                        // Extract trailing digits
                        let digits: String = stem
                            .chars()
                            .rev()
                            .take_while(|c| c.is_ascii_digit())
                            .collect();

                        let number_str: String = digits.chars().rev().collect();

                        if number_str.is_empty() {
                            // If no trailing digits, try finding ANY digits
                            let any_digits: String =
                                stem.chars().filter(|c| c.is_ascii_digit()).collect();
                            any_digits.parse::<usize>().unwrap_or(0)
                        } else {
                            number_str.parse::<usize>().unwrap_or(0)
                        }
                    });

                    // Calculate frame index
                    // Calculate frame index
                    if !frames.is_empty() {
                        // Use folder FPS or fallback
                        let fps = folder.fps.unwrap_or(12.0);
                        let elapsed = if let Some(start) = self.animation_start {
                            if self.paused {
                                // TODO: persistent pause time
                                0.0
                            } else {
                                start.elapsed().as_secs_f32()
                            }
                        } else {
                            0.0
                        };

                        let idx_raw = (elapsed * fps) as usize;
                        let mut normalized_idx = idx_raw;

                        // Determine per-direction frame count
                        let per_dir_count = if is_multi_direction_sheet
                            && frames.len() > 0
                            && frames.len() % 4 == 0
                        {
                            frames.len() / 4
                        } else {
                            frames.len()
                        };

                        // Lookup behavior for current mode
                        let behavior = folder
                            .behaviors
                            .as_ref()
                            .and_then(|b| b.get(&mode_str).or_else(|| b.get(&search_mode_str)));

                        // Apply Loop Range & Modifiers
                        if let Some(beh) = behavior {
                            normalized_idx =
                                hackers::sprites::animation_utils::calculate_variable_frame_index(
                                    elapsed,
                                    fps,
                                    per_dir_count,
                                    beh,
                                );
                        } else {
                            normalized_idx = normalized_idx % per_dir_count;
                        }

                        let mut final_idx = 0;
                        if is_multi_direction_sheet && frames.len() > 0 && frames.len() % 4 == 0 {
                            let per_dir = frames.len() / 4;
                            let safe_offset = dir_offset.min(3);
                            final_idx = (safe_offset * per_dir) + normalized_idx;
                        } else {
                            final_idx = normalized_idx;
                        }

                        if final_idx >= frames.len() {
                            final_idx = 0;
                        }

                        let key = frames[final_idx];

                        if let Some(tex) = self
                            .texture_cache
                            .get(key)
                            .and_then(|t| t.clone().into_option())
                        {
                            if let Some(img) = images.get(key) {
                                let w = img.width() as f32 * self.image_scale as f32 / 100.0 * zoom;
                                let h =
                                    img.height() as f32 * self.image_scale as f32 / 100.0 * zoom;

                                let center = to_screen(self.player.pos);
                                let off_x = self.x_offset * zoom;
                                let off_y = self.y_offset * zoom;
                                let p1 = [center[0] - w / 2.0 + off_x, center[1] - h / 2.0 + off_y];
                                let p2 = [center[0] + w / 2.0 + off_x, center[1] + h / 2.0 + off_y];

                                draw_bg.add_image(tex, p1.into(), p2.into());

                                // Draw frame info overlay (Restored)
                                let info =
                                    format!("Frame {}/{} ({})", final_idx + 1, frames.len(), key);
                                draw_bg.add_text(
                                    [p1[0] + 1.0, p1[1] - 24.0].into(),
                                    [0.0, 0.0, 0.0, 1.0],
                                    RStr::from_str(&info),
                                );
                                draw_bg.add_text(
                                    [p1[0], p1[1] - 25.0].into(),
                                    [1.0, 1.0, 1.0, 1.0],
                                    RStr::from_str(&info),
                                );

                                // Debug state overlay
                                let state_info = format!(
                                    "Pos: {:.1},{:.1} Dir: {:?}",
                                    self.player.pos[0],
                                    self.player.pos[1],
                                    self.player.facing_direction
                                );
                                draw_bg.add_text(
                                    [p1[0], p1[1] - 10.0].into(),
                                    [1.0, 1.0, 0.0, 1.0],
                                    RStr::from_str(&state_info),
                                );

                                // DRAW COLLISION POLYGON if enabled
                                if self.show_collision_debug {
                                    // Try to find the file config
                                    // search_mode_str is the mode (e.g. "run")
                                    // We need to find the filename. Usually "{mode}.png"
                                    let filename = format!("{}.png", search_mode_str);

                                    if let Some(file_config) =
                                        folder.files.as_ref().and_then(|files| {
                                            files.iter().find(|f| f.filename == filename)
                                        })
                                    {
                                        if let Some(collision_map) = &file_config.collision_data {
                                            // final_idx *should* match the frame index if the list hasn't been mixed
                                            // If we filtered by file, it should match.
                                            // The key list `frames` was filtered by `folder.name` and `search_mode_str`.
                                            // Assuming simple mapping for now.

                                            // We need to know which frame index WITHIN THE FILE this corresponds to.
                                            // Since `frames` is sorted by the trailing number, and `collision_data` is keyed by that same number (generated during split),
                                            // We can try to parse the key again or trust final_idx implies 0..N?
                                            // Actually, `collision_data` keys are the cell indices (0, 1, 2...).
                                            // If `frames` contains `file_0`, `file_1`, then `frames[0]` corresponds to `0`.

                                            // Let's parse the index from the key string to be safe
                                            let key_str = key.as_str();
                                            let file_frame_idx = if let Some(idx_str) =
                                                key_str.rsplitn(2, '_').next()
                                            {
                                                idx_str.parse::<usize>().unwrap_or(final_idx)
                                            } else {
                                                final_idx
                                            };

                                            if let Some(points) = collision_map.get(&file_frame_idx)
                                            {
                                                // Draw points!
                                                for i in 0..points.len() {
                                                    let p1_local = points[i];
                                                    let p2_local = points[(i + 1) % points.len()];

                                                    // Transform
                                                    // Local 0,0 is Top-Left of the Sprite Frame
                                                    // Sprite Render Logic:
                                                    // center of sprite on screen = `center` (player pos projected)
                                                    // top-left of sprite rect = `p1` (calculated above as center - w/2 + off)
                                                    // But wait, `p1` variable in this scope hides my loop var `p1`.
                                                    // The rect top-left is `p1` (line 1351).

                                                    // Apply scaling
                                                    let scale_x =
                                                        self.image_scale as f32 / 100.0 * zoom;
                                                    let scale_y =
                                                        self.image_scale as f32 / 100.0 * zoom;

                                                    let t_p1 = [
                                                        p1[0] + (p1_local[0] * scale_x),
                                                        p1[1] + (p1_local[1] * scale_y),
                                                    ];
                                                    let t_p2 = [
                                                        p1[0] + (p2_local[0] * scale_x),
                                                        p1[1] + (p2_local[1] * scale_y),
                                                    ];

                                                    draw_bg.add_line(
                                                        t_p1.into(),
                                                        t_p2.into(),
                                                        [0.0, 1.0, 1.0, 1.0], // Cyan for poly
                                                        2.0,
                                                    );

                                                    // Draw Points
                                                    draw_bg.add_circle(
                                                        t_p1.into(),
                                                        2.0,
                                                        [0.0, 1.0, 1.0, 0.8],
                                                        true,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Draw placeholder for player
            let p1 = to_screen([self.player.pos[0] - 10.0, self.player.pos[1] - 20.0]);
            let p2 = to_screen([self.player.pos[0] + 10.0, self.player.pos[1]]);
            draw_bg.add_rect(p1, p2, [0.0, 1.0, 0.0, 1.0], true);
        }

        // --- 4.5 Render Charge Bar ---
        if !self.charge_bar_sprite.is_empty() && self.player.jump_charge > 0.0 {
            if let Some(folder) = self
                .discovered_folders
                .iter()
                .find(|f| f.name == self.charge_bar_sprite)
            {
                if let Some(images) = &self.image_list {
                    // Get all frames for charge bar
                    let mut frames: Vec<_> = images
                        .keys()
                        .filter(|k| k.starts_with(&folder.name))
                        .collect();
                    frames.sort(); // Simple sort

                    if !frames.is_empty() {
                        // Simple animation
                        let fps = 15.0;
                        let elapsed = if let Some(start) = self.animation_start {
                            start.elapsed().as_secs_f32()
                        } else {
                            0.0
                        };
                        let idx = (elapsed * fps) as usize % frames.len();
                        let key = frames[idx];

                        if let Some(tex) = self
                            .texture_cache
                            .get(key)
                            .and_then(|t| t.clone().into_option())
                        {
                            if let Some(img) = images.get(key) {
                                let scale = self.charge_bar_scale * zoom;

                                // Let's assume charge_bar_scale is a direct multiplier? No, it's 0.1 to 5.0.
                                // Let's treat it as relative to original size.
                                let w = img.width() as f32
                                    * self.charge_bar_scale
                                    * (self.image_scale as f32 / 100.0)
                                    * zoom;
                                let h = img.height() as f32
                                    * self.charge_bar_scale
                                    * (self.image_scale as f32 / 100.0)
                                    * zoom;

                                // Position: Above head? Users head is roughly center - half_height.
                                // Let's put it fairly high up relative to the sprite center.
                                let center = to_screen(self.player.pos);
                                // Apply user offset to this too? Or keep it centered on player physics?
                                // User asked for control, maybe it should follow the sprite offset too?
                                // "control the items on screen".
                                // Let's make it follow the sprite offset, as it's visually part of the "unit".
                                let off_x = self.x_offset * zoom;
                                let off_y = self.y_offset * zoom;

                                let cx = center[0] + off_x;
                                let cy = center[1] + off_y - (50.0 * zoom); // 50px up from center as default

                                let p1 = [cx - w / 2.0, cy - h / 2.0];
                                let p2 = [cx + w / 2.0, cy + h / 2.0];

                                draw_fg.add_image(tex, p1.into(), p2.into());
                            }
                        }
                    }
                }
            }
        }

        // --- 4.6 Render Projectiles ---
        for p in &self.projectiles {
            let sprite_name = &p.sprite_path;
            if let Some(folder) = self
                .discovered_folders
                .iter()
                .find(|f| f.name == *sprite_name)
            {
                if let Some(images) = &self.image_list {
                    let mut frames: Vec<_> = images
                        .keys()
                        .filter(|k| k.starts_with(&folder.name))
                        .collect();
                    frames.sort();

                    if !frames.is_empty() {
                        let fps = 15.0;
                        let idx = (p.age * fps) as usize % frames.len();
                        let key = frames[idx];

                        if let Some(tex) = self
                            .texture_cache
                            .get(key)
                            .and_then(|t| t.clone().into_option())
                        {
                            if let Some(img) = images.get(key) {
                                let scale = p.scale;
                                let center = to_screen(p.sim.pos);
                                let w = img.width() as f32 * scale * zoom;
                                let h = img.height() as f32 * scale * zoom;
                                let p1 = [center[0] - w / 2.0, center[1] - h / 2.0];
                                let p2 = [center[0] + w / 2.0, center[1] + h / 2.0];
                                draw_bg.add_image(tex, p1.into(), p2.into());
                            }
                        }
                    }
                }
            }
        }

        // --- 5. Collision Debug ---
        // --- 5. Collision Debug ---
        if self.show_collision_debug {
            // Original AABB Debug
            let center = to_screen(self.player.pos);
            let half_w = (self.player.body.width / 2.0) * zoom;
            let half_h = (self.player.body.height / 2.0) * zoom;
            let off_x = self.player.body.collision_offset[0] * zoom;
            let off_y = self.player.body.collision_offset[1] * zoom;

            let bx1 = center[0] + off_x - half_w;
            let by1 = center[1] + off_y - half_h;
            let bx2 = center[0] + off_x + half_w;
            let by2 = center[1] + off_y + half_h;

            draw_bg.add_rect([bx1, by1], [bx2, by2], [1.0, 0.0, 0.0, 0.5], false); // Made AABB semi-transparent
            draw_bg.add_circle(center, 3.0, [1.0, 1.0, 0.0, 1.0], true);
        }

        // --- 6. Render Preview Overlay ---
        // Uses self.preview state
        if self.enable_preview && self.preview.selected_folder_index >= 0 {
            if let Some(folder) = self
                .discovered_folders
                .get(self.preview.selected_folder_index as usize)
            {
                if let Some(images) = &self.image_list {
                    let mut frames: Vec<_> = images
                        .keys()
                        .filter(|k| k.starts_with(&folder.name))
                        .collect();
                    frames.sort_by_key(|k| {
                        k.split('/')
                            .last()
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0)
                    });

                    if !frames.is_empty() {
                        let fps = if self.preview.fps_override > 0.0 {
                            self.preview.fps_override
                        } else {
                            12.0
                        };
                        let speed = self.preview.speed_override;
                        let elapsed = if let Some(start) = self.animation_start {
                            start.elapsed().as_secs_f32()
                        } else {
                            0.0
                        };

                        let idx = if self.preview.is_playing {
                            (elapsed * fps * speed) as usize % frames.len()
                        } else {
                            0 // or manual
                        };
                        let key = frames[idx];

                        // Preview Window
                        let px = 20.0;
                        let py = 200.0;
                        let size = 150.0;

                        draw_fg.add_rect(
                            [px, py].into(),
                            [px + size, py + size].into(),
                            [0.0, 0.0, 0.0, 0.5],
                            true,
                        );
                        draw_fg.add_text(
                            [px, py - 20.0].into(),
                            [1.0, 1.0, 1.0, 1.0],
                            RStr::from_str(&format!("Preview: {}", folder.name)),
                        );

                        if let Some(tex) = self
                            .texture_cache
                            .get(key)
                            .and_then(|t| t.clone().into_option())
                        {
                            if let Some(img) = images.get(key) {
                                let scale = self.preview.zoom;
                                let w = img.width() as f32 * scale;
                                let h = img.height() as f32 * scale;

                                let cx = px + size / 2.0 + self.preview.pan[0];
                                let cy = py + size / 2.0 + self.preview.pan[1];

                                let p1 = [cx - w / 2.0, cy - h / 2.0];
                                let p2 = [cx + w / 2.0, cy + h / 2.0];

                                draw_fg.add_image(tex, p1.into(), p2.into());
                            }
                        }

                        if self.preview.show_collision_box {
                            let cx = px + size / 2.0 + self.preview.pan[0];
                            let cy = py + size / 2.0 + self.preview.pan[1];

                            let half_w = (self.player.body.width / 2.0) * self.preview.zoom;
                            let half_h = (self.player.body.height / 2.0) * self.preview.zoom;
                            let off_x = self.player.body.collision_offset[0] * self.preview.zoom;
                            let off_y = self.player.body.collision_offset[1] * self.preview.zoom;

                            let bx1 = cx + off_x - half_w;
                            let by1 = cy + off_y - half_h;
                            let bx2 = cx + off_x + half_w;
                            let by2 = cy + off_y + half_h;

                            draw_fg.add_rect([bx1, by1], [bx2, by2], [1.0, 0.0, 0.0, 1.0], false);
                            draw_fg.add_circle([cx, cy], 3.0, [1.0, 1.0, 0.0, 1.0], true);
                        }
                    }
                }
            }
        }
    }
}

/// The callback to create the module instance.
pub extern "C" fn create_hack() -> StableHaCK_TO<'static, RBox<()>> {
    StableHaCK_TO::from_value(RpgClientHaCK::new(), TD_Opaque)
}

/// Export the root module.
#[export_root_module]
pub fn get_library() -> HackersModule_Ref {
    HackersModule { create_hack }.leak_into_prefix()
}
