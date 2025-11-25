use std::any::TypeId;

#[derive(Debug, Clone)]
pub enum HaCSEvent {
    /// Ask a module to open its window
    OpenWindow { module_id: TypeId },

    /// Ask a module to close its window
    CloseWindow { module_id: TypeId },

    /// Mark a group of modules as undocked (windowed)
    UndockGroup { path: Vec<String> },

    /// Mark a group as docked again (back into the menu)
    RedockGroup { path: Vec<String> },

    /// Rebuild the menu cache
    RebuildMenu,
}

impl crate::HaCKS {
    /// Queue an event to be handled later
    pub fn emit(&mut self, event: HaCSEvent) {
        self.event_bus.push(event);
    }

    /// Handle and clear all queued events
    pub fn process_events(&mut self) {
        let mut events = Vec::new();
        std::mem::swap(&mut events, &mut self.event_bus);

        for event in events {
            match event {
                HaCSEvent::OpenWindow { module_id } => {
                    if let Some(m) = self.hacs.get(&module_id) {
                        let mut m = m.borrow_mut();
                        m.set_show_window(true);
                        m.set_show_menu(false);
                    }
                }
                HaCSEvent::CloseWindow { module_id } => {
                    if let Some(m) = self.hacs.get(&module_id) {
                        m.borrow_mut().set_show_window(false);
                    }
                }
                HaCSEvent::UndockGroup { path } => {
                    self.windowed_groups.insert(path, true);
                }
                HaCSEvent::RedockGroup { path } => {
                    self.windowed_groups.remove(&path);
                }
                HaCSEvent::RebuildMenu => {
                    self.menu_dirty = true;
                }
            }
        }
    }
}


// /// Events that modules can send to each other
// pub enum HaCEvent {
//     /// Query another module's state (non-mut)
//     Query {
//         target: TypeId,
//         query: Box<dyn Any + Send>,
//     },
//     /// Request another module to change state
//     Command {
//         target: TypeId,
//         command: Box<dyn Any + Send>,
//     },
//     /// Broadcast to all modules
//     Broadcast {
//         event: Box<dyn Any + Send>,
//     },
//     /// Response from a query
//     Response {
//         from: TypeId,
//         data: Box<dyn Any + Send>,
//     },
// }

// /// Event bus for inter-module communication
// pub struct EventBus {
//     queue: VecDeque<HaCEvent>,
//     responses: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
// }

// impl EventBus {
//     pub fn new() -> Self {
//         Self {
//             queue: VecDeque::new(),
//             responses: HashMap::new(),
//         }
//     }

//     /// Send an event (called from within a module)
//     pub fn send(&mut self, event: HaCEvent) {
//         self.queue.push_back(event);
//     }

//     /// Query a module and get immediate response if available
//     pub fn query<T: 'static>(&mut self, target: TypeId, query: impl Any + Send + 'static) {
//         self.send(HaCEvent::Query {
//             target,
//             query: Box::new(query),
//         });
//     }

//     /// Send a command to another module
//     pub fn command(&mut self, target: TypeId, command: impl Any + Send + 'static) {
//         self.send(HaCEvent::Command {
//             target,
//             command: Box::new(command),
//         });
//     }

//     /// Broadcast to all modules
//     pub fn broadcast(&mut self, event: impl Any + Send + 'static) {
//         self.send(HaCEvent::Broadcast {
//             event: Box::new(event),
//         });
//     }

//     /// Add a response (called by modules handling queries)
//     pub fn respond(&mut self, from: TypeId, data: impl Any + Send + 'static) {
//         self.responses
//             .entry(from)
//             .or_default()
//             .push(Box::new(data));
//     }

//     /// Get responses from a specific module
//     pub fn get_responses(&mut self, from: TypeId) -> Vec<Box<dyn Any + Send>> {
//         self.responses.remove(&from).unwrap_or_default()
//     }

//     /// Check if there are pending events
//     pub fn has_events(&self) -> bool {
//         !self.queue.is_empty()
//     }

//     /// Get next event
//     fn pop_event(&mut self) -> Option<HaCEvent> {
//         self.queue.pop_front()
//     }

//     /// Clear all pending events (useful for cleanup)
//     pub fn clear(&mut self) {
//         self.queue.clear();
//         self.responses.clear();
//     }
// }

// use crate::{HaC, HaCS};

// impl HaCS {
//     /// Process all pending events
//     pub fn process_events(&mut self) {
//         while self.events.has_events() {
//             if let Some(event) = self.events.pop_event() {
//                 match event {
//                     HaCEvent::Query { target, query } => {
//                         if let Some(module) = self.hacs.get_mut(&target) {
//                             module.on_query(&query, &mut self.events);
//                         }
//                     }
//                     HaCEvent::Command { target, command } => {
//                         if let Some(module) = self.hacs.get_mut(&target) {
//                             module.on_command(&command);
//                         }
//                     }
//                     HaCEvent::Broadcast { event } => {
//                         for module in self.hacs.values_mut() {
//                             module.on_broadcast(&event);
//                         }
//                     }
//                     HaCEvent::Response { .. } => {
//                         // Responses are handled by get_responses()
//                     }
//                 }
//             }
//         }
//     }

//     /// Send an event during render/update
//     pub fn send_event(&mut self, event: HaCEvent) {
//         self.events.send(event);
//     }

//     /// Query another module's state (from within a module)
//     pub fn query_module<Q: 'static + Send>(&mut self, target: TypeId, query: Q) {
//         self.events.query(target, query);
//     }

//     /// Get event bus reference (for modules to use)
//     pub fn events(&mut self) -> &mut EventBus {
//         &mut self.events
//     }
// }

// // Add to HaC trait (src_hackrs_traits.rs)
// pub trait HaCEventHandler {
//     /// Handle a query from another module
//     fn on_query(&mut self, query: &dyn Any, bus: &mut EventBus) {
//         // Default: no-op
//     }

//     /// Handle a command from another module
//     fn on_command(&mut self, command: &dyn Any) {
//         // Default: no-op
//     }

//     /// Handle a broadcast event
//     fn on_broadcast(&mut self, event: &dyn Any) {
//         // Default: no-op
//     }
// }
