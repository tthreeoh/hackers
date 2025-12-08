# Sprite Animation System

# Sprite Animation System Overview

A flexible animation system that automatically discovers and loads sprites from the filesystem, supporting multiple formats and configurations.

## Quick Navigation

- **[Developer Guide](developer-guide.md)** - Implementing sprite support for entities
- **[User Guide](user-guide.md)** - Adding custom sprites (no programming)
- **[Sprite Configuration](sprite-config.md)** - JSON format reference
- **[API Reference](api-reference.md)** - Detailed method signatures
- **[Examples](examples.md)** - Real-world implementations

---

## What Is This?

A sprite animation system that:
- **Auto-discovers** sprites from folders
- **Supports multiple formats** - Individual frames, sprite sheets, DC6 files
- **Configurable** - JSON files control animation properties
- **User-friendly** - Non-programmers can add/modify sprites
- **Flexible** - Per-animation customization, state-based animations, layering

---

## Core Concepts

### Entity Types

The system organizes sprites by entity category:

- **Monsters** - Enemy creatures (Zombie, Skeleton, etc.)
- **Missiles** - Projectiles and effects (FrozenOrb, Lightning, etc.)
- **Players** - Player characters (Barbarian, Sorceress, etc.)
- **Objects** - Interactive entities (Chest, Door, etc.)

Each type has its own folder structure and registry.

---

### Animation Modes

Every entity supports 16 standard animation modes:

| Mode | Use | Behavior |
|------|-----|----------|
| Stand | Idle | Loops |
| Walk | Moving | Loops |
| Run | Fast movement | Loops, 1.5x speed |
| Attack1/2 | Combat | Loops, 1.5x speed |
| Cast | Spellcasting | Animated, 1.3x speed |
| Death | Dying | Plays once |
| Dead | Corpse | Static (last frame) |
| BeingHit | Taking damage | Animated |
| Block | Defending | Static |
| UseSkill1-4 | Abilities | Animated, 1.2x speed |
| BeingKnockback | Staggered | Animated |
| Sequence | Custom | Animated |

Different modes can have different frame counts, speeds, and files.

---

### Sprite Organization

Sprites live in folders named after entities:

```
assets/images/
├── monsters/
│   ├── Zombie/
│   └── Skeleton/
├── missiles/
│   ├── FrozenOrb/
│   └── Lightning/
└── ui/
    └── HealthGlobe/
```

Each folder can contain:
- Individual frame images (0.png, 1.png, ...)
- Sprite sheets (one image with multiple frames)
- DC6 files (Diablo II format)
- sprite.json (configuration file)

---

## How It Works

### 1. Discovery Phase (Initialization)

System scans configured paths for sprite folders:

```
custom/missiles/       ← Checked first (user overrides)
assets/images/missiles ← Default sprites
```

For each entity:
1. Search for matching folder name
2. Read `sprite.json` if present
3. Load images (frames, sheets, or DC6)
4. Build animation specifications
5. Store in registry

---

### 2. Runtime Phase (Drawing)

Game requests frames by entity ID, mode, and time:

```rust
let frame = registry.get_missile_frame(
    260,                     // Entity ID (FrozenOrb)
    UnitAnimationMode::Walk, // Animation mode
    elapsed_time            // Time in seconds
);
// Returns: "FrozenOrb/7"
```

System calculates which frame to show based on:
- Frame count
- FPS (frames per second)
- Speed multiplier
- Elapsed time
- Looping behavior

---

## Sprite Formats

### Individual Frames (Simplest)

One PNG per frame, numbered sequentially:

```
FrozenOrb/
├── 0.png
├── 1.png
├── ...
└── 13.png
```

**Best for:**
- Small animations (< 20 frames)
- Easy editing
- Simple projectiles

**Behavior:** All animation modes share these frames.

---

### Sprite Sheets (Flexible)

Multiple frames in a single image:

**Horizontal strip:**
```
[F0][F1][F2][F3][F4][F5]
```

**Grid (for many frames):**
```
[F0][F1][F2][F3]
[F4][F5][F6][F7]
[F8][F9][F10][F11]
```

**Best for:**
- Medium animations (8-64 frames)
- Organized storage
- Faster loading

---

### Modular Mode (Advanced)

Separate files for different animation modes:

```
Zombie/
├── Stand.png    (1 frame)
├── Walk.png     (8 frames)
├── Attack.png   (12 frames)
└── Death.png    (15 frames)
```

**Best for:**
- Complex entities
- Different animations per mode
- Per-animation customization

**Requires:** sprite.json to map modes to files.

---

## Configuration

### Minimal Configuration

For simple cases, just specify FPS:

```json
{
  "fps": 15.0
}
```

System auto-detects frame count and layout.

---

### Full Configuration

For complex entities:

```json
{
  "default_fps": 15.0,
  "files": {
    "Stand.png": { "frame_count": 1 },
    "Walk.png": { "frame_count": 8, "fps": 12.0 },
    "Attack.png": { "frame_count": 12, "fps": 24.0 }
  },
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "attack1": "Attack.png"
  }
}
```

Each file can override FPS, speed, and layout.

---

## Animation Types

### Time-Based (Standard)

Frames advance over time automatically:
- Walking characters
- Projectiles
- Attack animations

System handles looping, speed multipliers, and timing.

---

### State-Based (Special)

Frame determined by state value, not time:
- Health bars (0-100% → frames 0-100)
- Progress indicators
- Counters

**Configuration:**
```json
{
  "animation_mode": {
    "state_mapping": "percentage"
  }
}
```

---

## Key Features

### User Overrides

Users can replace sprites without modifying game files:

```
custom/missiles/FrozenOrb/  ← User's version (priority)
assets/images/missiles/FrozenOrb/  ← Default
```

First match wins.

---

### Progressive Loading

Large sprite sets load in chunks to prevent frame drops:
- Configurable batch size
- Respects frame time budget
- Shows progress

---

### Fallback Behavior

Graceful handling of missing assets:
1. Check for sprite.json → use it
2. Check for numbered frames → count them
3. Check for DC6 file → load it
4. Use default config → from code
5. Skip entity → log warning

Game continues even if sprites missing.

---

## Performance

### Loading
- Discovery: O(folders) filesystem scan
- Image loading: Progressive, respects frame budget
- Parsing: Simple JSON, minimal overhead

### Runtime
- Frame lookup: O(1) HashMap access
- Frame calculation: Simple arithmetic
- Memory: Images cached per entity type

### Optimization Strategies
- Batch similar entities together
- Reuse sprites between similar entities
- Use grid layouts for large animations
- Progressive loading for startup

---

## Typical Workflows

### For Developers

1. Implement `SpriteDiscoverable` trait
2. Register entities in `post_load_init()`
3. Load images
4. Use `get_*_frame()` at runtime

See: [Developer Guide](developer-guide.md)

---

### For Artists/Modders

1. Create sprite folder in `custom/`
2. Add PNG frames or sheets
3. Write `sprite.json` config
4. Launch game

See: [User Guide](user-guide.md)

---

### For Configuration

1. Use Sheet Analyzer tool to verify frames
2. Write sprite.json with settings
3. Test different FPS/speed values
4. Use mode_mapping for complex entities

See: [Sprite Configuration](sprite-config.md)

---

## Design Principles

### Auto-Discovery Over Registration

System scans folders instead of requiring manual registration:
- Add folder → sprites load automatically
- Remove folder → entity has no sprites
- Modify folder → changes take effect

---

### Configuration Over Code

Settings in JSON files, not hardcoded:
- Artists adjust FPS without programmer
- Users override without recompiling
- Easy to experiment and iterate

---

### Graceful Degradation

Missing assets don't crash:
- No sprites → entity invisible but functional
- No config → use sensible defaults
- Invalid JSON → fallback to defaults

---

## Common Use Cases

| Use Case | Approach | Example |
|----------|----------|---------|
| Simple projectile | Individual frames | FrozenOrb |
| Complex monster | Modular mode | Zombie |
| UI element | State-based | Health globe |
| Character equipment | Multi-layer | Player armor |
| Particle effects | Randomized | Fire particles |
| Large crowds | Batched rendering | Zombie horde |

---

## Limitations

- **Formats:** PNG and DC6 only (no JPEG, GIF, etc.)
- **Size:** Sprite sheets limited by image memory
- **Naming:** Strict folder name matching (case-sensitive)
- **JSON:** Must be valid (use validator)
- **Synchronization:** Multi-layer sprites must match frame counts

---

## Architecture

```
┌─────────────────────────────────────┐
│   Game Code (missiles.rs, etc.)    │
│   - Implements SpriteDiscoverable   │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│     UnitAnimationRegistry           │
│   - from_discovered()               │
│   - Scans filesystem                │
│   - Loads configurations            │
│   - Builds AnimationSpecs           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Image Storage (HashMap)           │
│   - "FrozenOrb/0" → DynamicImage    │
│   - "Zombie/Walk/3" → DynamicImage  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Runtime (get_*_frame)             │
│   - Calculate frame from time       │
│   - Return image key                │
│   - Draw to screen                  │
└─────────────────────────────────────┘
```

---

## Getting Started

**New to the system?**
- Read: [User Guide](user-guide.md) - Add custom sprites

**Implementing for new entity type?**
- Read: [Developer Guide](developer-guide.md) - Add sprite support

**Need configuration details?**
- Read: [Sprite Configuration](sprite-config.md) - JSON format

**Want code examples?**
- Read: [Examples](examples.md) - Real implementations

**Need method signatures?**
- Read: [API Reference](api-reference.md) - Complete reference

### Sprite Organization

Sprites are organized by **entity type** (monsters, missiles, objects, players):

```
assets/images/
├── monsters/
│   ├── Zombie/
│   ├── Skeleton/
│   └── ...
├── missiles/
│   ├── FrozenOrb/
│   ├── Lightning/
│   └── ...
└── ui/
    ├── HealthGlobe/
    └── ...
```

Each entity folder can contain:
- Individual frame images (`0.png`, `1.png`, ...)
- Sprite sheets (horizontal strips, grids)
- DC6 files (Diablo II format)
- Configuration file (`sprite.json`)

---

## Animation Modes

Every entity supports 16 standard animation modes:

| Mode | Common Use | Behavior |
|------|-----------|----------|
| `Stand` | Idle | Loops |
| `Walk` | Moving | Loops |
| `Run` | Fast movement | Loops, 1.5x speed |
| `Attack1/2` | Combat | Loops, 1.5x speed |
| `Cast` | Spellcasting | Animated, 1.3x speed |
| `Death` | Dying | Plays once |
| `Dead` | Corpse | Shows last frame |
| `BeingHit` | Taking damage | Animated |
| `Block` | Defending | Static |
| `UseSkill1-4` | Special abilities | Animated, 1.2x speed |
| `BeingKnockback` | Staggered | Animated |
| `Sequence` | Custom | Animated |

---

## File Formats

### Individual Frames (Simplest)

One PNG per frame, numbered sequentially:

```
FrozenOrb/
├── 0.png
├── 1.png
├── 2.png
...
└── 13.png
```

**Best for**: Small animations (< 20 frames), easy editing

All animation modes share the same frames. System auto-detects frame count.

---

### Sprite Sheets (Modular)

One image file per animation mode:

```
Zombie/
├── sprite.json
├── Stand.png      (1 frame)
├── Walk.png       (8 frames horizontal)
├── Attack.png     (12 frames horizontal)
└── Death.png      (15 frames horizontal)
```

**Best for**: Complex entities with distinct animations per mode

Requires `sprite.json` to map modes to files.

---

### Sprite Sheet Layouts

Images can be arranged as:

**Horizontal Strip** (default)
```
[F0][F1][F2][F3][F4][F5]
```
Good for: < 16 frames

**Vertical Strip**
```
[F0]
[F1]
[F2]
[F3]
```
Good for: Rarely used, special cases

**Grid**
```
[F0][F1][F2][F3]
[F4][F5][F6][F7]
[F8][F9][F10][F11]
```
Good for: 16-64 frames

---

## Configuration Files

### Basic sprite.json

For individual frames:
```json
{
  "fps": 15.0
}
```

For sprite sheets:
```json
{
  "frame_count": 14,
  "fps": 15.0,
  "layout": "horizontal"
}
```

---

### Modular Configuration

Different files for different animation modes:

```json
{
  "default_fps": 15.0,
  
  "files": {
    "Stand.png": {
      "frame_count": 1
    },
    "Walk.png": {
      "frame_count": 8,
      "fps": 12.0
    },
    "Attack.png": {
      "frame_count": 12,
      "fps": 24.0,
      "layout": "grid",
      "grid_columns": 4
    }
  },
  
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "run": "Walk.png",
    "attack1": "Attack.png",
    "attack2": "Attack.png"
  }
}
```

**Key features:**
- Each file can have its own FPS and layout
- Modes can share files (run uses walk animation)
- Override `default_fps` per file

---

### Mode-Specific Overrides

Fine-tune individual modes:

```json
{
  "default_fps": 15.0,
  
  "mode_overrides": {
    "walk": {
      "fps": 12.0,
      "speed": 1.0
    },
    "run": {
      "fps": 15.0,
      "speed": 1.5
    },
    "attack1": {
      "fps": 24.0,
      "speed": 1.0
    }
  }
}
```

---

## State-Based Animation

For entities that map state to frames (health bars, progress indicators):

```json
{
  "frame_count": 101,
  "fps": 1.0,
  "animation_mode": {
    "state_mapping": "percentage"
  }
}
```

**Mapping types:**

- `"percentage"` - 0.0 to 1.0 maps to frames (health globes)
- `"direct"` - State value N shows frame N
- `"ranges"` - Custom value ranges to frame ranges

Example with ranges:
```json
{
  "animation_mode": {
    "state_mapping": "ranges",
    "ranges": [
      {
        "min_value": 0.0,
        "max_value": 0.25,
        "start_frame": 0,
        "end_frame": 10
      },
      {
        "min_value": 0.25,
        "max_value": 1.0,
        "start_frame": 11,
        "end_frame": 50
      }
    ]
  }
}
```

---

## Code Integration

### Implementing SpriteDiscoverable

```rust
pub struct MissileWrapper(pub Missiles);

impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32  // Unique ID
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Missiles::FrozenOrb => "FrozenOrb",
            Missiles::Lightning => "Lightning",
            _ => ""  // No sprites
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
```

**Why the wrapper?** Rust's orphan rule prevents implementing external traits on external types. The newtype wrapper solves this.

---

### Initialization

```rust
fn post_load_init(&mut self) {
    // Collect entities
    let all_missiles: Vec<MissileWrapper> = Missiles::iter()
        .map(|m| MissileWrapper(m))
        .collect();
    
    // Define search paths (checked in order)
    let search_paths = vec![
        "custom/missiles".to_string(),    // User overrides
        "assets/images/missiles".to_string(),
    ];
    
    // Create registry with auto-discovery
    self.animation_registry = Some(
        UnitAnimationRegistry::from_discovered(
            &all_missiles,
            &search_paths,
            Some(&dc6_loader)
        )
    );
    
    self.animation_start = Some(std::time::Instant::now());
}
```

---

### Runtime Usage

**Time-based animation:**
```rust
let elapsed = get_elapsed_seconds(self.animation_start);

let frame_name = registry.get_missile_frame(
    missile.dw_class_id,  // Entity ID
    animation_mode,        // Walk, Attack, etc.
    elapsed               // Current time
);
// Returns: "FrozenOrb/7"
```

**State-based animation:**
```rust
let health_percent = player.health / player.max_health;

let frame = get_frame_for_state(
    frame_count,
    animation_mode,
    health_percent
);
// Returns: frame index directly
```

---

## System Behavior

### Fallback Chain

When loading sprites, the system tries:

1. **Modular mode** - Check for `sprite.json` with `files` section
2. **Individual frames** - Look for numbered PNGs (0.png, 1.png, ...)
3. **DC6 files** - Check for `.dc6` file (if loader provided)
4. **Default config** - Use `default_config()` from trait
5. **Skip** - Entity has no sprites, won't render

### Search Priority

Multiple paths are checked in order:
```rust
let search_paths = vec![
    "custom/missiles".to_string(),     // 1. Check here first
    "assets/images/missiles".to_string(),  // 2. Then here
];
```

This allows users to override default sprites without modifying game files.

---

## Performance

### Loading Strategy

Images load progressively:
```rust
let config = LoadingConfig {
    max_frame_time_ms: 16,  // ~60 FPS budget
    batch_size: 10,         // Images per frame
};

let mut loading_state = ImageLoadingState::new(
    discovered_folders,
    search_paths,
    config
);

// In update loop:
let has_more = loading_state.load_chunk(dc6_loader);
```

Prevents frame drops during initialization.

### Runtime Performance

- **Frame lookup**: O(1) HashMap access
- **Frame calculation**: Simple arithmetic
- **Memory**: Images cached, shared across all entities of same type

---

## Common Patterns

### Single Animation for All Modes

Simple entities (projectiles, effects):
```
FrozenOrb/
├── 0.png - 13.png   (14 frames)
└── sprite.json      (fps: 15.0)
```

Stand, Walk, Attack all use frames 0-13.

---

### Per-Mode Animations

Complex entities (monsters, players):
```
Zombie/
├── sprite.json
├── Stand.png    (1 frame, slow)
├── Walk.png     (8 frames, medium)
├── Attack.png   (12 frames, fast)
└── Death.png    (15 frames, slow)
```

Each mode has unique animation with custom timing.

---

### Sharing Animations

Multiple entities use same sprites:
```rust
fn sprite_folder_name(&self) -> &'static str {
    match self.0 {
        Missiles::FireBolt1 => "FireBolt",
        Missiles::FireBolt2 => "FireBolt",  // Same folder
        Missiles::FireBolt3 => "FireBolt",  // Same folder
        _ => ""
    }
}
```

Saves disk space and memory.

---

### Custom Search Paths

Users can add sprites:
```rust
let search_paths = if self.use_custom_path {
    vec![
        self.custom_path.clone(),  // User's folder
        "assets/images/monsters".to_string(),
    ]
} else {
    vec!["assets/images/monsters".to_string()]
};
```

---

## Troubleshooting

### Sprites Don't Show

**Check:**
1. Folder name matches `sprite_folder_name()` exactly (case-sensitive)
2. Files are actual PNGs, not renamed JPGs
3. `sprite.json` is valid JSON (use JSONLint)
4. Search paths include the folder
5. Logs show discovery (look for warnings)

### Wrong Animation Speed

**Fix:**
- Adjust `fps` in `sprite.json`
- Use `mode_overrides` for specific modes
- Check `speed` multiplier (1.0 = normal)

### Missing Frames

**Causes:**
- `frame_count` too low in config
- Numbered files have gaps (0,1,3 missing 2)
- Grid layout needs `grid_columns` specified

### Performance Issues

**Solutions:**
- Use grid layout for 20+ frames
- Enable progressive loading
- Reduce frame counts where possible
- Share sprites between similar entities

---

## Summary

**Simple icons**: No config needed, just images  
**Basic animations**: Individual frames + `sprite.json`  
**Complex entities**: Modular mode with per-animation files  
**State displays**: Use state-based animation mode  
**User customization**: Put sprites in custom/ folder

The system adapts to your needs - from static icons to complex multi-animation entities.