use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages texture uploads for plugins
/// Bridges the gap between upload requests (during plugin update/render)
/// and actual GPU uploads (during wgpu render pass)
#[derive(Clone)]
pub struct TextureManager {
    /// Pending uploads: (id, width, height, rgba_data)
    pending: Arc<Mutex<Vec<(u64, u32, u32, Vec<u8>)>>>,
    /// Map from our ID to imgui TextureId
    textures: Arc<Mutex<HashMap<u64, imgui::TextureId>>>,
    /// Next ID to assign
    next_id: Arc<Mutex<u64>>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::new())),
            textures: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Request texture upload (called from plugin via ImguiBackend)
    /// Returns ID that will map to real TextureId after flush
    pub fn request_upload(&self, data: &[u8], width: u32, height: u32) -> u64 {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let mut pending = self.pending.lock().unwrap();
        pending.push((id, width, height, data.to_vec()));

        id
    }

    /// Flush pending uploads to GPU (called during render)
    pub fn flush_uploads(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        let mut pending = self.pending.lock().unwrap();
        if pending.is_empty() {
            return;
        }

        let mut textures = self.textures.lock().unwrap();

        for (id, width, height, data) in pending.drain(..) {
            let texture_config = imgui_wgpu::TextureConfig {
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                ..Default::default()
            };

            let texture = imgui_wgpu::Texture::new(device, renderer, texture_config);
            texture.write(queue, &data, width, height);

            let texture_id = renderer.textures.insert(texture);
            textures.insert(id, texture_id);
        }
    }

    /// Get actual imgui TextureId (called from DrawList::add_image)
    pub fn get_texture_id(&self, id: u64) -> Option<imgui::TextureId> {
        self.textures.lock().unwrap().get(&id).copied()
    }

    /// Free a texture
    pub fn free_texture(&self, id: u64, renderer: &mut imgui_wgpu::Renderer) {
        if let Some(texture_id) = self.textures.lock().unwrap().remove(&id) {
            renderer.textures.remove(texture_id);
        }
    }
}
