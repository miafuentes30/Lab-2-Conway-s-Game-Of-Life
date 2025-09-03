mod framebuffer;

use framebuffer::{mouse_cell, Color, Framebuffer, BLACK, WHITE, KeyCode};
use minifb::{ScaleMode, Window, WindowOptions};
use std::thread;
use std::time::Duration;

const WIN_W: usize = 900;
const WIN_H: usize = 900;

const FB_W: usize = 160;
const FB_H: usize = 160;


#[derive(Clone, Copy)]
enum ThemeKind { Classic, Aqua, Sunset, Neon }

#[derive(Clone, Copy, PartialEq, Eq)]
enum WrapMode { DeadBorder, Torus }

#[derive(Clone, Copy)]
struct Style {
    theme: ThemeKind,
    show_grid: bool,
    show_checker: bool,
    show_trails: bool,
}

#[inline] fn rgb(r:u8,g:u8,b:u8)->Color { ((r as u32)<<16)|((g as u32)<<8)|(b as u32) }
#[inline] fn lerp(a:f32,b:f32,t:f32)->f32 { a + (b-a)*t.clamp(0.0,1.0) }
#[inline] fn lerp_u8(a:u8,b:u8,t:f32)->u8 { lerp(a as f32,b as f32,t).round() as u8 }

fn color_for_alive(age: u8, theme: ThemeKind) -> Color {
    match theme {
        ThemeKind::Classic => WHITE,
        ThemeKind::Aqua => {
            let t = (age as f32 / 20.0).min(1.0);
            let r = lerp_u8(180,  20, t);
            let g = lerp_u8(230, 220, t);
            let b = lerp_u8(255, 200, t);
            rgb(r,g,b)
        }
        ThemeKind::Sunset => {
            let t = (age as f32 / 25.0).min(1.0);
            let r = lerp_u8(255, 255, t);
            let g = lerp_u8(140,  30, t);
            let b = lerp_u8( 60, 160, t);
            rgb(r,g,b)
        }
        ThemeKind::Neon => {
            let x = (age as usize) % 24;
            let table: [Color; 24] = [
                rgb( 80,255, 80), rgb( 80,255,160), rgb( 80,255,240),
                rgb( 80,200,255), rgb( 80,120,255), rgb(120, 80,255),
                rgb(200, 80,255), rgb(255, 80,220), rgb(255, 80,140),
                rgb(255, 80, 80), rgb(255,140, 80), rgb(255,200, 80),
                rgb(255,255, 80), rgb(220,255, 80), rgb(160,255, 80),
                rgb( 80,255, 80), rgb( 80,255,160), rgb( 80,255,240),
                rgb( 80,200,255), rgb( 80,120,255), rgb(120, 80,255),
                rgb(200, 80,255), rgb(255, 80,220), rgb(255, 80,140),
            ];
            table[x]
        }
    }
}

fn color_for_dead(trail: u8, style: &Style) -> Color {
    if !style.show_trails || trail == 0 {
        return BLACK;
    }
    let t = (trail as f32 / 16.0).min(1.0);
    match style.theme {
        ThemeKind::Classic => {
            let v = lerp_u8(0, 80, t);
            rgb(v,v,v)
        }
        ThemeKind::Aqua => {
            let r = lerp_u8(  0,  15, t);
            let g = lerp_u8(  6,  40, t);
            let b = lerp_u8( 12,  60, t);
            rgb(r,g,b)
        }
        ThemeKind::Sunset => {
            let r = lerp_u8(  0,  40, t);
            let g = lerp_u8(  0,  18, t);
            let b = lerp_u8(  0,  28, t);
            rgb(r,g,b)
        }
        ThemeKind::Neon => {
            let v = lerp_u8(0, 50, t);
            rgb(v,0,v) 
        }
    }
}

fn checker(x: usize, y: usize, style: &Style) -> Color {
    if !style.show_checker { return BLACK; }
    let dark = rgb(10,10,12);
    let lite = rgb(16,16,20);
    if ((x >> 3) + (y >> 3)) & 1 == 0 { lite } else { dark }
}

fn overlay_grid(x: usize, y: usize, base: Color, style: &Style) -> Color {
    if !style.show_grid { return base; }
    if x % 8 == 0 || y % 8 == 0 {
        let (r,g,b) = ((base>>16)&255, (base>>8)&255, base&255);
        let r = ((r as u32 + 30).min(255)) as u8;
        let g = ((g as u32 + 30).min(255)) as u8;
        let b = ((b as u32 + 30).min(255)) as u8;
        return rgb(r,g,b);
    }
    base
}


fn main() {
    let mut window = Window::new(
        "Conway — Styled (minifb)",
        WIN_W, WIN_H,
        WindowOptions { resize: false, scale_mode: ScaleMode::Stretch, ..WindowOptions::default() },
    ).expect("no se pudo crear la ventana");

    let mut win_buffer = vec![0u32; WIN_W * WIN_H];
    let mut fb = Framebuffer::new(FB_W, FB_H, BLACK);

    let mut grid  = vec![0u8; FB_W * FB_H];    
    let mut next  = grid.clone();
    let mut ages  = vec![0u8; FB_W * FB_H];     
    seed_creative_pattern(&mut grid, FB_W as i32, FB_H as i32);

    let mut style = Style {
        theme: ThemeKind::Aqua,
        show_grid: true,
        show_checker: true,
        show_trails: true,
    };

    let mut wrap = WrapMode::Torus;
    let mut paused = false;
    let mut delay_ms: u64 = 60;

    while window.is_open() {
        use minifb::KeyRepeat::No;
        if window.is_key_pressed(KeyCode::Space, No) { paused = !paused; }
        if window.is_key_pressed(KeyCode::R, No)     { randomize(&mut grid); ages.fill(0); }
        if window.is_key_pressed(KeyCode::C, No)     { grid.fill(0); ages.fill(0); }
        if window.is_key_pressed(KeyCode::S, No)     {
            wrap = match wrap { WrapMode::DeadBorder => WrapMode::Torus, WrapMode::Torus => WrapMode::DeadBorder };
        }
        if window.is_key_pressed(KeyCode::Up, No)    { if delay_ms > 5 { delay_ms -= 5; } }
        if window.is_key_pressed(KeyCode::Down, No)  { delay_ms += 5; }

        if window.is_key_pressed(KeyCode::T, No) {
            style.theme = match style.theme {
                ThemeKind::Classic => ThemeKind::Aqua,
                ThemeKind::Aqua    => ThemeKind::Sunset,
                ThemeKind::Sunset  => ThemeKind::Neon,
                ThemeKind::Neon    => ThemeKind::Classic,
            };
        }
        if window.is_key_pressed(KeyCode::H, No) { style.show_grid    = !style.show_grid; }
        if window.is_key_pressed(KeyCode::B, No) { style.show_checker = !style.show_checker; }
        if window.is_key_pressed(KeyCode::V, No) { style.show_trails  = !style.show_trails; }

        if window.is_key_pressed(KeyCode::P, No) {
            stamp_pulsar(&mut grid, FB_W as i32 / 2 - 6, FB_H as i32 / 2 - 6, FB_W as i32);
        }
        if window.is_key_pressed(KeyCode::G, No) {
            if let Some((cx, cy)) = mouse_cell(&window, FB_W, FB_H, WIN_W, WIN_H) {
                stamp_glider(&mut grid, cx, cy, FB_W as i32);
            }
        }
        if window.is_key_pressed(KeyCode::L, No) {
            if let Some((cx, cy)) = mouse_cell(&window, FB_W, FB_H, WIN_W, WIN_H) {
                stamp_lwss(&mut grid, cx, cy, FB_W as i32);
            }
        }

        if !paused {
            step_life(&grid, &mut next, FB_W as i32, FB_H as i32, wrap);
            for i in 0..grid.len() {
                if next[i] == 1 {
                    ages[i] = ages[i].saturating_add(1).min(255);
                } else {
                    ages[i] = ages[i].saturating_sub(1);
                }
            }
            std::mem::swap(&mut grid, &mut next);
        }

        for y in 0..FB_H {
            for x in 0..FB_W {
                let i = y * FB_W + x;
                let alive = grid[i] == 1;
                let color = if alive {
                    color_for_alive(ages[i], style.theme)
                } else {
                    let base = if ages[i] == 0 { checker(x,y,&style) } else { color_for_dead(ages[i], &style) };
                    base
                };
                let color = overlay_grid(x,y,color,&style);
                fb.set_pixel(x as i32, y as i32, color);
            }
        }

        fb.blit_scaled(&mut win_buffer, WIN_W, WIN_H);
        window.update_with_buffer(&win_buffer, WIN_W, WIN_H).unwrap();
        thread::sleep(Duration::from_millis(delay_ms));
    }
}

#[inline] fn idx(x: i32, y: i32, w: i32) -> usize { (y * w + x) as usize }

fn count_neighbors(grid: &[u8], x: i32, y: i32, w: i32, h: i32, wrap: WrapMode) -> u8 {
    let mut n = 0u8;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let mut nx = x + dx;
            let mut ny = y + dy;
            match wrap {
                WrapMode::DeadBorder => {
                    if nx < 0 || nx >= w || ny < 0 || ny >= h { continue; }
                }
                WrapMode::Torus => {
                    if nx < 0 { nx = w - 1; } else if nx >= w { nx = 0; }
                    if ny < 0 { ny = h - 1; } else if ny >= h { ny = 0; }
                }
            }
            if grid[idx(nx, ny, w)] == 1 { n += 1; }
        }
    }
    n
}

fn step_life(curr: &[u8], next: &mut [u8], w: i32, h: i32, wrap: WrapMode) {
    for y in 0..h {
        for x in 0..w {
            let alive = curr[idx(x, y, w)] == 1;
            let n = count_neighbors(curr, x, y, w, h, wrap);
            next[idx(x, y, w)] = match (alive, n) {
                (true, 2) | (true, 3) => 1,
                (false, 3)            => 1,
                _                     => 0,
            };
        }
    }
}

fn set(grid: &mut [u8], x: i32, y: i32, w: i32) {
    if x < 0 || y < 0 { return; }
    let (x, y) = (x as usize, y as usize);
    let w_us = w as usize;
    let h_us = grid.len() / w_us;
    if x < w_us && y < h_us { grid[y * w_us + x] = 1; }
}

fn stamp_block(grid: &mut [u8], x: i32, y: i32, w: i32) { for dy in 0..2 { for dx in 0..2 { set(grid, x+dx, y+dy, w); } } }
fn stamp_blinker(grid: &mut [u8], x: i32, y: i32, w: i32) { set(grid, x-1, y, w); set(grid, x, y, w); set(grid, x+1, y, w); }
fn stamp_toad(grid: &mut [u8], x: i32, y: i32, w: i32) {
    for dx in 0..3 { set(grid, x+dx, y, w); }
    for dx in -1..2 { set(grid, x+dx, y+1, w); }
}
fn stamp_beacon(grid: &mut [u8], x: i32, y: i32, w: i32) { stamp_block(grid, x, y, w); stamp_block(grid, x+2, y+2, w); }
fn stamp_beehive(grid: &mut [u8], x: i32, y: i32, w: i32) {
    set(grid, x+1,y,w); set(grid, x+2,y,w);
    set(grid, x,y+1,w); set(grid, x+3,y+1,w);
    set(grid, x+1,y+2,w); set(grid, x+2,y+2,w);
}
fn stamp_loaf(grid: &mut [u8], x: i32, y: i32, w: i32) {
    set(grid, x+1,y,w); set(grid, x+2,y,w);
    set(grid, x,y+1,w); set(grid, x+3,y+1,w);
    set(grid, x+2,y+2,w); set(grid, x+4,y+2,w);
    set(grid, x+1,y+3,w); set(grid, x+3,y+3,w);
    set(grid, x+2,y+4,w);
}
fn stamp_boat(grid: &mut [u8], x: i32, y: i32, w: i32) {
    set(grid, x,y,w); set(grid, x+1,y,w);
    set(grid, x,y+1,w); set(grid, x+2,y+1,w);
    set(grid, x+1,y+2,w);
}
fn stamp_tub(grid: &mut [u8], x: i32, y: i32, w: i32) { set(grid, x+1,y,w); set(grid, x,y+1,w); set(grid, x+2,y+1,w); set(grid, x+1,y+2,w); }
fn stamp_glider(grid: &mut [u8], x: i32, y: i32, w: i32) {
    set(grid, x+1, y, w);
    set(grid, x+2, y+1, w);
    set(grid, x, y+2, w); set(grid, x+1, y+2, w); set(grid, x+2, y+2, w);
}
fn stamp_lwss(grid: &mut [u8], x: i32, y: i32, w: i32) {
    for dx in 1..5 { set(grid, x+dx, y, w); }
    set(grid, x, y+1, w); set(grid, x+4, y+1, w);
    set(grid, x+4, y+2, w);
    set(grid, x, y+3, w); set(grid, x+3, y+3, w);
}
fn stamp_pulsar(grid: &mut [u8], x: i32, y: i32, w: i32) {
    let offs = [
        (x+2,y),(x+3,y),(x+4,y),(x+8,y),(x+9,y),(x+10,y),
        (x,y+2),(x+5,y+2),(x+7,y+2),(x+12,y+2),
        (x,y+3),(x+5,y+3),(x+7,y+3),(x+12,y+3),
        (x,y+4),(x+5,y+4),(x+7,y+4),(x+12,y+4),
        (x+2,y+5),(x+3,y+5),(x+4,y+5),(x+8,y+5),(x+9,y+5),(x+10,y+5),
        (x+2,y+7),(x+3,y+7),(x+4,y+7),(x+8,y+7),(x+9,y+7),(x+10,y+7),
        (x,y+8),(x+5,y+8),(x+7,y+8),(x+12,y+8),
        (x,y+9),(x+5,y+9),(x+7,y+9),(x+12,y+9),
        (x,y+10),(x+5,y+10),(x+7,y+10),(x+12,y+10),
        (x+2,y+12),(x+3,y+12),(x+4,y+12),(x+8,y+12),(x+9,y+12),(x+10,y+12),
    ];
    for (sx, sy) in offs { set(grid, sx, sy, w); }
}

fn xorshift32() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static STATE: AtomicU32 = AtomicU32::new(2463534242);
    let mut x = STATE.load(Ordering::Relaxed);
    x ^= x << 13; x ^= x >> 17; x ^= x << 5;
    STATE.store(x, Ordering::Relaxed);
    x
}
fn randomize(grid: &mut [u8]) {
    for v in grid.iter_mut() { *v = if (xorshift32() & 1) == 1 { 1 } else { 0 }; }
}
fn seed_creative_pattern(grid: &mut [u8], w: i32, h: i32) {
    grid.fill(0);
    for y in (6..h-6).step_by(20) {
        for x in (6..w-6).step_by(20) {
            match (x + y) % 5 {
                0 => stamp_block(grid, x, y, w),
                1 => stamp_blinker(grid, x, y, w),
                2 => stamp_toad(grid, x, y, w),
                3 => stamp_glider(grid, x, y, w),
                _ => stamp_beehive(grid, x, y, w),
            }
        }
    }
    stamp_lwss(grid, 5, h/2 - 2, w);
    stamp_pulsar(grid, w - 20, h/2 - 6, w);
}
