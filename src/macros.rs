//automatic settings config per HaC
#[macro_export]
macro_rules! impl_hac_settings {
    ($module_type:ty, $key:literal) => {
        impl $crate::hack::ModuleSettings for $module_type {
            fn settings_key() -> &'static str {
                $key
            }
        }
    };
}

#[macro_export]
macro_rules! impl_hac_boilerplate {
    ($module_type:ty, $field:ident) => {
        fn is_menu_enabled(&self) -> bool {
            self.$field.is_menu_enabled
        }
        fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
            serde_json::to_value(self)
        }

        fn is_window_enabled(&self) -> bool {
            self.$field.is_window_enabled
        }

        fn set_show_menu(&mut self, truth: bool) -> bool {
            self.$field.is_menu_enabled = truth;
            self.$field.is_menu_enabled
        }

        fn set_show_window(&mut self, truth: bool) -> bool {
            self.$field.is_window_enabled = truth;
            self.$field.is_window_enabled
        }

        fn metadata(&self) -> &$crate::HaCMetadata {
            &self.$field
        }

        fn metadata_mut(&mut self) -> &mut $crate::HaCMetadata {
            &mut self.$field
        }

        fn is_render_enabled(&self) -> bool {
            self.$field.is_render_enabled
        }

        fn is_update_enabled(&self) -> bool {
            self.$field.is_update_enabled
        }

        fn menu_weight(&self) -> f32 {
            self.$field.menu_weight
        }

        fn window_weight(&self) -> f32 {
            self.$field.window_weight
        }

        fn draw_weight(&self) -> f32 {
            self.$field.draw_weight
        }

        fn update_weight(&self) -> f32 {
            self.$field.update_weight
        }

        fn set_menu_weight(&mut self, weight: f32) {
            self.$field.menu_weight = weight;
        }

        fn set_window_weight(&mut self, weight: f32) {
            self.$field.window_weight = weight;
        }

        fn set_draw_weight(&mut self, weight: f32) {
            self.$field.draw_weight = weight;
        }

        fn set_update_weight(&mut self, weight: f32) {
            self.$field.update_weight = weight;
        }

        fn set_render_enabled(&mut self, enabled: bool) {
            self.$field.is_render_enabled = enabled;
        }

        fn set_update_enabled(&mut self, enabled: bool) {
            self.$field.is_update_enabled = enabled;
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        
    };
}


#[macro_export]
macro_rules! declare_and_register_hacs {
    (
        $(
            $(#[$attr:meta])*
            $mod_name:ident => $module_path:path, $key:literal
        ),* $(,)?
    ) => {
        // Import necessary types
        use std::rc::Rc;
        use std::cell::RefCell;
        use std::collections::HashMap;
        use $crate::serde_json;

        // Declare submodules (attributes only go here)
        $(
            $(#[$attr])*
            pub mod $mod_name;
        )*

        // Implement HaC settings (no attributes here)
        $(
            $crate::impl_hac_settings!($module_path, $key);
        )*

        // Create modules (Rc<RefCell<_>>)
        pub fn create_modules() -> Vec<Rc<RefCell<dyn $crate::HaCK>>> {
            vec![
                $(
                    // No attributes on expressions
                    Rc::new(RefCell::new(<$module_path>::default())) as Rc<RefCell<dyn $crate::HaCK>>
                ),*
            ]
        }

        // Save settings
        pub fn save_all_settings(
            modules: &HashMap<std::any::TypeId, Rc<RefCell<dyn $crate::HaCK>>>
        ) -> HashMap<String, serde_json::Value> {
            let mut settings = HashMap::new();

            $(
                {
                    if let Some(hac_rc) = modules.get(&std::any::TypeId::of::<$module_path>()) {
                        if let Ok(hac_ref) = hac_rc.borrow() {
                            if let Some(m) = hac_ref.as_any().downcast_ref::<$module_path>() {
                                if let Ok(value) = serde_json::to_value(m) {
                                    settings.insert($key.to_string(), value);
                                }
                            }
                        }
                    }
                }
            )*

            settings
        }

        // Load settings
        pub fn load_all_settings(
            settings: &HashMap<String, serde_json::Value>
        ) -> Vec<Rc<RefCell<dyn $crate::HaCK>>> {
            use $crate::HaCK as _;
            vec![
                $(
                    // No attributes on expressions
                    Rc::new(RefCell::new({
                        let mut module = settings.get($key)
                            .and_then(|v| serde_json::from_value::<$module_path>(v.clone()).ok())
                            .unwrap_or_default();
                        module.post_load_init();
                        module
                    })) as Rc<RefCell<dyn $crate::HaCK>>
                ),*
            ]
        }
    };
}


#[macro_export]
macro_rules! impl_nac_for_hacs {
    ($container_type:ty) => {
        impl $crate::HaC for $container_type {
            fn name(&self) -> &str {
                &self.nac_data.name
            }

            fn update(&mut self, ctx: &mut $crate::UpdateContext<'_>) {
                for HaC in self.hacs_mut() {
                    HaC.update(ctx);
                }
            }

            fn render_menu(&mut self, rctx: &mut $crate::RenderUpdateContext) {
                for HaC in self.hacs_mut() {
                    HaC.render_menu(rctx);
                }
            }

            fn render_window(&mut self, rctx: &mut $crate::RenderUpdateContext) {
                for HaC in self.hacs_mut() {
                    HaC.render_window(rctx);
                }
            }

            fn on_unload(&mut self) {
                for HaC in self.hacs_mut() {
                    HaC.on_unload();
                }
            }

            fn exit(&mut self, ctx: &mut $crate::InitContext) {
                for HaC in self.hacs_mut() {
                    HaC.exit(ctx);
                }
            }

            fn init(&mut self, ctx: &mut $crate::InitContext) {
                for HaC in self.hacs_mut() {
                    HaC.init(ctx);
                }
            }

            fn post_load_init(&mut self) {
                for HaC in self.hacs_mut() {
                    HaC.post_load_init();
                }
            }

            fn before_render(&mut self) {
                for HaC in self.hacs_mut() {
                    HaC.before_render();
                }
            }

            fn nac_type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<$container_type>()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn metadata(&self) -> &$crate::HaCMetadata {
                &self.nac_data
            }

            fn metadata_mut(&mut self) -> &mut $crate::HaCMetadata {
                &mut self.nac_data
            }

            fn is_render_enabled(&self) -> bool {
                self.nac_data.is_render_enabled
            }

            fn is_update_enabled(&self) -> bool {
                self.nac_data.is_update_enabled
            }

            fn is_menu_enabled(&self) -> bool {
                self.nac_data.is_menu_enabled
            }

            fn is_window_enabled(&self) -> bool {
                self.nac_data.is_window_enabled
            }

            fn window_weight(&self) -> f32 {
                self.nac_data.window_weight
            }

            fn update_weight(&self) -> f32 {
                self.nac_data.update_weight
            }

            fn draw_weight(&self) -> f32 {
                self.nac_data.draw_weight
            }

            fn menu_weight(&self) -> f32 {
                self.nac_data.menu_weight
            }

            fn set_update_weight(&mut self, weight: f32) {
                self.nac_data.update_weight = weight;
            }

            fn set_window_weight(&mut self, weight: f32) {
                self.nac_data.window_weight = weight;
            }

            fn set_show_menu(&mut self, truth: bool) -> bool {
                self.nac_data.is_menu_enabled = truth;
                self.nac_data.is_menu_enabled
            }

            fn set_show_window(&mut self, truth: bool) -> bool {
                self.nac_data.is_window_enabled = truth;
                self.nac_data.is_window_enabled
            }

            fn set_menu_weight(&mut self, weight: f32) {
                self.nac_data.menu_weight = weight;
            }

            fn set_draw_weight(&mut self, weight: f32) {
                self.nac_data.draw_weight = weight;
            }

            fn set_render_enabled(&mut self, enabled: bool) {
                self.nac_data.is_render_enabled = enabled;
            }

            fn set_update_enabled(&mut self, enabled: bool) {
                self.nac_data.is_update_enabled = enabled;
            }
        }
    };
}