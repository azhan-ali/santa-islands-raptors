use turbo::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
pub enum StealthState {
    Menu,
    Playing,
    GameOver,
    Win,
}

#[turbo::serialize]
pub struct StealthDog {
    pub id: u8,
    pub x: f32,
    pub y: f32,
    pub patrol_min: f32,
    pub patrol_max: f32,
    pub patrol_dir: f32,
    pub alert: f32,
    pub is_patrol: bool,
    pub name: String,
}

#[turbo::serialize]
pub struct StealthStar {
    pub x: f32,
    pub y: f32,
}

#[turbo::serialize]
pub struct StealthWave {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub alpha: f32,
}

#[turbo::serialize]
pub struct StealthGame {
    pub state: StealthState,
    pub player_x: f32,
    pub player_y: f32,
    pub cam_x: f32,
    pub cam_y: f32,
    pub stars_collected: u32,
    pub gift1_done: bool, // Dog 1 Room
    pub gift2_done: bool, // Dog 2 Room
    pub gift3_done: bool, // Tree (Wolf)
    pub cookie_done: bool,
    pub exit_open: bool,
    pub time_elapsed: f32, // Timer
    pub dogs: Vec<StealthDog>,
    pub stars: Vec<StealthStar>,
    pub waves: Vec<StealthWave>,
    pub msg: String,
}

impl StealthGame {
    pub fn new() -> Self {
        // Map is 920x900.
        // Dogs: 
        // D1: 175, 175 (Sleep)
        // D2: 745, 450 (Sleep)
        // D3: 600, 700 (Patrol 500-800)
        
        let dogs = vec![
            StealthDog { id: 0, x: 175.0, y: 175.0, patrol_min: 0.0, patrol_max: 0.0, patrol_dir: 0.0, alert: 0.0, is_patrol: false, name: "Dog 1".to_string() },
            StealthDog { id: 1, x: 745.0, y: 450.0, patrol_min: 0.0, patrol_max: 0.0, patrol_dir: 0.0, alert: 0.0, is_patrol: false, name: "Dog 2".to_string() },
            StealthDog { id: 2, x: 600.0, y: 700.0, patrol_min: 500.0, patrol_max: 800.0, patrol_dir: 1.5, alert: 0.0, is_patrol: true, name: "Wolf".to_string() },
        ];

        let stars = vec![
            StealthStar { x: 100.0, y: 100.0 },
            StealthStar { x: 745.0, y: 100.0 },
            StealthStar { x: 100.0, y: 450.0 },
            StealthStar { x: 745.0, y: 700.0 },
            StealthStar { x: 250.0, y: 750.0 },
        ];

        Self {
            state: StealthState::Menu,
            player_x: 460.0,
            player_y: 250.0,
            cam_x: 0.0,
            cam_y: 0.0,
            stars_collected: 0,
            gift1_done: false,
            gift2_done: false,
            gift3_done: false,
            cookie_done: false,
            exit_open: false,
            time_elapsed: 0.0,
            dogs,
            stars,
            waves: vec![],
            msg: "".to_string(),
        }
    }

    pub fn update(&mut self) {
        let gp = gamepad::get(0);

        if self.state == StealthState::Menu {
            if gp.a.just_pressed() || gp.start.just_pressed() {
                self.reset();
                self.state = StealthState::Playing;
            }
            return;
        }
        if self.state == StealthState::GameOver || self.state == StealthState::Win {
             if gp.a.just_pressed() || gp.start.just_pressed() {
                self.state = StealthState::Menu;
             }
             return;
        }

        // --- PLAYING ---
        self.time_elapsed += 1.0 / 60.0;
        
        // Movement
        // No more Sneak button. Speed is constant "careful" speed.
        let speed = 3.0; 

        let mut dx = 0.0;
        let mut dy = 0.0;
        if gp.up.pressed() { dy -= speed; }
        if gp.down.pressed() { dy += speed; }
        if gp.left.pressed() { dx -= speed; }
        if gp.right.pressed() { dx += speed; }

        let nx = self.player_x + dx;
        let ny = self.player_y + dy;

        // Collision
        let walls = [
             (0.0, 0.0, 920.0, 20.0), (0.0, 880.0, 920.0, 20.0), (0.0, 0.0, 20.0, 900.0), (900.0, 0.0, 20.0, 900.0),
             (0.0, 300.0, 200.0, 20.0), (720.0, 300.0, 200.0, 20.0),
             (0.0, 550.0, 200.0, 20.0), (720.0, 550.0, 200.0, 20.0),
             (300.0, 0.0, 20.0, 200.0), (300.0, 300.0, 20.0, 300.0),
             (620.0, 0.0, 20.0, 200.0), (620.0, 300.0, 20.0, 250.0),
             (400.0, 600.0, 20.0, 300.0)
        ];
        
        let mut hit = false;
        for w in walls {
            if rect_circle_hit(w.0, w.1, w.2, w.3, nx, ny, 15.0) { hit = true; break; }
        }

        if !hit {
            self.player_x = nx;
            self.player_y = ny;
        }

        // Noise
        let moving = dx != 0.0 || dy != 0.0;
        if moving {
            // Constant low noise. Use patience (stop) to recover.
             if (rand() % 100) > 85 { // Less frequent waves
                 self.waves.push(StealthWave { x: self.player_x, y: self.player_y, r: 5.0, alpha: 0.6 });
             }
             
             for d in &mut self.dogs {
                 let dist = ((self.player_x - d.x).powi(2) + (self.player_y - d.y).powi(2)).sqrt();
                 if dist < 400.0 {
                     let factor = (400.0 - dist) / 400.0;
                     // Slower alert rise than before since you can't sneak
                     d.alert += 0.1 + (factor * 0.8); 
                 } else {
                     d.alert += 0.02;
                 }
             }
        } else {
             // Recovery (Patience)
             for d in &mut self.dogs {
                 if d.alert > 0.0 { d.alert -= 0.3; } // Fast recovery when stopped
             }
        }

        // Dogs Logic
        for d in &mut self.dogs {
            // Patrol
            if d.is_patrol {
                d.x += d.patrol_dir;
                if d.x > d.patrol_max || d.x < d.patrol_min { d.patrol_dir *= -1.0; }
                if ((self.player_x - d.x).powi(2) + (self.player_y - d.y).powi(2)).sqrt() < 50.0 {
                    d.alert = 100.0;
                }
            }
            if d.alert < 0.0 { d.alert = 0.0; }
            if d.alert >= 100.0 {
                self.state = StealthState::GameOver;
                self.msg = format!("{} WOKE UP!", d.name);
            }
        }

        // Interact (Space/A)
        if gp.a.just_pressed() || gp.start.just_pressed() {
            // Need to be reasonably close. 60px is generous.
            // Gift 1 (Dog 1): 175, 175 (Near Dog 1 but slightly offset? No, room center is ~150,150. Dog is 175,175. Let's aim for 150, 150)
            // Let's check proximity to Dog itself for "placing near dog".
            
            // Objective 1: Near Dog 1 (175, 175). Let's say within 100px of Dog 1.
            if !self.gift1_done && dist(self.player_x, self.player_y, 175.0, 175.0) < 100.0 {
                self.gift1_done = true;
            }
            
            // Objective 2: Near Dog 2 (745, 450). Within 100px.
            if !self.gift2_done && dist(self.player_x, self.player_y, 745.0, 450.0) < 100.0 {
                self.gift2_done = true;
            }

            // Objective 3: Near Tree/Wolf (800, 800) or Wolf Patrol (600, 700).
            // User said "Christmas tree ke pass". Tree is at 800,800.
            if !self.gift3_done && dist(self.player_x, self.player_y, 800.0, 800.0) < 80.0 {
                self.gift3_done = true;
            }

            // Cookie: 100, 750 (Kitchen Table area)
            if !self.cookie_done && dist(self.player_x, self.player_y, 100.0, 750.0) < 80.0 {
                self.cookie_done = true;
            }
        }
        
        // Stars
        let mut i = 0;
        while i < self.stars.len() {
            if dist(self.player_x, self.player_y, self.stars[i].x, self.stars[i].y) < 30.0 {
                self.stars.remove(i);
                self.stars_collected += 1;
            } else {
                i += 1;
            }
        }

        // Win Condition
        if self.gift1_done && self.gift2_done && self.gift3_done && self.cookie_done && self.stars_collected >= 5 {
             self.exit_open = true;
        }

        // Exit: 410, 20
        if self.exit_open && self.player_y < 50.0 && (self.player_x - 460.0).abs() < 50.0 {
             self.state = StealthState::Win;
             self.msg = format!("TIME: {:.1}s", self.time_elapsed);
        }

        // Waves Update
        let mut i = 0;
        while i < self.waves.len() {
            self.waves[i].r += 3.0;
            self.waves[i].alpha -= 0.03;
            if self.waves[i].alpha <= 0.0 {
                self.waves.remove(i);
            } else {
                i += 1;
            }
        }

        // Camera
        self.cam_x = (self.player_x - 256.0).clamp(0.0, 920.0 - 512.0);
        self.cam_y = (self.player_y - 144.0).clamp(0.0, 900.0 - 288.0);
    }

    fn reset(&mut self) {
        *self = Self::new();
        self.state = StealthState::Playing;
    }

    pub fn draw(&self) {
        // Clear
        rect!(w=512, h=288, color=0x111111FF); // #111

        if self.state == StealthState::Menu {
            text!("SILENT SANTA", x=160, y=80, font="large", color=0xE74C3CFF);
            text!("Mission:", x=200, y=120, font="medium", color=0xFFFFFFFF);
            text!("- Place 3 Gifts (Near Dogs & Tree)", x=140, y=145, font="small", color=0xAAAAAAFF);
            text!("- Eat Cookie (Kitchen)", x=140, y=160, font="small", color=0xAAAAAAFF);
            text!("- Collect 5 Stars", x=140, y=175, font="small", color=0xAAAAAAFF);
            text!("Use ARROWS to Move. STOP to quiet down.", x=120, y=200, font="small", color=0xFF00FFFF);
            text!("Press SPACE/A to Interact.", x=160, y=215, font="small", color=0xFF00FFFF);
            text!("Press Start to Begin", x=180, y=240, font="small", color=0x888888FF);
            return;
        }

        let cx = self.cam_x as i32;
        let cy = self.cam_y as i32;

        // Draw Map (Rooms)
        let rooms = [
            ("Master Bed", 50, 50, 250, 250, 0x3E2723FF),
            ("Main Hall", 300, 50, 320, 450, 0x212121FF),
            ("Kids Bed", 620, 50, 250, 250, 0x3E2723FF),
            ("Library", 50, 350, 250, 200, 0x263238FF),
            ("Dining", 620, 350, 250, 200, 0x4E342EFF),
            ("Kitchen", 50, 600, 350, 250, 0x37474FFF),
            ("Living", 450, 550, 420, 300, 0x5D4037FF)
        ];
        
        for r in rooms {
            rect!(x=r.1 - cx, y=r.2 - cy, w=r.3, h=r.4, color=r.5);
            text!(r.0, x=r.1 - cx + 10, y=r.2 - cy + 25, font="small", color=0xFFFFFF33);
        }

        // Walls
         let walls = [
             (0, 0, 920, 20), (0, 880, 920, 20), (0, 0, 20, 900), (900, 0, 20, 900),
             (0, 300, 200, 20), (720, 300, 200, 20),
             (0, 550, 200, 20), (720, 550, 200, 20),
             (300, 0, 20, 200), (300, 300, 20, 300),
             (620, 0, 20, 200), (620, 300, 20, 250),
             (400, 600, 20, 300)
        ];
        for w in walls {
             rect!(x=w.0 - cx, y=w.1 - cy, w=w.2, h=w.3, color=0x000000FF);
        }

        // Objectives
        
        // Gift 1 (Dog 1)
        if self.gift1_done {
            text!("GIFT", x=160 - cx, y=100 - cy, font="medium", color=0x2ECC71FF);
        } else {
            // Draw a marker for where to place it
            rect!(x=150 - cx, y=150 - cy, w=50, h=50, border_size=2, border_color=0xAAAAAAFF, color=0x00000000);
            text!("PLACE", x=155 - cx, y=165 - cy, font="small", color=0xAAAAAAFF);
        }

        // Gift 2 (Dog 2)
        if self.gift2_done {
            text!("GIFT", x=745 - cx, y=400 - cy, font="medium", color=0x2ECC71FF);
        } else {
             rect!(x=720 - cx, y=420 - cy, w=50, h=50, border_size=2, border_color=0xAAAAAAFF, color=0x00000000);
             text!("PLACE", x=725 - cx, y=435 - cy, font="small", color=0xAAAAAAFF);
        }

        // Gift 3 (Tree)
        let tx = 800 - cx;
        let ty = 800 - cy;
        if self.gift3_done { text!("GIFT", x=tx, y=ty, font="medium", color=0x2ECC71FF); }
        else { text!("TREE", x=tx, y=ty, font="medium", color=0x388E3CFF); }

        // Cookie
        let kx = 100 - cx;
        let ky = 750 - cy;
        if self.cookie_done { text!("YUM", x=kx, y=ky, font="medium", color=0xF1C40FFF); }
        else { text!("COOKIE", x=kx, y=ky, font="medium", color=0xD35400FF); }

        // Stars
        for s in &self.stars {
            text!("*", x=s.x as i32 - cx, y=s.y as i32 - cy, font="large", color=0xF1C40FFF);
        }

        // Dogs
        for d in &self.dogs {
            let dx = d.x as i32 - cx;
            let dy = d.y as i32 - cy;
            // Facing? (If patrol, use dir. Else default left/right)
            let face_right = if d.is_patrol { d.patrol_dir > 0.0 } else { true };
            
            // Animation for dogs
            let anim = (self.time_elapsed * 5.0) as u32 % 2; // 0 or 1
            
            // Draw Dog/Wolf
            if d.id == 2 {
                // -- WOLF (Gray/Dark) --
                let col = 0x616161FF; 
                let eye_col = 0xFF0000FF; // Red eyes
                
                // Body
                rect!(x=dx-12, y=dy-6, w=24, h=12, color=col);
                // Legs (Animated)
                if anim == 0 {
                    rect!(x=dx-10, y=dy+6, w=4, h=8, color=col); // Back L
                    rect!(x=dx+6, y=dy+6, w=4, h=8, color=col);  // Front L
                } else {
                    rect!(x=dx-8, y=dy+6, w=4, h=8, color=col); 
                    rect!(x=dx+8, y=dy+6, w=4, h=8, color=col);
                }
                
                // Tail
                let tx = if face_right { dx-14 } else { dx+12 };
                rect!(x=tx, y=dy-4, w=4, h=6, color=col);

                // Head
                if face_right {
                    rect!(x=dx+12, y=dy-10, w=10, h=10, color=col); // Head
                    rect!(x=dx+18, y=dy-6, w=6, h=6, color=col);   // Snout
                    rect!(x=dx+16, y=dy-8, w=2, h=2, color=eye_col); // Eye
                    rect!(x=dx+14, y=dy-14, w=4, h=4, color=col);  // Ear
                } else {
                    rect!(x=dx-22, y=dy-10, w=10, h=10, color=col); // Head
                    rect!(x=dx-26, y=dy-6, w=6, h=6, color=col);   // Snout
                    rect!(x=dx-18, y=dy-8, w=2, h=2, color=eye_col); // Eye
                    rect!(x=dx-18, y=dy-14, w=4, h=4, color=col);  // Ear
                }

            } else {
                // -- DOG (Brown/Beige) --
                let col = 0x8D6E63FF;     // Brown
                let spot_col = 0x5D4037FF; // Dark Brown spots
                
                // Sleeping Pose (Lying down)
                rect!(x=dx-14, y=dy, w=28, h=12, color=col); // Main Body
                // Spot on body
                rect!(x=dx-6, y=dy+2, w=6, h=6, color=spot_col);
                
                // Legs tucked in
                rect!(x=dx-10, y=dy+12, w=8, h=3, color=col);
                rect!(x=dx+4, y=dy+12, w=8, h=3, color=col);
                
                // Head resting (Front)
                rect!(x=dx+10, y=dy-4, w=12, h=10, color=col); 
                // Ears (Floppy)
                rect!(x=dx+10, y=dy-2, w=4, h=6, color=spot_col);
                rect!(x=dx+18, y=dy-2, w=4, h=6, color=spot_col);
                // Nose
                rect!(x=dx+14, y=dy+4, w=4, h=3, color=0x212121FF);
                // Closed Eyes
                rect!(x=dx+12, y=dy, w=3, h=1, color=0x3E2723FF);
                rect!(x=dx+17, y=dy, w=3, h=1, color=0x3E2723FF);
                
                // Zzz floating
                if (self.time_elapsed * 2.0) as i32 % 2 == 0 {
                    text!("z", x=dx, y=dy-20, font="small", color=0xFFFFFF88);
                }
            }
            
            // Bar above dog
            let bar_w = 40;
            let fill = (d.alert / 100.0 * 40.0) as u32;
            rect!(x=dx-20, y=dy-30, w=bar_w, h=5, color=0x555555FF);
            let col = if d.alert > 80.0 { 0xE74C3CFF } else { 0x2ECC71FF };
            rect!(x=dx-20, y=dy-30, w=fill, h=5, color=col);
        }

        // Draw Player (Santa) - High Detail Stealth Version
        let px = self.player_x as i32 - cx;
        let py = self.player_y as i32 - cy;
        
        // let direction = 1; // Unused
        
        // -- ANIMATION --
        let walk_cycle = (self.time_elapsed * 10.0) as u32 % 4; // 0, 1, 2, 3
        let bob_y = if walk_cycle % 2 == 0 { 1 } else { 0 };

        // -- BODY --
        rect!(x=px-5, y=py-8+bob_y, w=10, h=14, color=0xD32F2FFF); 
        rect!(x=px-5, y=py+2+bob_y, w=10, h=2, color=0x212121FF); 
        rect!(x=px-2, y=py+2+bob_y, w=4, h=2, color=0xF1C40FFF); // Buckle
        rect!(x=px-6, y=py+6+bob_y, w=12, h=2, color=0xFFFFFFFF);
        rect!(x=px-1, y=py-8+bob_y, w=2, h=14, color=0xFFFFFFFF);

        // -- LEGS / BOOTS --
        rect!(x=px-4, y=py+8+bob_y, w=4, h=4, color=0xD32F2FFF); 
        rect!(x=px-4, y=py+12+bob_y, w=4, h=3, color=0x212121FF); 
        rect!(x=px+2, y=py+8+bob_y, w=4, h=4, color=0xD32F2FFF); 
        rect!(x=px+2, y=py+12+bob_y, w=4, h=3, color=0x212121FF); 

        // -- HEAD --
        let hy = py - 18 + bob_y;
        rect!(x=px-4, y=hy, w=8, h=8, color=0xFFCC80FF); 
        rect!(x=px-5, y=hy+4, w=10, h=6, color=0xFFFFFFFF);
        rect!(x=px-2, y=hy+9, w=4, h=2, color=0xFFFFFFFF); 
        
        rect!(x=px-2, y=hy+2, w=1, h=1, color=0x000000FF);
        rect!(x=px+2, y=hy+2, w=1, h=1, color=0x000000FF);
        
        rect!(x=px-5, y=hy-4, w=10, h=4, color=0xD32F2FFF); 
        rect!(x=px-4, y=hy-7, w=6, h=3, color=0xD32F2FFF); 
        rect!(x=px+1, y=hy-9, w=4, h=3, color=0xD32F2FFF); 
        rect!(x=px+5, y=hy-8, w=3, h=3, color=0xFFFFFFFF); 
        
        // -- SACK --
        rect!(x=px-9, y=py-8+bob_y, w=6, h=12, color=0x795548FF); 
        rect!(x=px-7, y=py-10+bob_y, w=4, h=2, color=0x8D6E63FF); 
        
        // -- ARMS --
        rect!(x=px-6, y=py-6+bob_y, w=3, h=6, color=0xD32F2FFF);
        rect!(x=px-6, y=py+bob_y, w=3, h=3, color=0xFFCC80FF); 
        rect!(x=px+4, y=py-6+bob_y, w=3, h=6, color=0xD32F2FFF);
        rect!(x=px+4, y=py+bob_y, w=3, h=3, color=0xFFCC80FF);

        // -- FURNITURE & DECOR --
        
        // 1. Christmas Tree (At Gift 3 Location: 800, 800)
        let tx = 800 - cx;
        let ty = 800 - cy;
        // Trunk
        rect!(x=tx-4, y=ty+20, w=8, h=10, color=0x5D4037FF);
        // Leaves (Layers)
        rect!(x=tx-20, y=ty+5, w=40, h=15, color=0x2E7D32FF); // Bottom
        rect!(x=tx-15, y=ty-5, w=30, h=15, color=0x388E3CFF); // Mid
        rect!(x=tx-10, y=ty-15, w=20, h=15, color=0x4CAF50FF); // Top
        // Ornaments (Blinking?)
        let blink = (self.time_elapsed * 5.0) as u32 % 2 == 0;
        if blink {
             rect!(x=tx-8, y=ty, w=3, h=3, color=0xF1C40FFF);
             rect!(x=tx+5, y=ty+8, w=3, h=3, color=0xE91E63FF);
             rect!(x=tx, y=ty-10, w=3, h=3, color=0x03A9F4FF);
        }
        // Star on Top
        rect!(x=tx-3, y=ty-20, w=6, h=6, color=0xFFD700FF);
        
        // 2. Kitchen Table (Near Cookie: 100, 750)
        let kx = 100 - cx;
        let ky = 750 - cy;
        // Table Top
        rect!(x=kx-30, y=ky, w=60, h=30, color=0x8D6E63FF); // Brown
        rect!(x=kx-25, y=ky+5, w=50, h=20, color=0xA1887FFF); // Light Brown inlay
        // Legs
        rect!(x=kx-25, y=ky+30, w=5, h=15, color=0x5D4037FF);
        rect!(x=kx+20, y=ky+30, w=5, h=15, color=0x5D4037FF);
        
        // Cookie Plate
        circ!(x=kx, y=ky+15, d=10, color=0xFFFFFFFF); // Plate
        if !self.cookie_done {
            circ!(x=kx+1, y=ky+16, d=6, color=0xD35400FF); // Cookie
            rect!(x=kx+2, y=ky+17, w=1, h=1, color=0x3E2723FF); // Chip
            rect!(x=kx+4, y=ky+16, w=1, h=1, color=0x3E2723FF); // Chip
        }

        // 3. Simple Bed (Master Bed: 175, 120 approx)
        let bx = 150 - cx;
        let by_pos = 100 - cy;
        rect!(x=bx, y=by_pos, w=80, h=100, color=0x3F51B5FF); // Blanket
        rect!(x=bx, y=by_pos, w=80, h=30, color=0xFFFFFFFF); // Pillow area
        rect!(x=bx+10, y=by_pos+5, w=25, h=20, color=0xEEEEEEFF); // Pillow 1
        rect!(x=bx+45, y=by_pos+5, w=25, h=20, color=0xEEEEEEFF); // Pillow 2


        // Waves
        for w in &self.waves {
            let wx = w.x as i32 - cx;
            let wy = w.y as i32 - cy;
             let c = 0xFFFFFF00 | (w.alpha * 255.0) as u32;
             circ!(x=wx - w.r as i32, y=wy - w.r as i32, d=(w.r*2.0) as u32, color=c);
        }

        // HUD
        // Timer
        let time_txt = format!("Time: {:.1}", self.time_elapsed);
        text!(&time_txt, x=10, y=10, font="small", color=0xFFFFFFFF);
        
        // Stars
        let star_txt = format!("Stars: {}/5", self.stars_collected);
        text!(&star_txt, x=10, y=25, font="small", color=0xF1C40FFF);

        // Gifts Status
        let g1 = if self.gift1_done { "OK" } else { ".." };
        let g2 = if self.gift2_done { "OK" } else { ".." };
        let g3 = if self.gift3_done { "OK" } else { ".." };
        let ck = if self.cookie_done { "OK" } else { ".." };
        let status = format!("G1:{} G2:{} G3:{} C:{}", g1, g2, g3, ck);
        text!(&status, x=10, y=40, font="small", color=0xAAAAAAFF);

        // HUD Dog Bars (Fixed Right Side)
        text!("Dog 1", x=400, y=30, font="small", color=0xAAAAAAFF);
        rect!(x=440, y=32, w=60, h=6, color=0x444444FF);
        let fill1 = (self.dogs[0].alert / 100.0 * 60.0) as u32;
        let col1 = if self.dogs[0].alert > 80.0 { 0xE74C3CFF } else { 0x2ECC71FF };
        rect!(x=440, y=32, w=fill1, h=6, color=col1);

        text!("Dog 2", x=400, y=45, font="small", color=0xAAAAAAFF);
        rect!(x=440, y=47, w=60, h=6, color=0x444444FF);
        let fill2 = (self.dogs[1].alert / 100.0 * 60.0) as u32;
        let col2 = if self.dogs[1].alert > 80.0 { 0xE74C3CFF } else { 0x2ECC71FF };
        rect!(x=440, y=47, w=fill2, h=6, color=col2);

        text!("Wolf", x=400, y=60, font="small", color=0xAAAAAAFF);
        rect!(x=440, y=62, w=60, h=6, color=0x444444FF);
        let fill3 = (self.dogs[2].alert / 100.0 * 60.0) as u32;
        let col3 = if self.dogs[2].alert > 80.0 { 0xE74C3CFF } else { 0x2ECC71FF };
        rect!(x=440, y=62, w=fill3, h=6, color=col3);

        // Mini Map (Bottom Right)
        let mm_w = 120;
        let mm_h = 120;
        let mm_x = 380;
        let mm_y = 160;
        rect!(x=mm_x, y=mm_y, w=mm_w, h=mm_h, color=0x111111EE, border_radius=4);
        rect!(x=mm_x, y=mm_y, w=mm_w, h=mm_h, border_size=1, border_color=0x444444FF, color=0x00000000);
        
        // Scale factor: 920x900 -> 120x120 roughly 0.13
        let scale = 0.13;
        for r in rooms {
            rect!(x=mm_x + (r.1 as f32 * scale) as i32, 
                  y=mm_y + (r.2 as f32 * scale) as i32, 
                  w=(r.3 as f32 * scale) as u32, 
                  h=(r.4 as f32 * scale) as u32, 
                  color=0x444444FF);
        }
        // Player Dot
        rect!(x=mm_x + (self.player_x * scale) as i32, y=mm_y + (self.player_y * scale) as i32, w=2, h=2, color=0xFF0000FF);
        
        // Targets
        if !self.gift1_done { rect!(x=mm_x + (175.0 * scale) as i32, y=mm_y + (175.0 * scale) as i32, w=2, h=2, color=0x00FF00FF); }
        if !self.gift2_done { rect!(x=mm_x + (745.0 * scale) as i32, y=mm_y + (450.0 * scale) as i32, w=2, h=2, color=0x00FF00FF); }
        if !self.gift3_done { rect!(x=mm_x + (800.0 * scale) as i32, y=mm_y + (800.0 * scale) as i32, w=2, h=2, color=0x00FF00FF); }
        
        // Exit on Map (Top Center)
        let exit_col = if self.exit_open { 0x00FF00FF } else { 0x555555FF };
        rect!(x=mm_x + (460.0 * scale) as i32 - 2, y=mm_y + (20.0 * scale) as i32, w=4, h=2, color=exit_col);


        if self.exit_open {
             text!("EXIT OPEN!", x=200, y=10, font="medium", color=0x2ECC71FF);
        }

        // Game Over Overlay
        if self.state == StealthState::GameOver {
            rect!(x=100, y=100, w=312, h=100, color=0x000000EE);
            text!("CAUGHT!", x=200, y=120, font="large", color=0xE74C3CFF);
            text!(&self.msg, x=150, y=150, font="small", color=0xFFFFFFFF);
            text!("Press Start", x=200, y=180, font="small", color=0xAAAAAAFF);
        } else if self.state == StealthState::Win {
            rect!(x=100, y=100, w=312, h=100, color=0x000000EE);
            text!("MISSION COMPLETE", x=160, y=120, font="large", color=0x2ECC71FF);
             text!(&self.msg, x=190, y=150, font="small", color=0xFFFFFFFF);
            text!("Press Start", x=200, y=180, font="small", color=0xAAAAAAFF);
        }
    }
}

fn rect_circle_hit(rx: f32, ry: f32, rw: f32, rh: f32, cx: f32, cy: f32, cr: f32) -> bool {
    let tx = cx.clamp(rx, rx + rw);
    let ty = cy.clamp(ry, ry + rh);
    let dx = cx - tx;
    let dy = cy - ty;
    (dx*dx + dy*dy) < (cr*cr)
}

fn dist(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

fn rand() -> u32 {
    unsafe {
        static mut SEED: u32 = 77777;
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}
