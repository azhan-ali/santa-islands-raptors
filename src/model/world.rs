use turbo::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[turbo::serialize]
#[derive(Copy, PartialEq)]
pub struct Gift {
    pub x: f32,
    pub y: f32,
    pub collected: bool,
}

#[turbo::serialize]

pub struct World {
    pub platforms: Vec<Platform>,
    pub gifts: Vec<Gift>,
    pub camera_x: f32,
}

impl World {
    pub fn new() -> Self {
        let mut platforms = vec![];
        let mut gifts = vec![];

        // Ground
        for i in 0..20 {
            platforms.push(Platform {
                x: i as f32 * 64.0,
                y: 200.0,
                width: 64.0,
                height: 32.0,
            });
        }

        // Random platforms and gifts
        let mut rng = random::u32();
        for i in 1..20 {
            if rng % 3 == 0 {
                let px = i as f32 * 100.0 + (rng % 50) as f32;
                let py = 150.0 - (rng % 80) as f32;
                platforms.push(Platform {
                    x: px,
                    y: py,
                    width: 48.0,
                    height: 16.0,
                });
                
                // Add gift on platform
                gifts.push(Gift {
                    x: px + 16.0,
                    y: py - 16.0,
                    collected: false,
                });
            }
            rng = (rng * 1103515245 + 12345) % 2147483648;
        }

        Self {
            platforms,
            gifts,
            camera_x: 0.0,
        }
    }

    pub fn update(&mut self, player_x: f32) {
        // Camera follows player
        let target_cam = player_x - 100.0;
        if target_cam > self.camera_x {
            self.camera_x = target_cam;
        }
    }

    pub fn draw(&self) {
        // Draw sky
        rect!(w = screen().w(), h = screen().h(), color = 0x87CEEBff);

        // HUD


        // Draw Platforms
        for plat in &self.platforms {
            rect!(
                x = (plat.x - self.camera_x) as i32,
                y = plat.y as i32,
                w = plat.width as u32,
                h = plat.height as u32,
                color = 0xFFFFFFFF // Snowy white
            );
             // Top snow layer
            rect!(
                x = (plat.x - self.camera_x) as i32,
                y = plat.y as i32,
                w = plat.width as u32,
                h = 4,
                color = 0xE0F7FAFF
            );
        }

        // Draw Gifts
        for gift in &self.gifts {
            if !gift.collected {
                rect!(
                    x = (gift.x - self.camera_x) as i32,
                    y = gift.y as i32,
                    w = 12,
                    h = 12,
                    color = 0xFF0000FF // Red Box
                );
                rect!(
                    x = (gift.x - self.camera_x + 4.0) as i32,
                    y = gift.y as i32,
                    w = 4,
                    h = 12,
                    color = 0x00FF00FF // Green Ribbon
                );
            }
        }
    }
}
