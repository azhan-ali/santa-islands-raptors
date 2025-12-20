use turbo::*;


#[turbo::serialize]
#[derive(PartialEq)]
pub struct MPlayer {
    pub x: f32,
    pub y: f32,
    pub color: u32,
    pub id: u8,
    pub score: u32,
    pub radius: f32,
    pub boost_timer: u32,
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct House {
    pub x: f32,
    pub y: f32,
    pub points: u32,
    pub cooldown: u32,
    pub last_collected_by: Option<u8>,
    pub last_collection_time: u32, // in ticks
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct PowerUp {
    pub x: f32,
    pub y: f32,
    pub kind: u8, // 0 = Gift, 1 = Speed
    pub collected: bool,
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct MultiplayerGame {
    pub players: Vec<MPlayer>,
    pub houses: Vec<House>,
    pub obstacles: Vec<Obstacle>,
    pub powerups: Vec<PowerUp>,
    pub timer: u32, // in seconds (approx)
    pub game_over: bool,
    pub winner_text: String,
    pub frame_count: u32,
    pub next_level_timer: u32,
}

impl MultiplayerGame {
    pub fn new() -> Self {
        let mut game = Self {
            players: vec![],
            houses: vec![],
            obstacles: vec![],
            powerups: vec![],
            timer: 180, // 3 mins
            game_over: false,
            winner_text: "".to_string(),
            frame_count: 0,
            next_level_timer: 0,
        };
        game.init_level(1);
        game
    }

    fn init_level(&mut self, _level: u32) {
        // Reset players
        self.players = vec![
            MPlayer {
                x: 50.0,
                y: 50.0,
                color: 0xFF0000FF, // Red (Santa)
                id: 1,
                score: 0,
                radius: 8.0,
                boost_timer: 0,
            },
            MPlayer {
                x: 462.0,
                y: 238.0,
                color: 0x0000FFFF, // Blue (Rival)
                id: 2,
                score: 0,
                radius: 8.0,
                boost_timer: 0,
            },
        ];

        // Random Houses
        self.houses = vec![];
        let mut rng = random::u32();
        for _ in 0..10 {
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hx = 50.0 + (rng % 400) as f32;
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hy = 50.0 + (rng % 180) as f32;
            
            self.houses.push(House {
                x: hx,
                y: hy,
                points: 10,
                cooldown: 0,
                last_collected_by: None,
                last_collection_time: 0,
            });
        }
        
        // Timer reset
        self.timer = 180;
        self.game_over = false;
    }

    pub fn update(&mut self) {
        if self.game_over {
            // Wait for input to restart
            if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() {
                 self.init_level(1);
            }
            return;
        }

        self.frame_count += 1;
        if self.frame_count % 60 == 0 && self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.end_game();
        }

        // Spawn Powerups randomly
        if random::u32() % 500 == 0 {
             let px = 20.0 + (random::u32() % 470) as f32;
             let py = 20.0 + (random::u32() % 240) as f32;
             self.powerups.push(PowerUp {
                 x: px,
                 y: py,
                 kind: (random::u32() % 2) as u8,
                 collected: false,
             });
        }

        // Players Update
        for i in 0..self.players.len() {
            let (dx, dy) = self.get_input(i, self.players[i].id);
            let speed = if self.players[i].boost_timer > 0 { 3.0 } else { 2.0 };
            
            if self.players[i].boost_timer > 0 {
                self.players[i].boost_timer -= 1;
            }

            self.players[i].x += dx * speed;
            self.players[i].y += dy * speed;

            // Bounds
            let r = self.players[i].radius;
            self.players[i].x = self.players[i].x.clamp(r, 512.0 - r);
            self.players[i].y = self.players[i].y.clamp(r, 288.0 - r);
        }

        // House Interaction
        let current_tick = self.frame_count; // Approximation
        for house in self.houses.iter_mut() {
            if house.cooldown > 0 {
                house.cooldown -= 1;
            }
            
            for player in self.players.iter_mut() {
                let dist = ((player.x - house.x).powi(2) + (player.y - house.y).powi(2)).sqrt();
                if dist < (player.radius + 12.0) {
                    // Collection logic
                    if house.cooldown == 0 {
                        player.score += house.points;
                        house.cooldown = 300; // 5 seconds at 60fps
                        house.last_collected_by = Some(player.id);
                        house.last_collection_time = current_tick;
                        house.points = 10; // Reset points
                    } 
                    // Steal logic (Window 3s = 180 ticks)
                    else if house.cooldown > 0 {
                        if let Some(last_id) = house.last_collected_by {
                            if last_id != player.id && (current_tick - house.last_collection_time) < 180 {
                                // Steal!
                                house.last_collected_by = Some(player.id); // Prevent infinite steal loop
                                house.last_collection_time = 0; // Close window
                                player.score += 20; // Steal bonus
                                // In real implementation we'd deduct from other player, but accessing them is hard here due to borrow check if we iter mut.
                                // Simplification: Just bonus for now.
                            }
                        }
                    }
                }
            }
        }
        
        // Powerup Interaction
         for pu in self.powerups.iter_mut() {
             if pu.collected { continue; }
             for player in self.players.iter_mut() {
                let dist = ((player.x - pu.x).powi(2) + (player.y - pu.y).powi(2)).sqrt();
                if dist < (player.radius + 8.0) {
                    pu.collected = true;
                    if pu.kind == 0 { // Gift
                        player.score += 50;
                    } else { // Speed
                        player.boost_timer = 300; // 5 secs
                    }
                }
             }
         }
         self.powerups.retain(|p| !p.collected);
    }

    fn get_input(&self, index: usize, _id: u8) -> (f32, f32) {
        let gp = gamepad::get(index);
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        
        if gp.up.pressed() { dy -= 1.0; }
        if gp.down.pressed() { dy += 1.0; }
        if gp.left.pressed() { dx -= 1.0; }
        if gp.right.pressed() { dx += 1.0; }
        
        // Normalize
        if dx != 0.0 || dy != 0.0 {
            let mag = (dx*dx + dy*dy).sqrt();
            dx /= mag;
            dy /= mag;
        }
        (dx, dy)
    }
    
    fn end_game(&mut self) {
        self.game_over = true;
        let s1 = self.players[0].score;
        let s2 = self.players[1].score;
        if s1 > s2 {
            self.winner_text = "SANTA WINS!".to_string();
        } else if s2 > s1 {
            self.winner_text = "RIVAL WINS!".to_string();
        } else {
            self.winner_text = "DRAW!".to_string();
        }
    }

    pub fn draw(&self) {
        // BG
        rect!(w=512, h=288, color=0x276749FF); // Grass Green

        // Grid (Optional)
        for i in (0..512).step_by(50) {
             rect!(x=i, y=0, w=1, h=288, color=0xFFFFFF10);
        }

        // Houses
        for h in &self.houses {
            let color = if h.cooldown > 0 { 0x555555FF } else { 0x8B4513FF };
            let size = 24;
            rect!(x = h.x as i32 - size/2, y = h.y as i32 - size/2, w=size as u32, h=size as u32, color = color);
            
            // Roof
            // Turbo doesn't have easy triangle, use small rect on top
            rect!(x = h.x as i32 - size/2 + 2, y = h.y as i32 - size/2 - 6, w=size as u32 - 4, h=6, color = 0xA52A2AFF);

            if h.cooldown == 0 {
                 text!("+", x = h.x as i32 - 4, y = h.y as i32 - 30, color = 0xFFFFFFFF);
            }
        }
        
        // Powerups
        for pu in &self.powerups {
             let color = if pu.kind == 0 { 0xFF00FFFF } else { 0xFFFF00FF }; // Purple (Gift) or Yellow (Speed)
             circ!(d=10, x=pu.x as i32, y=pu.y as i32, color = color);
             if pu.kind == 0 { text!("?", x=pu.x as i32 - 3, y=pu.y as i32 - 4, font="small", color=0xFFFFFFFF); }
        }

        // Players
        for p in &self.players {
            circ!(d=16, x=p.x as i32, y=p.y as i32, color=p.color);
            // Label
            // text!(if p.id==1{"P1"}else{"P2"}, x=p.x as i32 - 6, y=p.y as i32 - 20, font="small", color=0xFFFFFFFF);
        }

        // HUD
        let p1_text = format!("P1: {}", self.players[0].score);
        text!(&p1_text, x=10, y=10, color=0xFF5555FF);
        
        let p2_text = format!("P2: {}", self.players[1].score);
        text!(&p2_text, x=450, y=10, color=0x5555FFFF); // Blue
        
        let mins = self.timer / 60;
        let secs = self.timer % 60;
        let time_text = format!("{:02}:{:02}", mins, secs);
        text!(&time_text, x=240, y=10, color=0xFFCC00FF);
        
        if self.game_over {
             // Overlay
             rect!(w=512, h=288, color=0x000000AA);
             text!(&self.winner_text, x=200, y=130, font="large", color=0xFFFFFFFF);
             text!("Press START to Restart", x=180, y=160, color=0xAAAAAAFF);
             text!("Press X to Exit", x=200, y=180, color=0xAAAAAAFF);
        }
    }
}
