// ============================================================
// Runtime Sync System - Define syncs dynamically via scripting
// ============================================================

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::any::TypeId;
use std::collections::HashMap;

/// A sync defined via JSON/scripting that operates on module data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptedSyncDefinition {
    pub id: String,
    pub source_module: String, // Module type name (e.g., "PanicButton")
    pub target_module: String, // Module type name (e.g., "Chicken")
    pub sync_type: SyncType,
    pub mappings: Vec<FieldMapping>,
    #[serde(default)]
    pub conditions: Vec<SyncCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    /// One-way: source -> target
    OneWay,
    /// Bidirectional with change tracking
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_field: String,
    pub target_field: String,
    #[serde(default)]
    pub field_type: FieldType,
    #[serde(default)]
    pub transform: Option<String>, // JS expression like "value * 100"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    Bool,
    Int,
    Float,
    String,
    #[serde(rename = "Vec<String>")]
    VecString,
    #[serde(rename = "Vec<i32>")]
    VecInt,
    #[serde(rename = "Vec<f32>")]
    VecFloat,
    Json, // Generic JSON value
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::Json
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
}

/// Runtime sync that operates on JSON representations of modules
pub struct RuntimeSync {
    definition: ScriptedSyncDefinition,
    last_source_state: Option<JsonValue>,
    last_target_state: Option<JsonValue>,
}

impl RuntimeSync {
    pub fn new(definition: ScriptedSyncDefinition) -> Self {
        Self {
            definition,
            last_source_state: None,
            last_target_state: None,
        }
    }

    /// Check if source or target changed
    fn detect_changes(&mut self, source_json: &JsonValue, target_json: &JsonValue) -> (bool, bool) {
        let source_changed = self
            .last_source_state
            .as_ref()
            .map(|last| last != source_json)
            .unwrap_or(true);

        let target_changed = self
            .last_target_state
            .as_ref()
            .map(|last| last != target_json)
            .unwrap_or(true);

        (source_changed, target_changed)
    }

    /// Apply sync from source to target
    fn apply_sync(
        &mut self,
        source_json: &JsonValue,
        target_json: &mut JsonValue,
    ) -> Result<bool, String> {
        // Check conditions
        if !self.check_conditions(source_json)? {
            return Ok(false);
        }

        // Apply field mappings
        for mapping in &self.definition.mappings {
            if let Some(source_value) = source_json.get(&mapping.source_field) {
                // Apply transform first (if any)
                let transformed = if let Some(transform) = &mapping.transform {
                    self.apply_transform(source_value, transform)?
                } else {
                    source_value.clone()
                };

                // Then apply type conversion
                let value = self.convert_type(&transformed, &mapping.field_type)?;

                if let Some(obj) = target_json.as_object_mut() {
                    obj.insert(mapping.target_field.clone(), value);
                }
            }
        }

        Ok(true)
    }

    /// Convert JSON value to target type
    fn convert_type(
        &self,
        value: &JsonValue,
        target_type: &FieldType,
    ) -> Result<JsonValue, String> {
        match target_type {
            FieldType::Bool => {
                if let Some(b) = value.as_bool() {
                    Ok(JsonValue::Bool(b))
                } else if let Some(n) = value.as_i64() {
                    Ok(JsonValue::Bool(n != 0))
                } else if let Some(s) = value.as_str() {
                    Ok(JsonValue::Bool(s.to_lowercase() == "true"))
                } else {
                    Err(format!("Cannot convert {:?} to bool", value))
                }
            }
            FieldType::Int => {
                if let Some(i) = value.as_i64() {
                    Ok(JsonValue::Number(i.into()))
                } else if let Some(f) = value.as_f64() {
                    Ok(JsonValue::Number((f as i64).into()))
                } else if let Some(s) = value.as_str() {
                    s.parse::<i64>()
                        .map(|i| JsonValue::Number(i.into()))
                        .map_err(|e| format!("Cannot parse '{}' as int: {}", s, e))
                } else {
                    Err(format!("Cannot convert {:?} to int", value))
                }
            }
            FieldType::Float => {
                if let Some(f) = value.as_f64() {
                    Ok(JsonValue::Number(
                        serde_json::Number::from_f64(f)
                            .ok_or_else(|| "Invalid float value".to_string())?,
                    ))
                } else if let Some(i) = value.as_i64() {
                    Ok(JsonValue::Number(
                        serde_json::Number::from_f64(i as f64)
                            .ok_or_else(|| "Invalid float value".to_string())?,
                    ))
                } else if let Some(s) = value.as_str() {
                    s.parse::<f64>()
                        .and_then(|f| {
                            serde_json::Number::from_f64(f)
                                .ok_or_else(|| s.parse::<f64>().unwrap_err())
                        })
                        .map(JsonValue::Number)
                        .map_err(|e| format!("Cannot parse '{}' as float: {}", s, e))
                } else {
                    Err(format!("Cannot convert {:?} to float", value))
                }
            }
            FieldType::String => {
                if let Some(s) = value.as_str() {
                    Ok(JsonValue::String(s.to_string()))
                } else {
                    Ok(JsonValue::String(value.to_string()))
                }
            }
            FieldType::VecString => {
                if let Some(arr) = value.as_array() {
                    let strings: Result<Vec<String>, String> = arr
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .map(|s| s.to_string())
                                .ok_or_else(|| format!("Array element is not a string: {:?}", v))
                        })
                        .collect();
                    Ok(JsonValue::Array(
                        strings?.into_iter().map(JsonValue::String).collect(),
                    ))
                } else {
                    Err(format!("Cannot convert {:?} to Vec<String>", value))
                }
            }
            FieldType::VecInt => {
                if let Some(arr) = value.as_array() {
                    let ints: Result<Vec<i64>, String> = arr
                        .iter()
                        .map(|v| {
                            v.as_i64()
                                .ok_or_else(|| format!("Array element is not an int: {:?}", v))
                        })
                        .collect();
                    Ok(JsonValue::Array(
                        ints?
                            .into_iter()
                            .map(|i| JsonValue::Number(i.into()))
                            .collect(),
                    ))
                } else {
                    Err(format!("Cannot convert {:?} to Vec<i32>", value))
                }
            }
            FieldType::VecFloat => {
                if let Some(arr) = value.as_array() {
                    let floats: Result<Vec<f64>, String> = arr
                        .iter()
                        .map(|v| {
                            v.as_f64()
                                .ok_or_else(|| format!("Array element is not a float: {:?}", v))
                        })
                        .collect();
                    let numbers: Result<Vec<serde_json::Number>, String> = floats?
                        .into_iter()
                        .map(|f| {
                            serde_json::Number::from_f64(f)
                                .ok_or_else(|| "Invalid float value".to_string())
                        })
                        .collect();
                    Ok(JsonValue::Array(
                        numbers?.into_iter().map(JsonValue::Number).collect(),
                    ))
                } else {
                    Err(format!("Cannot convert {:?} to Vec<f32>", value))
                }
            }
            FieldType::Json => Ok(value.clone()),
        }
    }

    fn check_conditions(&self, json: &JsonValue) -> Result<bool, String> {
        for condition in &self.definition.conditions {
            if let Some(field_value) = json.get(&condition.field) {
                let matches = match &condition.operator {
                    ConditionOperator::Equals => field_value == &condition.value,
                    ConditionOperator::NotEquals => field_value != &condition.value,
                    ConditionOperator::GreaterThan => {
                        if let (Some(a), Some(b)) = (field_value.as_f64(), condition.value.as_f64())
                        {
                            a > b
                        } else {
                            false
                        }
                    }
                    ConditionOperator::LessThan => {
                        if let (Some(a), Some(b)) = (field_value.as_f64(), condition.value.as_f64())
                        {
                            a < b
                        } else {
                            false
                        }
                    }
                    ConditionOperator::Contains => {
                        if let (Some(s), Some(needle)) =
                            (field_value.as_str(), condition.value.as_str())
                        {
                            s.contains(needle)
                        } else {
                            false
                        }
                    }
                };

                if !matches {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn apply_transform(&self, value: &JsonValue, _transform: &str) -> Result<JsonValue, String> {
        // TODO: Integrate with JS engine here
        // For now, just return the value as-is
        Ok(value.clone())
    }
}

/// Manager for runtime-defined syncs
pub struct RuntimeSyncManager {
    syncs: Vec<RuntimeSync>,
    module_name_to_id: HashMap<String, TypeId>,
    schema_registry: SchemaRegistry,
}

impl RuntimeSyncManager {
    pub fn new() -> Self {
        Self {
            syncs: Vec::new(),
            module_name_to_id: HashMap::new(),
            schema_registry: SchemaRegistry::new(),
        }
    }

    /// Register a module type name to TypeId mapping
    pub fn register_module_type<T: crate::HaCK + 'static>(&mut self, name: impl Into<String>) {
        self.module_name_to_id
            .insert(name.into(), TypeId::of::<T>());
    }

    /// Register a module schema
    pub fn register_schema(&mut self, schema: ModuleSchema) {
        self.schema_registry.register_schema(schema);
    }

    /// Get available modules
    pub fn list_modules(&self) -> Vec<String> {
        self.schema_registry.list_modules()
    }

    /// Get schema for a module
    pub fn get_schema(&self, module_name: &str) -> Option<&ModuleSchema> {
        self.schema_registry.get_schema(module_name)
    }

    /// Get compatible field mappings between two modules
    pub fn get_compatible_fields(
        &self,
        source: &str,
        target: &str,
    ) -> Vec<(String, String, FieldType)> {
        self.schema_registry.get_compatible_fields(source, target)
    }

    /// Add a runtime sync from JSON definition
    pub fn add_sync(&mut self, definition: ScriptedSyncDefinition) {
        self.syncs.push(RuntimeSync::new(definition));
    }

    /// Remove a sync by ID
    pub fn remove_sync(&mut self, id: &str) -> bool {
        if let Some(pos) = self.syncs.iter().position(|s| s.definition.id == id) {
            self.syncs.remove(pos);
            true
        } else {
            false
        }
    }

    /// List all registered syncs
    pub fn list_syncs(&self) -> Vec<ScriptedSyncDefinition> {
        self.syncs.iter().map(|s| s.definition.clone()).collect()
    }

    /// Load syncs from JSON
    pub fn load_syncs_from_json(&mut self, json: &str) -> Result<(), String> {
        let definitions: Vec<ScriptedSyncDefinition> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse sync definitions: {}", e))?;

        for def in definitions {
            self.add_sync(def);
        }

        Ok(())
    }

    pub fn apply_all(&mut self, hacs: &crate::HaCKS) {
        for sync in &mut self.syncs {
            if let Err(e) = Self::apply_sync_static(sync, hacs, &self.module_name_to_id) {
                log::error!("Sync error ({}): {}", sync.definition.id, e);
            }
        }
    }
    #[allow(unused)]
    fn apply_sync(&self, sync: &mut RuntimeSync, hacs: &crate::HaCKS) -> Result<(), String> {
        Self::apply_sync_static(sync, hacs, &self.module_name_to_id)
    }

    fn apply_sync_static(
        sync: &mut RuntimeSync,
        hacs: &crate::HaCKS,
        module_name_to_id: &HashMap<String, TypeId>,
    ) -> Result<(), String> {
        // Get source and target TypeIds
        let source_id = module_name_to_id
            .get(&sync.definition.source_module)
            .ok_or_else(|| format!("Unknown source module: {}", sync.definition.source_module))?;

        let target_id = module_name_to_id
            .get(&sync.definition.target_module)
            .ok_or_else(|| format!("Unknown target module: {}", sync.definition.target_module))?;

        // Get JSON representations
        let source_json = {
            let module_rc = hacs.get_module_by_type_id(*source_id).ok_or_else(|| {
                format!("Source module not found: {}", sync.definition.source_module)
            })?;

            let module = module_rc.borrow();
            module
                .to_json()
                .map_err(|e| format!("Failed to serialize source: {}", e))?
        };

        let mut target_json = {
            let module_rc = hacs.get_module_by_type_id(*target_id).ok_or_else(|| {
                format!("Target module not found: {}", sync.definition.target_module)
            })?;

            let module = module_rc.borrow();
            module
                .to_json()
                .map_err(|e| format!("Failed to serialize target: {}", e))?
        };

        // Detect changes
        let (source_changed, target_changed) = sync.detect_changes(&source_json, &target_json);

        // Apply sync based on type and changes
        let should_sync = match (&sync.definition.sync_type, source_changed, target_changed) {
            (SyncType::OneWay, true, _) => true,
            (SyncType::Bidirectional, true, false) => true,
            (SyncType::Bidirectional, false, true) => {
                // Reverse sync (target -> source)
                // For now, skip reverse sync - would need reverse mappings
                false
            }
            _ => false,
        };

        if should_sync {
            if sync.apply_sync(&source_json, &mut target_json)? {
                // Update the target module with new JSON
                Self::update_module_from_json_static(hacs, target_id, &target_json)?;

                // Update tracking
                sync.last_source_state = Some(source_json);
                sync.last_target_state = Some(target_json);
            }
        }

        Ok(())
    }

    #[allow(unused)]
    fn update_module_from_json(
        &self,
        hacs: &crate::HaCKS,
        type_id: &TypeId,
        json: &JsonValue,
    ) -> Result<(), String> {
        Self::update_module_from_json_static(hacs, type_id, json)
    }

    fn update_module_from_json_static(
        _hacs: &crate::HaCKS,
        type_id: &TypeId,
        json: &JsonValue,
    ) -> Result<(), String> {
        // Get the module and deserialize into it
        // This requires modules to support from_json or similar
        // For now, we'll just log that we would update it
        log::info!("Would update module {:?} with: {}", type_id, json);

        // TODO: Implement actual deserialization
        // This is tricky because we need to deserialize into the concrete type
        // One approach: Add a `update_from_json` method to HaCK trait

        Ok(())
    }
}

// ============================================================
// Module Schema System - Introspect field types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSchema {
    pub module_name: String,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: FieldType,
    pub description: Option<String>,
    pub readable: bool,
    pub writable: bool,
}

/// Trait for modules to expose their schema
pub trait ModuleSchemaProvider {
    fn get_schema() -> ModuleSchema;
}

// ============================================================
// Macro to auto-generate schema from struct
// ============================================================

#[macro_export]
macro_rules! impl_schema {
    ($type:ty, $name:expr, [
        $(($field:ident: $field_type:expr, $readable:expr, $writable:expr $(, $desc:expr)?)),* $(,)?
    ]) => {
        impl $crate::runtime_sync::ModuleSchemaProvider for $type {
            fn get_schema() -> $crate::runtime_sync::ModuleSchema {
                $crate::runtime_sync::ModuleSchema {
                    module_name: $name.to_string(),
                    fields: vec![
                        $(
                            $crate::runtime_sync::FieldSchema {
                                name: stringify!($field).to_string(),
                                field_type: $field_type,
                                description: impl_schema!(@desc $($desc)?),
                                readable: $readable,
                                writable: $writable,
                            }
                        ),*
                    ],
                }
            }
        }
    };

    (@desc $desc:expr) => { Some($desc.to_string()) };
    (@desc) => { None };
}

// ============================================================
// Schema Registry
// ============================================================

pub struct SchemaRegistry {
    schemas: HashMap<String, ModuleSchema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register_schema(&mut self, schema: ModuleSchema) {
        self.schemas.insert(schema.module_name.clone(), schema);
    }

    pub fn get_schema(&self, module_name: &str) -> Option<&ModuleSchema> {
        self.schemas.get(module_name)
    }

    pub fn list_modules(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// Get all syncable field pairs between two modules
    pub fn get_compatible_fields(
        &self,
        source: &str,
        target: &str,
    ) -> Vec<(String, String, FieldType)> {
        let source_schema = match self.get_schema(source) {
            Some(s) => s,
            None => return vec![],
        };

        let target_schema = match self.get_schema(target) {
            Some(s) => s,
            None => return vec![],
        };

        let mut compatible = Vec::new();

        for source_field in &source_schema.fields {
            if !source_field.readable {
                continue;
            }

            for target_field in &target_schema.fields {
                if !target_field.writable {
                    continue;
                }

                // Allow syncing if types match or are compatible
                if self.types_compatible(&source_field.field_type, &target_field.field_type) {
                    compatible.push((
                        source_field.name.clone(),
                        target_field.name.clone(),
                        target_field.field_type.clone(),
                    ));
                }
            }
        }

        compatible
    }

    fn types_compatible(&self, source: &FieldType, target: &FieldType) -> bool {
        match (source, target) {
            // Exact match
            (a, b) if a == b => true,
            // Numeric conversions
            (FieldType::Int, FieldType::Float) => true,
            (FieldType::Float, FieldType::Int) => true,
            // Bool to numeric
            (FieldType::Bool, FieldType::Int) => true,
            (FieldType::Bool, FieldType::Float) => true,
            // String conversions
            (_, FieldType::String) => true,
            // JSON catch-all
            (_, FieldType::Json) => true,
            (FieldType::Json, _) => true,
            _ => false,
        }
    }
}

impl Default for RuntimeSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}
