// animation_utils.rs - Complete file with extended animation system

use std::collections::HashMap;

use log::{debug, info};

use crate::sprites::{discover_sprite_folders, DiscoveredFolder, GameFileLoader};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitType {
    Player,
    NPC,
    Object,
    Missile,
    Item,
    Tile,
    Unused,
}

/// Animation configuration - describes how to animate a series of frames
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationConfig {
    pub name: String,
    pub folder_path: String,
    pub frame_count: usize,
    pub fps: f32,
}

impl AnimationConfig {
    pub fn new(
        name: impl Into<String>,
        folder: impl Into<String>,
        frame_count: usize,
        fps: f32,
    ) -> Self {
        Self {
            name: name.into(),
            folder_path: folder.into(),
            frame_count,
            fps,
        }
    }

    pub fn get_frame_index(&self, elapsed_seconds: f32) -> usize {
        let frame = (elapsed_seconds * self.fps) as usize;
        frame % self.frame_count
    }

    pub fn get_frame_name(&self, frame_index: usize) -> String {
        format!("{}/{}", self.folder_path, frame_index)
    }

    pub fn get_current_frame_name(&self, elapsed_seconds: f32) -> String {
        self.get_frame_name(self.get_frame_index(elapsed_seconds))
    }
}

/// Unit animation modes from the game engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum UnitAnimationMode {
    Death = 0,
    Stand = 1,
    Walk = 2,
    BeingHit = 3,
    Attack1 = 4,
    Attack2 = 5,
    Block = 6,
    Cast = 7,
    UseSkill1 = 8,
    UseSkill2 = 9,
    UseSkill3 = 10,
    UseSkill4 = 11,
    Dead = 12,
    BeingKnockback = 13,
    Sequence = 14,
    Run = 15,
}

const ALL_ANIMATION_MODES: [UnitAnimationMode; 16] = [
    UnitAnimationMode::Death,
    UnitAnimationMode::Stand,
    UnitAnimationMode::Walk,
    UnitAnimationMode::BeingHit,
    UnitAnimationMode::Attack1,
    UnitAnimationMode::Attack2,
    UnitAnimationMode::Block,
    UnitAnimationMode::Cast,
    UnitAnimationMode::UseSkill1,
    UnitAnimationMode::UseSkill2,
    UnitAnimationMode::UseSkill3,
    UnitAnimationMode::UseSkill4,
    UnitAnimationMode::Dead,
    UnitAnimationMode::BeingKnockback,
    UnitAnimationMode::Sequence,
    UnitAnimationMode::Run,
];

/// Trait for types that can be registered in the animation system
pub trait SpriteDiscoverable {
    /// Get the unique ID for this sprite
    fn sprite_id(&self) -> u32;

    /// Get the folder name to search for (e.g., "FrozenOrb", "gloam")
    fn sprite_folder_name(&self) -> &'static str;

    /// Get the unit type
    fn unit_type(&self) -> UnitType;

    /// Optional: Get default animation settings if no sprite.json exists
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        None
    }
}

#[derive(Clone)]
pub struct DefaultSpriteConfig {
    pub frame_count: usize,
    pub fps: f32,
}

impl UnitAnimationMode {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Death),
            1 => Some(Self::Stand),
            2 => Some(Self::Walk),
            3 => Some(Self::BeingHit),
            4 => Some(Self::Attack1),
            5 => Some(Self::Attack2),
            6 => Some(Self::Block),
            7 => Some(Self::Cast),
            8 => Some(Self::UseSkill1),
            9 => Some(Self::UseSkill2),
            10 => Some(Self::UseSkill3),
            11 => Some(Self::UseSkill4),
            12 => Some(Self::Dead),
            13 => Some(Self::BeingKnockback),
            14 => Some(Self::Sequence),
            15 => Some(Self::Run),
            _ => None,
        }
    }
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Death),
            1 => Some(Self::Stand),
            2 => Some(Self::Walk),
            3 => Some(Self::BeingHit),
            4 => Some(Self::Attack1),
            5 => Some(Self::Attack2),
            6 => Some(Self::Block),
            7 => Some(Self::Cast),
            8 => Some(Self::UseSkill1),
            9 => Some(Self::UseSkill2),
            10 => Some(Self::UseSkill3),
            11 => Some(Self::UseSkill4),
            12 => Some(Self::Dead),
            13 => Some(Self::BeingKnockback),
            14 => Some(Self::Sequence),
            15 => Some(Self::Run),
            _ => None,
        }
    }

    pub fn is_animated(&self) -> bool {
        matches!(
            self,
            Self::Stand
                | Self::Walk
                | Self::Run
                | Self::Attack1
                | Self::Attack2
                | Self::Cast
                | Self::UseSkill1
                | Self::UseSkill2
                | Self::UseSkill3
                | Self::UseSkill4
                | Self::BeingKnockback
                | Self::BeingHit
        )
    }

    pub fn is_static(&self) -> bool {
        matches!(self, Self::Block | Self::Dead)
    }

    pub fn is_transition(&self) -> bool {
        matches!(self, Self::Death)
    }

    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Self::Walk => 1.0,
            Self::Run => 1.5,
            Self::Attack1 | Self::Attack2 => 1.5,
            Self::Cast => 1.3,
            Self::UseSkill1 | Self::UseSkill2 | Self::UseSkill3 | Self::UseSkill4 => 1.2,
            _ => 1.0,
        }
    }
}

/// Calculate the frame index based on animation mode behavior
pub fn calculate_frame_index(
    mode: &UnitAnimationMode,
    config: &AnimationConfig,
    elapsed_time: f32,
) -> usize {
    let result = if mode.is_static() {
        match mode {
            UnitAnimationMode::Dead => config.frame_count.saturating_sub(1),
            _ => 0,
        }
    } else if mode.is_transition() {
        let speed = mode.speed_multiplier();
        let adjusted_time = elapsed_time * speed;
        let frame = (adjusted_time * config.fps) as usize;
        frame.min(config.frame_count.saturating_sub(1))
    } else {
        let speed = mode.speed_multiplier();
        let adjusted_time = elapsed_time * speed;
        config.get_frame_index(adjusted_time)
    };
    // debug!("calculate_frame_index: mode={:?}, elapsed={}, fps={}, frame_count={}, result={}",
    //     mode, elapsed_time, config.fps, config.frame_count, result);
    result
}

/// Specification for loading a unit's animations
#[derive(Clone, Debug, PartialEq)]
pub struct UnitAnimationSpec {
    pub unit_id: u32,
    pub unit_type: UnitType,
    pub mode_folders: HashMap<UnitAnimationMode, AnimationConfig>,
}

impl UnitAnimationSpec {
    /// Get the animation config for a specific mode
    pub fn get_config(&self, mode: UnitAnimationMode) -> Option<&AnimationConfig> {
        self.mode_folders.get(&mode)
    }

    /// Get the current frame name for this unit in a specific mode
    pub fn get_frame_name(&self, mode: UnitAnimationMode, elapsed_time: f32) -> Option<String> {
        let config = self.get_config(mode)?;
        let frame_index = calculate_frame_index(&mode, config, elapsed_time);
        Some(config.get_frame_name(frame_index))
    }
}

/// Registry mapping unit types to their complete animation specifications
#[derive(Clone, PartialEq)]
pub struct UnitAnimationRegistry {
    monsters: HashMap<u32, UnitAnimationSpec>,
    missiles: HashMap<u32, UnitAnimationSpec>,
    players: HashMap<u32, UnitAnimationSpec>,
    objects: HashMap<u32, UnitAnimationSpec>,
    // Add more as needed
}

impl UnitAnimationRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            monsters: HashMap::new(),
            missiles: HashMap::new(),
            players: HashMap::new(),
            objects: HashMap::new(),
        };
        registry.register_all();
        registry
    }
    pub fn from_discovered<T>(
        items: &[T],
        search_paths: &[String],
        game_loader: Option<&dyn GameFileLoader>,
    ) -> Self
    where
        T: SpriteDiscoverable,
    {
        let mut registry = Self::new();

        // Discover all folders in search paths
        let discovered_folders = discover_sprite_folders(search_paths);

        // Build a quick lookup: folder_name -> DiscoveredFolder
        let folder_map: HashMap<String, &DiscoveredFolder> = discovered_folders
            .iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        // Register each item
        for item in items {
            let folder_name = item.sprite_folder_name();

            // Skip if no folder name specified
            if folder_name.is_empty() {
                continue;
            }

            // Try to find discovered folder
            if let Some(discovered) = folder_map.get(folder_name) {
                registry.register_from_discovered(item, discovered);
            } else if let Some(default_config) = item.default_config() {
                // Fall back to default config if no folder found
                registry.register_from_default(item, &default_config);
            }
        }

        registry
    }

    fn register_from_discovered<T: SpriteDiscoverable>(
        &mut self,
        item: &T,
        discovered: &DiscoveredFolder,
    ) {
        let mut mode_folders = HashMap::new();

        if discovered.is_modular() {
            self.register_modular(item, discovered, &mut mode_folders);
        } else {
            self.register_legacy(item, discovered, &mut mode_folders);
        }

        let spec = UnitAnimationSpec {
            unit_id: item.sprite_id(),
            unit_type: item.unit_type(),
            mode_folders,
        };

        // Insert based on unit_type
        match item.unit_type() {
            UnitType::Missile => {
                self.missiles.insert(item.sprite_id(), spec);
            }
            UnitType::NPC => {
                self.monsters.insert(item.sprite_id(), spec);
            }
            UnitType::Player => {
                self.players.insert(item.sprite_id(), spec);
            }
            UnitType::Object => {
                self.objects.insert(item.sprite_id(), spec);
            }
            _ => {}
        }
    }

    fn register_modular<T: SpriteDiscoverable>(
        &self,
        item: &T,
        discovered: &DiscoveredFolder,
        mode_folders: &mut HashMap<UnitAnimationMode, AnimationConfig>,
    ) {
        if let Some(files) = &discovered.files {
            for mode in ALL_ANIMATION_MODES {
                let mode_name = format!("{:?}", mode);

                if let Some(filename) = discovered.get_file_for_mode(&mode_name) {
                    if let Some(file_info) = discovered.get_file_info(filename) {
                        let config = AnimationConfig {
                            name: format!("{}_{:?}", item.sprite_folder_name(), mode),
                            folder_path: format!(
                                "{}/{}",
                                discovered.name,
                                std::path::Path::new(filename)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(filename)
                            ),
                            frame_count: file_info.frame_count,
                            fps: file_info.fps.unwrap_or(15.0),
                        };
                        mode_folders.insert(mode, config);
                    }
                }
            }
        }

        if mode_folders.is_empty() {
            self.register_legacy(item, discovered, mode_folders);
        }
    }

    fn register_legacy<T: SpriteDiscoverable>(
        &self,
        item: &T,
        discovered: &DiscoveredFolder,
        mode_folders: &mut HashMap<UnitAnimationMode, AnimationConfig>,
    ) {
        if let Some(frame_count) = discovered.frame_count {
            let config = AnimationConfig {
                name: item.sprite_folder_name().to_string(),
                folder_path: discovered.name.clone(),
                frame_count,
                fps: discovered.fps.unwrap_or(15.0),
            };

            for mode in ALL_ANIMATION_MODES {
                mode_folders.insert(mode, config.clone());
            }
        }
    }

    fn register_from_default<T: SpriteDiscoverable>(
        &mut self,
        item: &T,
        default_config: &DefaultSpriteConfig,
    ) {
        let mut mode_folders = HashMap::new();

        let config = AnimationConfig {
            name: item.sprite_folder_name().to_string(),
            folder_path: item.sprite_folder_name().to_string(),
            frame_count: default_config.frame_count,
            fps: default_config.fps,
        };

        for mode in ALL_ANIMATION_MODES {
            mode_folders.insert(mode, config.clone());
        }

        let spec = UnitAnimationSpec {
            unit_id: item.sprite_id(),
            unit_type: item.unit_type(),
            mode_folders,
        };

        match item.unit_type() {
            UnitType::Missile => {
                self.missiles.insert(item.sprite_id(), spec);
            }
            UnitType::NPC => {
                self.monsters.insert(item.sprite_id(), spec);
            }
            UnitType::Player => {
                self.players.insert(item.sprite_id(), spec);
            }
            UnitType::Object => {
                self.objects.insert(item.sprite_id(), spec);
            }
            _ => {}
        }
    }

    // Lookup methods
    pub fn get_monster_spec(&self, unit_id: u32) -> Option<&UnitAnimationSpec> {
        self.monsters.get(&unit_id)
    }

    pub fn get_missile_spec(&self, unit_id: u32) -> Option<&UnitAnimationSpec> {
        self.missiles.get(&unit_id)
    }

    pub fn get_player_spec(&self, unit_id: u32) -> Option<&UnitAnimationSpec> {
        self.players.get(&unit_id)
    }

    pub fn get_object_spec(&self, unit_id: u32) -> Option<&UnitAnimationSpec> {
        self.objects.get(&unit_id)
    }

    // Generic lookup by UnitType
    pub fn get_spec(&self, unit_type: UnitType, unit_id: u32) -> Option<&UnitAnimationSpec> {
        match unit_type {
            UnitType::NPC => self.get_monster_spec(unit_id),
            UnitType::Missile => self.get_missile_spec(unit_id),
            UnitType::Player => self.get_player_spec(unit_id),
            UnitType::Object => self.get_object_spec(unit_id),
            _ => None,
        }
    }

    fn register_all(&mut self) {
        // ===== MONSTERS WITH SINGLE SPRITE FOR ALL MODES =====

        // Gloam - uses same sprite for all modes
        let gloam_ids = vec![118, 119, 120, 121, 639, 640, 641, 733];
        for id in gloam_ids {
            self.register_monster_simple(id, "gloam", 2, 3.0);
        }

        // Fetish - uses same sprite for all modes
        let chucky_ids = vec![
            141, 142, 143, 144, 145, 212, 213, 214, 215, 216, 396, 397, 398, 399, 400, 407, 656,
            657, 658, 659, 660, 661, 690, 691,
        ];
        for id in chucky_ids {
            self.register_monster_simple(id, "chucky", 3, 4.0);
        }

        // ===== MONSTERS WITH MODE-SPECIFIC SPRITES =====

        // Example: Diablo with different sprites per mode
        // self.register_monster_with_modes(58, vec![
        //     (UnitAnimationMode::Stand, "diablo/stand", 1, 1.0),
        //     (UnitAnimationMode::Walk, "diablo/walk", 8, 8.0),
        //     (UnitAnimationMode::Attack1, "diablo/attack1", 12, 10.0),
        //     (UnitAnimationMode::Death, "diablo/death", 10, 8.0),
        // ]);

        // ===== MISSILES =====
        let frozenorb_ids = vec![
            260,
            // 261,
            // 262,
            // 263
        ];
        for id in frozenorb_ids {
            self.register_missile_simple(id, "FrozenOrb", 14, 15.0)
        }

        // self.register_missile_simple(
        //     260,
        //     "FrozenOrb",
        //     14,
        //     15.0
        // );
        // Example: Fireball missile
        // self.register_missile_simple(0, "fireball", 16, 24.0);

        // Example: Lightning bolt with travel/hit modes
        // self.register_missile_with_modes(1, vec![
        //     (MissileMode::Travel, "lightning/travel", 8, 16.0),
        //     (MissileMode::Hit, "lightning/hit", 6, 12.0),
        // ]);
    }

    // ===== MONSTER REGISTRATION HELPERS =====

    /// Register a monster that uses the same sprite folder for all animation modes
    fn register_monster_simple(
        &mut self,
        unit_type: u32,
        folder: &str,
        frame_count: usize,
        fps: f32,
    ) {
        let mut mode_folders = HashMap::new();

        let all_modes = [
            UnitAnimationMode::Death,
            UnitAnimationMode::Stand,
            UnitAnimationMode::Walk,
            UnitAnimationMode::BeingHit,
            UnitAnimationMode::Attack1,
            UnitAnimationMode::Attack2,
            UnitAnimationMode::Block,
            UnitAnimationMode::Cast,
            UnitAnimationMode::UseSkill1,
            UnitAnimationMode::UseSkill2,
            UnitAnimationMode::UseSkill3,
            UnitAnimationMode::UseSkill4,
            UnitAnimationMode::Dead,
            UnitAnimationMode::BeingKnockback,
            UnitAnimationMode::Sequence,
            UnitAnimationMode::Run,
        ];

        let config =
            AnimationConfig::new(format!("Monster_{}", unit_type), folder, frame_count, fps);

        for mode in all_modes {
            mode_folders.insert(mode, config.clone());
        }

        self.monsters.insert(
            unit_type,
            UnitAnimationSpec {
                unit_id: unit_type,
                unit_type: UnitType::NPC,
                mode_folders,
            },
        );
    }

    /// Register a monster with mode-specific sprite folders
    fn register_monster_with_modes(
        &mut self,
        unit_type: u32,
        modes: Vec<(UnitAnimationMode, &str, usize, f32)>,
    ) {
        let mut mode_folders = HashMap::new();

        for (mode, folder, frame_count, fps) in modes {
            let config = AnimationConfig::new(
                format!("Monster_{}_{:?}", unit_type, mode),
                folder,
                frame_count,
                fps,
            );
            mode_folders.insert(mode, config);
        }

        self.monsters.insert(
            unit_type,
            UnitAnimationSpec {
                unit_id: unit_type,
                unit_type: UnitType::NPC,
                mode_folders,
            },
        );
    }

    // ===== MISSILE REGISTRATION HELPERS =====

    pub fn register_missile_simple(
        &mut self,
        missile_id: u32,
        folder_name: &str,
        frame_count: usize,
        fps: f32,
    ) {
        let full_path = folder_name.to_string();

        let mut mode_folders = HashMap::new();

        // Create the animation config once
        let config = AnimationConfig {
            name: format!("missile_{}", missile_id),
            folder_path: full_path,
            frame_count,
            fps,
        };

        // Register the SAME animation for ALL modes
        let all_modes = [
            UnitAnimationMode::Death,
            UnitAnimationMode::Stand,
            UnitAnimationMode::Walk,
            UnitAnimationMode::BeingHit,
            UnitAnimationMode::Attack1,
            UnitAnimationMode::Attack2,
            UnitAnimationMode::Block,
            UnitAnimationMode::Cast,
            UnitAnimationMode::UseSkill1,
            UnitAnimationMode::UseSkill2,
            UnitAnimationMode::UseSkill3,
            UnitAnimationMode::UseSkill4,
            UnitAnimationMode::Dead,
            UnitAnimationMode::BeingKnockback,
            UnitAnimationMode::Sequence,
            UnitAnimationMode::Run,
        ];

        // Insert the same config for every mode
        for mode in all_modes {
            mode_folders.insert(mode, config.clone());
        }

        let spec = UnitAnimationSpec {
            unit_id: missile_id,
            unit_type: UnitType::Missile,
            mode_folders,
        };

        self.missiles.insert(missile_id, spec);
    }

    pub fn register_missile_with_modes(
        &mut self,
        missile_id: u32,
        modes: Vec<(UnitAnimationMode, &str, usize, f32)>,
    ) {
        let mut mode_folders = HashMap::new();

        for (mode, folder_name, frame_count, fps) in modes {
            // Store folder name directly (search path provides base directory)
            let config = AnimationConfig {
                name: format!("Missile_{}_{:?}", missile_id, mode),
                folder_path: folder_name.to_string(),
                frame_count,
                fps,
            };
            mode_folders.insert(mode, config);
        }

        self.missiles.insert(
            missile_id,
            UnitAnimationSpec {
                unit_id: missile_id,
                unit_type: UnitType::Missile,
                mode_folders,
            },
        );
    }

    /// Get the frame name for a monster at a specific time
    pub fn get_monster_frame(
        &self,
        unit_type: u32,
        mode: UnitAnimationMode,
        elapsed_time: f32,
    ) -> Option<String> {
        self.get_monster_spec(unit_type)?
            .get_frame_name(mode, elapsed_time)
    }

    /// Get the frame name for a missile at a specific time
    pub fn get_missile_frame(
        &self,
        missile_type: u32,
        mode: UnitAnimationMode,
        elapsed_time: f32,
    ) -> Option<String> {
        self.get_missile_spec(missile_type)?
            .get_frame_name(mode, elapsed_time)
    }

    /// Get all folders that need to be loaded for this registry
    pub fn get_all_folders_to_load(&self) -> Vec<(String, usize)> {
        let mut folders = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Collect all unique folders from monsters
        for spec in self.monsters.values() {
            for config in spec.mode_folders.values() {
                let folder_key = format!("{}:{}", config.folder_path, config.frame_count);
                if seen.insert(folder_key) {
                    folders.push((config.folder_path.clone(), config.frame_count));
                }
            }
        }

        // Collect all unique folders from missiles
        for spec in self.missiles.values() {
            for config in spec.mode_folders.values() {
                let folder_key = format!("{}:{}", config.folder_path, config.frame_count);
                if seen.insert(folder_key) {
                    folders.push((config.folder_path.clone(), config.frame_count));
                }
            }
        }

        folders
    }
}

/// Get the current animation frame name for a unit
/// Combines unit state with animation config to produce the full frame name
pub fn get_current_frame_name_for_unit(
    animation_mode: u8,
    anim_config: &AnimationConfig,
    elapsed_time: f32,
) -> String {
    let anim_mode =
        UnitAnimationMode::from_u8(animation_mode).unwrap_or(UnitAnimationMode::BeingHit);

    let frame_index = calculate_frame_index(&anim_mode, anim_config, elapsed_time);
    anim_config.get_frame_name(frame_index)
}

/// Helper to get elapsed time from an optional Instant
pub fn get_elapsed_seconds(start: Option<std::time::Instant>) -> f32 {
    start.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0)
}
