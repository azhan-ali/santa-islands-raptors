use turbo::*;

pub fn draw_title(x: i32, y: i32, scale: u32) {
    let color_main = 0xFF0000FF; // Red
    let color_shadow = 0xFFD700FF; // Gold

    // S
    draw_char(x, y, scale, 0, color_main, color_shadow);
    // A
    draw_char(x + (6 * scale as i32), y, scale, 1, color_main, color_shadow);
    // N
    draw_char(x + (12 * scale as i32), y, scale, 2, color_main, color_shadow);
    // T
    draw_char(x + (18 * scale as i32), y, scale, 3, color_main, color_shadow);
    // A
    draw_char(x + (24 * scale as i32), y, scale, 1, color_main, color_shadow);

    let offset_x = 32 * scale as i32;
    // I
    draw_char(x + offset_x, y, scale, 4, color_main, color_shadow);
    // S
    draw_char(x + offset_x + (6 * scale as i32), y, scale, 0, color_main, color_shadow);
    // L
    draw_char(x + offset_x + (12 * scale as i32), y, scale, 5, color_main, color_shadow);
    // A
    draw_char(x + offset_x + (18 * scale as i32), y, scale, 1, color_main, color_shadow);
    // N
    draw_char(x + offset_x + (24 * scale as i32), y, scale, 2, color_main, color_shadow);
    // D
    draw_char(x + offset_x + (30 * scale as i32), y, scale, 6, color_main, color_shadow);
    // S
    draw_char(x + offset_x + (36 * scale as i32), y, scale, 0, color_main, color_shadow);
}

fn draw_char(x: i32, y: i32, scale: u32, char_id: u8, color: u32, shadow: u32) {
    // 5x5 Grid representation
    let pattern = match char_id {
        0 => 0b01111_10000_01110_00001_11110, // S
        1 => 0b01110_10001_11111_10001_10001, // A
        2 => 0b10001_11001_10101_10011_10001, // N
        3 => 0b11111_00100_00100_00100_00100, // T
        4 => 0b01110_00100_00100_00100_01110, // I
        5 => 0b10000_10000_10000_10000_11111, // L
        6 => 0b11110_10001_10001_10001_11110, // D
        _ => 0,
    };

    let s = scale as i32;
    for row in 0..5 {
        for col in 0..5 {
            if (pattern >> ((4 - row) * 5 + (4 - col))) & 1 == 1 {
                // Shadow
                rect!(x = x + (col * s) + s, y = y + (row * s) + s, w = scale, h = scale, color = shadow);
                // Main
                rect!(x = x + (col * s), y = y + (row * s), w = scale, h = scale, color = color);
            }
        }
    }
}