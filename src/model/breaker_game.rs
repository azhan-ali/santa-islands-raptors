use turbo::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
pub enum BreakerState {
    Menu,
    Playing,
    GameOver,
}

#[turbo::serialize]
pub struct BreakerBall {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
    pub active: bool,
}

#[turbo::serialize]
pub struct BreakerPaddle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,
}

#[turbo::serialize]
pub struct BreakerBrick {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: u32,
    pub active: bool,
}

#[turbo::serialize]
pub struct BreakerSnow {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub v: f32,
}

#[turbo::serialize]
pub struct BreakerGame {
    pub score: i32,
    pub lives: i32,
    pub level: i32,
    pub state: BreakerState,
    pub menu_selection: u8, // 0=Easy, 1=Medium, 2=Hard, 3=VeryHard
    pub paddle: BreakerPaddle,
    pub ball: BreakerBall,
    pub bricks: Vec<BreakerBrick>,
    pub snow: Vec<BreakerSnow>,
}

impl BreakerGame {
    pub fn new() -> Self {
        let mut game = Self {
            score: 0,
            lives: 3,
            level: 1,
            state: BreakerState::Menu,
            menu_selection: 1, // Default Medium
            paddle: BreakerPaddle {
                x: 0.0, // Set in reset_paddle
                y: 0.0,
                w: 60.0,
                h: 10.0,
                speed: 6.0,
            },
            ball: BreakerBall {
                x: 0.0,
                y: 0.0,
                r: 5.0,
                dx: 0.0,
                dy: 0.0,
                speed: 4.0,
                active: false,
            },
            bricks: vec![],
            snow: vec![],
        };
        
        // Init Snow
        for _ in 0..50 {
            game.snow.push(BreakerSnow {
                x: (rand() % 512) as f32,
                y: (rand() % 288) as f32,
                r: (rand() % 2 + 1) as f32,
                v: (rand() % 2 + 1) as f32,
            });
        }

        game.reset_paddle();
        game.reset_ball();
        // game.build_level(); // Build level when game starts
        
        game
    }

    fn reset_paddle(&mut self) {
        self.paddle.x = (512.0 - self.paddle.w) / 2.0;
        self.paddle.y = 288.0 - 30.0;
    }

    fn reset_ball(&mut self) {
        self.ball.active = false;
        self.ball.x = self.paddle.x + self.paddle.w / 2.0;
        self.ball.y = self.paddle.y - self.ball.r - 2.0;
    }

    fn launch_ball(&mut self) {
        self.ball.active = true;
        self.ball.dy = -self.ball.speed;
        // Random angle
        let r = (rand() % 100) as f32 / 100.0; // 0.0 - 1.0
        self.ball.dx = (r - 0.5) * 4.0;
    }

    fn build_level(&mut self) {
        self.bricks.clear();
        let size = 18.0; // Reduced from 25.0
        let gap = 5.0;
        let cols = ((512.0 - 40.0) / (size + gap)) as i32;
        let start_x = (512.0 - (cols as f32 * (size + gap))) / 2.0 + gap / 2.0;
        let rows = 4 + self.level;
        
        // Start Y higher (smaller bricks also helps)
        // Original Y=40.0. Level Y=40.0
        // Total height with 5 rows: 5 * (18+5) = 115. Y_bottom = 155.
        // Screen Height 288. Gap to paddle(~258) = 100. Good.
        let y_start = 40.0;
        
        let colors = [0xE74C3CFF, 0x2ECC71FF, 0x3498DBFF, 0x9B59B6FF, 0xF1C40FFF];

        for r in 0..rows {
            for c in 0..cols {
                if (r+c) % 6 == 0 { continue; } // Pattern

                self.bricks.push(BreakerBrick {
                    x: start_x + c as f32 * (size + gap),
                    y: y_start + r as f32 * (size + gap),
                    w: size,
                    h: size,
                    color: colors[(r as usize) % colors.len()],
                    active: true,
                });
            }
        }
    }

    pub fn update(&mut self) {
        let gp = gamepad::get(0);

        if self.state == BreakerState::Menu {
            if gp.up.just_pressed() && self.menu_selection > 0 {
                self.menu_selection -= 1;
            }
            if gp.down.just_pressed() && self.menu_selection < 3 {
                self.menu_selection += 1;
            }
            
            if gp.a.just_pressed() || gp.start.just_pressed() {
                self.lives = match self.menu_selection {
                    0 => 6, // Easy
                    1 => 3, // Medium
                    2 => 2, // Hard
                    3 => 1, // Very Hard
                    _ => 3,
                };
                self.score = 0;
                self.level = 1;
                self.build_level();
                self.reset_paddle();
                self.reset_ball();
                self.ball.speed = 4.0; // Reset speed
                self.state = BreakerState::Playing;
            }
            return;
        }

        if self.state == BreakerState::GameOver {
            if gp.start.just_pressed() || gp.a.just_pressed() {
                self.state = BreakerState::Menu; // Go back to menu
            }
            return;
        }

        // --- PLAYING STATE ---

        // Snow
        for s in &mut self.snow {
            s.y += s.v;
            if s.y > 288.0 { s.y = -5.0; }
        }

        // Paddle
        if gp.left.pressed() && self.paddle.x > 0.0 { self.paddle.x -= self.paddle.speed; }
        if gp.right.pressed() && self.paddle.x + self.paddle.w < 512.0 { self.paddle.x += self.paddle.speed; }

        // Ball Logic
        if !self.ball.active {
            self.ball.x = self.paddle.x + self.paddle.w / 2.0;
            self.ball.y = self.paddle.y - self.ball.r - 2.0;
            
            if gp.a.just_pressed() || gp.start.just_pressed() {
                self.launch_ball();
            }
        } else {
            // Move Ball
            self.ball.x += self.ball.dx;
            self.ball.y += self.ball.dy;

            // Walls
            if self.ball.x + self.ball.r > 512.0 { 
                self.ball.x = 512.0 - self.ball.r; 
                self.ball.dx *= -1.0; 
            }
            if self.ball.x - self.ball.r < 0.0 { 
                self.ball.x = self.ball.r; 
                self.ball.dx *= -1.0; 
            }
            if self.ball.y - self.ball.r < 0.0 { 
                self.ball.y = self.ball.r; 
                self.ball.dy *= -1.0; 
            }

            // Floor
            if self.ball.y - self.ball.r > 288.0 {
                self.lives -= 1;
                if self.lives <= 0 {
                    self.state = BreakerState::GameOver;
                } else {
                    self.reset_ball();
                }
            }

            // Paddle Collision
            let b = &mut self.ball; // Mutable borrow ball
            let p = &self.paddle;
            if b.y + b.r >= p.y && b.y - b.r <= p.y + p.h &&
               b.x >= p.x && b.x <= p.x + p.w {
                
                b.dy = -b.dy.abs(); // Bounce Up
                
                // Angular bounce
                let hit_point = b.x - (p.x + p.w / 2.0);
                b.dx = hit_point * 0.2;
            }

            // Brick Collision
            let b_x = self.ball.x;
            let b_y = self.ball.y;
            let b_r = self.ball.r;
            
            let mut hit_idx = None;
            for (i, br) in self.bricks.iter().enumerate() {
                if !br.active { continue; }
                
                if b_x + b_r > br.x && b_x - b_r < br.x + br.w &&
                   b_y + b_r > br.y && b_y - b_r < br.y + br.h {
                       hit_idx = Some(i);
                       break;
                   }
            }
            
            if let Some(i) = hit_idx {
                self.bricks[i].active = false;
                self.ball.dy *= -1.0;
                self.score += 100;
                
                // Check Level Clear
                let remaining = self.bricks.iter().filter(|b| b.active).count();
                if remaining == 0 {
                    self.level += 1;
                    self.ball.speed += 1.0;
                    self.build_level();
                    self.reset_ball();
                }
            }
        }
    }

    pub fn draw(&self) {
        // BG
        rect!(w=512, h=288, color=0x000000FF);

        // Menu Draw
        if self.state == BreakerState::Menu {
            let title = "SANTA BREAKER";
            text!(title, x=180, y=50, font="large", color=0xE74C3CFF);
            text!("Select Difficulty:", x=180, y=100, font="medium", color=0xFFFFFFFF);

            let options = ["EASY (6 Lives)", "MEDIUM (3 Lives)", "HARD (2 Lives)", "VERY HARD (1 Life)"];
            for (i, opt) in options.iter().enumerate() {
                let y = 140 + i as i32 * 30;
                let color = if self.menu_selection == i as u8 { 0xFFFF00FF } else { 0xAAAAAAFF };
                // Cursor
                if self.menu_selection == i as u8 {
                    text!(">", x=140, y=y, font="medium", color=0xFFFF00FF);
                }
                text!(opt, x=160, y=y, font="medium", color=color);
            }
            text!("Press Start/A", x=200, y=260, font="small", color=0x888888FF);
            return;
        }

        // Play/GameOver Draw
        // Snow
        for s in &self.snow {
            circ!(x=s.x as i32, y=s.y as i32, d=(s.r * 2.0) as u32, color=0xFFFFFF80);
        }

        // Paddle
        rect!(x=self.paddle.x as i32, y=self.paddle.y as i32, w=self.paddle.w as u32, h=self.paddle.h as u32, color=0xC0392BFF);
        rect!(x=self.paddle.x as i32, y=self.paddle.y as i32 + 2, w=self.paddle.w as u32, h=2, color=0xF1C40FFF);

        // Ball (Santa Face)
        let bx = self.ball.x as i32;
        let by = self.ball.y as i32;
        // let br = self.ball.r as i32; // Unused
        
        // Ensure size is big enough for face. Radius 5 is diameter 10.
        // Let's draw a slightly larger sprite centered at bx, by.
        // Original logic checks collision with r=5. 
        // We will draw a 12x12 sprite centered.
        
        let sx = bx - 6;
        let sy = by - 6;
        
        // Head
        rect!(x=sx+2, y=sy+4, w=8, h=6, color=0xFFCCBCFF); 
        // Beard
        rect!(x=sx+1, y=sy+8, w=10, h=4, color=0xFFFFFFFF);
        // Hat
        rect!(x=sx, y=sy, w=12, h=4, color=0xD32F2FFF);
        rect!(x=sx+10, y=sy+1, w=2, h=2, color=0xFFFFFFFF); // Pom
        
        // Debug/Hitbox (Optional, commented out)
        // circ!(x=bx - br, y=by - br, d=(br*2) as u32, color=0xFF000044);

        // Bricks
        for b in &self.bricks {
            if !b.active { continue; }
            rect!(x=b.x as i32, y=b.y as i32, w=b.w as u32, h=b.h as u32, color=b.color);
            // Ribbon
            rect!(x=b.x as i32 + b.w as i32 / 2 - 2, y=b.y as i32, w=4, h=b.h as u32, color=0xFFFFFF66);
            rect!(x=b.x as i32, y=b.y as i32 + b.h as i32 / 2 - 2, w=b.w as u32, h=4, color=0xFFFFFF66);
        }

        // HUD
        let score_txt = format!("SCORE: {}", self.score);
        text!(&score_txt, x=10, y=10, font="medium", color=0xFFFFFFFF);
        let lives_txt = format!("LIVES: {}", self.lives);
        text!(&lives_txt, x=440, y=10, font="medium", color=0xFFFFFFFF);
        
        if self.state == BreakerState::Playing && !self.ball.active {
             text!("PRESS START", x=210, y=200, font="small", color=0xAAAAAAFF);
        }

        // Game Over
        if self.state == BreakerState::GameOver {
            rect!(x=156, y=94, w=200, h=100, color=0x000000EE);
            rect!(x=156, y=94, w=200, h=100, border_size=2, border_color=0xFF0000FF, color=0x00000000);
            text!("GAME OVER", x=210, y=110, font="large", color=0xFF0000FF);
            let final_score_txt = format!("Score: {}", self.score);
            text!(&final_score_txt, x=200, y=140, font="medium", color=0xFFFFFFFF);
            text!("Press START", x=210, y=170, font="small", color=0xAAAAAAFF);
        }
    }
}

fn rand() -> u32 {
    unsafe {
        static mut SEED: u32 = 54321;
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}
