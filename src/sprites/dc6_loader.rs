use image::{DynamicImage, RgbaImage};

use crate::sprites::GameFileLoader;

// use crate::images::GameFileLoader;

struct DC6Header {
    version: u32,
    flags: u32,
    encoding: u32,
    termination: [u8; 4],
    directions: u32,
    frames_per_direction: u32,
}

struct DC6FrameHeader {
    flip: u32,
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
    unknown: u32,
    next_block: u32,
    length: u32,
}

struct DC6File {
    header: DC6Header,
    frame_headers: Vec<DC6FrameHeader>,
    frame_data: Vec<Vec<u8>>,
}

impl DC6File {
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 24 {
            return Err("DC6 file too small".to_string());
        }
        
        let mut offset = 0;
        
        // Read header (24 bytes)
        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let encoding = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let termination = [data[12], data[13], data[14], data[15]];
        let directions = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let frames_per_direction = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        
        offset += 24;
        
        let header = DC6Header {
            version,
            flags,
            encoding,
            termination,
            directions,
            frames_per_direction,
        };
        
        let total_frames = (directions * frames_per_direction) as usize;
        
        // Read frame pointers
        let mut frame_pointers = Vec::new();
        for _ in 0..total_frames {
            if offset + 4 > data.len() {
                return Err("Invalid frame pointer".to_string());
            }
            let ptr = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
            ]) as usize;
            frame_pointers.push(ptr);
            offset += 4;
        }
        
        // Read frame headers and data
        let mut frame_headers = Vec::new();
        let mut frame_data = Vec::new();
        
        for frame_ptr in frame_pointers {
            if frame_ptr + 32 > data.len() {
                return Err("Invalid frame header position".to_string());
            }
            
            let mut fh_offset = frame_ptr;
            
            let flip = u32::from_le_bytes([
                data[fh_offset], data[fh_offset + 1], data[fh_offset + 2], data[fh_offset + 3]
            ]);
            let width = u32::from_le_bytes([
                data[fh_offset + 4], data[fh_offset + 5], data[fh_offset + 6], data[fh_offset + 7]
            ]);
            let height = u32::from_le_bytes([
                data[fh_offset + 8], data[fh_offset + 9], data[fh_offset + 10], data[fh_offset + 11]
            ]);
            let offset_x = i32::from_le_bytes([
                data[fh_offset + 12], data[fh_offset + 13], data[fh_offset + 14], data[fh_offset + 15]
            ]);
            let offset_y = i32::from_le_bytes([
                data[fh_offset + 16], data[fh_offset + 17], data[fh_offset + 18], data[fh_offset + 19]
            ]);
            let unknown = u32::from_le_bytes([
                data[fh_offset + 20], data[fh_offset + 21], data[fh_offset + 22], data[fh_offset + 23]
            ]);
            let next_block = u32::from_le_bytes([
                data[fh_offset + 24], data[fh_offset + 25], data[fh_offset + 26], data[fh_offset + 27]
            ]);
            let length = u32::from_le_bytes([
                data[fh_offset + 28], data[fh_offset + 29], data[fh_offset + 30], data[fh_offset + 31]
            ]);
            
            fh_offset += 32;
            
            frame_headers.push(DC6FrameHeader {
                flip,
                width,
                height,
                offset_x,
                offset_y,
                unknown,
                next_block,
                length,
            });
            
            if fh_offset + length as usize > data.len() {
                return Err("Invalid frame data length".to_string());
            }
            
            let pixel_data = data[fh_offset..fh_offset + length as usize].to_vec();
            frame_data.push(pixel_data);
        }
        
        Ok(DC6File {
            header,
            frame_headers,
            frame_data,
        })
    }
    
    fn get_frame(&self, frame_index: usize) -> Option<DynamicImage> {
        if frame_index >= self.frame_headers.len() {
            return None;
        }
        
        let header = &self.frame_headers[frame_index];
        let data = &self.frame_data[frame_index];
        
        let pixels = Self::decode_frame(data, header.width, header.height)?;
        let rgba_pixels = Self::apply_palette(&pixels);
        
        let img = RgbaImage::from_raw(header.width, header.height, rgba_pixels)?;
        Some(DynamicImage::ImageRgba8(img))
    }
    
    fn decode_frame(data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
        let mut pixels = vec![0u8; (width * height) as usize];
        let mut pixel_index = 0;
        let mut data_index = 0;
        
        while data_index < data.len() && pixel_index < pixels.len() {
            let val = data[data_index];
            data_index += 1;
            
            if val == 0x80 {
                if data_index >= data.len() {
                    break;
                }
                let count = data[data_index] as usize;
                data_index += 1;
                pixel_index += count;
            } else if (val & 0x80) != 0 {
                let count = (val & 0x7F) as usize;
                if data_index >= data.len() {
                    break;
                }
                let color = data[data_index];
                data_index += 1;
                
                for _ in 0..count {
                    if pixel_index < pixels.len() {
                        pixels[pixel_index] = color;
                        pixel_index += 1;
                    }
                }
            } else {
                let count = val as usize;
                for _ in 0..count {
                    if data_index >= data.len() || pixel_index >= pixels.len() {
                        break;
                    }
                    pixels[pixel_index] = data[data_index];
                    pixel_index += 1;
                    data_index += 1;
                }
            }
        }
        
        Some(pixels)
    }
    
    fn apply_palette(indexed: &[u8]) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(indexed.len() * 4);
        
        for &index in indexed {
            if index == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                rgba.extend_from_slice(&[index, index, index, 255]);
            }
        }
        
        rgba
    }
}

pub struct DC6Loader;

impl GameFileLoader for DC6Loader {
    fn load_frame(&self, data: &[u8], frame_index: usize) -> Result<DynamicImage, String> {
        let dc6 = DC6File::from_bytes(data)?;
        dc6.get_frame(frame_index).ok_or_else(|| "Frame not found".to_string())
    }
    
    fn file_extension(&self) -> &str {
        "dc6"
    }
}