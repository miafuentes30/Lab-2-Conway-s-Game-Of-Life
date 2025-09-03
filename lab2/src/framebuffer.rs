use minifb::{MouseMode, Window};

pub type Color = u32;

pub const BLACK: Color = 0x000000;
pub const WHITE: Color = 0xFFFFFF;

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<Color>,
    current_color: Color,
    pub background_color: Color,
}

impl Framebuffer {
    pub fn new(w: usize, h: usize, background_color: Color) -> Self {
        Self {
            w,
            h,
            pixels: vec![background_color; w * h],
            current_color: WHITE,
            background_color,
        }
    }

    #[inline]
    pub fn clear(&mut self, color: Color) {
        self.background_color = color;
        self.pixels.fill(color);
    }

    #[inline]
    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    #[inline]
    pub fn point(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return;
        }
        self.pixels[y * self.w + x] = self.current_color;
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return;
        }
        self.pixels[y * self.w + x] = color;
    }

    #[inline]
    pub fn get_color(&self, x: i32, y: i32) -> Color {
        if x < 0 || y < 0 {
            return self.background_color;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return self.background_color;
        }
        self.pixels[y * self.w + x]
    }

    pub fn blit_scaled(&self, out: &mut [u32], win_w: usize, win_h: usize) {
        let sx = (win_w / self.w).max(1);
        let sy = (win_h / self.h).max(1);
        let draw_w = self.w * sx;
        let draw_h = self.h * sy;


        out.fill(0x000000);

        let x_off = (win_w - draw_w) / 2;
        let y_off = (win_h - draw_h) / 2;

        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.pixels[y * self.w + x];
                let px0 = x_off + x * sx;
                let py0 = y_off + y * sy;
                for yy in 0..sy {
                    let row = (py0 + yy) * win_w;
                    for xx in 0..sx {
                        out[row + px0 + xx] = c;
                    }
                }
            }
        }
    }
}

pub fn mouse_cell(window: &Window, fb_w: usize, fb_h: usize, win_w: usize, win_h: usize) -> Option<(i32, i32)> {
    if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
        let mx = mx as usize;
        let my = my as usize;

        let sx = (win_w / fb_w).max(1);
        let sy = (win_h / fb_h).max(1);
        let draw_w = fb_w * sx;
        let draw_h = fb_h * sy;
        let x_off = (win_w - draw_w) / 2;
        let y_off = (win_h - draw_h) / 2;

        if mx < x_off || my < y_off || mx >= x_off + draw_w || my >= y_off + draw_h {
            return None;
        }
        let cx = (mx - x_off) / sx;
        let cy = (my - y_off) / sy;
        return Some((cx as i32, cy as i32));
    }
    None
}

pub use minifb::Key as KeyCode;
