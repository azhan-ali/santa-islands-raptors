use turbo::*;

#[turbo::serialize]
#[derive(Copy)] // Keep Copy if needed, remove conflicting ones
pub enum GiftType {
    Blue,
    Green,
    Purple,
}

#[turbo::serialize]
pub struct FactoryGift {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: GiftType,
    pub color: u32,
}

#[turbo::serialize]
pub struct FactoryParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub color: u32,
}

#[turbo::serialize]
pub struct FactoryMessage {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: u32,
    pub life: f32,
}

#[turbo::serialize]
pub struct FactorySnow {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub v: f32,
}

#[turbo::serialize]
pub struct FactoryGame {
    pub score: i32,
    pub time_left: f32,
    pub game_over: bool,
    pub player_x: f32,
    pub player_y: f32,
    // 0=None, 1=Blue, 2=Green, 3=Purple
    pub held_gift_type: u8, 
    pub held_gift_color: u32,
    
    pub gifts: Vec<FactoryGift>,
    pub particles: Vec<FactoryParticle>,
    pub messages: Vec<FactoryMessage>,
    pub snow: Vec<FactorySnow>,
    
    pub spawn_timer: u32,
    pub belt_anim_offset: f32,
}

impl FactoryGame {

    pub fn new() -> Self {
        
        // Init Snow

        // Actually, just loop and push after creation if I change structure,
        // or just vec![] and loop.
        
        // Let's rewrite the method slightly to be cleaner.
        let mut game = Self {
            score: 0,
            time_left: 60.0,
            game_over: false,
            player_x: 256.0,
            player_y: 144.0,
            held_gift_type: 0,
            held_gift_color: 0,
            gifts: vec![],
            particles: vec![],
            messages: vec![],
            spawn_timer: 0,
            belt_anim_offset: 0.0,
            snow: vec![],
        };
        
        for _ in 0..50 {
             game.snow.push(FactorySnow {
                x: (rand() % 512) as f32,
                y: (rand() % 288) as f32,
                r: (rand() % 2 + 1) as f32,
                v: (rand() % 2 + 1) as f32,
             });
        }
        
        game
    }

    pub fn update(&mut self) {
        if self.game_over {
            if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() {
                *self = Self::new(); // Restart
            }
            return;
        }

        // Timer
        let dt = 1.0 / 60.0;
        self.time_left -= dt;
        if self.time_left <= 0.0 {
            self.game_over = true;
        }

        // Player Movement
        let speed = 4.0;
        let gp = gamepad::get(0);
        
        if gp.left.pressed() { self.player_x -= speed; }
        if gp.right.pressed() { self.player_x += speed; }
        if gp.up.pressed() { self.player_y -= speed; }
        if gp.down.pressed() { self.player_y += speed; }

        // Bounds (Screen 512x288)
        self.player_x = self.player_x.clamp(10.0, 502.0);
        self.player_y = self.player_y.clamp(10.0, 278.0);

        // Belt & Spawning
        self.belt_anim_offset = (self.belt_anim_offset + 1.0) % 60.0;
        self.spawn_timer += 1;
        
        // Spawn every ~1.5s (90 frames)
        if self.spawn_timer > 90 {
            self.spawn_timer = 0;
            self.spawn_gift();
        }

        // Move Gifts
        let belt_speed = 1.5;
        for g in &mut self.gifts {
            g.x += belt_speed;
        }
        // Despawn offscreen
        self.gifts.retain(|g| g.x < 550.0);

        // Interaction
        if gp.a.just_pressed() || gp.start.just_pressed() {
            self.interact();
        }

        // Particles
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= 0.02;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Messages
        for m in &mut self.messages {
            m.y -= 0.5;
            m.life -= 0.02;
        }
        self.messages.retain(|m| m.life > 0.0);
        
        // Snow
        for s in &mut self.snow {
            s.y += s.v;
            if s.y > 288.0 { s.y = -5.0; s.x = (rand() % 512) as f32; }
        }
    }

    fn spawn_gift(&mut self) {
        // Types: 1=Blue, 2=Green, 3=Purple
        let t = (rand() % 3) + 1;
        let color = match t {
            1 => 0x3498DBFF, // Blue
            2 => 0x2ECC71FF, // Green
            3 => 0x9B59B6FF, // Purple
            _ => 0xFFFFFFFF,
        };
        
        self.gifts.push(FactoryGift {
            x: -30.0,
            y: 40.0, // Belt Y
            w: 24.0,
            h: 24.0,
            kind: match t {
                1 => GiftType::Blue,
                2 => GiftType::Green,
                _ => GiftType::Purple,
            },
            color,
        });
    }

    fn interact(&mut self) {
        // Drop Logic
        if self.held_gift_type != 0 {
            // Check Bins
            // Bins Y=240? Screen H=288. Let's say Bins at Y=230.
            // Bin 1 (Blue): x=80
            // Bin 2 (Green): x=256-40 = 216
            // Bin 3 (Purple): x=380
            let bin_y = 220.0;
            let bin_w = 60.0;
            let bin_h = 50.0;
            
            // Define Bins coords (Center X)
            let bins = [
                (100.0, 1, 0x3498DBFF), 
                (256.0, 2, 0x2ECC71FF), 
                (412.0, 3, 0x9B59B6FF)
            ];

            // let mut dropped = false;
            for (bx, btype, bcol) in bins {
                 // Check overlap
                 if self.player_x > bx - bin_w/2.0 && self.player_x < bx + bin_w/2.0 &&
                    self.player_y > bin_y - bin_h/2.0 && self.player_y < bin_y + bin_h/2.0 {
                        
                        if self.held_gift_type == btype {
                            self.score += 100;
                            self.spawn_msg("+100", bx, bin_y, 0xFFFF00FF);
                            self.spawn_particles(bx, bin_y, bcol, 10);
                        } else {
                            self.score -= 50;
                            self.spawn_msg("-50 WRONG", bx, bin_y, 0xFF0000FF);
                        }
                        self.held_gift_type = 0;
                        // dropped = true;
                        break;
                    }
            }
        }
        // Grab Logic
        else {
            // Check Gifts on Belt
            let mut grabbed_idx = None;
            for (i, g) in self.gifts.iter().enumerate() {
                // Distance Check
                let d = ((self.player_x - g.x).powi(2) + (self.player_y - g.y).powi(2)).sqrt();
                if d < 40.0 {
                    grabbed_idx = Some(i);
                    break;
                }
            }
            if let Some(i) = grabbed_idx {
                let g = self.gifts.remove(i);
                self.held_gift_type = match g.kind {
                    GiftType::Blue => 1,
                    GiftType::Green => 2,
                    GiftType::Purple => 3,
                };
                self.held_gift_color = g.color;
                self.spawn_msg("GOT IT!", self.player_x, self.player_y - 20.0, 0xFFFFFFFF);
            }
        }
    }

    fn spawn_msg(&mut self, text: &str, x: f32, y: f32, color: u32) {
        self.messages.push(FactoryMessage {
            x, y, text: text.to_string(), color, life: 1.0
        });
    }

    fn spawn_particles(&mut self, x: f32, y: f32, color: u32, count: usize) {
        for _ in 0..count {
             let vx = ((rand() % 100) as f32 - 50.0) / 10.0;
             let vy = ((rand() % 100) as f32 - 50.0) / 10.0;
             self.particles.push(FactoryParticle {
                 x, y, vx, vy, life: 1.0, color
             });
        }
    }

    pub fn draw(&self) {
        // Clear Black
        rect!(w=512, h=288, color=0x000000FF);
        
        // Background Snow
        for s in &self.snow {
             circ!(x=s.x as i32, y=s.y as i32, d=(s.r * 2.0) as u32, color=0xFFFFFF66);
        }

        // Belt (Top)
        let belt_y = 40;
        rect!(x=0, y=belt_y, w=512, h=40, color=0x333333FF);
        
        // Rollers
        for i in (0..520).step_by(40) {
            let offset = self.belt_anim_offset as i32 % 40;
            circ!(x=i - 20 + offset, y=belt_y+20, d=10, color=0x555555FF);
        }

        // Bins (Bottom)
        let bins = [
                (100.0, 0x3498DBFF, "BLUE"), 
                (256.0, 0x2ECC71FF, "GREEN"), 
                (412.0, 0x9B59B6FF, "PURPLE")
        ];
        let bin_y = 220;
        let bin_w = 60;
        let bin_h = 50;

        for (bx, col, label) in bins {
             let x = (bx as i32) - bin_w/2;
             let y = bin_y - bin_h/2;
             rect!(x=x, y=y, w=bin_w as u32, h=bin_h as u32, color=col);
             rect!(x=x, y=y, w=bin_w as u32, h=bin_h as u32, border_radius=0, color=0x00000000, border_size=2, border_color=0xFFFFFFFF);
             text!(label, x=x+10, y=y+15, font="small", color=0xFFFFFFFF);
        }

        // Gifts
        for g in &self.gifts {
            rect!(x=g.x as i32, y=g.y as i32, w=g.w as u32, h=g.h as u32, color=g.color);
            // Ribbon?
            rect!(x=g.x as i32 + 10, y=g.y as i32, w=4, h=g.h as u32, color=0xFFFFFF80);
            rect!(x=g.x as i32, y=g.y as i32 + 10, w=g.w as u32, h=4, color=0xFFFFFF80);
        }

        // Player (Santa) - Detailed
        let px = self.player_x as i32;
        let py = self.player_y as i32;
        
        // let direction = 1; // Removed unused variable 
        // For Factory, he moves left/right/up/down. Let's make him face "front" or slightly side.
        // Let's use Front Facing similar to Multiplayer design but simpler.
        
        // Body (Red Coat)
        rect!(x=px-6, y=py, w=12, h=14, color=0xD32F2FFF); 
        // White Fur (Center)
        rect!(x=px-2, y=py, w=4, h=14, color=0xFFFFFFFF);
        // Belt
        rect!(x=px-6, y=py+8, w=12, h=2, color=0x000000FF);
        rect!(x=px-2, y=py+8, w=4, h=2, color=0xF1C40FFF); // Buckle
        
        // Head
        rect!(x=px-5, y=py-10, w=10, h=10, color=0xFFCCBCFF); 
        // Beard
        rect!(x=px-5, y=py-4, w=10, h=6, color=0xFFFFFFFF);
        
        // Hat
        rect!(x=px-6, y=py-13, w=12, h=4, color=0xD32F2FFF); 
        rect!(x=px+4, y=py-12, w=4, h=4, color=0xFFFFFFFF); // Pom
        
        // Arms (Holding Gift?)
        if self.held_gift_type != 0 {
             // Arms up holding
             rect!(x=px-9, y=py+2, w=3, h=8, color=0xD32F2FFF);
             rect!(x=px+6, y=py+2, w=3, h=8, color=0xD32F2FFF);
             
             // The Held Gift (Above Head)
             rect!(x=px-8, y=py-22, w=16, h=16, color=self.held_gift_color);
             rect!(x=px-2, y=py-22, w=4, h=16, color=0xFFFFFF80); // Ribbon V
             rect!(x=px-8, y=py-16, w=16, h=4, color=0xFFFFFF80); // Ribbon H
             
        } else {
             // Arms Down
             rect!(x=px-9, y=py+4, w=3, h=8, color=0xD32F2FFF);
             rect!(x=px+6, y=py+4, w=3, h=8, color=0xD32F2FFF);
             // Hands
             rect!(x=px-9, y=py+12, w=3, h=3, color=0xFFCCBCFF);
             rect!(x=px+6, y=py+12, w=3, h=3, color=0xFFCCBCFF);
        }
        
        // Boots
        rect!(x=px-6, y=py+14, w=4, h=4, color=0x000000FF);
        rect!(x=px+2, y=py+14, w=4, h=4, color=0x000000FF);

        // Particles
        for p in &self.particles {
            rect!(x=p.x as i32, y=p.y as i32, w=2, h=2, color=p.color);
        }
        
        // Messages
        for m in &self.messages {
            text!(&m.text, x=m.x as i32, y=m.y as i32, font="medium", color=m.color);
        }

        // HUD
        let score_txt = format!("SCORE: {}", self.score);
        text!(&score_txt, x=10, y=10, font="medium", color=0xFFFFFFFF);
        let time_txt = format!("TIME: {:.0}", self.time_left);
        text!(&time_txt, x=400, y=10, font="medium", color=0xFFFFFFFF);

        // Game Over Screen
        if self.game_over {
            rect!(x=156, y=94, w=200, h=100, color=0x000000EE); // Box
            rect!(x=156, y=94, w=200, h=100, border_size=2, border_color=0xFFFFFFFF, color=0x00000000);
            text!("TIME'S UP!", x=210, y=110, font="large", color=0xFF0000FF);
            let final_score_txt = format!("Final Score: {}", self.score);
            text!(&final_score_txt, x=180, y=140, font="medium", color=0xFFFFFFFF);
            text!("Press START to Retry", x=180, y=170, font="small", color=0xAAAAAAFF);
        }
    }
}

fn rand() -> u32 {
    unsafe {
        static mut SEED: u32 = 12345;
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}
