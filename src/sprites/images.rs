use image::DynamicImage;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Configuration for progressive loading
#[derive(Debug, Clone, PartialEq)]
pub struct LoadingConfig {
    /// Maximum time to spend loading per frame (in milliseconds)
    pub max_frame_time_ms: u64,
    /// How many images to try loading per batch
    pub batch_size: usize,
}

impl Default for LoadingConfig {
    fn default() -> Self {
        Self {
            max_frame_time_ms: 16, // ~60 FPS budget
            batch_size: 10,
        }
    }
}

/// State machine for progressive image loading
#[derive(Debug, Clone, PartialEq)]
pub struct ImageLoadingState {
    /// Folders we need to load
    pending_folders: Vec<DiscoveredFolder>,
    /// Current folder being loaded
    current_folder_index: usize,
    /// Current frame within folder
    current_frame_index: usize,
    /// Images loaded so far
    loaded_images: HashMap<String, DynamicImage>,
    /// Configuration
    config: LoadingConfig,
    /// Paths to search
    search_paths: Vec<String>,
    /// Game loader
    game_loader_available: bool,
    /// Total progress
    total_frames: usize,
    frames_loaded: usize,
}

impl ImageLoadingState {
    pub fn new(
        discovered_folders: Vec<DiscoveredFolder>,
        search_paths: Vec<String>,
        config: LoadingConfig,
    ) -> Self {
        let total_frames = discovered_folders
            .iter()
            .map(|f| f.frame_count.unwrap_or(0))
            .sum();

        Self {
            pending_folders: discovered_folders,
            current_folder_index: 0,
            current_frame_index: 0,
            loaded_images: HashMap::new(),
            config,
            search_paths,
            game_loader_available: true,
            total_frames,
            frames_loaded: 0,
        }
    }

    /// Check if loading is complete
    pub fn is_complete(&self) -> bool {
        self.current_folder_index >= self.pending_folders.len()
    }

    /// Get loading progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_frames == 0 {
            1.0
        } else {
            self.frames_loaded as f32 / self.total_frames as f32
        }
    }

    /// Get current loading status message
    pub fn status_message(&self) -> String {
        if self.is_complete() {
            format!(
                "Loading complete: {} images loaded",
                self.loaded_images.len()
            )
        } else if let Some(folder) = self.pending_folders.get(self.current_folder_index) {
            format!(
                "Loading {}: frame {}/{} ({:.1}%)",
                folder.name,
                self.current_frame_index,
                folder.frame_count.unwrap_or(0),
                self.progress() * 100.0
            )
        } else {
            "Initializing...".to_string()
        }
    }

    /// Load a chunk of images (call this each frame)
    /// Returns true if more work remains
    pub fn load_chunk(&mut self, game_loader: Option<&dyn GameFileLoader>) -> bool {
        if self.is_complete() {
            return false;
        }

        let start_time = Instant::now();
        let frame_budget = Duration::from_millis(self.config.max_frame_time_ms);
        let mut images_this_batch = 0;

        while !self.is_complete()
            && images_this_batch < self.config.batch_size
            && start_time.elapsed() < frame_budget
        {
            // ---- Extract folder pointer safely ----
            let (frame_count, is_modular, folder_ptr) = {
                let folder = &self.pending_folders[self.current_folder_index];
                (
                    folder.frame_count.unwrap_or(0),
                    folder.is_modular(),
                    folder as *const DiscoveredFolder,
                )
            };
            // ---- immutable borrow ends HERE ----

            if self.current_frame_index >= frame_count {
                self.current_folder_index += 1;
                self.current_frame_index = 0;
                continue;
            }

            // Now reborrow safely
            let folder: &DiscoveredFolder = unsafe { &*folder_ptr };

            if is_modular {
                self.load_modular_frame(folder, game_loader);
            } else {
                self.load_legacy_frame(folder, game_loader);
            }

            self.current_frame_index += 1;
            self.frames_loaded += 1;
            images_this_batch += 1;
        }

        !self.is_complete()
    }

    fn load_legacy_frame(
        &mut self,
        folder: &DiscoveredFolder,
        game_loader: Option<&dyn GameFileLoader>,
    ) {
        let frame_idx = self.current_frame_index;
        let frame_name = format!("{}/{}", folder.name, frame_idx);

        for base_path in &self.search_paths {
            // Try game-specific format (DC6, etc.)
            if let Some(loader) = game_loader {
                let game_path =
                    format!("{}/{}.{}", base_path, folder.name, loader.file_extension());
                if let Ok(data) = std::fs::read(&game_path) {
                    if let Ok(img) = loader.load_frame(&data, frame_idx) {
                        self.loaded_images.insert(frame_name, img);
                        return;
                    }
                }
            }

            // Try PNG sheet
            let sheet_path = format!("{}/{}/sheet.png", base_path, folder.name);
            if let Ok(data) = std::fs::read(&sheet_path) {
                if let Ok(sheet_img) = image::load_from_memory(&data) {
                    if let Some(layout) = &folder.layout {
                        let frame_count = folder.frame_count.unwrap_or(1);
                        if let Some(frame) =
                            layout.extract_frame(&sheet_img, frame_idx, frame_count)
                        {
                            self.loaded_images.insert(frame_name, frame);
                            return;
                        }
                    }
                }
            }

            // Try individual PNG frame
            let png_path = format!("{}/{}/{}.png", base_path, folder.name, frame_idx);
            if let Ok(data) = std::fs::read(&png_path) {
                if let Ok(img) = image::load_from_memory(&data) {
                    self.loaded_images.insert(frame_name, img);
                    return;
                }
            }
        }
    }

    fn load_modular_frame(
        &mut self,
        folder: &DiscoveredFolder,
        _game_loader: Option<&dyn GameFileLoader>,
    ) {
        if let Some(files) = &folder.files {
            if let Some(file_info) = files.get(self.current_frame_index) {
                for base_path in &self.search_paths {
                    let file_path = format!("{}/{}/{}", base_path, folder.name, file_info.filename);

                    if let Ok(data) = std::fs::read(&file_path) {
                        if let Ok(sheet_img) = image::load_from_memory(&data) {
                            let layout = file_info
                                .layout
                                .as_ref()
                                .unwrap_or(&SpriteSheetLayout::HorizontalStrip);

                            let prefix = std::path::Path::new(&file_info.filename)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&file_info.filename);

                            // NEW: Handle cell_indices if specified
                            if let Some(cell_indices) = &file_info.cell_indices {
                                // Load ONLY the specified cells
                                let total_cells =
                                    cell_indices.iter().max().map(|m| m + 1).unwrap_or(0);

                                for (output_frame_idx, &cell_idx) in cell_indices.iter().enumerate()
                                {
                                    if let Some(frame) =
                                        layout.extract_frame(&sheet_img, cell_idx, total_cells)
                                    {
                                        let frame_name = format!(
                                            "{}/{}/{}",
                                            folder.name, prefix, output_frame_idx
                                        );
                                        self.loaded_images.insert(frame_name, frame);
                                    }
                                }
                            } else {
                                // Load all frames sequentially (existing behavior)
                                for frame_idx in 0..file_info.frame_count {
                                    if let Some(frame) = layout.extract_frame(
                                        &sheet_img,
                                        frame_idx,
                                        file_info.frame_count,
                                    ) {
                                        let frame_name =
                                            format!("{}/{}/{}", folder.name, prefix, frame_idx);
                                        self.loaded_images.insert(frame_name, frame);
                                    }
                                }
                            }

                            break;
                        }
                    }
                }
            }
        }
    }

    /// Get the currently loaded images (can be called during loading)
    pub fn get_images(&self) -> &HashMap<String, DynamicImage> {
        &self.loaded_images
    }

    /// Consume and return final image map
    pub fn into_images(self) -> HashMap<String, DynamicImage> {
        self.loaded_images
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredFolder {
    pub name: String,
    pub source_type: SourceType,

    // For single-file mode (legacy)
    pub frame_count: Option<usize>,
    pub layout: Option<SpriteSheetLayout>,
    pub fps: Option<f32>,
    pub speed: Option<f32>,

    // For modular mode (new)
    pub files: Option<Vec<DiscoveredFile>>,
    pub mode_mapping: Option<HashMap<String, String>>,

    // Shared
    pub mode_overrides: Option<ModeOverrides>,
    pub animation_mode: Option<AnimationMode>,
    pub behaviors: Option<HashMap<String, AnimationBehavior>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredFile {
    pub filename: String,
    pub frame_count: usize,
    pub layout: Option<SpriteSheetLayout>,
    pub fps: Option<f32>,
    pub speed: Option<f32>,
    pub cell_indices: Option<Vec<usize>>,
    pub collision_data: Option<HashMap<usize, Vec<[f32; 2]>>>,
}

impl DiscoveredFolder {
    /// Check if this folder uses modular file mode
    pub fn is_modular(&self) -> bool {
        self.files.is_some()
    }

    /// Get the filename for a specific mode
    pub fn get_file_for_mode(&self, mode_name: &str) -> Option<&str> {
        if let Some(mapping) = &self.mode_mapping {
            mapping.get(mode_name).map(|s| s.as_str())
        } else {
            // Convention fallback
            if let Some(files) = &self.files {
                let convention_name = format!("{}.png", mode_name);
                for file in files {
                    if file.filename == convention_name {
                        return Some(&file.filename);
                    }
                }
            }
            None
        }
    }

    /// Get file info for a specific filename
    pub fn get_file_info(&self, filename: &str) -> Option<&DiscoveredFile> {
        self.files.as_ref()?.iter().find(|f| f.filename == filename)
    }
}

pub fn discover_folder_contents(
    folder_path: &std::path::PathBuf,
    config: Option<SpriteConfig>,
) -> DiscoveredFolder {
    let folder_name = folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    if let Some(ref cfg) = config {
        if cfg.is_modular() {
            let mut discovered_files = Vec::new();

            if let Some(files) = &cfg.files {
                for (filename, file_config) in files {
                    let file_path = folder_path.join(filename);

                    if file_path.exists() {
                        // Determine frame count
                        let frame_count = if let Some(indices) = &file_config.cell_indices {
                            // Use number of specified indices as frame count
                            indices.len()
                        } else {
                            file_config
                                .frame_count
                                .or_else(|| count_frames_in_file(&file_path))
                                .unwrap_or(0)
                        };

                        if frame_count > 0 {
                            let layout_str =
                                file_config.layout.as_ref().or(cfg.default_layout.as_ref());
                            let layout = layout_str
                                .and_then(|l| parse_layout_string(l, file_config.grid_columns));

                            let fps = file_config.fps.or(cfg.default_fps);
                            let speed = file_config.speed.or(cfg.default_speed);

                            discovered_files.push(DiscoveredFile {
                                filename: filename.clone(),
                                frame_count,
                                layout,
                                fps,
                                speed,
                                cell_indices: file_config.cell_indices.clone(),
                                collision_data: file_config.collision_data.clone(),
                            });
                        }
                    }
                }
            }

            // Calculate total frame count for modular folders
            let total_frame_count: usize = discovered_files.iter().map(|f| f.frame_count).sum();

            return DiscoveredFolder {
                name: folder_name,
                source_type: SourceType::PNGFrames,
                frame_count: Some(total_frame_count),
                layout: None,
                fps: None,
                speed: None,
                files: Some(discovered_files),
                mode_mapping: cfg.mode_mapping.clone(),
                mode_overrides: cfg.mode_overrides.clone(),
                animation_mode: cfg.animation_mode.clone(),
                behaviors: cfg.behaviors.clone(),
            };
        }
    }

    // Legacy single-file mode (unchanged)
    let layout = if let Some(ref c) = config {
        c.layout
            .as_ref()
            .and_then(|l| parse_layout_string(l, c.grid_columns))
    } else {
        None
    };

    DiscoveredFolder {
        name: folder_name,
        source_type: SourceType::PNGFrames,
        frame_count: config.as_ref().and_then(|c| c.frame_count),
        layout,
        fps: config.as_ref().and_then(|c| c.fps),
        speed: config.as_ref().and_then(|c| c.speed),
        files: None,
        mode_mapping: None,
        mode_overrides: config.as_ref().and_then(|c| c.mode_overrides.clone()),
        animation_mode: config.and_then(|c| c.animation_mode),
        behaviors: None,
    }
}
fn parse_layout_string(layout: &str, grid_columns: Option<usize>) -> Option<SpriteSheetLayout> {
    match layout.to_lowercase().as_str() {
        "horizontal" => Some(SpriteSheetLayout::HorizontalStrip),
        "vertical" => Some(SpriteSheetLayout::VerticalStrip),
        "grid" => grid_columns.map(|cols| SpriteSheetLayout::Grid { columns: cols }),
        _ => None,
    }
}

fn count_frames_in_file(file_path: &std::path::PathBuf) -> Option<usize> {
    // Try to detect if it's a DC6 file
    if file_path.extension()? == "dc6" {
        if let Ok(data) = std::fs::read(file_path) {
            return get_dc6_frame_count(&data);
        }
    }

    // For sheets, we need the config to know the frame count
    // For individual frame files (0.png, 1.png), count them
    if let Some(parent) = file_path.parent() {
        return Some(count_png_frames(&parent.to_path_buf()));
    }

    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceType {
    DC6,
    PNGSheet,
    PNGFrames,
    Embedded,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileConfig {
    #[serde(default)]
    pub frame_count: Option<usize>,
    #[serde(default)]
    pub fps: Option<f32>,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub grid_columns: Option<usize>,
    #[serde(default)]
    pub cell_indices: Option<Vec<usize>>,
    #[serde(default)]
    pub collision_data: Option<HashMap<usize, Vec<[f32; 2]>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AnimationMode {
    /// State-based animation (direct frame mapping)
    StateBased {
        #[serde(rename = "state_mapping")]
        mapping: StateMapping,
    },
    /// Time-based animation (current behavior, default)
    TimeBased {
        #[serde(default)]
        fps: Option<f32>,
        #[serde(default)]
        looping: Option<bool>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StateMapping {
    /// Direct 1:1 mapping (value 0-100 maps to frame 0-100)
    Direct,
    /// Percentage (value 0.0-1.0 maps to frame range)
    Percentage,
    /// Custom ranges (e.g., 0-25% = frames 0-5, 26-50% = frames 6-10)
    Ranges(Vec<StateRange>),
    /// Automatic 4-way directional (divides frames equally into Up/Left/Down/Right)
    #[serde(rename = "directional_4way")]
    Directional4Way,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StateRange {
    pub min_value: f32,
    pub max_value: f32,
    pub start_frame: usize,
    pub end_frame: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpriteConfig {
    // ===== GLOBAL DEFAULTS (apply to all files if not overridden) =====
    #[serde(default)]
    pub default_fps: Option<f32>,
    #[serde(default)]
    pub default_speed: Option<f32>,
    #[serde(default)]
    pub default_layout: Option<String>,

    // ===== LEGACY SINGLE-FILE MODE (backward compatible) =====
    #[serde(default)]
    pub frame_count: Option<usize>,
    #[serde(default)]
    pub fps: Option<f32>,
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub grid_columns: Option<usize>,

    // ===== MODULAR MULTI-FILE MODE (new) =====
    /// File definitions - maps filename to its properties
    #[serde(default)]
    pub files: Option<HashMap<String, FileConfig>>,

    /// Mode mapping - maps animation mode names to filenames
    #[serde(default)]
    pub mode_mapping: Option<HashMap<String, String>>,

    // ===== PER-MODE OVERRIDES (legacy, for single-file mode) =====
    #[serde(default)]
    pub mode_overrides: Option<ModeOverrides>,
    #[serde(default)]
    pub animation_mode: Option<AnimationMode>,
    #[serde(default)]
    pub transitions: Option<HashMap<String, TransitionConfig>>,
    #[serde(default)]
    pub behaviors: Option<HashMap<String, AnimationBehavior>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameModifier {
    /// Multiplier for frame duration. 1.0 = normal, 2.0 = 2x duration (slower), 0.5 = 0.5x duration (faster).
    #[serde(default)]
    pub duration_scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeedScaling {
    /// Linear increase in playback speed per second.
    /// e.g. 0.5 means speed increases by 50% every second.
    #[serde(default)]
    pub ramp_per_second: f32,
    /// Maximum playback speed multiplier.
    #[serde(default)]
    pub max_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationBehavior {
    #[serde(default)]
    pub loop_range: Option<(usize, usize)>,
    #[serde(default)]
    pub force_loop: bool,
    #[serde(default)]
    pub frame_modifiers: Option<HashMap<usize, FrameModifier>>,
    #[serde(default)]
    pub speed_scaling: Option<SpeedScaling>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Number of frames to skip at start of animation
    #[serde(default)]
    pub skip_frames: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModeOverrides {
    #[serde(default)]
    pub walk: Option<ModeConfig>,
    #[serde(default)]
    pub run: Option<ModeConfig>,
    #[serde(default)]
    pub attack1: Option<ModeConfig>,
    #[serde(default)]
    pub attack2: Option<ModeConfig>,
    #[serde(default)]
    pub cast: Option<ModeConfig>,
    #[serde(default)]
    pub death: Option<ModeConfig>,
    #[serde(default)]
    pub stand: Option<ModeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModeConfig {
    #[serde(default)]
    pub fps: Option<f32>,
    #[serde(default)]
    pub speed: Option<f32>,
}

impl SpriteConfig {
    /// Check if this config uses modular file mode
    pub fn is_modular(&self) -> bool {
        self.files.is_some()
    }

    /// Get the filename for a specific animation mode
    /// Returns None if not in modular mode or mapping doesn't exist
    pub fn get_file_for_mode(&self, mode_name: &str) -> Option<&str> {
        self.mode_mapping.as_ref().and_then(|mapping| {
            // Try exact match first
            if let Some(s) = mapping.get(mode_name) {
                return Some(s.as_str());
            }
            // Try lowercase match
            let lower = mode_name.to_lowercase();
            mapping.get(&lower).map(|s| s.as_str())
        })
    }

    /// Get the file configuration for a specific filename
    pub fn get_file_config(&self, filename: &str) -> Option<&FileConfig> {
        self.files.as_ref().and_then(|files| files.get(filename))
    }

    /// Get frame count for a specific file, with fallback to defaults
    pub fn get_frame_count_for_file(&self, filename: &str) -> Option<usize> {
        if let Some(file_config) = self.get_file_config(filename) {
            if file_config.frame_count.is_some() {
                return file_config.frame_count;
            }
        }
        // Fall back to legacy single-file frame_count
        self.frame_count
    }

    /// Get FPS for a specific file, with fallback chain
    pub fn get_fps_for_file(&self, filename: &str) -> Option<f32> {
        // 1. Check file-specific config
        if let Some(file_config) = self.get_file_config(filename) {
            if file_config.fps.is_some() {
                return file_config.fps;
            }
        }
        // 2. Fall back to default_fps
        if self.default_fps.is_some() {
            return self.default_fps;
        }
        // 3. Fall back to legacy fps
        self.fps
    }

    /// Get speed for a specific file, with fallback chain
    pub fn get_speed_for_file(&self, filename: &str) -> f32 {
        // 1. Check file-specific config
        if let Some(file_config) = self.get_file_config(filename) {
            if let Some(speed) = file_config.speed {
                return speed;
            }
        }
        // 2. Fall back to default_speed
        if let Some(speed) = self.default_speed {
            return speed;
        }
        // 3. Fall back to legacy speed
        if let Some(speed) = self.speed {
            return speed;
        }
        // 4. Ultimate fallback
        1.0
    }

    /// Get layout for a specific file
    pub fn get_layout_for_file(&self, filename: &str) -> Option<String> {
        // 1. Check file-specific config
        if let Some(file_config) = self.get_file_config(filename) {
            if file_config.layout.is_some() {
                return file_config.layout.clone();
            }
        }
        // 2. Fall back to default_layout
        if self.default_layout.is_some() {
            return self.default_layout.clone();
        }
        // 3. Fall back to legacy layout
        self.layout.clone()
    }

    /// Get grid columns for a specific file
    pub fn get_grid_columns_for_file(&self, filename: &str) -> Option<usize> {
        if let Some(file_config) = self.get_file_config(filename) {
            if file_config.grid_columns.is_some() {
                return file_config.grid_columns;
            }
        }
        self.grid_columns
    }

    // ===== LEGACY METHODS (for backward compatibility) =====

    /// Get effective FPS for a mode (legacy single-file mode)
    pub fn get_fps_for_mode(&self, mode_name: &str) -> Option<f32> {
        // In modular mode, redirect to file-based lookup
        if self.is_modular() {
            if let Some(filename) = self.get_file_for_mode(mode_name) {
                return self.get_fps_for_file(filename);
            }
            // Convention fallback: try "{mode}.png"
            let convention_filename = format!("{}.png", mode_name);
            return self.get_fps_for_file(&convention_filename);
        }

        // Legacy single-file mode with mode_overrides
        if let Some(overrides) = &self.mode_overrides {
            let mode_config = match mode_name.to_lowercase().as_str() {
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
                if config.fps.is_some() {
                    return config.fps;
                }
            }
        }
        self.fps
    }

    /// Get effective speed multiplier for a mode (legacy single-file mode)
    pub fn get_speed_for_mode(&self, mode_name: &str) -> f32 {
        // In modular mode, redirect to file-based lookup
        if self.is_modular() {
            if let Some(filename) = self.get_file_for_mode(mode_name) {
                return self.get_speed_for_file(filename);
            }
            // Convention fallback
            let convention_filename = format!("{}.png", mode_name);
            return self.get_speed_for_file(&convention_filename);
        }

        // Legacy single-file mode with mode_overrides
        if let Some(overrides) = &self.mode_overrides {
            let mode_config = match mode_name.to_lowercase().as_str() {
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
                if config.speed.is_some() {
                    return config.speed.unwrap();
                }
            }
        }
        self.speed.unwrap_or(1.0)
    }

    /// Get all filenames defined in this config
    pub fn get_all_filenames(&self) -> Vec<String> {
        if let Some(files) = &self.files {
            files.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

pub enum ImageCategory {
    Counter,
    BuffIcon,
    Sent,
    Elements,
    Monsters,
    Missiles,
    MonsterUI,
}

/// Trait for loading game-specific file formats (DC6, DDS, etc.)
pub trait GameFileLoader {
    fn load_frame(&self, data: &[u8], frame_index: usize) -> Result<DynamicImage, String>;
    fn file_extension(&self) -> &str;
}

/// Sprite sheet layout types
#[derive(Clone, Debug, PartialEq)]
pub enum SpriteSheetLayout {
    /// Horizontal strip (all frames in one row)
    HorizontalStrip,
    /// Vertical strip (all frames in one column)
    VerticalStrip,
    /// Grid layout (specify columns, rows calculated from frame_count)
    Grid { columns: usize },
}

impl SpriteSheetLayout {
    /// Extract a specific frame from a sprite sheet
    pub fn extract_frame(
        &self,
        sheet: &DynamicImage,
        cell_index: usize,
        total_cells: usize,
    ) -> Option<DynamicImage> {
        match self {
            SpriteSheetLayout::HorizontalStrip => {
                let sheet_width = sheet.width();
                let sheet_height = sheet.height();
                let frame_width = sheet_width / total_cells as u32;

                if frame_width > 0 && cell_index < total_cells {
                    let x = cell_index as u32 * frame_width;
                    Some(sheet.crop_imm(x, 0, frame_width, sheet_height))
                } else {
                    None
                }
            }

            SpriteSheetLayout::VerticalStrip => {
                let sheet_width = sheet.width();
                let sheet_height = sheet.height();
                let frame_height = sheet_height / total_cells as u32;

                if frame_height > 0 && cell_index < total_cells {
                    let y = cell_index as u32 * frame_height;
                    Some(sheet.crop_imm(0, y, sheet_width, frame_height))
                } else {
                    None
                }
            }

            SpriteSheetLayout::Grid { columns } => {
                let sheet_width = sheet.width();
                let sheet_height = sheet.height();

                let rows = (total_cells + columns - 1) / columns;

                if cell_index < total_cells {
                    let col = cell_index % columns;
                    let row = cell_index / columns;

                    let x = (col as u32 * sheet_width) / *columns as u32;
                    let y = (row as u32 * sheet_height) / rows as u32;

                    let x_end = ((col as u32 + 1) * sheet_width) / *columns as u32;
                    let y_end = ((row as u32 + 1) * sheet_height) / rows as u32;

                    let frame_width = x_end - x;
                    let frame_height = y_end - y;

                    Some(sheet.crop_imm(x, y, frame_width, frame_height))
                } else {
                    None
                }
            }
        }
    }
}

/// Load images with user override support
/// Priority: custom_paths > default_paths, discovered folders > registry defaults
pub fn load_images_with_overrides(
    custom_paths: &[String],
    default_paths: &[String],
    embedded_category: Option<ImageCategory>,
    registry_folders: &[(String, usize)],
    game_loader: Option<&dyn GameFileLoader>,
) -> HashMap<String, DynamicImage> {
    let mut images = HashMap::new();

    // Combine paths with custom first (for priority)
    let mut all_paths = custom_paths.to_vec();
    all_paths.extend_from_slice(default_paths);

    // Load embedded images
    if let Some(category) = embedded_category {
        let embedded = ImageLoader::load_category(category);
        info!("Loaded {} embedded images", embedded.len());
        images.extend(embedded);
    }

    // Discover filesystem folders
    let discovered = discover_sprite_folders(&all_paths);
    info!("Discovered {} folders from filesystem", discovered.len());

    // Load discovered folders (handles both modular and legacy)
    for folder in &discovered {
        if folder.is_modular() {
            info!("Loading modular folder: {}", folder.name);
            let modular_images =
                load_all_frames_modular(&all_paths, &folder.name, folder, game_loader);
            info!("  Loaded {} modular frames", modular_images.len());
            images.extend(modular_images);
        } else {
            if let Some(frame_count) = folder.frame_count {
                info!(
                    "Loading legacy folder: {} ({} frames)",
                    folder.name, frame_count
                );
                let legacy_images = ImageLoader::load_frames(
                    &all_paths,
                    &folder.name,
                    frame_count,
                    game_loader,
                    folder.layout.clone(),
                );
                info!("  Loaded {} legacy frames", legacy_images.len());
                images.extend(legacy_images);
            }
        }
    }

    // Load registry defaults (only for folders NOT already discovered)
    let discovered_names: std::collections::HashSet<String> =
        discovered.iter().map(|f| f.name.clone()).collect();

    let default_folders: Vec<_> = registry_folders
        .iter()
        .filter(|(name, _)| !discovered_names.contains(name))
        .cloned()
        .collect();

    if !default_folders.is_empty() {
        info!(
            "Loading {} default folders (not overridden)",
            default_folders.len()
        );
        let default_images = ImageLoader::load_folders(&all_paths, &default_folders, game_loader);
        info!("  Loaded {} default frames", default_images.len());
        images.extend(default_images);
    }

    info!("Total images loaded: {}", images.len());
    images
}

/// Load ALL frames from a modular folder (all files, all modes)
pub fn load_all_frames_modular(
    base_paths: &[String],
    folder: &str,
    discovered: &DiscoveredFolder,
    game_loader: Option<&dyn GameFileLoader>,
) -> HashMap<String, DynamicImage> {
    let mut all_images = HashMap::new();

    if !discovered.is_modular() {
        if let Some(frame_count) = discovered.frame_count {
            return ImageLoader::load_frames(
                base_paths,
                folder,
                frame_count,
                game_loader,
                discovered.layout.clone(),
            );
        }
        return all_images;
    }

    if let Some(files) = &discovered.files {
        for file_info in files {
            for base_path in base_paths {
                let file_path = format!("{}/{}/{}", base_path, folder, file_info.filename);

                if let Ok(data) = std::fs::read(&file_path) {
                    if let Ok(sheet_img) = image::load_from_memory(&data) {
                        let layout = file_info
                            .layout
                            .as_ref()
                            .unwrap_or(&SpriteSheetLayout::HorizontalStrip);

                        let prefix = std::path::Path::new(&file_info.filename)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&file_info.filename);

                        // NEW: Handle cell_indices
                        if let Some(cell_indices) = &file_info.cell_indices {
                            let total_cells = cell_indices.iter().max().map(|m| m + 1).unwrap_or(0);

                            for (output_frame_idx, &cell_idx) in cell_indices.iter().enumerate() {
                                if let Some(frame) =
                                    layout.extract_frame(&sheet_img, cell_idx, total_cells)
                                {
                                    let frame_name =
                                        format!("{}/{}/{}", folder, prefix, output_frame_idx);
                                    all_images.insert(frame_name, frame);
                                }
                            }
                        } else {
                            // Sequential loading (existing behavior)
                            for frame_idx in 0..file_info.frame_count {
                                if let Some(frame) = layout.extract_frame(
                                    &sheet_img,
                                    frame_idx,
                                    file_info.frame_count,
                                ) {
                                    let frame_name = format!("{}/{}/{}", folder, prefix, frame_idx);
                                    all_images.insert(frame_name, frame);
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    all_images
}

pub fn get_frame_for_state(
    frame_count: usize,
    animation_mode: Option<&AnimationMode>,
    state_value: f32,
    animation_progress: Option<f32>, // NEW: 0.0-1.0 for animating within the range
) -> usize {
    let Some(AnimationMode::StateBased { mapping }) = animation_mode else {
        // Not a state-based animation, return first frame
        return 0;
    };

    if frame_count == 0 {
        return 0;
    }

    match mapping {
        StateMapping::Direct => {
            // Direct mapping: value 73 -> frame 73
            (state_value as usize).min(frame_count - 1)
        }
        StateMapping::Percentage => {
            // Percentage: 0.0-1.0 maps to full frame range
            let clamped = state_value.clamp(0.0, 1.0);
            ((clamped * (frame_count - 1) as f32).round() as usize).min(frame_count - 1)
        }
        StateMapping::Directional4Way => {
            // Auto-divide frames into 4 equal directional ranges
            // 0.0-0.25 = Up, 0.25-0.50 = Left, 0.50-0.75 = Down, 0.75-1.00 = Right
            let frames_per_direction = frame_count / 4;
            if frames_per_direction == 0 {
                return 0;
            }

            // Determine which direction based on state_value
            let direction_index = ((state_value.clamp(0.0, 0.999) * 4.0) as usize).min(3);
            let direction_start = direction_index * frames_per_direction;
            let direction_end = direction_start + frames_per_direction - 1;

            // If animation_progress is provided, cycle through frames in this direction
            if let Some(progress) = animation_progress {
                let frame_offset = (progress as usize) % frames_per_direction;
                return (direction_start + frame_offset).min(frame_count - 1);
            }

            // Otherwise, return first frame of the direction
            direction_start.min(frame_count - 1)
        }
        StateMapping::Ranges(ranges) => {
            // Find which range the value falls into
            for range in ranges {
                if state_value >= range.min_value && state_value <= range.max_value {
                    let frame_span = range.end_frame.saturating_sub(range.start_frame) + 1;

                    // If animation_progress is provided, use it to cycle through frames in the range
                    if let Some(progress) = animation_progress {
                        // progress is a frame number (e.g., 0.0, 1.5, 2.3, etc.)
                        // Convert to frame offset within this range
                        let frame_offset = (progress as usize) % frame_span;
                        return (range.start_frame + frame_offset)
                            .min(range.end_frame)
                            .min(frame_count - 1);
                    }

                    // Otherwise, map state_value position within range to a frame
                    let range_span = range.max_value - range.min_value;
                    if range_span == 0.0 {
                        return range.start_frame.min(frame_count - 1);
                    }
                    let range_progress = (state_value - range.min_value) / range_span;
                    let frame_offset = (range_progress * (frame_span - 1) as f32).round() as usize;
                    return (range.start_frame + frame_offset)
                        .min(range.end_frame)
                        .min(frame_count - 1);
                }
            }
            // Fallback if no range matches
            0
        }
    }
}

macro_rules! define_images {
    (
        $(
            $category:ident: $base_path:literal => [
                $( $name:literal => $file:literal ),* $(,)?
            ]
        ),* $(,)?
    ) => {
        pub struct ImageLoader;

    impl ImageLoader {
        /// Load sprite configuration from a folder
        pub fn load_sprite_config(folder_path: &std::path::PathBuf) -> Option<SpriteConfig> {
            let config_path = folder_path.join("sprite.json");
            if config_path.exists() {
                 if let Ok(file) = std::fs::File::open(config_path) {
                     if let Ok(config) = serde_json::from_reader(file) {
                         return Some(config);
                     }
                 }
            }
            None
        }
    }

        impl ImageLoader {
            /// Load a single embedded image by name
            pub fn load_image(name: &str) -> Option<DynamicImage> {
                let data = Self::get_image_data(name)?;
                image::load_from_memory(data).ok()
            }

            /// Load multiple embedded images by names
            pub fn load_images(names: &[&str]) -> HashMap<String, DynamicImage> {
                names
                    .iter()
                    .filter_map(|&name| {
                        Self::load_image(name).map(|img| (name.to_string(), img))
                    })
                    .collect()
            }

            /// Load all embedded images in a category
            pub fn load_category(category: ImageCategory) -> HashMap<String, DynamicImage> {
                let names = Self::get_category_names(category);
                Self::load_images(&names)
            }

            /// Load animation frames from disk - JUST loads the pixels
            /// Caller provides folder names and frame counts
            /// No FPS, no timing logic, just: "load folder X with N frames"
            pub fn load_frames(
                base_paths: &[String],
                folder: &str,
                frame_count: usize,
                game_loader: Option<&dyn GameFileLoader>,
                sprite_layout: Option<SpriteSheetLayout>,
            ) -> HashMap<String, DynamicImage> {
                let mut images = HashMap::new();

                for frame_idx in 0..frame_count {
                    let frame_name = format!("{}/{}", folder, frame_idx);

                    for base_path in base_paths {
                        // Priority 1: Game-specific format (DC6, DDS, etc.)
                        if let Some(loader) = game_loader {
                            let game_path = format!("{}/{}.{}", base_path, folder, loader.file_extension());
                            if let Ok(data) = std::fs::read(&game_path) {
                                if let Ok(img) = loader.load_frame(&data, frame_idx) {
                                    images.insert(frame_name.clone(), img);
                                    break;
                                }
                            }
                        }

                        // Priority 2: PNG sprite sheet
                        let sheet_path = format!("{}/{}/sheet.png", base_path, folder);
                        if let Ok(data) = std::fs::read(&sheet_path) {
                            if let Ok(sheet_img) = image::load_from_memory(&data) {
                                // Use provided layout or default to horizontal strip
                                let layout = sprite_layout.as_ref()
                                    .unwrap_or(&SpriteSheetLayout::HorizontalStrip);

                                if let Some(frame) = layout.extract_frame(&sheet_img, frame_idx, frame_count) {
                                    images.insert(frame_name.clone(), frame);
                                    break;
                                }
                            }
                        }

                        // Priority 3: Individual PNG frames
                        let png_path = format!("{}/{}/{}.png", base_path, folder, frame_idx);
                        if let Ok(data) = std::fs::read(&png_path) {
                            if let Ok(img) = image::load_from_memory(&data) {
                                images.insert(frame_name.clone(), img);
                                break;
                            }
                        }
                    }
                }

                images
            }

            /// Convenience: Load multiple folders at once
            pub fn load_folders(
                base_paths: &[String],
                folders: &[(String, usize)], // (folder_name, frame_count)
                game_loader: Option<&dyn GameFileLoader>
            ) -> HashMap<String, DynamicImage> {
                let mut all_images = HashMap::new();

                for (folder, frame_count) in folders {
                    let images = Self::load_frames(base_paths, folder, *frame_count, game_loader, None);
                    all_images.extend(images);
                }

                all_images
            }

            /// Load folders with custom sprite sheet layouts
            pub fn load_folders_with_layouts(
                base_paths: &[String],
                folders: &[(String, usize, Option<SpriteSheetLayout>)], // (folder, frame_count, layout)
                game_loader: Option<&dyn GameFileLoader>
            ) -> HashMap<String, DynamicImage> {
                let mut all_images = HashMap::new();

                for (folder, frame_count, layout) in folders {
                    let images = Self::load_frames(
                        base_paths,
                        folder,
                        *frame_count,
                        game_loader,
                        layout.clone()
                    );
                    all_images.extend(images);
                }

                all_images
            }

            fn get_image_data(name: &str) -> Option<&'static [u8]> {
                match name {
                    $(
                        $(
                            $name => Some(include_bytes!(concat!($base_path, $file))),
                        )*
                    )*
                    _ => None,
                }
            }

            pub fn get_category_names(category: ImageCategory) -> Vec<&'static str> {
                match category {
                    $(
                        ImageCategory::$category => vec![$( $name ),*],
                    )*
                }
            }

            fn all_image_names() -> Vec<&'static str> {
                let mut names = Vec::new();
                $(
                    names.extend(Self::get_category_names(ImageCategory::$category));
                )*
                names
            }
        }

        fn ensure_collision_data(dir_path: &PathBuf, config: &mut SpriteConfig) -> bool {
            let mut modified = false;

            if let Some(files) = &mut config.files {
                let filenames: Vec<String> = files.keys().cloned().collect();

                for filename in filenames {
                    // Temporarily take the config out to avoid double borrow (if needed, but here we can just target)
                    if let Some(file_config) = files.get_mut(&filename) {
                        let mut needs_generation = file_config.collision_data.is_none();
                        if !needs_generation {
                            if let Some(data) = &file_config.collision_data {
                                if data.values().any(|v| v.len() < 3) {
                                    needs_generation = true;
                                }
                            }
                        }

                        if needs_generation {
                            let file_path = dir_path.join(&filename);
                            if let Ok(data) = std::fs::read(&file_path) {
                                if let Ok(sheet) = image::load_from_memory(&data) {
                                    let layout_str = file_config
                                        .layout
                                        .as_ref()
                                        .or(config.default_layout.as_ref());
                                    let cols = file_config.grid_columns.or(config.grid_columns);

                                    let layout = layout_str
                                        .and_then(|l| parse_layout_string(l, cols))
                                        .unwrap_or(SpriteSheetLayout::HorizontalStrip);

                                    let mut generated_map = HashMap::new();

                                    if let Some(indices) = &file_config.cell_indices {
                                        let max_cell =
                                            indices.iter().max().map(|x| x + 1).unwrap_or(0);
                                        for (out_idx, &cell_idx) in indices.iter().enumerate() {
                                            if let Some(frame) =
                                                layout.extract_frame(&sheet, cell_idx, max_cell)
                                            {
                                                if let Some(poly) =
                                                    generate_collision_polygon(&frame)
                                                {
                                                    generated_map.insert(out_idx, poly);
                                                }
                                            }
                                        }
                                    } else {
                                        let count = file_config.frame_count.unwrap_or(1);

                                        for i in 0..count {
                                            if let Some(frame) =
                                                layout.extract_frame(&sheet, i, count)
                                            {
                                                if let Some(poly) =
                                                    generate_collision_polygon(&frame)
                                                {
                                                    generated_map.insert(i, poly);
                                                }
                                            }
                                        }
                                    }

                                    if !generated_map.is_empty() {
                                        file_config.collision_data = Some(generated_map);
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            modified
        }

        pub fn load_sprite_config(dir_path: &PathBuf) -> Option<SpriteConfig> {
            let config_path = dir_path.join("sprite.json");
            if !config_path.exists() {
                return None;
            }

            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<SpriteConfig>(&content) {
                        Ok(mut config) => {
                            // Ensure collision data logic
                            if ensure_collision_data(dir_path, &mut config) {
                                // Save back
                                if let Ok(new_content) = serde_json::to_string_pretty(&config) {
                                    let _ = std::fs::write(&config_path, new_content);
                                    info!(
                                        "Updated sprite.json with generated collision data for {:?}",
                                        dir_path.file_name()
                                    );
                                }
                            }

                            info!("Loaded sprite.json for {:?}", dir_path.file_name());
                            Some(config)
                        }
                        Err(e) => {
                            info!("Failed to parse sprite.json: {}", e);
                            None
                        }
                    }
                }
                Err(_) => None,
            }
        }

        /// Parse layout string from config to SpriteSheetLayout
        pub fn parse_sprite_layout(config: &Option<SpriteConfig>) -> Option<SpriteSheetLayout> {
            config.as_ref().and_then(|c| {
                c.layout.as_ref().and_then(|layout_str| {
                    match layout_str.to_lowercase().as_str() {
                        "horizontal" => Some(SpriteSheetLayout::HorizontalStrip),
                        "vertical" => Some(SpriteSheetLayout::VerticalStrip),
                        "grid" => {
                            if let Some(cols) = c.grid_columns {
                                Some(SpriteSheetLayout::Grid { columns: cols })
                            } else {
                                info!("Grid layout specified but no grid_columns provided");
                                None
                            }
                        }
                        _ => {
                            info!("Unknown layout type: {}", layout_str);
                            None
                        }
                    }
                })
            })
        }

        /// Count numbered PNG frames in a directory (0.png, 1.png, etc.)
        pub fn count_png_frames(dir_path: &PathBuf) -> usize {
            let mut count = 0;

            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("png") {
                        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                            // Check if filename is a number
                            if filename.parse::<usize>().is_ok() {
                                count += 1;
                            }
                        }
                    }
                }
            }

            count
        }

        /// Get frame count from DC6 header
        pub fn get_dc6_frame_count(data: &[u8]) -> Option<usize> {
            if data.len() < 24 {
                return None;
            }

            let directions = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
            let frames_per_direction = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);

            Some((directions * frames_per_direction) as usize)
        }

        /// Discover all sprite folders in given search paths
        pub fn discover_sprite_folders(search_paths: &[String]) -> Vec<DiscoveredFolder> {
            let mut folders = Vec::new();

            // Check embedded images first
            let embedded = ImageLoader::load_category(ImageCategory::Monsters);
            let mut folder_frames: HashMap<String, usize> = HashMap::new();

            for key in embedded.keys() {
                if let Some(folder) = key.split('/').next() {
                    *folder_frames.entry(folder.to_string()).or_insert(0) += 1;
                }
            }

            folders.extend(folder_frames.into_iter().map(|(name, count)| DiscoveredFolder {
                name,
                source_type: SourceType::Embedded,
                frame_count: Some(count),  // Changed to Option
                layout: None,
                fps: None,
                speed: None,
                files: None,  // NEW
                mode_mapping: None,  // NEW
                mode_overrides: None,
                animation_mode: None,
                behaviors: None,
            }));

            // Scan filesystem paths
            for base_path in search_paths {
                folders.extend(scan_path_for_folders(base_path));
            }

            // Deduplicate by name (keep first occurrence)
            let mut seen = std::collections::HashSet::new();
            folders.retain(|f| seen.insert(f.name.clone()));

            folders.sort_by(|a, b| a.name.cmp(&b.name));
            folders
        }

        fn scan_path_for_folders(base_path: &str) -> Vec<DiscoveredFolder> {
            let mut folders = Vec::new();
            let path = PathBuf::from(base_path);

            if !path.exists() {
                return folders;
            }

            // Helper function to process a directory
            let process_dir = |dir_path: &PathBuf, parent_prefix: Option<&str>| -> Vec<DiscoveredFolder> {
                let mut local_folders = Vec::new();

                if let Ok(entries) = std::fs::read_dir(dir_path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();

                        // Check for .dc6 files
                        if entry_path.extension().and_then(|s| s.to_str()) == Some("dc6") {
                            if let Some(folder_name) = entry_path.file_stem().and_then(|s| s.to_str()) {
                                if let Ok(data) = std::fs::read(&entry_path) {
                                    if let Some(frame_count) = get_dc6_frame_count(&data) {
                                        let full_name = if let Some(prefix) = parent_prefix {
                                            format!("{}/{}", prefix, folder_name)
                                        } else {
                                            folder_name.to_string()
                                        };

                                        local_folders.push(DiscoveredFolder {
                                            name: full_name,
                                            source_type: SourceType::DC6,
                                            frame_count: Some(frame_count),
                                            layout: None,
                                            fps: None,
                                            speed: None,
                                            files: None,
                                            mode_mapping: None,
                                            mode_overrides: None,
                                            animation_mode: None,
                                            behaviors: None,
                                        });
                                    }
                                }
                            }
                        }

                        // Check for subdirectories (PNG frames or sheets)
                        if entry_path.is_dir() {
                            if let Some(folder_name) = entry_path.file_name().and_then(|s| s.to_str()) {
                                let config = load_sprite_config(&entry_path);

                                // Use discover_folder_contents to properly handle both legacy and modular configs
                                let mut discovered = discover_folder_contents(&entry_path, config);

                                // Apply parent prefix if needed
                                if let Some(prefix) = parent_prefix {
                                    discovered.name = format!("{}/{}", prefix, discovered.name);
                                }

                                // Only add if it has content (frames or files)
                                if discovered.frame_count.unwrap_or(0) > 0 || discovered.files.is_some() {
                                    local_folders.push(discovered);
                                }
                            }
                        }
                    }
                }

                local_folders
            };

            // Level 1: Process the base directory itself
            folders.extend(process_dir(&path, None));

            // Level 2: Process subdirectories (like "monsters" and "missiles")
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();

                    if entry_path.is_dir() {
                        if let Some(subdir_name) = entry_path.file_name().and_then(|s| s.to_str()) {
                            // Process this subdirectory, passing its name as prefix
                            folders.extend(process_dir(&entry_path, Some(subdir_name)));
                        }
                    }
                }
            }

            folders
        }
    }
    }

// Define embedded images - NO animation configs here!
define_images! {
    Counter: "../assets/images/counter/" => [
        // "TPScroll" => "town_portal_scroll.png",
        // "IDScroll" => "identify_scroll.png",
        // "Key" => "key.png",
    ],

    BuffIcon: "../assets/images/icons/" => [
        // "AmplifyDamage" => "AmplifyDamage.png",
        // "Armageddon" => "Armageddon.png",
        // "Attract" => "Attract.png",
        // "Avoid" => "Avoid.png",
        // "AxeMastery" => "AxeMastery.png",
        // "Barbs" => "Barbs.png",
        // "BattleCommand" => "BattleCommand.png",
        // "BattleOrders" => "BattleOrders.png",
        // "Bear" => "Bear.png",
        // "Berserk" => "Berserk.png",
        // "BladeMastery" => "BladeMastery.png",
        // "BladeShield" => "BladeShield.png",
        // "BladesOfIce" => "BladesOfIce.png",
        // "Blaze" => "Blaze.png",
        // "BlessedAim" => "BlessedAim.png",
        // "BloodMana" => "BloodMana.png",
        // "BoneArmor" => "BoneArmor.png",
        // "ChillingArmor" => "ChillingArmor.png",
        // "ClawMastery" => "ClawMastery.png",
        // "ClawsOfThunder" => "ClawsOfThunder.png",
        // "Cleansing" => "Cleansing.png",
        // "Cloaked" => "Cloaked.png",
        // "CloakOfShadows" => "CloakofShadows.png",
        // "CobraStrike" => "CobraStrike.png",
        // "Cold" => "Cold.png",
        // "ColdMastery" => "ColdMastery.png",
        // "Concentration" => "Concentration.png",
        // "Confuse" => "Confuse.png",
        // "Conversion" => "Conversion.png",
        // "Convicted" => "Convicted.png",
        // "Conviction" => "Conviction.png",
        // "CriticalStrike" => "CriticalStrike.png",
        // "CycloneArmor" => "CycloneArmor.png",
        // "Decoy" => "Decoy.png",
        // "Decrepify" => "Decrepify.png",
        // "DefenseCurse" => "DefenseCurse.png",
        // "Defiance" => "Defiance.png",
        // "DimVision" => "DimVision.png",
        // "Dodge" => "Dodge.png",
        // "Enchant" => "Enchant.png",
        // "EnergyShield" => "EnergyShield.png",
        // "Evade" => "Evade.png",
        // "Fade" => "Fade.png",
        // "Fanaticism" => "Fanaticism.png",
        // "FenrisRage" => "FenrisRage.png",
        // "FeralRage" => "FeralRage.png",
        // "FireMastery" => "FireMastery.png",
        // "FistsOfFire" => "FistsOfFire.png",
        // "Frenzy" => "Frenzy.png",
        // "FrozenArmor" => "FrozenArmor.png",
        // "HolyFire" => "HolyFire.png",
        // "HolyShield" => "HolyShield.png",
        // "HolyShock" => "HolyShock.png",
        // "HolyWind" => "HolyWind.png",
        // "Hurricane" => "Hurricane.png",
        // "Impale" => "Impale.png",
        // "IncreasedSpeed" => "IncreasedSpeed.png",
        // "IncreasedStamina" => "IncreasedStamina.png",
        // "Inferno" => "Inferno.png",
        // "InnerSight" => "InnerSight.png",
        // "IronMaiden" => "IronMaiden.png",
        // "IronSkin" => "IronSkin.png",
        // "LifeTap" => "LifeTap.png",
        // "LightningMastery" => "LightningMastery.png",
        // "LowerResist" => "LowerResist.png",
        // "MaceMastery" => "MaceMastery.png",
        // "Meditation" => "Meditation.png",
        // "Might" => "Might.png",
        // "NaturalResistance" => "NaturalResistance.png",
        // "OakSage" => "OakSage.png",
        // "Penetrate" => "Penetrate.png",
        // "PhoenixStrike" => "PhoenixStrike.png",
        // "Pierce" => "Pierce.png",
        // "Poison" => "Poison.png",
        // "PolearmMastery" => "PolearmMastery.png",
        // "Prayer" => "Prayer.png",
        // "Quickness" => "Quickness.png",
        // "Redemption" => "Redemption.png",
        // "ResistAll" => "ResistAll.png",
        // "ResistCold" => "ResistCold.png",
        // "ResistFire" => "ResistFire.png",
        // "ResistLight" => "ResistLight.png",
        // "Salvation" => "Salvation.png",
        // "Sanctuary" => "Sanctuary.png",
        // "ShadowMaster" => "ShadowMaster.png",
        // "ShadowWarrior" => "ShadowWarrior.png",
        // "ShiverArmor" => "ShiverArmor.png",
        // "Shout" => "Shout.png",
        // "SlowMissiles" => "SlowMissiles.png",
        // "Slowed" => "Slowed.png",
        // "SpearMastery" => "SpearMastery.png",
        // "Stamina" => "Stamina.png",
        // "Teleport" => "Teleport.png",
        // "Terror" => "Terror.png",
        // "Thorns" => "Thorns.png",
        // "ThrowingMastery" => "ThrowingMastery.png",
        // "ThunderStorm" => "ThunderStorm.png",
        // "TigerStrike" => "TigerStrike.png",
        // "Valkyrie" => "Valkyrie.png",
        // "VenomClaws" => "VenomClaws.png",
        // "Vigor" => "Vigor.png",
        // "Warmth" => "Warmth.png",
        // "Weaken" => "Weaken.png",
        // "WeaponBlock" => "WeaponBlock.png",
        // "Whirlwind" => "Whirlwind.png",
        // "Wolf" => "Wolf.png",
        // "Wolverine" => "Wolverine.png",
    ],

    Sent: "../assets/images/sent/" => [
        // "Sent" => "Sent.png",
        // "SentFire" => "SentFire.png",
        // "SentIce" => "SentIce.png",
        // "SentLightning" => "SentLightning.png",
        // "SentPoison" => "SentPoison.png",
        // "SentMagic" => "SentMagic.png",
        // "SentPhysical" => "SentPhysical.png",
        // "SentSfx" => "SentSfx.png",
        // "SentFxTrigger" => "SentFxTrigger.png",
    ],

    Elements: "../assets/images/elements/" => [
        // "ElementFire" => "fire.png",
        // "ElementIce" => "ice.png",
        // "ElementLightning" => "lightning.png",
        // "ElementPoison" => "poison.png",
        // "ElementMagic" => "magic.png",
        // "ElementPhysical" => "physical.png",
        // "ElementFX" => "fx.png",
        // "ElementSFX" => "sfx.png",
    ],

    Monsters: "../assets/images/monsters/" => [
        // "MonsterSuperUnique" => "super_unique.png",
        // "MonsterUnique" => "unique.png",
        // "MonsterChampion" => "champion.png",
        // "MonsterRegular" => "regular.png",
    ],

    Missiles: "../assets/images/missiles/" => [],

    MonsterUI: "../assets/images/monster_ui" => []
}

pub fn generate_collision_polygon(image: &DynamicImage) -> Option<Vec<[f32; 2]>> {
    use image::GenericImageView;
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return None;
    }
    let threshold = 10;
    let mut start_point = None;
    'search: for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y)[3] > threshold {
                start_point = Some((x, y));
                break 'search;
            }
        }
    }
    let start = if let Some(p) = start_point {
        p
    } else {
        return None;
    };

    let mut contour = Vec::new();
    contour.push([start.0 as f32, start.1 as f32]);

    let moore_offsets = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];

    let mut current = start;
    let mut search_start_idx = 0;

    let max_iters = (width * height) as usize * 4;
    let mut iters = 0;

    loop {
        iters += 1;
        if iters > max_iters {
            break;
        }

        let mut found_next = false;
        let mut next_pixel = current;
        let mut found_idx = 0;

        for i in 0..8 {
            let idx = (search_start_idx + i) % 8;
            let off = moore_offsets[idx];
            let nx = current.0 as i32 + off.0;
            let ny = current.1 as i32 + off.1;

            let is_foreground = if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                image.get_pixel(nx as u32, ny as u32)[3] > threshold
            } else {
                false
            };

            if is_foreground {
                found_next = true;
                next_pixel = (nx as u32, ny as u32);
                found_idx = idx;
                break;
            }
        }

        if !found_next {
            break;
        }

        if current == start && contour.len() > 1 {
            let second = contour[1];
            if next_pixel.0 as f32 == second[0] && next_pixel.1 as f32 == second[1] {
                break;
            }
        }

        contour.push([next_pixel.0 as f32, next_pixel.1 as f32]);
        current = next_pixel;
        search_start_idx = (found_idx + 6) % 8;
    }

    Some(simplify_polygon(&contour, 2.0))
}

pub fn simplify_polygon(points: &[[f32; 2]], epsilon: f32) -> Vec<[f32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut dmax = 0.0;
    let mut index = 0;
    let end = points.len() - 1;
    for i in 1..end {
        let d = perpendicular_distance(&points[i], &points[0], &points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }
    let mut result_list = Vec::new();
    if dmax > epsilon {
        let rec_results1 = simplify_polygon(&points[0..=index], epsilon);
        let rec_results2 = simplify_polygon(&points[index..=end], epsilon);
        result_list.extend_from_slice(&rec_results1[..rec_results1.len() - 1]);
        result_list.extend_from_slice(&rec_results2);
    } else {
        result_list.push(points[0]);
        result_list.push(points[end]);
    }
    result_list
}

fn perpendicular_distance(pt: &[f32; 2], line_start: &[f32; 2], line_end: &[f32; 2]) -> f32 {
    let dx = line_end[0] - line_start[0];
    let dy = line_end[1] - line_start[1];
    let numerator = ((dy * pt[0]) - (dx * pt[1]) + (line_end[0] * line_start[1])
        - (line_end[1] * line_start[0]))
        .abs();
    let denominator = (dy * dy + dx * dx).sqrt();
    if denominator == 0.0 {
        return 0.0;
    }
    numerator / denominator
}

#[path = "images_test.rs"]
#[cfg(test)]
mod images_test;
