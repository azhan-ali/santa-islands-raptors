use turbo::*;
use crate::model::world::World;

#[turbo::serialize]
#[derive(PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub width: f32,
    pub height: f32,
    pub on_ground: bool,
    pub score: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: 50.0,
            y: 100.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            width: 16.0,
            height: 24.0,
            on_ground: false,
            score: 0,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        // Horizontal Movement
        if gamepad::get(0).left.pressed() {
            self.velocity_x = -3.0;
        } else if gamepad::get(0).right.pressed() {
            self.velocity_x = 3.0;
        } else {
            self.velocity_x = 0.0;
        }

        // Jump
        if (gamepad::get(0).a.just_pressed() || gamepad::get(0).up.just_pressed()) && self.on_ground {
            self.velocity_y = -7.0;
            self.on_ground = false;
        }

        // Apply Gravity
        self.velocity_y += 0.3;

        // Move X
        self.x += self.velocity_x;
        self.resolve_collisions_x(world);

        // Move Y
        self.y += self.velocity_y;
        self.on_ground = false; // Assume falling until collision proves otherwise
        self.resolve_collisions_y(world);

        // Collect Gifts
        for gift in world.gifts.iter_mut() {
            if !gift.collected {
                if self.check_aabb(self.x, self.y, self.width, self.height, gift.x, gift.y, 12.0, 12.0) {
                    gift.collected = true;
                    self.score += 100;
                }
            }
        }
        
        // Fall off world
        if self.y > 300.0 {
            // Respawn
            self.x = world.camera_x + 50.0;
            self.y = 0.0;
            self.velocity_y = 0.0;
            self.score = self.score.saturating_sub(50);
        }
    }

    fn resolve_collisions_x(&mut self, world: &World) {
        for plat in &world.platforms {
            if self.check_aabb(self.x, self.y, self.width, self.height, plat.x, plat.y, plat.width, plat.height) {
                if self.velocity_x > 0.0 {
                    self.x = plat.x - self.width;
                } else if self.velocity_x < 0.0 {
                    self.x = plat.x + plat.width;
                }
                self.velocity_x = 0.0;
            }
        }
    }

    fn resolve_collisions_y(&mut self, world: &World) {
        for plat in &world.platforms {
            if self.check_aabb(self.x, self.y, self.width, self.height, plat.x, plat.y, plat.width, plat.height) {
                if self.velocity_y > 0.0 {
                    self.y = plat.y - self.height;
                    self.on_ground = true;
                } else if self.velocity_y < 0.0 {
                    self.y = plat.y + plat.height;
                }
                self.velocity_y = 0.0;
            }
        }
    }

    fn check_aabb(&self, x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
        x1 < x2 + w2 &&
        x1 + w1 > x2 &&
        y1 < y2 + h2 &&
        y1 + h1 > y2
    }

    pub fn draw(&self, camera_x: f32) {
        // Draw Santa (Red Body)
        rect!(
            x = (self.x - camera_x) as i32,
            y = self.y as i32,
            w = self.width as u32,
            h = self.height as u32,
            color = 0xFF0000FF
        );
        // Face
        rect!(
            x = (self.x - camera_x + 4.0) as i32,
            y = (self.y + 4.0) as i32,
            w = 8,
            h = 8,
            color = 0xFFCCAAFF
        );
        // Beard
         rect!(
            x = (self.x - camera_x + 4.0) as i32,
            y = (self.y + 12.0) as i32,
            w = 8,
            h = 6,
            color = 0xFFFFFFFF
        );
        // Hat PomPom
        rect!(
            x = (self.x - camera_x + 12.0) as i32,
            y = self.y as i32,
            w = 4,
            h = 4,
            color = 0xFFFFFFFF
        );
    }
}
