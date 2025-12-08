# Developer Guide: Adding Sprite Support

Step-by-step guide for adding sprite animations to entity types.

## When to Use This Guide

You're implementing sprite support for an entity type (monsters, missiles, objects, players) that needs:
- Animated sprites
- Multiple animation modes (walk, attack, cast, etc.)
- User-customizable sprites
- Support for multiple sprite formats

**Not covered here:** Simple static icons (see icon system instead)

---

## Three-Step Process

1. **Implement `SpriteDiscoverable` trait** - Tell the system about your entities
2. **Register during initialization** - Set up auto-discovery
3. **Use at runtime** - Get frames and render

---

## Step 1: Implement SpriteDiscoverable

Create a wrapper type and implement the trait:

```rust
use util::sprites::*;
use game::{Missiles, UnitType};

// Wrapper to satisfy Rust's orphan rule
pub struct MissileWrapper(pub Missiles);

impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Missiles::FrozenOrb => "FrozenOrb",
            Missiles::Lightning => "Lightning",
            Missiles::Meteor => "Meteor",
            _ => ""  // No sprites for this entity
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

### Method Details

#### `sprite_id() -> u32`

Returns unique numeric identifier.

**Requirements:**
- Must be unique within the entity's `UnitType`
- Should remain stable across versions
- Typically the entity's internal ID

```rust
fn sprite_id(&self) -> u32 {
    self.0 as u32  // Cast enum to its discriminant
}
```

---

#### `sprite_folder_name() -> &'static str`

Returns the folder name containing sprite files.

**Rules:**
- Use PascalCase: `FrozenOrb`, not `frozen_orb`
- Must match actual folder name exactly (case-sensitive)
- Return `""` for entities without sprites
- No special characters or spaces

```rust
fn sprite_folder_name(&self) -> &'static str {
    match self.0 {
        Missiles::FrozenOrb => "FrozenOrb",
        Missiles::NewMissile => "",  // Not implemented yet
        _ => ""
    }
}
```

**Tip:** Use constants to avoid typos:
```rust
const FROZEN_ORB: &str = "FrozenOrb";

fn sprite_folder_name(&self) -> &'static str {
    match self.0 {
        Missiles::FrozenOrb => FROZEN_ORB,
        _ => ""
    }
}
```

---

#### `unit_type() -> UnitType`

Specifies the entity category.

```rust
fn unit_type(&self) -> UnitType {
    UnitType::Missile  // or Monster, Player, Object
}
```

Determines which registry storage is used and search path priorities.

---

#### `default_config() -> Option<DefaultSpriteConfig>`

Provides fallback configuration when `sprite.json` is missing.

**Return `Some(...)`** when:
- You want graceful fallback if files are missing
- Most entities have similar frame counts/FPS
- Entity has known animation properties

```rust
fn default_config(&self) -> Option<DefaultSpriteConfig> {
    Some(DefaultSpriteConfig {
        frame_count: 14,
        fps: 15.0,
    })
}
```

**Return `None`** when:
- Entity uses DC6 files (frame count read from file)
- Configuration must come from `sprite.json`
- You want failures to be explicit

```rust
fn default_config(&self) -> Option<DefaultSpriteConfig> {
    None  // Must have sprite.json or DC6 file
}
```

**Per-entity defaults:**
```rust
fn default_config(&self) -> Option<DefaultSpriteConfig> {
    match self.0 {
        Missiles::Arrow => Some(DefaultSpriteConfig {
            frame_count: 4,
            fps: 10.0,
        }),
        Missiles::FrozenOrb => Some(DefaultSpriteConfig {
            frame_count: 14,
            fps: 15.0,
        }),
        _ => None
    }
}
```

---

## Step 2: Register During Initialization

In your module's `post_load_init()`:

```rust
fn post_load_init(&mut self) {
    // Collect all entities
    let all_missiles: Vec<MissileWrapper> = Missiles::iter()
        .map(|m| MissileWrapper(m))
        .collect();
    
    // Define search paths (checked in order)
    let search_paths = vec![
        "custom/missiles".to_string(),    // User overrides first
        "assets/images/missiles".to_string(),
    ];
    
    // Create DC6 loader (if needed)
    let dc6_loader = DC6Loader;
    
    // Create registry with auto-discovery
    self.animation_registry = Some(
        UnitAnimationRegistry::from_discovered(
            &all_missiles,
            &search_paths,
            Some(&dc6_loader as &dyn GameFileLoader)
        )
    );
    
    // Start animation timer
    self.animation_start = Some(std::time::Instant::now());
}
```

**What happens:**
1. System iterates through all wrapped entities
2. For each entity, searches paths for matching folder
3. Loads `sprite.json` or uses default config
4. Builds `AnimationSpec` for each animation mode
5. Stores in registry: `missiles[260] = AnimationSpec { ... }`

**Search path priority:** First match wins. Put user customization folders first.

---

## Step 3: Use at Runtime

### Time-Based Animation

For most entities (projectiles, monsters, characters):

```rust
fn draw(&self, draw_bg: &mut DrawListMut) {
    let elapsed = get_elapsed_seconds(self.animation_start);
    
    for missile in missiles {
        // Get animation mode from entity state
        let anim_mode = UnitAnimationMode::from_u8(missile.animation_mode as u8)
            .unwrap_or(UnitAnimationMode::Stand);
        
        // Get current frame name
        let frame_name = registry.get_missile_frame(
            missile.dw_class_id,  // Entity ID (e.g., 260)
            anim_mode,            // Animation mode
            elapsed               // Time in seconds
        );
        // Returns: "FrozenOrb/7"
        
        // Load and draw
        if let Some(img) = self.image_list.get(&frame_name) {
            self.draw_image(draw_bg, img, x, y, scale, tint, alpha);
        }
    }
}
```

---

### State-Based Animation

For UI elements that map state to frames (health bars, progress):

```rust
fn draw_health_globe(&self, draw_bg: &mut DrawListMut, health_percent: f32) {
    let frame = get_frame_for_state(
        frame_count,
        animation_mode,
        health_percent  // 0.0 to 1.0
    );
    
    let frame_name = format!("HealthGlobe/{}", frame);
    
    if let Some(img) = self.image_list.get(&frame_name) {
        self.draw_image(draw_bg, img, x, y, scale, tint, alpha);
    }
}
```

---

## Loading Images

After registry creation, load actual image data:

```rust
fn load_images(&mut self) {
    let search_paths = vec![
        "assets/images/missiles".to_string(),
    ];
    
    // Discover what's on disk
    let discovered = discover_sprite_folders(&search_paths);
    
    // Get list of folders we need from registry
    let needed_folders = self.animation_registry
        .as_ref()
        .map(|r| r.get_all_folders_to_load())
        .unwrap_or_default();
    
    let dc6_loader = DC6Loader;
    let mut all_images = HashMap::new();
    
    for discovered_folder in discovered {
        // Check if this folder is needed
        let is_needed = needed_folders.iter()
            .any(|(name, _)| name == &discovered_folder.name);
        
        if !is_needed {
            continue;
        }
        
        // Load frames
        if discovered_folder.is_modular() {
            // Modular: separate files per animation mode
            let images = load_all_frames_modular(
                &search_paths,
                &discovered_folder.name,
                &discovered_folder,
                Some(&dc6_loader as &dyn GameFileLoader)
            );
            all_images.extend(images);
        } else if let Some(frame_count) = discovered_folder.frame_count {
            // Individual frames or sprite sheets
            let frames = ImageLoader::load_frames(
                &search_paths,
                &discovered_folder.name,
                frame_count,
                Some(&dc6_loader as &dyn GameFileLoader),
                discovered_folder.layout.clone()
            );
            all_images.extend(frames);
        }
    }
    
    self.image_list = Some(all_images);
}
```

---

## Progressive Loading

For large image sets, load progressively to avoid frame drops:

```rust
fn post_load_init(&mut self) {
    // ... setup registry as before ...
    
    // Setup progressive loading
    let discovered = discover_sprite_folders(&search_paths);
    
    let config = LoadingConfig {
        max_frame_time_ms: 16,  // ~60 FPS budget
        batch_size: 10,         // Images per frame
    };
    
    self.loading_state = Some(ImageLoadingState::new(
        discovered,
        search_paths,
        config,
    ));
    
    // Start with embedded images
    self.image_list = Some(ImageLoader::load_category(ImageCategory::Monsters));
    self.is_loading = true;
}

fn update(&mut self) {
    if !self.is_loading {
        return;
    }
    
    let Some(loading_state) = &mut self.loading_state else {
        self.is_loading = false;
        return;
    };
    
    // Load chunk this frame
    let dc6_loader = DC6Loader;
    let has_more = loading_state.load_chunk(
        Some(&dc6_loader as &dyn GameFileLoader)
    );
    
    // Merge new images
    if let Some(existing) = &mut self.image_list {
        for (key, img) in loading_state.get_images() {
            existing.entry(key.clone()).or_insert_with(|| img.clone());
        }
    }
    
    if !has_more {
        // Complete - transfer final images
        self.image_list = Some(loading_state.into_images());
        self.is_loading = false;
    }
}
```

---

## Common Patterns

### Sharing Sprites Between Entities

Multiple entities use same folder:

```rust
impl SpriteDiscoverable for MissileWrapper {
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Missiles::FireBolt1 => "FireBolt",
            Missiles::FireBolt2 => "FireBolt",  // Same sprites
            Missiles::FireBolt3 => "FireBolt",  // Same sprites
            _ => ""
        }
    }
    
    fn sprite_id(&self) -> u32 {
        self.0 as u32  // Different IDs, shared sprites
    }
}
```

---

### Dynamic Configuration

Adjust defaults based on entity properties:

```rust
fn default_config(&self) -> Option<DefaultSpriteConfig> {
    let (frame_count, fps) = match self.0 {
        Missiles::SlowMissile => (8, 10.0),
        Missiles::FastMissile => (8, 20.0),
        Missiles::ComplexMissile => (16, 15.0),
        _ => return None,
    };
    
    Some(DefaultSpriteConfig { frame_count, fps })
}
```

---

### Conditional Loading

Load sprites only when needed:

```rust
impl SpriteDiscoverable for MonsterWrapper {
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Monsters::Zombie => "Zombie",
            Monsters::Skeleton => "Skeleton",
            // Generic for others
            _ => "GenericMonster"
        }
    }
}
```

---

### Custom Search Paths

Allow user configuration:

```rust
fn post_load_init(&mut self) {
    let mut search_paths = vec![];
    
    // User's custom path first
    if self.use_custom_path && !self.custom_path.is_empty() {
        search_paths.push(self.custom_path.clone());
    }
    
    // Default paths
    search_paths.extend_from_slice(&[
        "assets/images/monsters".to_string(),
    ]);
    
    // ... rest of initialization
}
```

---

## Testing

### Basic Trait Test

```rust
#[test]
fn test_sprite_discoverable() {
    let missile = MissileWrapper(Missiles::FrozenOrb);
    
    assert_eq!(missile.sprite_id(), 260);
    assert_eq!(missile.sprite_folder_name(), "FrozenOrb");
    assert_eq!(missile.unit_type(), UnitType::Missile);
    
    let config = missile.default_config().unwrap();
    assert_eq!(config.frame_count, 14);
    assert_eq!(config.fps, 15.0);
}
```

---

### Integration Test

```rust
#[test]
fn test_full_discovery() {
    let missiles = vec![
        MissileWrapper(Missiles::FrozenOrb),
        MissileWrapper(Missiles::Lightning),
    ];
    
    let paths = vec!["test_assets/missiles".to_string()];
    
    let registry = UnitAnimationRegistry::from_discovered(
        &missiles,
        &paths,
        None
    );
    
    assert!(registry.has_missile(260));
    
    let frame = registry.get_missile_frame(
        260,
        UnitAnimationMode::Walk,
        0.0
    );
    assert!(frame.starts_with("FrozenOrb/"));
}
```

---

## Troubleshooting

### Issue: Sprites Don't Appear

**Check:**
1. `sprite_folder_name()` matches actual folder exactly
2. Folder exists in search paths
3. Files are valid PNGs or DC6s
4. `sprite.json` is valid JSON
5. Check logs for warnings

**Debug:**
```rust
info!("Discovered {} folders", discovered.len());
for folder in discovered {
    info!("  - {}: {} frames", folder.name, folder.frame_count.unwrap_or(0));
}
```

---

### Issue: Wrong Frames Displaying

**Causes:**
- `frame_count` in config doesn't match actual frames
- Animation mode mapping incorrect
- Frame calculation off (check FPS/speed)

**Fix:**
Use the Sheet Analyzer tool to verify frame counts.

---

### Issue: Performance Problems

**Solutions:**
- Enable progressive loading for 100+ images
- Use grid layouts for 20+ frame animations
- Share sprites between similar entities
- Reduce frame counts where possible

---

## Summary Checklist

Adding a new entity type:

- [ ] Create wrapper struct
- [ ] Implement `SpriteDiscoverable` trait
  - [ ] `sprite_id()` returns unique ID
  - [ ] `sprite_folder_name()` matches folder exactly
  - [ ] `unit_type()` correct category
  - [ ] `default_config()` provides fallback
- [ ] Register in `post_load_init()`
- [ ] Load images (immediate or progressive)
- [ ] Use at runtime (`get_missile_frame` etc.)
- [ ] Test with actual sprites
- [ ] Document special cases

---

## Next Steps

- **[Sprite Configuration](sprite-config.md)** - JSON format details
- **[API Reference](api-reference.md)** - Method signatures
- **[Examples](examples.md)** - Real implementations