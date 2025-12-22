use turbo::*;

#[turbo::serialize]
pub struct SleighStar {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub speed: f32,
}

#[turbo::serialize]
pub struct SleighBullet {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,
}

#[turbo::serialize]
pub struct SleighEnemy {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub color: u32,
}

#[turbo::serialize]
pub struct SleighParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub color: u32,
}

#[turbo::serialize]
pub struct SleighGame {
    pub score: i32,
    pub lives: i32,
    pub game_over: bool,
    pub player_x: f32,
    pub player_y: f32,
    pub player_w: f32,
    pub player_h: f32,
    
    // Difficulty
    pub enemy_base_speed: f32,
    pub spawn_rate: u32,
    pub fire_delay: u32,
    pub last_shot_frame: u32,
    pub difficulty_timer: u32,
    
    pub frame_count: u32,

    pub bullets: Vec<SleighBullet>,
    pub enemies: Vec<SleighEnemy>,
    pub particles: Vec<SleighParticle>,
    pub stars: Vec<SleighStar>,
}

impl SleighGame {
    pub fn new() -> Self {
        let mut game = Self {
            score: 0,
            lives: 3,
            game_over: false,
            player_x: 30.0,
            player_y: 130.0,
            player_w: 60.0, // Scaled down from 80
            player_h: 30.0, // Scaled down from 50
            enemy_base_speed: 2.0,
            spawn_rate: 100,
            fire_delay: 20, // Frames (approx 200ms at 60fps is 12 frames, but lets settle on 20)
            last_shot_frame: 0,
            difficulty_timer: 0,
            frame_count: 0,
            bullets: vec![],
            enemies: vec![],
            particles: vec![],
            stars: vec![],
        };
        
        // Init Stars
        for _ in 0..100 {
            game.stars.push(SleighStar {
                x: (rand() % 512) as f32,
                y: (rand() % 288) as f32,
                size: (rand() % 2 + 1) as f32,
                speed: (rand() % 4 + 1) as f32,
            });
        }
        
        game
    }

    pub fn update(&mut self) {
        if self.game_over {
            if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() {
                *self = Self::new();
            }
            return;
        }

        self.frame_count += 1;
        self.difficulty_timer += 1;

        // Difficulty Increase (Every ~10s = 600 frames)
        if self.difficulty_timer > 600 {
            self.difficulty_timer = 0;
            self.enemy_base_speed += 0.5;
            if self.spawn_rate > 30 { self.spawn_rate -= 10; }
            if self.fire_delay > 10 { self.fire_delay -= 2; }
            
            // Apply to existing? No, keeps simple.
            // Notify? We can draw text.
        }

        // Player Move
        // Speed 3.0
        let speed = 3.0;
        let gp = gamepad::get(0);
        if gp.left.pressed() && self.player_x > 0.0 { self.player_x -= speed; }
        if gp.right.pressed() && self.player_x < 512.0 - self.player_w { self.player_x += speed; }
        if gp.up.pressed() && self.player_y > 0.0 { self.player_y -= speed; }
        if gp.down.pressed() && self.player_y < 288.0 - self.player_h { self.player_y += speed; }

        // Shoot (Auto-fire space held or single press)
        if gp.a.pressed() || gp.start.pressed() { // A or Start or Space mapped to A usually
            if self.frame_count - self.last_shot_frame > self.fire_delay {
                self.bullets.push(SleighBullet {
                    x: self.player_x + self.player_w,
                    y: self.player_y + 15.0, // Center-ish
                    w: 8.0,
                    h: 3.0,
                    speed: 7.0,
                });
                self.last_shot_frame = self.frame_count;
            }
        }

        // Update Stars
        for star in &mut self.stars {
            star.x -= star.speed;
            if star.x < 0.0 { star.x = 512.0; star.y = (rand() % 288) as f32; }
        }

        // Update Bullets
        for b in &mut self.bullets {
            b.x += b.speed;
        }
        self.bullets.retain(|b| b.x < 520.0);

        // Update Enemies
        // Spawn
        if self.frame_count % self.spawn_rate == 0 {
             let hp = (rand() % 5 + 1) as i32 + (self.score / 500);
             let _eh = 30.0;
             let ey = (rand() % (288 - 40)) as f32 + 10.0;
             let base_s = self.enemy_base_speed;
             
             // Random HSL-ish Color logic
             // Just simple predefined colors for now
             let colors = [0xFF0000FF, 0x00FF00FF, 0x0000FFFF, 0xFFFF00FF, 0xFF00FFFF, 0x00FFFFFF];
             let col = colors[(rand() % 6) as usize];

             self.enemies.push(SleighEnemy {
                 x: 520.0,
                 y: ey,
                 w: 40.0,
                 h: 40.0,
                 speed: base_s + ((rand() % 20) as f32 / 10.0),
                 hp,
                 max_hp: hp,
                 color: col,
             });
        }

        for i in (0..self.enemies.len()).rev() {
            let mut remove = false;
            // Move
            self.enemies[i].x -= self.enemies[i].speed;

            // Collision Player
            // Clone to avoid borrow issues when modifying self.enemies later
            let e = self.enemies[i].clone(); 
            if aabb_intersect(self.player_x, self.player_y, self.player_w, self.player_h,
                              e.x, e.y, e.w, e.h) {
                
                self.lives -= 1;
                self.create_particles(self.player_x + self.player_w/2.0, self.player_y + self.player_h/2.0, 0xFF0000FF, 10);
                remove = true;
                if self.lives <= 0 { self.game_over = true; }
            }
            
            // Collision Bullets
            if !remove {
                // Use indices to avoid double borrow
                let mut hit_idx = None;
                for j in (0..self.bullets.len()).rev() {
                    let b = &self.bullets[j];
                     if aabb_intersect(b.x, b.y, b.w, b.h, e.x, e.y, e.w, e.h) {
                          hit_idx = Some(j);
                          self.enemies[i].hp -= 1;
                          if self.enemies[i].hp <= 0 {
                               self.score += self.enemies[i].max_hp * 10;
                               // We need to create particles, but we can't call self.create_particles here easily due to borrow of 'e' (from enemies[i]) and 'self.enemies' index
                               // Defer or inline logic? Inline is easier for now.
                               // Inline particle creation logic to avoid method call borrow issues
                               for _ in 0..8 {
                                    let vx = ((rand() % 100) as f32 - 50.0) / 10.0;
                                    let vy = ((rand() % 100) as f32 - 50.0) / 10.0;
                                    self.particles.push(SleighParticle {
                                        x: self.enemies[i].x + self.enemies[i].w/2.0, 
                                        y: self.enemies[i].y + self.enemies[i].h/2.0, 
                                        vx, vy, life: 1.0, color: self.enemies[i].color
                                    });
                               }
                               remove = true;
                          }
                          break; 
                     }
                }
                if let Some(idx) = hit_idx {
                    self.bullets.remove(idx);
                }
            }

            if remove || self.enemies[i].x < -50.0 {
                self.enemies.remove(i);
            }
        }

        // Update Particles
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= 0.05;
        }
        self.particles.retain(|p| p.life > 0.0);
    }
    
    fn create_particles(&mut self, x: f32, y: f32, color: u32, count: usize) {
        for _ in 0..count {
             let vx = ((rand() % 100) as f32 - 50.0) / 10.0;
             let vy = ((rand() % 100) as f32 - 50.0) / 10.0;
             self.particles.push(SleighParticle {
                 x, y, vx, vy, life: 1.0, color
             });
        }
    }

    pub fn draw(&self) {
        // BG
        rect!(w=512, h=288, color=0x000000FF);

        // Stars
        for s in &self.stars {
            rect!(x=s.x as i32, y=s.y as i32, w=s.size as u32, h=s.size as u32, color=0xFFFFFFFF);
        }

        // Player (Sleigh + Reindeer)
        let px = self.player_x as i32;
        let py = self.player_y as i32;

        // Player (Santa + Sleigh + Reindeer) - Pixel Art
        // Animation Frame (0 or 1)
        let anim = (self.frame_count / 10) % 2; 

        // -- Reindeer (Leading the sleigh) --
        // Position relative to player_x/y
        let rx = px + 40;
        let ry = py + 5;

        // Reindeer Legs (Animated)
        let leg_color = 0x6D4C41FF;
        if anim == 0 {
            rect!(x=rx+5,  y=ry+15, w=3, h=10, color=leg_color); // Front Left
            rect!(x=rx+15, y=ry+15, w=3, h=10, color=leg_color); // Back Left
        } else {
            rect!(x=rx+2,  y=ry+14, w=3, h=8, color=leg_color);  // Front Left (Raised)
            rect!(x=rx+18, y=ry+14, w=3, h=8, color=leg_color);  // Back Left (Raised)
        }

        // Reindeer Body
        rect!(x=rx, y=ry+5, w=25, h=12, color=0x8D6E63FF); // Main body
        rect!(x=rx-2, y=ry+4, w=4, h=6, color=0xFFFFFFFF); // Tail

        // Reindeer Head & Neck
        rect!(x=rx+20, y=ry-2, w=8, h=10, color=0x8D6E63FF); // Neck
        rect!(x=rx+22, y=ry-6, w=10, h=9, color=0x8D6E63FF); // Head
        
        // Antlers
        rect!(x=rx+26, y=ry-10, w=2, h=4, color=0xD7CCC8FF);
        rect!(x=rx+28, y=ry-9, w=4, h=2, color=0xD7CCC8FF);

        // Nose (Rudolph Red!)
        rect!(x=rx+32, y=ry-2, w=3, h=3, color=0xFF0000FF);

        // Reins (Connecting to sleigh)
        rect!(x=px+30, y=ry+6, w=15, h=1, color=0xF1C40FFF);


        // -- Sleigh --
        let sx = px;
        let sy = py + 10;

        // Runners (Gold)
        rect!(x=sx, y=sy+15, w=40, h=2, color=0xF1C40FFF); // Bottom runner
        rect!(x=sx, y=sy+10, w=2, h=5, color=0xF1C40FFF);  // Support Back
        rect!(x=sx+30, y=sy+10, w=2, h=5, color=0xF1C40FFF); // Support Front
        rect!(x=sx+38, y=sy+10, w=2, h=5, color=0xF1C40FFF); // Curved tip support

        // Sleigh Body (Red)
        rect!(x=sx, y=sy, w=35, h=12, color=0xB71C1CFF); 
        rect!(x=sx-2, y=sy, w=2, h=14, color=0xD32F2FFF); // Back rest
        rect!(x=sx+35, y=sy+5, w=3, h=7, color=0xD32F2FFF); // Front curve
        
        // Ornament / Trim
        rect!(x=sx, y=sy+4, w=35, h=2, color=0xFFD700FF);


        // -- Santa --
        let santa_x = sx + 10;
        let santa_y = sy - 8;

        // Body
        rect!(x=santa_x, y=santa_y+5, w=14, h=10, color=0xD32F2FFF); // Red Coat
        rect!(x=santa_x+4, y=santa_y+5, w=6, h=10, color=0xFFFFFFFF); // White Fur center/beard flow

        // Head
        rect!(x=santa_x+2, y=santa_y, w=10, h=8, color=0xFFCCBCFF); // Face
        rect!(x=santa_x+2, y=santa_y+5, w=10, h=4, color=0xFFFFFFFF); // Beard
        
        // Hat
        rect!(x=santa_x, y=santa_y-4, w=14, h=4, color=0xD32F2FFF); // Hat Red
        rect!(x=santa_x+12, y=santa_y-2, w=4, h=4, color=0xFFFFFFFF); // Pom Pom

        // -- Sack of Gifts --
        let sack_x = sx + 2;
        let sack_y = sy - 5;
        rect!(x=sack_x, y=sack_y, w=8, h=10, color=0x795548FF); // Brown Sack
        rect!(x=sack_x+1, y=sack_y-2, w=6, h=2, color=0xA1887FFF); // Top tied


        // Bullets
        for b in &self.bullets {
            rect!(x=b.x as i32, y=b.y as i32, w=b.w as u32, h=b.h as u32, color=0xFFFF00FF);
        }

        // Enemies
        for e in &self.enemies {
            let ex = e.x as i32;
            let ey = e.y as i32;
            let ew = e.w as u32;
            let eh = e.h as u32;
            
            rect!(x=ex, y=ey, w=ew, h=eh, color=e.color);
            // Ribbon
            rect!(x=ex+ew as i32/2 - 2, y=ey, w=4, h=eh, color=0xFFFFFF66);
            rect!(x=ex, y=ey+eh as i32/2 - 2, w=ew, h=4, color=0xFFFFFF66);
            
            // Health Number
            let val = e.hp;
            let txt = format!("{}", val);
            // Large font (approx 16px wide). Box 40 wide.
            // Center: (40 - 16)/2 = 12.
            let offset = if val > 9 { 4 } else { 12 }; 
            text!(&txt, x=ex + offset, y=ey + 10, font="large", color=0x000000FF);
        }

        // Particles
        for p in &self.particles {
             rect!(x=p.x as i32, y=p.y as i32, w=2, h=2, color=p.color);
        }

        // HUD
        let score_txt = format!("SCORE: {}", self.score);
        text!(&score_txt, x=10, y=10, font="medium", color=0x00FFFFFF);
        
        let lives_txt = format!("LIVES: {}", self.lives);
        text!(&lives_txt, x=440, y=10, font="medium", color=0xFF0000FF);

        // Game Over
        if self.game_over {
            rect!(x=156, y=94, w=200, h=100, color=0x000000EE);
            rect!(x=156, y=94, w=200, h=100, border_size=2, border_color=0xFF0000FF, color=0x00000000);
            text!("MISSION FAILED", x=190, y=110, font="large", color=0xFF0000FF);
            let final_score = format!("Final Score: {}", self.score);
            text!(&final_score, x=180, y=140, font="medium", color=0xFFFFFFFF);
            text!("Press START to Retry", x=180, y=170, font="small", color=0xAAAAAAFF);
        }
    }
}

fn aabb_intersect(x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

fn rand() -> u32 {
    unsafe {
        static mut SEED: u32 = 98765;
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}
