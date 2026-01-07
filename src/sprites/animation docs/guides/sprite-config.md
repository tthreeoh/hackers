# Sprite Configuration Reference

Complete guide to `sprite.json` configuration format.

## Overview

The `sprite.json` file configures how sprites are loaded and animated. It's optional - the system can auto-detect many settings, but explicit configuration gives you more control.

**When to use:**
- Multiple sprite files per entity
- Custom FPS or speed per animation mode
- Grid layouts
- State-based animations
- Overriding auto-detection

---

## File Location

Place `sprite.json` in the entity's sprite folder:

```
assets/images/missiles/FrozenOrb/
├── sprite.json          ← Configuration file
├── 0.png - 13.png      ← Sprite frames
```

---

## Basic Configuration

### Minimal (Auto-Detection)

For numbered frames (0.png, 1.png, ...), you only need FPS:

```json
{
  "fps": 15.0
}
```

System auto-detects:
- Frame count (counts files)
- Layout (horizontal strip)

---

### Explicit Frame Count

For sprite sheets, specify frame count:

```json
{
  "frame_count": 14,
  "fps": 15.0,
  "layout": "horizontal"
}
```

---

### Speed Multiplier

Adjust playback speed without changing FPS:

```json
{
  "frame_count": 14,
  "fps": 15.0,
  "speed": 1.5
}
```

- `1.0` = normal speed
- `1.5` = 50% faster
- `0.5` = 50% slower

**Speed affects**: Frame timing only, not FPS value

---

## Sprite Sheet Layouts

### Horizontal Strip (Default)

Frames arranged left-to-right:

```
[Frame0][Frame1][Frame2][Frame3]
```

```json
{
  "frame_count": 4,
  "layout": "horizontal"
}
```

**Best for**: 1-16 frames

---

### Vertical Strip

Frames arranged top-to-bottom:

```
[Frame0]
[Frame1]
[Frame2]
[Frame3]
```

```json
{
  "frame_count": 4,
  "layout": "vertical"
}
```

**Best for**: Rarely used, special vertical animations

---

### Grid Layout

Frames arranged in 2D grid:

```
[F0][F1][F2][F3]
[F4][F5][F6][F7]
[F8][F9][F10][F11]
```

```json
{
  "frame_count": 12,
  "layout": "grid",
  "grid_columns": 4
}
```

**Best for**: 16-64 frames

**Required fields:**
- `layout: "grid"`
- `grid_columns`: Number of columns

System calculates rows automatically: `rows = ceil(frame_count / grid_columns)`

---

## Modular Configuration

Different files for different animation modes.

### Basic Modular

```json
{
  "default_fps": 15.0,
  
  "files": {
    "Stand.png": {
      "frame_count": 1
    },
    "Walk.png": {
      "frame_count": 8
    },
    "Attack.png": {
      "frame_count": 12
    }
  },
  
  "mode_mapping": {
    "stand": "Stand.png",
    "walk": "Walk.png",
    "attack1": "Attack.png",
    "attack2": "Attack.png"
  }
}
```

**Key points:**
- Each file has its own `frame_count`
- Modes map to filenames
- Multiple modes can share files

---

### Per-File Overrides

Each file can override FPS, speed, and layout:

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
    "Attack.png": {
      "frame_count": 16,
      "fps": 24.0,
      "layout": "grid",
      "grid_columns": 4
    }
  }
}
```

**Inheritance:**
- Files inherit `default_fps` unless they override it
- Files inherit `default_layout` unless they specify their own

---

### Mode Mapping

Map animation modes to files:

```json
{
  "mode_mapping": {
    "stand": "Idle.png",
    "walk": "Walk.png",
    "run": "Walk.png",
    "attack1": "Attack.png",
    "attack2": "Attack.png",
    "cast": "Cast.png",
    "death": "Death.png",
    "dead": "Death.png"
  }
}
```

**Available modes:**
- `stand`, `walk`, `run`
- `attack1`, `attack2`
- `cast`
- `death`, `dead`
- `beinghit`, `block`
- `useskill1`, `useskill2`, `useskill3`, `useskill4`
- `beingknockback`
- `sequence`

**Modes not mapped**: Fall back to first file or default animation

---

## Mode-Specific Overrides

Fine-tune individual modes without separate files:

```json
{
  "frame_count": 14,
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
    },
    "cast": {
      "fps": 18.0,
      "speed": 1.3
    }
  }
}
```

**Use when:**
- All modes use same frames
- Just need different timing per mode
- Don't want separate sprite files

---

## State-Based Animation

For UI elements that map state values to frames (health bars, progress indicators).

### Percentage Mapping

Maps 0.0-1.0 to frames:

```json
{
  "frame_count": 101,
  "fps": 1.0,
  "animation_mode": {
    "state_mapping": "percentage"
  }
}
```

**Use for:**
- Health/mana globes (0% = frame 0, 100% = frame 100)
- Progress bars
- Percentage displays

**Behavior:**
- Value 0.0 → frame 0
- Value 0.5 → frame 50
- Value 1.0 → frame 100

---

### Direct Mapping

State value N directly shows frame N:

```json
{
  "frame_count": 50,
  "fps": 1.0,
  "animation_mode": {
    "state_mapping": "direct"
  }
}
```

**Use for:**
- Item stacks (count = frame)
- Direct numeric indicators
- Discrete states

**Behavior:**
- Value 0 → frame 0
- Value 25 → frame 25
- Value 49 → frame 49

---

### Range Mapping

Custom value ranges map to frame ranges:

```json
{
  "frame_count": 50,
  "fps": 1.0,
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
        "max_value": 0.5,
        "start_frame": 11,
        "end_frame": 25
      },
      {
        "min_value": 0.5,
        "max_value": 1.0,
        "start_frame": 26,
        "end_frame": 49
      }
    ]
  }
}
```

**Use for:**
- Non-linear mappings
- Different frame densities per range
- Complex state displays

**Behavior:**
- Finds matching range for value
- Interpolates within that range's frames
- Falls outside all ranges → uses nearest

---

## Complete Examples

### Simple Projectile

All modes share same animation:

```json
{
  "fps": 15.0,
  "speed": 1.0
}
```

Folder structure:
```
FrozenOrb/
├── sprite.json
└── 0.png - 13.png
```

---

### Complex Monster

Different animation per mode:

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
    "dead": "Death.png",
    "beinghit": "BeingHit.png"
  }
}
```

Folder structure:
```
Zombie/
├── sprite.json
├── Stand.png
├── Walk.png
├── Run.png
├── Attack1.png
├── Attack2.png
├── Death.png
└── BeingHit.png
```

---

### Health Globe

State-based percentage:

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

Folder structure:
```
HealthGlobe/
├── sprite.json
└── sheet.png  (6464x64, 101 frames)
```

---

### Grid Layout Animation

Many frames in 2D grid:

```json
{
  "frame_count": 64,
  "fps": 15.0,
  "layout": "grid",
  "grid_columns": 8
}
```

Image layout:
```
8 columns × 8 rows = 64 frames
[F0 ][F1 ][F2 ][F3 ][F4 ][F5 ][F6 ][F7 ]
[F8 ][F9 ][F10][F11][F12][F13][F14][F15]
[F16][F17][F18][F19][F20][F21][F22][F23]
[F24][F25][F26][F27][F28][F29][F30][F31]
[F32][F33][F34][F35][F36][F37][F38][F39]
[F40][F41][F42][F43][F44][F45][F46][F47]
[F48][F49][F50][F51][F52][F53][F54][F55]
[F56][F57][F58][F59][F60][F61][F62][F63]
```

---

## Field Reference

### Top-Level Fields

| Field            | Type    | Required | Default      | Description                  |
| ---------------- | ------- | -------- | ------------ | ---------------------------- |
| `fps`            | number  | No       | 15.0         | Frames per second            |
| `speed`          | number  | No       | 1.0          | Speed multiplier             |
| `frame_count`    | integer | No       | Auto         | Total frames                 |
| `layout`         | string  | No       | "horizontal" | Frame layout type            |
| `grid_columns`   | integer | If grid  | -            | Columns in grid              |
| `default_fps`    | number  | No       | 15.0         | Default FPS for modular      |
| `default_layout` | string  | No       | "horizontal" | Default layout for modular   |
| `files`          | object  | No       | -            | Modular file definitions     |
| `mode_mapping`   | object  | No       | -            | Mode to file mapping         |
| `mode_overrides` | object  | No       | -            | Per-mode settings            |
| `animation_mode` | object  | No       | -            | State-based config           |
| `transitions`    | object  | No       | -            | Transitions between modes    |
| `behaviors`      | object  | No       | -            | Advanced animation behaviors |

---

### Transitions Object

Map "from_mode->to_mode" keys to transition rules.

```json
"transitions": {
  "run->jump": {
    "skip_frames": 2
  }
}
```

| Field         | Type    | Description                       |
| ------------- | ------- | --------------------------------- |
| `skip_frames` | integer | Number of frames to skip at start |

---

### Behaviors Object

Map mode names to behavior rules.

```json
"behaviors": {
  "jump": {
    "loop_range": [4, 5],
    "force_loop": false
  }
}
```

| Field        | Type       | Description                                                              |
| ------------ | ---------- | ------------------------------------------------------------------------ |
| `loop_range` | [int, int] | Range of frames to loop [start, end]                                     |
| `force_loop` | boolean    | If true, loops immediately. If false, plays intro (0..start) then loops. |

---

### File Object Fields

| Field          | Type    | Required | Description              |
| -------------- | ------- | -------- | ------------------------ |
| `frame_count`  | integer | Yes      | Frames in this file      |
| `fps`          | number  | No       | Overrides default_fps    |
| `speed`        | number  | No       | Speed multiplier         |
| `layout`       | string  | No       | Overrides default_layout |
| `grid_columns` | integer | If grid  | Columns in grid          |

---

### Mode Override Fields

| Field   | Type   | Description         |
| ------- | ------ | ------------------- |
| `fps`   | number | FPS for this mode   |
| `speed` | number | Speed for this mode |

---

### Animation Mode Fields

| Field           | Type   | Description                         |
| --------------- | ------ | ----------------------------------- |
| `state_mapping` | string | "percentage", "direct", or "ranges" |
| `ranges`        | array  | Array of range objects (if ranges)  |

---

### Range Object Fields

| Field         | Type    | Description          |
| ------------- | ------- | -------------------- |
| `min_value`   | number  | Minimum state value  |
| `max_value`   | number  | Maximum state value  |
| `start_frame` | integer | First frame in range |
| `end_frame`   | integer | Last frame in range  |

---

## Validation Rules

### File Names

- Must end in `.png` or `.dc6`
- Case-sensitive on Linux/Mac
- No special characters except `-`, `_`, and `.`

### Frame Counts

- Must be positive integer
- For grids: `frame_count <= grid_columns * rows`
- For strips: `frame_count` matches actual frames

### FPS Values

- Must be positive number
- Typical: 10-30 FPS
- Very slow: 1-5 FPS
- Very fast: 30-60 FPS

### Speed Values

- Must be positive number
- `1.0` = normal speed
- `< 1.0` = slower
- `> 1.0` = faster

### Layout Values

- `"horizontal"` (default)
- `"vertical"`
- `"grid"` (requires `grid_columns`)

---

## Common Mistakes

### ❌ Missing Grid Columns

```json
{
  "frame_count": 64,
  "layout": "grid"
  // Missing: "grid_columns"!
}
```

**Error:** `grid_columns required for grid layout`

**Fix:**
```json
{
  "frame_count": 64,
  "layout": "grid",
  "grid_columns": 8
}
```

---

### ❌ Wrong Frame Count

Config says 8 frames, sheet has 10:

```json
{
  "frame_count": 8  // Sheet actually has 10 frames
}
```

**Result:** Last 2 frames never show

**Fix:** Use Sheet Analyzer tool to count frames

---

### ❌ Inconsistent Naming

```json
{
  "files": {
    "Stand.png": { "frame_count": 1 }
  },
  "mode_mapping": {
    "stand": "stand.png"  // Wrong case!
  }
}
```

**Fix:** Match file names exactly

---

### ❌ Missing Mode Mapping

```json
{
  "files": {
    "Walk.png": { "frame_count": 8 }
  }
  // No mode_mapping!
}
```

**Result:** System doesn't know which file to use for which mode

**Fix:** Add mode_mapping or rely on filename matching

---

## Tools

### Sheet Analyzer

Web tool to analyze sprite sheets:

1. Open `tools/sheet-analyzer.html`
2. Upload sprite sheet
3. Configure layout settings
4. View frame grid overlay
5. Save generated `sprite.json`

**Features:**
- Auto-detects frame count
- Shows frame boundaries
- Generates configuration
- Validates layouts

---

### Globe Generator

Generates state-based animations:

1. Open `tools/globe-generator.html`
2. Choose style and colors
3. Generate frames
4. Download sprite sheet and config

---

## Tips

- Start simple: Just FPS, add complexity as needed
- Use Sheet Analyzer to verify frame counts
- Test with one mode before configuring all modes
- Keep configs minimal - system auto-detects when possible
- Document non-obvious choices with comments (JSON doesn't support them, but keep notes elsewhere)

---

## See Also

- **[Developer Guide](developer-guide.md)** - Implementing sprite support
- **[API Reference](api-reference.md)** - Method details
- **[Examples](examples.md)** - Real-world configs