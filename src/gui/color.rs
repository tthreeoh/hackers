use crate::HaCKS;

impl HaCKS {
    pub fn get_bar_color(&self, order: usize, active_count: usize, progress: f32) -> [f32; 4] {
        match self.color_scheme {
            0 => {
                // Warm-Cool gradient (orange → blue)
                let r = 0.95 - progress * 0.8;
                let g = 0.55 - progress * 0.15;
                let b = 0.2 + progress * 0.7;
                let a = 0.7 + progress * 0.3;
                [r, g, b, a]
            }
            1 => {
                // Blue-Cyan gradient
                let r = 0.2 + progress * 0.5;
                let g = 0.4 + progress * 0.4;
                let b = 0.9 - progress * 0.3;
                let a = 0.7 + progress * 0.3;
                [r, g, b, a]
            }
            2 => {
                // Muted gradient
                let r = 0.15 + progress * 0.6;
                let g = 0.35 + progress * 0.4;
                let b = 0.65 - progress * 0.2;
                let a = 0.75 + progress * 0.25;
                [r, g, b, a]
            }
            3 => {
                // Sunset (orange → pink → purple)
                let r = 0.95 - progress * 0.4;
                let g = 0.6 - progress * 0.6;
                let b = 0.2 + progress * 0.6;
                let a = 0.75 + progress * 0.25;
                [r, g, b, a]
            }
            4 => {
                // Forest (dark green → light green)
                let r = 0.1 + progress * 0.3;
                let g = 0.3 + progress * 0.5;
                let b = 0.1 + progress * 0.2;
                let a = 0.75 + progress * 0.25;
                [r, g, b, a]
            }
            5 => {
                // Neon (vibrant, high saturation)
                let r = 0.2 + progress * 0.8;
                let g = 0.1 + progress * 0.9;
                let b = 0.9 - progress * 0.5;
                let a = 0.8 + progress * 0.2;
                [r, g, b, a]
            }
            6 => {
                // Pastel (soft, desaturated)
                let r = 0.7 + progress * 0.2;
                let g = 0.6 + progress * 0.3;
                let b = 0.8 - progress * 0.2;
                let a = 0.7 + progress * 0.3;
                [r, g, b, a]
            }
            7 => {
                // Grayscale
                let gray = 0.3 + progress * 0.6;
                let a = 0.7 + progress * 0.3;
                [gray, gray, gray, a]
            }
            8 => {
                // Monochrome Blue (all blues, varying brightness)
                let r = 0.1 + progress * 0.15;
                let g = 0.3 + progress * 0.4;
                let b = 0.8 + progress * 0.2;
                let a = 0.75 + progress * 0.25;
                [r, g, b, a]
                
            }
            _=> self.hue_to_rgb((order as f32 / active_count.max(1) as f32) * 360.0),
        }
    }

    fn hue_to_rgb(&self, hue: f32) -> [f32; 4] {
        let h = hue / 60.0;
        let c = 0.7; // Saturation
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        
        [r + 0.3, g + 0.3, b + 0.3, 1.0]
    }

}