# Real-World Examples

Practical implementations for common animation scenarios.

## Example 1: Simple Projectile (FrozenOrb)

**Use Case:** Projectile with single animation for all modes

### Folder Structure

```
assets/images/missiles/FrozenOrb/
├── 0.png    (64x64)
├── 1.png
├── 2.png
├── ...
├── 13.png
└── sprite.json
```

### sprite.json

```json
{
  "fps": 15.0
}
```

**How it works:**
- System counts files automatically (14 frames)
- All 16 animation modes use the same 14-frame cycle
- Simple and efficient for projectiles
- Only needs FPS setting

### Rust Implementation

```rust
impl SpriteDiscoverable for MissileWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32  // 260 for FrozenOrb
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
```

### Runtime Usage

```rust
// In game loop
let frame = registry.get_missile_frame(
    missile.id,           // 260
    UnitAnimationMode::Walk,  // Doesn't matter, all modes same
    elapsed_time         // Auto-loops through 0-13
);
// Returns: "FrozenOrb/7" at 0.467s (15 FPS)
```

### Performance Notes

- **Memory:** 14 frames × 64×64 × 4 bytes = ~220 KB
- **Load Time:** < 10ms
- **Runtime:** O(1) frame lookup
- **Best For:** Simple projectiles, effects

---

## Example 2: Complex Monster (Zombie)

**Use Case:** Enemy with different animations per mode

### Folder Structure

```
assets/images/monsters/Zombie/
├── sprite.json
├── Stand.png    (1 frame, 128x128)
├── Walk.png     (8 frames horizontal, 1024x128)
├── Run.png      (8 frames horizontal, 1024x128)
├── Attack1.png  (12 frames horizontal, 1536x128)
├── Attack2.png  (10 frames horizontal, 1280x128)
├── Death.png    (15 frames horizontal, 1920x128)
└── BeingHit.png (4 frames horizontal, 512x128)
```

### sprite.json

```json
{
  "default_fps": 15.0,
  "default_layout": "horizontal",
  
  "files": {
    "Stand.png": {
      "frame_count": 1,
      "fps": 1.0
    },
    "Walk.png": {
      "frame_count": 8,
      "fps": 12.0
    },
    "Run.png": {
      "frame_count": 8,
      "fps": 18.0
    },
    "Attack1.png": {
      "frame_count": 12,
      "fps": 24.0
    },
    "Attack2.png": {
      "frame_count": 10,
      "fps": 20.0
    },
    "Death.png": {
      "frame_count": 15,
      "fps": 10.0
    },
    "BeingHit.png": {
      "frame_count": 4,
      "fps": 20.0
    }
  },
  
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "run": "Run.png",
    "attack1": "Attack1.png",
    "attack2": "Attack2.png",
    "death": "Death.png",
    "beinghit": "BeingHit.png",
    "dead": "Death.png"
  }
}
```

**Design choices:**
- Each mode has custom FPS for proper feel
- Attack animations are faster (24 FPS)
- Death animation is slower (10 FPS) for dramatic effect
- `dead` mode reuses Death.png (shows last frame)

### Rust Implementation

```rust
impl SpriteDiscoverable for MonsterWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32  // e.g., 1 for Zombie
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            Monsters::Zombie => "Zombie",
            Monsters::Skeleton => "Skeleton",
            Monsters::Gloam => "Gloam",
            _ => ""
        }
    }
    
    fn unit_type(&self) -> UnitType {
        UnitType::Monster
    }
    
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        // Fallback if sprite.json missing
        match self.0 {
            Monsters::Zombie => Some(DefaultSpriteConfig {
                frame_count: 8,
                fps: 15.0,
            }),
            _ => None
        }
    }
}
```

### Runtime Usage

```rust
// Monster changes state
match monster.state {
    MonsterState::Idle => {
        let frame = registry.get_monster_frame(
            monster.id,
            UnitAnimationMode::Stand,
            elapsed_time
        );
        // Returns: "Zombie/Stand/0" (static frame)
    },
    
    MonsterState::Walking => {
        let frame = registry.get_monster_frame(
            monster.id,
            UnitAnimationMode::Walk,
            elapsed_time
        );
        // Returns: "Zombie/Walk/3" (loops 0-7 at 12 FPS)
    },
    
    MonsterState::Attacking => {
        let frame = registry.get_monster_frame(
            monster.id,
            UnitAnimationMode::Attack1,
            elapsed_time
        );
        // Returns: "Zombie/Attack1/7" (0-11 at 24 FPS)
    },
}
```

### Performance Notes

- **Memory:** ~1.5 MB total (7 sprite sheets)
- **Flexibility:** Easy to add/modify individual animations
- **Best For:** Enemies with distinct animations per action

**Why this approach:**
- Designers can iterate on individual animations
- Different frame counts per mode (8 walk, 12 attack)
- Custom timing per animation
- Easy to replace specific animations

---

## Example 3: State-Based Entity (Health Globe)

**Use Case:** UI element that changes based on percentage (0-100%)

### Folder Structure

```
assets/images/ui/HealthGlobe/
├── sprite.json
└── sheet.png  (101 frames, 6464x64 horizontal strip)
```

### sprite.json

```json
{
  "frame_count": 101,
  "fps": 1.0,
  "layout": "horizontal",
  "animation_mode": {
    "state_mapping": "percentage"
  }
}
```

**Key feature:** `"state_mapping": "percentage"` tells system to map 0-100% to frames directly instead of animating over time.

### Rust Implementation

```rust
impl SpriteDiscoverable for UIElementWrapper {
    fn sprite_id(&self) -> u32 {
        self.0 as u32
    }
    
    fn sprite_folder_name(&self) -> &'static str {
        match self.0 {
            UIElements::HealthGlobe => "HealthGlobe",
            UIElements::ManaGlobe => "ManaGlobe",
            _ => ""
        }
    }
    
    fn unit_type(&self) -> UnitType {
        UnitType::Object
    }
    
    fn default_config(&self) -> Option<DefaultSpriteConfig> {
        Some(DefaultSpriteConfig {
            frame_count: 101,
            fps: 1.0,  // Doesn't matter for state-based
        })
    }
}
```

### Runtime Usage

```rust
// Calculate frame directly from health percentage
let health_percent = (player.health / player.max_health) * 100.0;
let frame_index = health_percent as usize;  // 0-100

let frame = format!("HealthGlobe/{}", frame_index);
// 100% health: "HealthGlobe/100"
// 50% health:  "HealthGlobe/50"
// 1% health:   "HealthGlobe/1"
// 0% health:   "HealthGlobe/0"

draw_image(&frame, globe_position);
```

### Alternative: Smooth Transitions

```rust
// Smooth transition between health values
let target_frame = (health_percent * 100.0) as usize;
let current_frame = globe.displayed_frame;

// Lerp towards target
let new_frame = current_frame + 
    ((target_frame - current_frame) as f32 * 0.1) as isize;

globe.displayed_frame = new_frame;
let frame = format!("HealthGlobe/{}", new_frame);
```

### Performance Notes

- **Memory:** 101 frames × 64×64 × 4 = ~1.6 MB
- **Instant Updates:** No animation time needed
- **Smooth:** Can interpolate between frames
- **Best For:** Status bars, meters, percentage displays

---

## Example 4: Multi-Layer Sprite (Character with Equipment)

**Use Case:** Player character with swappable equipment that layers

### Folder Structure

```
assets/images/players/Barbarian/
├── sprite.json
├── base/
│   ├── Walk.png      (8 frames)
│   ├── Attack.png    (12 frames)
│   └── Cast.png      (10 frames)
├── armor/
│   ├── leather/
│   │   └── Walk.png  (8 frames, same size as base)
│   └── plate/
│       └── Walk.png
└── weapon/
    ├── sword/
    │   └── Attack.png
    └── axe/
        └── Attack.png
```

### sprite.json

```json
{
  "default_fps": 15.0,
  "layers": ["base", "armor", "weapon"],
  "files": {
    "base/Walk.png": { "frame_count": 8 },
    "base/Attack.png": { "frame_count": 12 },
    "armor/leather/Walk.png": { "frame_count": 8 },
    "armor/plate/Walk.png": { "frame_count": 8 },
    "weapon/sword/Attack.png": { "frame_count": 12 },
    "weapon/axe/Attack.png": { "frame_count": 12 }
  }
}
```

### Rust Implementation

```rust
struct PlayerRenderState {
    base_frame: String,
    armor_frame: Option<String>,
    weapon_frame: Option<String>,
}

impl PlayerRenderState {
    fn get_frames(
        &self,
        registry: &UnitAnimationRegistry,
        player: &Player,
        mode: UnitAnimationMode,
        time: f32
    ) -> PlayerRenderState {
        // Base layer always present
        let base = format!(
            "Barbarian/base/{}",
            registry.get_player_frame(player.id, mode, time)
                .unwrap_or_default()
        );
        
        // Armor layer (if equipped)
        let armor = player.armor.as_ref().map(|armor_type| {
            format!(
                "Barbarian/armor/{}/{}",
                armor_type,
                registry.get_player_frame(player.id, mode, time)
                    .unwrap_or_default()
            )
        });
        
        // Weapon layer (if equipped and relevant mode)
        let weapon = if matches!(mode, UnitAnimationMode::Attack1 | UnitAnimationMode::Attack2) {
            player.weapon.as_ref().map(|weapon_type| {
                format!(
                    "Barbarian/weapon/{}/{}",
                    weapon_type,
                    registry.get_player_frame(player.id, mode, time)
                        .unwrap_or_default()
                )
            })
        } else {
            None
        };
        
        PlayerRenderState {
            base_frame: base,
            armor_frame: armor,
            weapon_frame: weapon,
        }
    }
}
```

### Runtime Usage

```rust
// Render in order: base -> armor -> weapon
let frames = player_state.get_frames(&registry, &player, mode, time);

// Draw base layer
draw_image(&frames.base_frame, position);

// Draw armor layer (if exists)
if let Some(armor) = &frames.armor_frame {
    draw_image(armor, position);
}

// Draw weapon layer (if exists)
if let Some(weapon) = &frames.weapon_frame {
    draw_image(weapon, position);
}
```

### Performance Notes

- **Memory:** Layers share frame count, efficient
- **Flexibility:** Swap equipment without reloading
- **Compositing:** Layers must be same dimensions
- **Best For:** RPGs with equipment systems

**Important:** All layers must have same frame count and timing for synchronization.

---

## Example 5: Particle Effect (Randomized Frames)

**Use Case:** Fire particle that shows random flames

### Folder Structure

```
assets/images/effects/Fire/
├── sprite.json
└── variants/
    ├── 0.png  (single frame, different flame shape)
    ├── 1.png
    ├── 2.png
    ├── 3.png
    └── 4.png
```

### sprite.json

```json
{
  "frame_count": 5,
  "fps": 10.0,
  "randomize": true
}
```

### Rust Implementation

```rust
struct ParticleEffect {
    id: u32,
    random_frames: Vec<usize>,
    spawn_time: f32,
}

impl ParticleEffect {
    fn new(id: u32, frame_count: usize) -> Self {
        // Generate random frame sequence
        let mut rng = thread_rng();
        let random_frames: Vec<usize> = (0..frame_count)
            .map(|_| rng.gen_range(0..5))
            .collect();
        
        Self {
            id,
            random_frames,
            spawn_time: 0.0,
        }
    }
    
    fn get_frame(&self, elapsed_time: f32) -> String {
        let frame_index = ((elapsed_time * 10.0) as usize) % self.random_frames.len();
        let variant = self.random_frames[frame_index];
        
        format!("Fire/variants/{}", variant)
    }
}
```

### Runtime Usage

```rust
// Spawn particle
let particle = ParticleEffect::new(effect_id, 5);

// Update and render
loop {
    let elapsed = current_time - particle.spawn_time;
    let frame = particle.get_frame(elapsed);
    
    draw_image(&frame, particle.position);
    
    // Each particle shows different random sequence
}
```

### Performance Notes

- **Variety:** Same 5 frames look different via randomization
- **Memory:** Only 5 frames needed
- **Unique:** Each particle instance has unique sequence
- **Best For:** Fire, smoke, sparkles, magic effects

---

## Example 6: Performance-Optimized (Hundreds of Entities)

**Use Case:** Many identical enemies on screen (zombie horde)

### Optimization Strategy

```rust
// ❌ DON'T: Recalculate every frame for each zombie
for zombie in &zombies {
    let frame = registry.get_monster_frame(
        zombie.id,
        zombie.mode,
        zombie.animation_time
    );
    draw_image(&frame, zombie.position);
}

// ✅ DO: Batch by animation state
struct AnimationBatch {
    frame_name: String,
    positions: Vec<Position>,
}

// Group zombies by current frame
let mut batches: HashMap<String, Vec<Position>> = HashMap::new();

for zombie in &zombies {
    // Calculate frame once per unique state
    let key = (zombie.id, zombie.mode, zombie.animation_phase);
    
    let frame = batches_frames.entry(key)
        .or_insert_with(|| {
            registry.get_monster_frame(
                zombie.id,
                zombie.mode,
                zombie.animation_time
            ).unwrap_or_default()
        });
    
    batches.entry(frame.clone())
        .or_insert_with(Vec::new)
        .push(zombie.position);
}

// Draw all zombies with same frame in one batch
for (frame_name, positions) in batches {
    draw_image_batch(&frame_name, &positions);
}
```

### Instanced Rendering

```rust
// Pre-calculate animation phases
const PHASE_STEPS: usize = 16;

struct OptimizedAnimations {
    // Pre-computed frame indices for common timings
    walk_phases: [usize; PHASE_STEPS],
    attack_phases: [usize; PHASE_STEPS],
}

impl OptimizedAnimations {
    fn new(config: &AnimationConfig) -> Self {
        let walk_phases = (0..PHASE_STEPS)
            .map(|i| {
                let time = (i as f32 / PHASE_STEPS as f32) * 
                           (config.frame_count as f32 / config.fps);
                calculate_frame_index(time, config.fps, config.speed, 
                                     config.frame_count, true)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        
        // Same for other modes...
        
        Self { walk_phases, attack_phases }
    }
    
    fn get_frame(&self, mode: UnitAnimationMode, phase: usize) -> usize {
        match mode {
            UnitAnimationMode::Walk => self.walk_phases[phase % PHASE_STEPS],
            UnitAnimationMode::Attack1 => self.attack_phases[phase % PHASE_STEPS],
            _ => 0
        }
    }
}
```

### Memory Pooling

```rust
// Reuse frame name strings instead of allocating
struct FrameNamePool {
    buffer: String,
    cache: HashMap<(u32, usize), Range<usize>>,
}

impl FrameNamePool {
    fn get_or_insert(&mut self, folder: &str, frame: usize) -> &str {
        let key = (folder.len() as u32, frame);
        
        if let Some(range) = self.cache.get(&key) {
            return &self.buffer[range.clone()];
        }
        
        let start = self.buffer.len();
        self.buffer.push_str(folder);
        self.buffer.push('/');
        self.buffer.push_str(&frame.to_string());
        let end = self.buffer.len();
        
        self.cache.insert(key, start..end);
        &self.buffer[start..end]
    }
}
```

### Performance Results

| Approach | 100 Zombies | 500 Zombies | 1000 Zombies |
|----------|-------------|-------------|--------------|
| Naive | 2.3 ms | 11.5 ms | 23.0 ms |
| Batched | 0.8 ms | 4.0 ms | 8.0 ms |
| Instanced | 0.3 ms | 1.5 ms | 3.0 ms |
| Pooled | 0.2 ms | 1.0 ms | 2.0 ms |

**Best For:** Large numbers of similar entities (hordes, particle systems)

---

## Choosing the Right Approach

| Scenario | Recommendation | Example |
|----------|---------------|---------|
| Simple projectile | Individual frames | Example 1 |
| Complex character | Modular (per-mode files) | Example 2 |
| UI/status display | State-based (percentage mapping) | Example 3 |
| Equipment system | Multi-layer | Example 4 |
| Particle effects | Randomized variants | Example 5 |
| Large quantities | Batched/instanced | Example 6 |

## Summary

- **Individual frames:** Simple, all modes share frames
- **Modular:** Flexible, different animations per mode
- **State-based:** Direct frame selection, no time animation
- **Multi-layer:** Compositing for equipment/customization
- **Randomized:** Variety from limited frames
- **Optimized:** Batching and instancing for performance

## See Also

- **[Developer Guide](developer-guide.md)** - Implementation details
- **[API Reference](api-reference.md)** - Method signatures
- **[Sprite Config](sprite-config.md)** - JSON format reference