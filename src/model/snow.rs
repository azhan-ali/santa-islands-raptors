use turbo::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
pub struct SnowFlake {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub size: u32,
}

impl SnowFlake {
    pub fn new() -> Self {
        Self {
            x: (random::u32() % screen().w()) as f32,
            y: -((random::u32() % screen().h()) as f32),
            speed: (random::u32() % 3 + 2) as f32 * 0.5, // 1.0 to 2.5
            size: if random::u32() % 10 < 8 { 1 } else { 2 }, // Mostly 1px, some 2px
        }
    }

    pub fn update(&mut self) {
        self.y += self.speed;
        if self.y > screen().h() as f32 {
            self.y = -((random::u32() % 10) as f32);
            self.x = (random::u32() % screen().w()) as f32;
        }
    }

    pub fn draw(&self) {
        let color = if self.size == 1 { 0xAAAAAAFF } else { 0xFFFFFFFF };
        rect!(
            x = self.x as i32,
            y = self.y as i32,
            w = self.size,
            h = self.size,
            color = color
        );
    }
}
