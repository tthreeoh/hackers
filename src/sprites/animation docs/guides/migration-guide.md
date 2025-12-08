# Migration Guide

Upgrading from manual registration to auto-discovery system.

## Why Migrate?

### Old System Pain Points

- **Manual Registration:** Register every entity individually
```rust
// 600+ lines of this...
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
registry.register_missile_simple(261, "FrozenOrbBolt", 14, 15.0);
registry.register_missile_simple(262, "BlizzardMissile1", 14, 15.0);
// ... hundreds more ...
```

- **Brittle:** Typo in folder name → silent failure
- **Tedious:** Adding new entity requires code change
- **No User Overrides:** Can't swap sprites without code
- **Duplicated Config:** FPS values repeated everywhere

### New System Benefits

✅ **Auto-Discovery:** Drop folder → automatic registration  
✅ **User-Friendly:** Non-programmers can add sprites  
✅ **Flexible:** Per-mode configurations  
✅ **Maintainable:** Config in JSON, not code  
✅ **Extensible:** Easy to add new entities  

---

## Breaking Changes

### 1. Registration Method Changed

**Old:**
```rust
let mut registry = UnitAnimationRegistry::new();
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
```

**New:**
```rust
let all_missiles = Missiles::iter().map(MissileWrapper).collect();
let registry = UnitAnimationRegistry::from_discovered(
    &all_missiles,
    &search_paths,
    None
);
```

### 2. Wrapper Required

**Old:**
```rust
// Direct enum usage
match missile {
    Missiles::FrozenOrb => { /* ... */ }
}
```

**New:**
```rust
// Wrap for trait implementation
impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32
    }
    // ...
}
```

### 3. Config in JSON, Not Code

**Old:**
```rust
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
```

**New:**
```json
{
  "frame_count": 14,
  "fps": 15.0
}
```

### 4. No More `register_*` Methods

Removed:
- `register_missile_simple()`
- `register_monster_simple()`
- `register_player_simple()`
- `register_object_simple()`

Replaced by: `from_discovered()`

---

## Step-by-Step Migration

### Step 1: Audit Current Registrations

Extract all your current registration calls:

```bash
grep -r "register_.*_simple" src/ > old_registrations.txt
```

Example output:
```
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
registry.register_missile_simple(261, "FrozenOrbBolt", 14, 15.0);
registry.register_monster_simple(1, "Zombie", 8, 12.0);
```

### Step 2: Create Wrapper Structs

**For Missiles:**
```rust
// In missiles.rs or new wrappers.rs
pub struct MissileWrapper(pub Missiles);

impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        // Extract from your old registration calls
        match self.0 {
            Missiles::FrozenOrb => "FrozenOrb",       // ID 260
            Missiles::FrozenOrbBolt => "FrozenOrbBolt", // ID 261
            // ... rest of your missiles
            _ => ""
        }
    }
    
    fn unit_type(&self) -> UnitType {
        UnitType::Missile
    }
    
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        Some(DefaultSpriteConfig {
            frame_count: 14,  // Most common value
            fps: 15.0,        // Most common value
        })
    }
}
```

**Tip:** Generate from old registrations:
```python
# Quick script to convert
import re

with open('old_registrations.txt') as f:
    for line in f:
        match = re.search(r'register_missile_simple\((\d+), "([^"]+)", (\d+), ([\d.]+)\)', line)
        if match:
            id, name, frames, fps = match.groups()
            print(f'Missiles::{name} => "{name}",  // ID {id}, {frames}f @ {fps}fps')
```

### Step 3: Create sprite.json Files

For each entity folder, create a `sprite.json`:

**Simple case (all defaults):**
```json
{
  "frame_count": 14,
  "fps": 15.0
}
```

**If using custom FPS:**
```json
{
  "frame_count": 14,
  "fps": 20.0
}
```

**Automated generation:**
```python
import json
import os

registrations = [
    (260, "FrozenOrb", 14, 15.0),
    (261, "FrozenOrbBolt", 14, 15.0),
    # ... your registrations
]

for id, name, frames, fps in registrations:
    folder = f"assets/images/missiles/{name}"
    if os.path.exists(folder):
        config = {
            "frame_count": frames,
            "fps": fps
        }
        with open(f"{folder}/sprite.json", 'w') as f:
            json.dump(config, f, indent=2)
```

### Step 4: Update Initialization Code

**Old:**
```rust
fn initialize_animations(&mut self) {
    let mut registry = UnitAnimationRegistry::new();
    
    // Missiles
    registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
    registry.register_missile_simple(261, "FrozenOrbBolt", 14, 15.0);
    // ... 600 more lines ...
    
    // Monsters
    registry.register_monster_simple(1, "Zombie", 8, 12.0);
    registry.register_monster_simple(2, "Skeleton", 8, 12.0);
    // ... 200 more lines ...
    
    self.animation_registry = Some(registry);
}
```

**New:**
```rust
fn initialize_animations(&mut self, game_loader: &GameLoader) {
    // Collect all missiles
    let all_missiles: Vec<MissileWrapper> = Missiles::iter()
        .map(|m| MissileWrapper(m))
        .collect();
    
    // Collect all monsters
    let all_monsters: Vec<MonsterWrapper> = Monsters::iter()
        .map(|m| MonsterWrapper(m))
        .collect();
    
    // Define search paths
    let missile_paths = vec![
        "custom/missiles".to_string(),     // User overrides first
        "assets/images/missiles".to_string(),
    ];
    
    let monster_paths = vec![
        "custom/monsters".to_string(),
        "assets/images/monsters".to_string(),
    ];
    
    // Auto-discover everything
    let mut registry = UnitAnimationRegistry::from_discovered(
        &all_missiles,
        &missile_paths,
        Some(&game_loader.dc6_loader)
    );
    
    registry.add_discovered(
        &all_monsters,
        &monster_paths,
        Some(&game_loader.dc6_loader)
    );
    
    self.animation_registry = Some(registry);
}
```

**Lines of code:**  
Old: ~1000 lines  
New: ~30 lines

### Step 5: Test Each Entity Type

Create a test harness:

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn verify_all_missiles_discovered() {
        let all_missiles: Vec<MissileWrapper> = Missiles::iter()
            .map(MissileWrapper)
            .collect();
        
        let paths = vec!["assets/images/missiles".to_string()];
        let registry = UnitAnimationRegistry::from_discovered(
            &all_missiles,
            &paths,
            None
        );
        
        // Verify key missiles
        assert!(registry.has_missile(260), "FrozenOrb not found");
        assert!(registry.has_missile(261), "FrozenOrbBolt not found");
        
        // Get a frame to verify it works
        let frame = registry.get_missile_frame(
            260,
            UnitAnimationMode::Walk,
            0.0
        );
        assert_eq!(frame, "FrozenOrb/0");
    }
    
    #[test]
    fn compare_old_and_new_frame_calculation() {
        // Old system result
        let old_frame = old_calculate_frame(260, 1.5);
        
        // New system result
        let registry = /* ... */;
        let new_frame = registry.get_missile_frame(260, UnitAnimationMode::Walk, 1.5);
        
        // Should match
        assert_eq!(old_frame, new_frame);
    }
}
```

### Step 6: Handle Missing Entities

Some entities might not have sprite folders. The new system handles this gracefully:

```rust
impl SpriteDiscoverable for MissileWrapper {
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Missiles::FrozenOrb => "FrozenOrb",
            
            // Not implemented yet
            Missiles::NewMissile => "",  // Empty = skip
            
            _ => ""
        }
    }
    
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        match self.0 {
            Missiles::FrozenOrb => Some(DefaultSpriteConfig {
                frame_count: 14,
                fps: 15.0,
            }),
            
            // No default for new missile
            _ => None
        }
    }
}
```

**Result:** NewMissile won't crash, just won't have sprites.

---

## Handling Existing sprite.json Files

### Case 1: Already Have Modular Setup

If you already use modular layout:

```
FrozenOrb/
├── Stand.png
├── Walk.png
└── Attack.png
```

Just add `sprite.json`:
```json
{
  "default_fps": 15.0,
  "files": {
    "Stand.png": { "frame_count": 1 },
    "Walk.png": { "frame_count": 8 },
    "Attack.png": { "frame_count": 12 }
  }
}
```

### Case 2: Have Legacy Numbered Frames

Keep your existing structure:
```
FrozenOrb/
├── 0.png
├── 1.png
├── ...
└── 13.png
```

Add minimal `sprite.json`:
```json
{
  "fps": 15.0
}
```

**Frame count auto-detected from files!**

### Case 3: Mixed Formats

Some folders have JSON, some don't:

```
missiles/
├── FrozenOrb/          (has sprite.json)
├── Lightning/          (no JSON, use defaults)
└── Meteor/             (no JSON, use defaults)
```

System handles this automatically:
1. Checks for sprite.json → Use it
2. No JSON? → Use `default_config()`
3. No defaults? → Log warning, skip

---

## Updating sprite.json for New Format

### Old Format (if you had custom configs)

```json
{
  "frame_count": 14,
  "fps": 15.0,
  "layout": "horizontal"
}
```

### New Format (modular support)

```json
{
  "default_fps": 15.0,
  "default_layout": "horizontal",
  
  "files": {
    "Stand.png": { "frame_count": 1 },
    "Walk.png": { "frame_count": 8, "fps": 12.0 },
    "Attack.png": { "frame_count": 12, "fps": 20.0 }
  },
  
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "attack1": "Attack.png"
  }
}
```

**Backward Compatible:** Old format still works!

### Migration Script

```python
import json
import os
from pathlib import Path

def migrate_sprite_json(folder):
    json_path = Path(folder) / "sprite.json"
    
    if not json_path.exists():
        return
    
    with open(json_path) as f:
        config = json.load(f)
    
    # Already new format?
    if "files" in config or "default_fps" in config:
        return
    
    # Check if modular (has mode-specific files)
    has_stand = (Path(folder) / "Stand.png").exists()
    has_walk = (Path(folder) / "Walk.png").exists()
    
    if has_stand or has_walk:
        # Convert to modular format
        new_config = {
            "default_fps": config.get("fps", 15.0),
            "default_layout": config.get("layout", "horizontal"),
            "files": {}
        }
        
        # Find all PNG files
        for png in Path(folder).glob("*.png"):
            # Extract frame count from image or use default
            frame_count = config.get("frame_count", 8)
            
            new_config["files"][png.name] = {
                "frame_count": frame_count
            }
        
        with open(json_path, 'w') as f:
            json.dump(new_config, f, indent=2)
        
        print(f"Migrated: {folder}")

# Run on all folders
for folder in Path("assets/images/missiles").iterdir():
    if folder.is_dir():
        migrate_sprite_json(folder)
```

---

## Testing the Migration

### Visual Comparison Tool

```rust
// Create a side-by-side comparison
fn test_migration_visually() {
    // Load old system
    let old_registry = load_old_system();
    
    // Load new system
    let new_registry = load_new_system();
    
    // Compare for each entity
    for missile in Missiles::iter() {
        let id = missile as u32;
        
        for mode in UnitAnimationMode::iter() {
            for frame_idx in 0..20 {
                let time = frame_idx as f32 * 0.1;
                
                let old_frame = old_registry.get_missile_frame(id, mode, time);
                let new_frame = new_registry.get_missile_frame(id, mode, time);
                
                if old_frame != new_frame {
                    println!("MISMATCH: Missile {}, Mode {:?}, Time {:.1}",
                             id, mode, time);
                    println!("  Old: {}", old_frame);
                    println!("  New: {}", new_frame);
                }
            }
        }
    }
}
```

### Automated Test Suite

```rust
#[test]
fn test_all_entities_load() {
    let registry = create_new_registry();
    
    // Test missiles
    for missile in Missiles::iter() {
        let id = missile as u32;
        let wrapper = MissileWrapper(missile);
        
        if wrapper.sprite_folder_name().is_empty() {
            continue;  // Skip entities without sprites
        }
        
        assert!(
            registry.has_missile(id),
            "Missile {} ({}) not loaded",
            id,
            wrapper.sprite_folder_name()
        );
    }
    
    // Same for monsters, players, objects...
}

#[test]
fn test_frame_calculations_match() {
    let test_cases = vec![
        (260, UnitAnimationMode::Walk, 0.0, "FrozenOrb/0"),
        (260, UnitAnimationMode::Walk, 0.5, "FrozenOrb/7"),
        (260, UnitAnimationMode::Walk, 1.0, "FrozenOrb/14"),
    ];
    
    let registry = create_new_registry();
    
    for (id, mode, time, expected) in test_cases {
        let actual = registry.get_missile_frame(id, mode, time);
        assert_eq!(actual, expected, "Frame mismatch at t={}", time);
    }
}
```

---

## Rollback Plan

If migration fails, you can rollback:

### 1. Keep Old Code (Temporarily)

```rust
#[cfg(feature = "old-animation-system")]
fn initialize_old_system(&mut self) {
    // Old registration code
}

#[cfg(not(feature = "old-animation-system"))]
fn initialize_new_system(&mut self) {
    // New discovery code
}
```

Compile with:
```bash
cargo build --features old-animation-system  # Use old
cargo build                                   # Use new
```

### 2. Git Branches

```bash
git checkout -b migration-auto-discovery
# ... make changes ...
git commit -m "Migrate to auto-discovery system"

# If problems:
git checkout main  # Back to old system
```

### 3. Runtime Toggle

```rust
enum AnimationSystemMode {
    Legacy,
    AutoDiscovery,
}

impl GameState {
    fn initialize_animations(&mut self, mode: AnimationSystemMode) {
        match mode {
            AnimationSystemMode::Legacy => self.init_legacy(),
            AnimationSystemMode::AutoDiscovery => self.init_autodiscovery(),
        }
    }
}
```

---

## Common Issues

### Issue 1: Missing Sprite Folders

**Symptoms:** Console shows "Sprite folder not found: XYZ"

**Solution:** Either:
1. Create the folder with sprites
2. Return "" from `sprite_folder_name()` to skip
3. Provide `default_config()` for graceful fallback

### Issue 2: Typos in Folder Names

**Old:**
```rust
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
// Folder: FrozenOrb ✅
```

**New:**
```rust
Missiles::FrozenOrb => "FrozenOrbb",  // Typo! ❌
// Folder: FrozenOrb
```

**Solution:** Use constants or check against folder list

### Issue 3: Performance Regression

**Problem:** Discovery slower than old hardcoded registration?

**Analysis:**
- Old system: 0 disk I/O (hardcoded)
- New system: Scans folders, reads JSON

**Solution:** Cache registry after first load
```rust
// Save registry state after discovery
registry.save_to_cache("animation_cache.bin");

// Next startup: load from cache
if let Ok(registry) = UnitAnimationRegistry::load_from_cache("animation_cache.bin") {
    return registry;
}
// Fall back to discovery if cache invalid/missing
```

### Issue 4: Breaking Existing Saves

**Problem:** Old saves reference frame names differently?

**Solution:** Add compatibility layer
```rust
fn normalize_frame_name(old_name: &str) -> String {
    // Old: "frozen_orb_7"
    // New: "FrozenOrb/7"
    
    let parts: Vec<&str> = old_name.split('_').collect();
    if let Some(num) = parts.last() {
        if num.parse::<usize>().is_ok() {
            let name = parts[..parts.len()-1].join("_");
            return format!("{}/{}", to_pascal_case(&name), num);
        }
    }
    old_name.to_string()
}
```

---

## Migration Checklist

- [ ] Audit all `register_*_simple()` calls
- [ ] Create wrapper structs for each entity type
- [ ] Implement `SpriteDiscoverable` trait
- [ ] Create `sprite.json` files for all folders
- [ ] Update initialization code to use `from_discovered()`
- [ ] Write migration tests
- [ ] Run visual comparison tool
- [ ] Test with actual game
- [ ] Verify performance
- [ ] Document any custom configurations
- [ ] Train team on new system
- [ ] Archive old registration code
- [ ] Update documentation

---

## Benefits After Migration

### Before
```rust
// 1000+ lines of registration code
registry.register_missile_simple(260, "FrozenOrb", 14, 15.0);
// ... 599 more missiles ...
registry.register_monster_simple(1, "Zombie", 8, 12.0);
// ... 199 more monsters ...
```

### After
```rust
// 30 lines of discovery code
let registry = UnitAnimationRegistry::from_discovered(
    &all_entities,
    &search_paths,
    Some(&dc6_loader)
);
```

**Results:**
- 97% less code
- Users can add sprites
- Per-mode configurations
- Easy to maintain
- Self-documenting (JSON files)

---

## Next Steps

After successful migration:

1. **Document Custom Sprites** - Create user guide for sprite installation
2. **Create Templates** - Provide example sprite.json files
3. **Build Tools** - Use [Sheet Analyzer](../tools/sheet-analyzer.html) for config generation
4. **Monitor Performance** - Profile animation system under load
5. **Gather Feedback** - Ask team about new workflow

## See Also

- **[Developer Guide](../animation/developer-guide.html)** - Detailed implementation
- **[Examples](./examples.html)** - Real-world usage patterns
- **[User Guide](./user-guide.html)** - For non-programmers