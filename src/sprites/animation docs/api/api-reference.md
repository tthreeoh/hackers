# API Reference

Complete reference for the Sprite Animation System.

## Table of Contents

- [Traits](#traits)
  - [SpriteDiscoverable](#spritediscoverable)
- [Structs](#structs)
  - [UnitAnimationRegistry](#unitanimationregistry)
  - [UnitAnimationSpec](#unitanimationspec)
  - [AnimationConfig](#animationconfig)
  - [DefaultSpriteConfig](#defaultspriteconfig)
- [Enums](#enums)
  - [UnitAnimationMode](#unitanimationmode)
  - [UnitType](#unittype)
  - [SpriteSheetLayout](#spritesheetlayout)
- [Functions](#functions)
  - [calculate_frame_index](#calculate_frame_index)
  - [get_elapsed_seconds](#get_elapsed_seconds)

---

## Traits

### SpriteDiscoverable

Trait that enables automatic sprite discovery and loading for game entities.

**Module:** `crate::animation`

```rust
pub trait SpriteDiscoverable {
    fn sprite_id(&self) -> u32;
    fn sprite_folder_name(&self) -> &'static str;
    fn unit_type(&self) -> UnitType;
    fn default_config(&self) -> Option<DefaultSpriteConfig>;
}
```

#### Methods

##### `sprite_id() -> u32`

Returns the unique numeric identifier for this entity.

**Returns:** Unique ID as `u32`

**Requirements:**
- Must be unique within the entity's `UnitType`
- Should remain stable across versions
- Typically matches game's internal entity ID

**Example:**
```rust
fn sprite_id(&self) -> u32 {
    self.0 as u32  // Convert enum to ID
}
```

---

##### `sprite_folder_name() -> &'static str`

Returns the folder name containing sprite files for this entity.

**Returns:** Static string reference

**Behavior:**
- System searches for this folder name in configured search paths
- Empty string ("") indicates no sprites for this entity
- Folder name is case-sensitive

**Example:**
```rust
fn sprite_folder_name(&self) -> &'static str {
    match self.0 {
        Missiles::FrozenOrb => "FrozenOrb",
        Missiles::Lightning => "Lightning",
        _ => ""  // No sprites
    }
}
```

**Best Practices:**
- Use PascalCase
- Make names descriptive
- Avoid special characters

---

##### `unit_type() -> UnitType`

Specifies the category of this entity.

**Returns:** `UnitType` enum value

**Purpose:**
- Determines which registry storage to use
- Affects search path priorities
- Groups related entities

**Example:**
```rust
fn unit_type(&self) -> UnitType {
    UnitType::Missile
}
```

---

##### `default_config() -> Option<DefaultSpriteConfig>`

Provides fallback configuration when sprite.json is missing or invalid.

**Returns:** `Option<DefaultSpriteConfig>`
- `Some(config)` - Use this configuration as fallback
- `None` - Must have sprite.json or will fail

**Example:**
```rust
fn default_config(&self) -> Option<DefaultSpriteConfig> {
    Some(DefaultSpriteConfig {
        frame_count: 14,
        fps: 15.0,
    })
}
```

**When to return None:**
- Entity uses DC6 files (frame count in file header)
- Configuration must always come from sprite.json
- Entity should fail explicitly if files missing

---

## Structs

### UnitAnimationRegistry

Central registry storing animation specifications for all entities.

**Module:** `crate::animation`

```rust
pub struct UnitAnimationRegistry {
    monsters: HashMap<u32, UnitAnimationSpec>,
    missiles: HashMap<u32, UnitAnimationSpec>,
    players: HashMap<u32, UnitAnimationSpec>,
    objects: HashMap<u32, UnitAnimationSpec>,
}
```

#### Methods

##### `from_discovered()`

Creates a new registry by discovering and loading sprites.

**Signature:**
```rust
pub fn from_discovered<T: SpriteDiscoverable>(
    entities: &[T],
    search_paths: &[String],
    dc6_loader: Option<&DC6Loader>
) -> Self
```

**Parameters:**
- `entities: &[T]` - Slice of entities implementing `SpriteDiscoverable`
- `search_paths: &[String]` - Paths to search for sprite folders (checked in order)
- `dc6_loader: Option<&DC6Loader>` - Optional DC6 file loader

**Returns:** `UnitAnimationRegistry` with loaded sprites

**Behavior:**
1. Iterates through all entities
2. For each entity, searches paths for matching folder
3. Loads sprite.json or uses default config
4. Builds AnimationSpec for each animation mode
5. Stores in appropriate HashMap by unit type

**Example:**
```rust
let missiles = Missiles::iter()
    .map(MissileWrapper)
    .collect::<Vec<_>>();

let registry = UnitAnimationRegistry::from_discovered(
    &missiles,
    &["assets/images/missiles".to_string()],
    Some(&dc6_loader)
);
```

---

##### `get_missile_frame()`

Gets the current frame name for a missile at given time and mode.

**Signature:**
```rust
pub fn get_missile_frame(
    &self,
    missile_id: u32,
    mode: UnitAnimationMode,
    elapsed_time: f32
) -> Option<String>
```

**Parameters:**
- `missile_id: u32` - Missile's unique ID
- `mode: UnitAnimationMode` - Current animation mode
- `elapsed_time: f32` - Time in seconds since animation started

**Returns:** `Option<String>` - Frame name in format `"FolderName/frame_index"`, or `None` if not found

**Example:**
```rust
let frame = registry.get_missile_frame(
    260,
    UnitAnimationMode::Walk,
    1.5
);
// Returns: Some("FrozenOrb/22") (assuming 15 FPS)
```

---

##### `get_monster_frame()`

Gets the current frame name for a monster.

**Signature:**
```rust
pub fn get_monster_frame(
    &self,
    monster_id: u32,
    mode: UnitAnimationMode,
    elapsed_time: f32
) -> Option<String>
```

**Parameters:** Same as `get_missile_frame()`

---

##### `get_player_frame()`

Gets the current frame name for a player character.

**Signature:**
```rust
pub fn get_player_frame(
    &self,
    player_id: u32,
    mode: UnitAnimationMode,
    elapsed_time: f32
) -> Option<String>
```

**Parameters:** Same as `get_missile_frame()`

---

##### `get_object_frame()`

Gets the current frame name for an object.

**Signature:**
```rust
pub fn get_object_frame(
    &self,
    object_id: u32,
    mode: UnitAnimationMode,
    elapsed_time: f32
) -> Option<String>
```

**Parameters:** Same as `get_missile_frame()`

---

##### `has_missile()`, `has_monster()`, etc.

Check if entity has loaded sprites.

**Signature:**
```rust
pub fn has_missile(&self, id: u32) -> bool
pub fn has_monster(&self, id: u32) -> bool
pub fn has_player(&self, id: u32) -> bool
pub fn has_object(&self, id: u32) -> bool
```

**Returns:** `true` if entity found in registry

**Example:**
```rust
if registry.has_missile(260) {
    let frame = registry.get_missile_frame(260, mode, time);
}
```

---

### UnitAnimationSpec

Complete animation specification for a single entity.

**Module:** `crate::animation`

```rust
pub struct UnitAnimationSpec {
    pub folder_name: String,
    pub animations: HashMap<UnitAnimationMode, AnimationConfig>,
}
```

#### Fields

- `folder_name: String` - Name of the sprite folder
- `animations: HashMap<UnitAnimationMode, AnimationConfig>` - Map of mode to configuration

#### Methods

##### `get_config()`

Gets the animation configuration for a specific mode.

**Signature:**
```rust
pub fn get_config(&self, mode: UnitAnimationMode) -> Option<&AnimationConfig>
```

**Returns:** Reference to `AnimationConfig` or `None` if mode not configured

---

##### `get_frame_at_time()`

Calculates which frame should be displayed at given time.

**Signature:**
```rust
pub fn get_frame_at_time(
    &self,
    mode: UnitAnimationMode,
    elapsed_time: f32
) -> Option<usize>
```

**Parameters:**
- `mode: UnitAnimationMode` - Animation mode
- `elapsed_time: f32` - Time in seconds

**Returns:** Frame index or `None` if mode not found

---

### AnimationConfig

Configuration for a single animation mode.

**Module:** `crate::animation`

```rust
pub struct AnimationConfig {
    pub frame_count: usize,
    pub fps: f32,
    pub speed: f32,
    pub layout: SpriteSheetLayout,
    pub file_name: Option<String>,
}
```

#### Fields

- `frame_count: usize` - Total number of frames
- `fps: f32` - Frames per second playback rate
- `speed: f32` - Speed multiplier (1.0 = normal, 2.0 = 2x speed)
- `layout: SpriteSheetLayout` - How frames are arranged in sprite sheet
- `file_name: Option<String>` - Specific file for modular mode, `None` for single animation

#### Methods

##### `frame_duration()`

Calculates duration of one frame in seconds.

**Signature:**
```rust
pub fn frame_duration(&self) -> f32
```

**Returns:** Duration as `f32`

**Formula:** `1.0 / (fps * speed)`

**Example:**
```rust
let config = AnimationConfig {
    fps: 15.0,
    speed: 1.5,
    // ...
};
let duration = config.frame_duration();
// Returns: 0.0444... (1.0 / 22.5)
```

---

### DefaultSpriteConfig

Simple fallback configuration.

**Module:** `crate::animation`

```rust
pub struct DefaultSpriteConfig {
    pub frame_count: usize,
    pub fps: f32,
}
```

#### Fields

- `frame_count: usize` - Number of frames
- `fps: f32` - Frames per second

**Usage:** Returned by `SpriteDiscoverable::default_config()`

---

## Enums

### UnitAnimationMode

All available animation modes.

**Module:** `crate::animation`

```rust
#[repr(u8)]
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
```

#### Variants

| Mode | ID | Type | Speed | Description |
|------|-----|------|-------|-------------|
| `Death` | 0 | Transition | 1.0x | Death animation (plays once) |
| `Stand` | 1 | Looping | 1.0x | Idle stance |
| `Walk` | 2 | Looping | 1.0x | Walking movement |
| `BeingHit` | 3 | Animated | 1.0x | Taking damage |
| `Attack1` | 4 | Looping | 1.5x | Primary attack |
| `Attack2` | 5 | Looping | 1.5x | Secondary attack |
| `Block` | 6 | Static | 1.0x | Blocking/defending |
| `Cast` | 7 | Animated | 1.3x | Spell casting |
| `UseSkill1` | 8 | Animated | 1.2x | Special ability 1 |
| `UseSkill2` | 9 | Animated | 1.2x | Special ability 2 |
| `UseSkill3` | 10 | Animated | 1.2x | Special ability 3 |
| `UseSkill4` | 11 | Animated | 1.2x | Special ability 4 |
| `Dead` | 12 | Static | 1.0x | Corpse (last frame) |
| `BeingKnockback` | 13 | Animated | 1.0x | Being knocked back |
| `Sequence` | 14 | Animated | 1.0x | Custom sequence |
| `Run` | 15 | Looping | 1.5x | Running movement |

#### Methods

##### `speed_multiplier()`

Gets the default speed multiplier for this mode.

**Signature:**
```rust
pub fn speed_multiplier(&self) -> f32
```

**Returns:** Speed multiplier

**Example:**
```rust
let speed = UnitAnimationMode::Run.speed_multiplier();
// Returns: 1.5
```

---

##### `from_u8()`

Converts u8 to UnitAnimationMode.

**Signature:**
```rust
pub fn from_u8(value: u8) -> Option<UnitAnimationMode>
```

**Returns:** `Option<UnitAnimationMode>`

**Example:**
```rust
let mode = UnitAnimationMode::from_u8(2);
// Returns: Some(UnitAnimationMode::Walk)
```

---

### UnitType

Categories of game entities.

**Module:** `crate::animation`

```rust
pub enum UnitType {
    Monster,
    Missile,
    Player,
    Object,
}
```

#### Variants

- `Monster` - Enemy creatures
- `Missile` - Projectiles and area effects
- `Player` - Player characters
- `Object` - Interactive objects (chests, doors, etc.)

**Usage:** Determines which HashMap in registry to use

---

### SpriteSheetLayout

Defines how frames are arranged in a sprite sheet.

**Module:** `crate::animation`

```rust
pub enum SpriteSheetLayout {
    HorizontalStrip,
    VerticalStrip,
    Grid { columns: usize },
}
```

#### Variants

##### `HorizontalStrip`

Frames arranged in a single horizontal row.

**Best For:** < 16 frames

**Example:**
```
[Frame0][Frame1][Frame2][Frame3]
```

---

##### `VerticalStrip`

Frames arranged in a single vertical column.

**Best For:** Rarely used, vertical scrolling

**Example:**
```
[Frame0]
[Frame1]
[Frame2]
[Frame3]
```

---

##### `Grid { columns: usize }`

Frames arranged in a 2D grid.

**Best For:** 16-64 frames

**Parameters:**
- `columns: usize` - Number of columns in grid

**Example:**
```
Grid { columns: 4 }

[F0][F1][F2][F3]
[F4][F5][F6][F7]
[F8][F9][F10][F11]
```

---

## Functions

### calculate_frame_index

Calculates which frame to display at a given time.

**Signature:**
```rust
pub fn calculate_frame_index(
    elapsed_time: f32,
    fps: f32,
    speed: f32,
    frame_count: usize,
    looping: bool
) -> usize
```

**Parameters:**
- `elapsed_time: f32` - Time in seconds since animation started
- `fps: f32` - Frames per second
- `speed: f32` - Speed multiplier
- `frame_count: usize` - Total frames in animation
- `looping: bool` - Whether animation loops

**Returns:** Frame index (0-based)

**Formula:**
```
effective_fps = fps * speed
total_frames_passed = elapsed_time * effective_fps

if looping:
    frame_index = total_frames_passed % frame_count
else:
    frame_index = min(total_frames_passed, frame_count - 1)
```

**Example:**
```rust
let frame = calculate_frame_index(
    1.0,    // 1 second elapsed
    15.0,   // 15 FPS
    1.5,    // 1.5x speed
    8,      // 8 total frames
    true    // looping
);
// Calculation: 1.0 * (15.0 * 1.5) = 22.5 frames
// Result: 22 % 8 = 6
```

---

### get_elapsed_seconds

Helper to convert game time to seconds.

**Signature:**
```rust
pub fn get_elapsed_seconds(
    start_time: Option<Instant>
) -> f32
```

**Parameters:**
- `start_time: Option<Instant>` - When animation started

**Returns:** Elapsed time as `f32` seconds, or 0.0 if `None`

**Example:**
```rust
let start = Some(Instant::now());
// ... game loop ...
let elapsed = get_elapsed_seconds(start);
```

---

### get_frame_for_state

Maps state value to frame index (state-based animation).

**Signature:**
```rust
pub fn get_frame_for_state(
    frame_count: usize,
    animation_mode: Option<&AnimationMode>,
    state_value: f32
) -> usize
```

**Parameters:**
- `frame_count: usize` - Total frames
- `animation_mode: Option<&AnimationMode>` - Animation mode config
- `state_value: f32` - Current state value

**Returns:** Frame index

**Example:**
```rust
// Health globe at 50%
let frame = get_frame_for_state(
    101,                    // 101 frames (0-100%)
    Some(&AnimationMode::StateBased {
        mapping: StateMapping::Percentage
    }),
    0.5                     // 50%
);
// Returns: 50
```

---

## Usage Examples

### Complete Entity Setup

```rust
// 1. Define wrapper
pub struct MissileWrapper(pub Missiles);

// 2. Implement trait
impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Missiles::FrozenOrb => "FrozenOrb",
            _ => ""
        }
    }
    
    fn unit_type(&self) -> UnitType {
        UnitType::Missile
    }
    
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        Some(DefaultSpriteConfig {
            frame_count: 14,
            fps: 15.0,
        })
    }
}

// 3. Initialize registry
let missiles = Missiles::iter().map(MissileWrapper).collect();
let registry = UnitAnimationRegistry::from_discovered(
    &missiles,
    &["assets/images/missiles".to_string()],
    None
);

// 4. Use at runtime
let frame = registry.get_missile_frame(
    260,
    UnitAnimationMode::Walk,
    elapsed_time
);
```

---

### Frame Calculation

```rust
// Get configuration
let spec = registry.get_missile_spec(260).unwrap();
let config = spec.get_config(UnitAnimationMode::Walk).unwrap();

// Calculate frame
let frame_index = calculate_frame_index(
    elapsed_time,
    config.fps,
    config.speed,
    config.frame_count,
    true  // looping
);

// Build frame name
let frame_name = format!("{}/{}", spec.folder_name, frame_index);
```

---

## See Also

- **[Developer Guide](developer-guide.md)** - Implementation walkthrough
- **[Examples](examples.md)** - Real-world code samples
- **[Sprite Configuration](sprite-config.md)** - sprite.json reference