# Hackers Framework Internals

## Overview

**hackers** is the core framework that provides a plugin architecture. It defines the `HaCK` trait, the `HaCKS` container, GUI abstractions, and inter-module communication systems.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    hackers crate                        │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │  HaCK Trait (hackrs/hack.rs)                      │  │
│  │  - Lifecycle methods                              │  │
│  │  - Rendering methods                              │  │
│  │  - Metadata access                                │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │  HaCKS Container (hackrs/HaCKS.rs)                │  │
│  │  - Module storage (HashMap<TypeId, RefCell>)      │  │
│  │  - Lifecycle orchestration                        │  │
│  │  - Sync registry                                  │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │  GUI Abstraction (gui/)                           │  │
│  │  - UiBackend trait                                │  │
│  │  - DrawList trait                                 │  │
│  │  - ImguiBackend implementation                    │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Utilities                                        │  │
│  │  - Hotkey Manager                                 │  │
│  │  - Font Registry                                  │  │
│  │  - State Tracker                                  │  │
│  │  - Access Control                                 │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## HaCK Trait Deep Dive

### Trait Definition

```rust
pub trait HaCK: ErasedSerialize + Send + 'static {
    // === Identity ===
    fn name(&self) -> &str;
    fn nac_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    
    // === Lifecycle ===
    fn init(&mut self) {}
    fn post_load_init(&mut self) {}
    fn on_load(&mut self) {}
    fn on_unload(&mut self) {}
    fn exit(&mut self) {}
    
    // === Runtime ===
    fn update(&mut self, hacs: &HaCKS) {}
    fn before_render(&mut self, ui: &dyn UiBackend) {}
    
    // === Rendering ===
    fn render_menu(&mut self, ui: &dyn UiBackend) {}
    fn render_window(&mut self, ui: &dyn UiBackend) {}
    fn render_draw(&mut self, ui: &dyn UiBackend, 
                   fg_draw: &mut dyn DrawList, 
                   bg_draw: &mut dyn DrawList) {}
    
    // === Metadata ===
    fn metadata(&self) -> &HaCMetadata;
    fn metadata_mut(&mut self) -> &mut HaCMetadata;
    
    // === Type Casting ===
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    // === Serialization ===
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
    fn to_json_data_only(&self) -> Result<serde_json::Value, serde_json::Error>;
}
```

### Trait Bounds

- **`ErasedSerialize`**: Enables type-erased serialization via `erased_serde`
- **`Send`**: Allows transfer between threads (required for async operations)
- **`'static`**: No borrowed references (modules must own their data)

### Boilerplate Macro

The `impl_hac_boilerplate!` macro generates common implementations:

```rust
macro_rules! impl_hac_boilerplate {
    ($struct_name:ident, $metadata_field:ident) => {
        fn metadata(&self) -> &HaCMetadata {
            &self.$metadata_field
        }
        
        fn metadata_mut(&mut self) -> &mut HaCMetadata {
            &mut self.$metadata_field
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        
        fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
            serde_json::to_value(self)
        }
        
        fn to_json_data_only(&self) -> Result<serde_json::Value, serde_json::Error> {
            // Serialize without metadata
            // Implementation details...
        }
        
        fn nac_type_id(&self) -> TypeId {
            TypeId::of::<$struct_name>()
        }
    };
}
```

## HaCKS Container

### Internal Structure

```rust
pub struct HaCKS {
    // Module storage: TypeId -> RefCell<Box<dyn HaCK>>
    pub(crate) hacs: HashMap<TypeId, RefCell<Box<dyn HaCK>>>,
    
    // Event bus for inter-module communication, not implemented yet
    event_bus: EventBus,
    
    // Sync registry for Rust -> JS data sharing
    sync_registry: Option<SyncRegistry>,
    
    // Access control for module permissions
    access_manager: AccessManager,
}
```

### Module Storage

Modules are stored in a `HashMap` keyed by `TypeId`:

```rust
// Insert module
let type_id = TypeId::of::<MyModule>();
hacs.insert(type_id, RefCell::new(Box::new(module)));

// Retrieve module
if let Some(cell) = hacs.get(&TypeId::of::<MyModule>()) {
    let module = cell.borrow();  // Ref<Box<dyn HaCK>>
    // Use module...
}
```

**Why `RefCell`?**
- Enables interior mutability
- Allows borrowing modules at runtime
- Enforces Rust's borrowing rules dynamically

**Trade-offs**:
- Runtime borrow checking (can panic if violated)
- Slight performance overhead
- Enables flexible module access patterns

### Lifecycle Orchestration

```rust
impl HaCKS {
    pub fn init(&mut self) {
        for (_, module) in &self.hacs {
            module.borrow_mut().init();
        }
    }
    
    pub fn update(&self) {
        // Sort by update_weight (descending)
        let mut modules: Vec<_> = self.hacs.values().collect();
        modules.sort_by(|a, b| {
            let weight_a = a.borrow().metadata().update_weight;
            let weight_b = b.borrow().metadata().update_weight;
            weight_b.partial_cmp(&weight_a).unwrap()
        });
        
        for module in modules {
            let mut m = module.borrow_mut();
            if m.metadata().is_update_enabled {
                m.update(self);
            }
        }
    }
    
    pub fn render_menu(&self, ui: &dyn UiBackend) {
        // Similar to update, but for menu rendering
        for (_, module) in &self.hacs {
            let mut m = module.borrow_mut();
            if m.metadata().is_menu_enabled {
                m.render_menu(ui);
            }
        }
    }
}
```

### Module Access

**Type-Safe Retrieval**:

```rust
pub fn get_module<T: HaCK + 'static>(&self) -> Option<Ref<'_, T>> {
    let type_id = TypeId::of::<T>();
    let cell = self.hacs.get(&type_id)?;
    
    let borrowed = cell.borrow();
    
    // Downcast Box<dyn HaCK> to &T
    let any = borrowed.as_any();
    let concrete = any.downcast_ref::<T>()?;
    
    // Return Ref that keeps borrow alive
    Some(Ref::map(borrowed, |_| concrete))
}
```

**Mutable Access**:

```rust
pub fn get_module_mut<T: HaCK + 'static>(&self) -> Option<RefMut<'_, T>> {
    let type_id = TypeId::of::<T>();
    let cell = self.hacs.get(&type_id)?;
    
    let borrowed = cell.borrow_mut();
    
    // Downcast and return mutable reference
    Some(RefMut::map(borrowed, |b| {
        b.as_any_mut().downcast_mut::<T>().unwrap()
    }))
}
```
<!-- 
## Event Bus

### Architecture

```rust
pub struct EventBus {
    // TypeId -> Vec<Box<dyn Any>>
    events: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl EventBus {
    pub fn send<T: 'static>(&mut self, event: T) {
        let type_id = TypeId::of::<T>();
        self.events.entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(event));
    }
    
    pub fn receive<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let events = self.events.get_mut(&type_id)?;
        
        if events.is_empty() {
            return None;
        }
        
        let boxed = events.remove(0);
        let any = boxed.downcast::<T>().ok()?;
        Some(*any)
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
```

### Usage Pattern

```rust
// Module A: Send event
#[derive(Clone)]
struct ItemPickedEvent {
    item_id: u32,
}

hacs.send_event(ItemPickedEvent { item_id: 123 });

// Module B: Receive event
if let Some(event) = hacs.receive_event::<ItemPickedEvent>() {
    log::info!("Item {} was picked", event.item_id);
}
```

**Event Lifecycle**:
1. Events sent during frame are queued
2. `process_events()` called at end of frame
3. Events cleared for next frame -->

## Sync Registry

### Purpose

Bridge Rust module state to JavaScript runtime.

### Architecture

```rust
pub trait SyncProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn sync_to_js(&self, hacs: &HaCKS) -> Option<serde_json::Value>;
}

pub struct SyncRegistry {
    providers: HashMap<&'static str, Box<dyn SyncProvider>>,
}

impl SyncRegistry {
    pub fn register_sync_provider(&mut self, provider: Box<dyn SyncProvider>) {
        let name = provider.name();
        self.providers.insert(name, provider);
    }
    
    pub fn sync_all(&self, hacs: &HaCKS) -> serde_json::Value {
        let mut data = serde_json::Map::new();
        
        for (name, provider) in &self.providers {
            if let Some(value) = provider.sync_to_js(hacs) {
                data.insert(name.to_string(), value);
            }
        }
        
        serde_json::Value::Object(data)
    }
}
```

### Example Implementation

```rust
pub struct StateSync;

impl SyncProvider for StateSync {
    fn name(&self) -> &'static str {
        "state"
    }
    
    fn sync_to_js(&self, hacs: &HaCKS) -> Option<serde_json::Value> {
        let state = hacs.get_module::<State>()?;
        
        Some(serde_json::json!({
            "in_game": state.in_game,
            "in_menu": state.in_menu,
            "player_name": state.player_name,
            "area": format!("{:?}", state.current_area),
        }))
    }
}
```

## GUI Abstraction

### UiBackend Trait

Abstracts ImGui operations for backend independence:

```rust
pub trait UiBackend {
    // Windows
    fn window(&self, name: &str) -> WindowBuilder;
    
    // Widgets
    fn text(&self, text: &str);
    fn button(&self, label: &str) -> bool;
    fn checkbox(&self, label: &str, value: &mut bool) -> bool;
    fn slider_float(&self, label: &str, value: &mut f32, min: f32, max: f32) -> bool;
    fn input_text(&self, label: &str, buf: &mut String) -> bool;
    
    // Layout
    fn same_line(&self);
    fn separator(&self);
    fn spacing(&self);
    
    // Trees
    fn tree_node(&self, label: &str) -> TreeNodeBuilder;
    
    // Tables
    fn begin_table(&self, id: &str, column_count: usize) -> Option<TableToken>;
    
    // Input
    fn is_key_pressed(&self, key: Key) -> bool;
    fn is_mouse_clicked(&self, button: MouseButton) -> bool;
    
    // ... many more methods
}
```

### Builder Pattern

Builders enable fluent API for complex widgets:

```rust
pub struct WindowBuilder<'a> {
    ui: &'a dyn UiBackend,
    name: String,
    size: Option<[f32; 2]>,
    position: Option<[f32; 2]>,
    flags: WindowFlags,
}

impl<'a> WindowBuilder<'a> {
    pub fn size(mut self, size: [f32; 2], condition: Condition) -> Self {
        self.size = Some(size);
        self
    }
    
    pub fn position(mut self, pos: [f32; 2], condition: Condition) -> Self {
        self.position = Some(pos);
        self
    }
    
    pub fn flags(mut self, flags: WindowFlags) -> Self {
        self.flags = flags;
        self
    }
    
    pub fn begin(self) -> Option<WindowToken<'a>> {
        // Create window and return RAII token
        // ...
    }
}
```

### RAII Tokens

Tokens ensure proper cleanup:

```rust
pub struct WindowToken<'a> {
    ui: &'a dyn UiBackend,
}

impl<'a> Drop for WindowToken<'a> {
    fn drop(&mut self) {
        // Automatically call end_window()
        self.ui.end_window();
    }
}
```

Usage:

```rust
if let Some(_token) = ui.window("My Window").begin() {
    ui.text("Content");
    // Window automatically ended when token drops
}
```

### DrawList Trait

Abstracts drawing operations:

```rust
pub trait DrawList {
    fn add_line(&mut self, p1: [f32; 2], p2: [f32; 2], color: [f32; 4]) -> LineBuilder;
    fn add_rect(&mut self, min: [f32; 2], max: [f32; 2], color: [f32; 4]) -> RectBuilder;
    fn add_circle(&mut self, center: [f32; 2], radius: f32, color: [f32; 4]) -> CircleBuilder;
    fn add_text(&mut self, pos: [f32; 2], color: [f32; 4], text: &str);
    // ... more drawing primitives
}
```

## Access Control

### Purpose

Control which modules can access sensitive operations.

### Architecture

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessLevel {
    Public,      // Anyone can access
    Protected,   // Trusted modules only
    Private,     // Owner module only
}

pub struct AccessManager {
    permissions: HashMap<TypeId, HashMap<String, AccessLevel>>,
}

impl AccessManager {
    pub fn grant_access(&mut self, module_id: TypeId, resource: &str, level: AccessLevel) {
        self.permissions.entry(module_id)
            .or_insert_with(HashMap::new)
            .insert(resource.to_string(), level);
    }
    
    pub fn check_access(&self, module_id: TypeId, resource: &str, required: AccessLevel) -> bool {
        if let Some(perms) = self.permissions.get(&module_id) {
            if let Some(&level) = perms.get(resource) {
                return level >= required;
            }
        }
        false
    }
}
```

## State Tracker

### Purpose

Track game state changes and notify modules.

### Architecture

```rust
pub struct StateTracker<T: Clone> {
    current: T,
    previous: Option<T>,
    changed: bool,
}

impl<T: Clone + PartialEq> StateTracker<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: initial,
            previous: None,
            changed: false,
        }
    }
    
    pub fn update(&mut self, new_value: T) {
        if self.current != new_value {
            self.previous = Some(self.current.clone());
            self.current = new_value;
            self.changed = true;
        } else {
            self.changed = false;
        }
    }
    
    pub fn get(&self) -> &T {
        &self.current
    }
    
    pub fn changed(&self) -> bool {
        self.changed
    }
    
    pub fn previous(&self) -> Option<&T> {
        self.previous.as_ref()
    }
}
```

## Hotkey Manager

### Architecture

```rust
pub struct Hotkey {
    key: Key,
    modifiers: Modifiers,
}

pub struct HotkeyManager {
    hotkeys: HashMap<String, (Hotkey, Instant, Duration)>,
}

impl HotkeyManager {
    pub fn register(&mut self, id: &str, hotkey: Hotkey, cooldown: Duration) {
        self.hotkeys.insert(
            id.to_string(),
            (hotkey, Instant::now(), cooldown)
        );
    }
    
    pub fn is_pressed(&mut self, id: &str) -> bool {
        if let Some((hotkey, last_press, cooldown)) = self.hotkeys.get_mut(id) {
            if last_press.elapsed() >= *cooldown {
                if hotkey.is_pressed() {
                    *last_press = Instant::now();
                    return true;
                }
            }
        }
        false
    }
}
```

## Font Registry

### Architecture

```rust
pub struct FontRegistry {
    fonts: HashMap<String, FontHandle>,
    default_font: Option<FontHandle>,
}

impl FontRegistry {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font: None,
        }
    }
    
    pub fn add_font(&mut self, name: &str, data: &[u8], size: f32) -> &mut Self {
        let handle = load_font(data, size);
        self.fonts.insert(name.to_string(), handle);
        self
    }
    
    pub fn set_default(&mut self, name: &str) -> &mut Self {
        if let Some(handle) = self.fonts.get(name) {
            self.default_font = Some(*handle);
        }
        self
    }
    
    pub fn get(&self, name: &str) -> Option<FontHandle> {
        self.fonts.get(name).copied()
    }
}
```

## Best Practices

### Module Design

1. **Single Responsibility**: Each module should have one clear purpose
2. **Minimal Dependencies**: Reduce coupling between modules
3. **Fail Gracefully**: Handle missing dependencies without panicking

### Performance

1. **Lazy Initialization**: Defer expensive operations until needed
2. **Caching**: Store computed results when possible
3. **Weight Tuning**: Use weights to control execution order

### Thread Safety

1. **Avoid Shared Mutable State**: Use message passing instead
2. **RefCell Discipline**: Don't hold borrows across await points
3. **Send + Sync**: Ensure types can be safely shared

### Error Handling

1. **Option/Result**: Use Rust's error types, don't panic
2. **Logging**: Log errors for debugging
3. **Fallbacks**: Provide default behavior when operations fail
