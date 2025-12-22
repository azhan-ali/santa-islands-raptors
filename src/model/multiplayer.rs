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
    pub invuln_timer: u32,
    pub name: String,
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
    pub respawn_timer: u32,
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
pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: u32,
    pub life: u32,
}

#[turbo::serialize]
#[derive(PartialEq, Copy)]
pub struct Decor {
    pub x: f32,
    pub y: f32,
    pub kind: u8, // 0 = Tree, 1 = SnowPile
}

#[turbo::serialize]
#[derive(PartialEq, Copy)]
pub struct EnvSnow {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub size: u32,
}

#[turbo::serialize]
#[derive(PartialEq, Copy)]
pub struct MParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: u32,
    pub color: u32,
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct MultiplayerGame {
    pub players: Vec<MPlayer>,
    pub houses: Vec<House>,
    pub obstacles: Vec<Obstacle>,
    pub powerups: Vec<PowerUp>,
    pub particles: Vec<MParticle>,
    pub env_snow: Vec<EnvSnow>,
    pub decors: Vec<Decor>,
    pub floating_texts: Vec<FloatingText>, // Added
    pub timer: u32,
    pub game_over: bool,
    pub winner_text: String,
    pub frame_count: u32,
    pub next_level_timer: u32,
    pub last_pickup_pos: (f32, f32), // Track last pickup to avoid respawn nearby
    pub p1_name: String,
    pub p2_name: String,
    pub max_time_minutes: u32,
    pub current_level: u32,
    // Level 2 Specifics
    pub shuffle_timer: u32,
    pub is_shuffling: bool,
    pub shuffle_pause_timer: u32,
}

impl MultiplayerGame {
    pub fn new(p1: String, p2: String, minutes: u32, level: u32) -> Self {
        let mut game = Self {
            players: vec![],
            houses: vec![],
            obstacles: vec![],
            powerups: vec![],
            particles: vec![],
            env_snow: (0..60).map(|_| EnvSnow {
                x: (random::u32() % 512) as f32,
                y: (random::u32() % 288) as f32,
                speed: (random::u32() % 10) as f32 / 20.0 + 0.5, // Faster on bright bg
                size: (random::u32() % 2) + 2,
            }).collect(),
            decors: vec![], // Init in init_level
            floating_texts: vec![],
            timer: minutes * 60,
            game_over: false,
            winner_text: "".to_string(),
            frame_count: 0,
            next_level_timer: 0,
            last_pickup_pos: (0.0, 0.0),
            p1_name: p1,
            p2_name: p2,
            max_time_minutes: minutes,
            current_level: level,
            shuffle_timer: 0,
            is_shuffling: false,
            shuffle_pause_timer: 0,
        };
        game.init_level(level);
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
                invuln_timer: 0,
                name: self.p1_name.clone(),
            },
            MPlayer {
                x: 462.0,
                y: 238.0,
                color: 0x0000FFFF, // Blue (Rival)
                id: 2,
                score: 0,
                radius: 8.0,
                boost_timer: 0,
                invuln_timer: 0,
                name: self.p2_name.clone(),
            },
        ];

        // Level Configuration
        let house_count = if _level == 2 { 14 } else { 10 };
        
        // Random Houses
        self.generate_houses(house_count);
        
        // Level 2: Obstacles
        self.obstacles = vec![];
        if _level == 2 {
            let mut rng = random::u32();
            let mut attempts = 0;
            while self.obstacles.len() < 8 && attempts < 100 {
                attempts += 1;
                rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                let ox = 60.0 + (rng % 392) as f32; 
                rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                let oy = 60.0 + (rng % 168) as f32;
                
                // Avoid overlap with houses
                let mut safe = true;
                for h in &self.houses {
                   if ((h.x - ox).powi(2) + (h.y - oy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                }
                
                // Avoid overlap with existing obstacles
                if safe {
                    for o in &self.obstacles {
                        if ((o.x - ox).powi(2) + (o.y - oy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                    }
                }
                
                if safe {
                    self.obstacles.push(Obstacle { x: ox, y: oy, w: 24.0, h: 24.0, respawn_timer: 0 });
                }
            }
        }
        
        // Level 2: Dynamic Shuffle Init
        if _level == 2 {
            self.shuffle_timer = 15 * 60; // 15 seconds
        } else {
            self.shuffle_timer = 0;
        }
        self.is_shuffling = false;
        self.shuffle_pause_timer = 0;

        // Generate Decor (Trees and Piles)
        self.decors = vec![];
        for _ in 0..20 {
            // Random Trees
            self.decors.push(Decor {
                x: (random::u32() % 500 + 10) as f32,
                y: (random::u32() % 260 + 10) as f32,
                kind: 0,
            });
            // Random Piles
            if random::u32() % 2 == 0 {
                self.decors.push(Decor {
                    x: (random::u32() % 500 + 10) as f32,
                    y: (random::u32() % 260 + 10) as f32,
                    kind: 1,
                });
            }
        }
        
        // Reset particles and texts
        self.particles = vec![];
        self.floating_texts = vec![];
        
        // Timer reset
        self.timer = self.max_time_minutes * 60;
        self.game_over = false;
    }

    pub fn update(&mut self) {
        if self.game_over {
            // Wait for input to restart
            if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() {
                 self.init_level(self.current_level);
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

        // Spawn Powerups
        let mut kinds_to_spawn = vec![];
        
        // Gift (0): Every 6s (360 frames)
        if self.frame_count % 360 == 0 { kinds_to_spawn.push(0); }
        
        // Speed (1): Every 10s (600 frames)
        if self.frame_count % 600 == 0 { kinds_to_spawn.push(1); }

        for kind in kinds_to_spawn {
             // Check if already exists
             let exists = self.powerups.iter().any(|p| p.kind == kind);
             
             if !exists {
                 let mut px = 0.0;
                 let mut py = 0.0;
                 let mut safe = false;
                 let mut attempts = 0;
                 
                 while !safe && attempts < 15 {
                     attempts += 1;
                     px = 40.0 + (random::u32() % 432) as f32;
                     py = 40.0 + (random::u32() % 208) as f32;
                     
                     safe = true;
                     
                     // 1. Not on houses
                     for h in &self.houses {
                         let d = ((h.x - px).powi(2) + (h.y - py).powi(2)).sqrt();
                         if d < 45.0 { safe = false; break; }
                     }
                     
                     // 2. Not near players
                     if safe {
                         for p in &self.players {
                             let d = ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt();
                             if d < 60.0 { safe = false; break; }
                         }
                     }
                     
                     // 3. Not near last pickup (avoid camping)
                     if safe {
                         let (lx, ly) = self.last_pickup_pos;
                         let d = ((lx - px).powi(2) + (ly - py).powi(2)).sqrt();
                         if d < 80.0 { safe = false; }
                     }
                 }
                 
                 if safe {
                     self.powerups.push(PowerUp {
                         x: px,
                         y: py,
                         kind,
                         collected: false,
                     });
                 }
             }
        }

        // Players Update
        // Freeze movement if shuffling (Level 2 pause)
        if !self.is_shuffling {
            for i in 0..self.players.len() {
                let (dx, dy) = self.get_input(i, self.players[i].id);
                let speed = if self.players[i].boost_timer > 0 { 3.0 } else { 2.0 };
                
                if self.players[i].boost_timer > 0 {
                    self.players[i].boost_timer -= 1;
                }
                if self.players[i].invuln_timer > 0 {
                    self.players[i].invuln_timer -= 1;
                }

                self.players[i].x += dx * speed;
                self.players[i].y += dy * speed;

                // Bounds
                let r = self.players[i].radius;
                self.players[i].x = self.players[i].x.clamp(r, 512.0 - r);
                self.players[i].y = self.players[i].y.clamp(r, 288.0 - r);
            }
        } else {
             // Still notify invuln timer tick if frozen? Usually yes.
             for p in self.players.iter_mut() {
                 if p.invuln_timer > 0 { p.invuln_timer -= 1; }
             }
        }

        // House Interaction
        let mut sparkle_reqs = vec![]; // Defer spawning
        let current_tick = self.frame_count;
        // House Interaction logic merged into the loop below to prevent double updates

        
        // Re-write interaction loop to handle defer properly
        // We need score pops.
        let mut score_pops = vec![];

        for house in self.houses.iter_mut() {
            if house.cooldown > 0 { house.cooldown -= 1; }
            else if self.frame_count % 120 == 0 && house.points < 50 { house.points += 5; } // Charge

            for player in self.players.iter_mut() {
                 let dist = ((player.x - house.x).powi(2) + (player.y - house.y).powi(2)).sqrt();
                 if dist < (player.radius + 12.0) && house.cooldown == 0 {
                        player.score += house.points;
                        
                        // Queue Pop
                        score_pops.push(FloatingText {
                            x: house.x,
                            y: house.y - 20.0,
                            text: format!("+{}", house.points),
                            color: 0xE0F7FAFF, // Icy White
                            life: 60,
                        });

                        house.cooldown = 300; 
                        house.last_collected_by = Some(player.id);
                        house.last_collection_time = current_tick;
                        house.points = 5; // Reset to 5
                        
                        turbo::audio::play("coin");
                        sparkle_reqs.push((house.x, house.y));
                 }
            }
        }

        // Apply pops
        for pop in score_pops {
            self.floating_texts.push(pop);
        }
        
        // Update Floating Texts
        for t in self.floating_texts.iter_mut() {
            t.y -= 1.0; // Float up
            if t.life > 0 { t.life -= 1; }
        }
        self.floating_texts.retain(|t| t.life > 0);
        
        
        // Powerup Interaction
         for pu in self.powerups.iter_mut() {
             if pu.collected { continue; }
             for player in self.players.iter_mut() {
                let dist = ((player.x - pu.x).powi(2) + (player.y - pu.y).powi(2)).sqrt();
                if dist < (player.radius + 15.0) {
                     pu.collected = true;
                     self.last_pickup_pos = (pu.x, pu.y); // Record location
                     
                     if pu.kind == 0 {
                         // Gift: Points
                         player.score += 50;
                         // Text Pop
                         self.floating_texts.push(FloatingText {
                            x: pu.x,
                            y: pu.y - 10.0,
                            text: "+50".to_string(),
                            color: 0xFFD700FF,
                            life: 60,
                        });
                     } else {
                         // Bolt: Speed
                         player.boost_timer = 300; // 5s
                         // Text Pop
                         self.floating_texts.push(FloatingText {
                            x: pu.x,
                            y: pu.y - 10.0,
                            text: "SPEED!".to_string(),
                            color: 0x00E5FFFF,
                            life: 60,
                        });
                     }
                     sparkle_reqs.push((pu.x, pu.y));
                     turbo::audio::play("coin");
                }
             }
         }

        // Process deferred sparkles (Houses & Powerups)
        for (sx, sy) in sparkle_reqs {
            self.spawn_sparkles(sx, sy);
        }

        // Update Particles
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            if p.life > 0 { p.life -= 1; }
        }
        self.particles.retain(|p| p.life > 0);
        
        // Update Env Snow
        for s in self.env_snow.iter_mut() {
            s.y += s.speed;
            if s.y > 288.0 {
                s.y = -5.0;
                s.x = (random::u32() % 512) as f32;
            }
        }

         self.powerups.retain(|p| !p.collected);
         
         // Level 2 Mechanics
         if self.current_level == 2 {
             self.update_level2();
         }
    }
    
    fn generate_houses(&mut self, count: usize) {
        self.houses = vec![];
        let mut rng = random::u32();
        let mut attempts = 0;
        
        while self.houses.len() < count && attempts < 200 {
            attempts += 1;
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hx = 40.0 + (rng % 432) as f32; // Inset from edges
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hy = 40.0 + (rng % 180) as f32; // Keep somewhat central vertically
            
            // Check overlap with existing houses
            let mut overlap = false;
            for h in &self.houses {
                let d = ((h.x - hx).powi(2) + (h.y - hy).powi(2)).sqrt();
                if d < 60.0 { 
                    overlap = true;
                    break;
                }
            }
            
            // Check overlap with obstacles (if they exist yet, though usually houses gen first)
            if !overlap {
                for o in &self.obstacles {
                     let d = ((o.x - hx).powi(2) + (o.y - hy).powi(2)).sqrt();
                     if d < 60.0 { overlap = true; break; }
                }
            }
            
            if !overlap {
                self.houses.push(House {
                    x: hx,
                    y: hy,
                    points: 5,
                    cooldown: 0,
                    last_collected_by: None,
                    last_collection_time: 0,
                });
            }
        }
    }
    
    fn update_level2(&mut self) {
        // Shuffle Logic
        if self.is_shuffling {
            if self.shuffle_pause_timer > 0 {
                self.shuffle_pause_timer -= 1;
            } else {
                // Time to shuffle!
                self.generate_houses(14);
                self.is_shuffling = false;
                self.shuffle_timer = 15 * 60;
                
                // Alert Text
                self.floating_texts.push(FloatingText {
                    x: 256.0,
                    y: 144.0,
                    text: "HOUSES MOVED!".to_string(),
                    color: 0xFFFF00FF,
                    life: 120,
                });
            }
        } else {
            if self.shuffle_timer > 0 {
                self.shuffle_timer -= 1;
                if self.shuffle_timer == 60 { // 1 sec warning
                     self.floating_texts.push(FloatingText {
                        x: 256.0,
                        y: 144.0,
                        text: "SHUFFLING SOON...".to_string(),
                        color: 0xFFA500FF,
                        life: 60,
                    });
                }
            } else {
                self.is_shuffling = true;
                self.shuffle_pause_timer = 60; // 1 sec pause
            }
        }
        
        // Obstacle Collision & Respawn
        let mut penalties = vec![];
        let mut explosions = vec![];
        
        // Snapshot existing positions to check for overlaps during respawn
        let obstacle_positions: Vec<(f32, f32)> = self.obstacles.iter().map(|o| (o.x, o.y)).collect();
        
        // Handle Respawning Obstacles
        for (i, o) in self.obstacles.iter_mut().enumerate() {
            if o.respawn_timer > 0 {
                o.respawn_timer -= 1;
                if o.respawn_timer == 0 {
                     // Respawn Logic
                     let mut rng = random::u32();
                     let mut attempts = 0;
                     let mut placed = false;
                     while !placed && attempts < 50 {
                         attempts += 1;
                         rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                         let ox = 60.0 + (rng % 392) as f32; 
                         rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                         let oy = 60.0 + (rng % 168) as f32;
                         
                         let mut safe = true;
                         // Check houses
                         for h in &self.houses {
                            if ((h.x - ox).powi(2) + (h.y - oy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                         }
                         // Check players (don't spawn on top)
                         if safe {
                             for p in &self.players {
                                 if ((p.x - ox).powi(2) + (p.y - oy).powi(2)).sqrt() < 60.0 { safe = false; break; }
                             }
                         }
                         // Check other obstacles
                         if safe {
                             for (idx, (ex, ey)) in obstacle_positions.iter().enumerate() {
                                 if i != idx { // Don't check against self (old pos)
                                     if ((ex - ox).powi(2) + (ey - oy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                                 }
                             }
                         }
                         
                         if safe {
                             o.x = ox;
                             o.y = oy;
                             placed = true;
                         }
                     }
                }
                continue; // Don't collide if respawning
            }
            
            // Collision Check
            for p in self.players.iter_mut() {
                if p.invuln_timer > 0 { continue; }

                // Circle collision for Bomb (radius vs radius)
                // Bomb radius approx w/2 = 12
                let bomb_radius = 12.0;
                let dx = p.x - (o.x + o.w/2.0);
                let dy = p.y - (o.y + o.h/2.0);
                let dist = (dx*dx + dy*dy).sqrt();

                if dist < (p.radius + bomb_radius) {
                    // Hit!
                    if p.score >= 20 { p.score -= 20; } else { p.score = 0; }
                    p.invuln_timer = 60; // 1s invuln
                    
                    penalties.push((p.x, p.y));
                    explosions.push((o.x + o.w/2.0, o.y + o.h/2.0));
                    
                    // Trigger Respawn
                    o.respawn_timer = 30; // 0.5s delay
                    o.x = -1000.0; // Hide
                }
            }
        }
        
        // Apply penalties (Sound & Text)
        for (px, py) in penalties {
            self.floating_texts.push(FloatingText {
                x: px,
                y: py - 20.0,
                text: "-20".to_string(),
                color: 0xFF0000FF, // Red
                life: 60,
            });
        }
        
        // Apply Explosions
        for (ex, ey) in explosions {
             turbo::audio::play("projectile_hit"); 
             self.spawn_explosion(ex, ey);
        }
    }

    fn spawn_explosion(&mut self, x: f32, y: f32) {
        for _ in 0..12 {
            let angle = (random::u32() % 360) as f32 * 3.14 / 180.0;
            let speed = (random::u32() % 30) as f32 / 10.0 + 2.0;
            self.particles.push(MParticle {
                x, y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 20 + (random::u32() % 15),
                color: 0xFF5722FF, // Orange/Red Boom
            });
        }
    }

    fn spawn_sparkles(&mut self, x: f32, y: f32) {
        for _ in 0..8 {
            let angle = (random::u32() % 360) as f32 * 3.14 / 180.0;
            let speed = (random::u32() % 20) as f32 / 10.0 + 1.0;
            self.particles.push(MParticle {
                x, y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 30 + (random::u32() % 20),
                color: 0xFFFF00FF, // Yellow sparkles
            });
        }
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
        // 1. Background (Light Green Winter - Mint/Pastel)
        rect!(w=512, h=288, color=0xC8E6C9FF); 

        // 2. Decor (Trees & Piles) - Draw BEFORE Houses/Players
        for d in &self.decors {
            let dx = d.x as i32;
            let dy = d.y as i32;
            if d.kind == 0 { // Small Festive Tree
                 // Trunk
                 rect!(x=dx-1, y=dy+4, w=2, h=4, color=0x5D4037FF); 
                 
                 // Bottom Layer
                 rect!(x=dx-5, y=dy, w=10, h=4, color=0x2E7D32FF);
                 rect!(x=dx-6, y=dy+1, w=1, h=2, color=0xFFFFFFFF); // Snow tip L
                 rect!(x=dx+5, y=dy+1, w=1, h=2, color=0xFFFFFFFF); // Snow tip R

                 // Mid Layer
                 rect!(x=dx-4, y=dy-3, w=8, h=3, color=0x2E7D32FF);
                 rect!(x=dx-5, y=dy-2, w=1, h=2, color=0xFFFFFFFF); // Snow tip L
                 rect!(x=dx+4, y=dy-2, w=1, h=2, color=0xFFFFFFFF); // Snow tip R

                 // Top Layer
                 rect!(x=dx-2, y=dy-6, w=4, h=3, color=0x2E7D32FF);
                 
                 // Star / Top
                 rect!(x=dx-1, y=dy-7, w=2, h=2, color=0xFFD700FF); // Gold Tip

                 // Ornaments (Red/Gold dots)
                 if (dx + dy) % 3 == 0 {
                    rect!(x=dx-2, y=dy+2, w=1, h=1, color=0xD32F2FFF); // Red
                    rect!(x=dx+1, y=dy-1, w=1, h=1, color=0xFFD700FF); // Gold
                 } else {
                    rect!(x=dx+2, y=dy+2, w=1, h=1, color=0xD32F2FFF);
                    rect!(x=dx-1, y=dy-2, w=1, h=1, color=0xFFFFFFFF); // White bulb
                 }

            } else { // Snow Pile
                 rect!(x=dx, y=dy, w=12, h=8, color=0xFFFFFFFF);
                 rect!(x=dx+2, y=dy+2, w=8, h=4, color=0xEEEEEEFF); // Shading
            }
        }

        // 3. Environmental Snow (White for visibility on Green)
        for s in &self.env_snow {
            let col = 0xFFFFFFAA; // White Translucent
            rect!(x=s.x as i32, y=s.y as i32, w=s.size, h=s.size, color=col);
        }

        // Houses
        for h in &self.houses {
            let x = h.x as i32;
            let y = h.y as i32;
            let active = h.cooldown == 0;
            
            // Draw Procedural House
            // ... (Code same as before, omitted changes)
            
            // Draw Glow if active (Soft blue glow instead of pulsing yellow)
            if active {
                 circ!(x=x, y=y, d=32, color=0x00E5FF22); // Static soft glow
            }
            
            // ... (Procedural House Drawing Block - Same as existing, assume preserved by context match or manual re-insertion if needed. Wait, ReplaceFileContent replaces everything in range. I need to include the House Draw logic or use a generic "Draw House" comment if I don't want to rewrite it.
            // Actually, I should rewrite the House Draw logic to be safe, or select a smaller chunk range.
            // The previous chunk was huge. Let's target the "Floating Points" section specifically?)
            
            // RE-INSERTING PROCEDURAL HOUSE LOGIC (Compressed for tool limit):
            let w = 24; let h_body = 18;
            let wall_color = if active { 0xB22222FF } else { 0x444444FF };
            let roof_color = if active { 0xFFFFFFFF } else { 0x777777FF };
            let door_color = if active { 0x5D4037FF } else { 0x222222FF };
            let win_color = if active { 0xFFD700FF } else { 0x333300FF };
            rect!(x=x-w/2, y=y-h_body/2, w=w as u32, h=h_body as u32, color=wall_color);
            rect!(x=x-4, y=y+h_body/2-10, w=8, h=10, color=door_color);
            rect!(x=x-9, y=y-3, w=4, h=5, color=win_color);
            rect!(x=x+5, y=y-3, w=4, h=5, color=win_color);
            let rx = x-w/2; let ry = y-h_body/2;
            rect!(x=rx-2, y=ry-4, w=(w+4) as u32, h=4, color=roof_color);
            rect!(x=rx+2, y=ry-8, w=(w-4) as u32, h=4, color=roof_color);
            rect!(x=rx+6, y=ry-12, w=(w-12) as u32, h=4, color=roof_color);
            rect!(x=x+6, y=ry-14, w=4, h=10, color=if active { 0x8B4513FF } else { 0x333333FF });
            rect!(x=x+5, y=ry-16, w=6, h=2, color=roof_color);

            // Floating Points Indicator (Festive Board - White Text & Snowflakes)
            if active {
                 let label = format!("+{}", h.points);
                 let bob = (self.frame_count as f32 * 0.1).sin() * 2.0;

                 let label_w = (label.len() * 7) as i32; 
                 let w = label_w + 8;
                 let h = 11; 
                 
                 let bx = (x as i32) - w/2;
                 let by = (y as i32) - 30 + (bob as i32); 

                 // Posts
                 rect!(x=bx+3, y=by+h, w=2, h=4, color=0x5D4037FF);
                 rect!(x=bx+w-5, y=by+h, w=2, h=4, color=0x5D4037FF);

                 // Wood Border
                 rect!(x=bx-1, y=by-1, w=w as u32 + 2, h=h as u32 + 2, color=0x8D6E63FF); 

                 // Red Inner Board
                 rect!(x=bx, y=by, w=w as u32, h=h as u32, color=0xD32F2FFF); 

                 // Text (White)
                 text!(&label, x = bx + 3, y = by + 2, font="medium", color=0xFFFFFFFF);

                 // Border Decor (Lights & Snowflakes)
                 let bulb_yellow = 0xFFEB3BFF; 
                 let snow_white = 0xFFFFFFFF; 
                 
                 // Top/Bottom
                 for i in (1..w+1).step_by(3) {
                     let col = if i % 2 == 0 { bulb_yellow } else { snow_white };
                     rect!(x=bx+i, y=by-2, w=2, h=2, color=col);
                     rect!(x=bx+i, y=by+h, w=2, h=2, color=col);
                 }
                 // Sides
                 for j in (1..h+1).step_by(3) {
                     let col = if j % 2 == 0 { snow_white } else { bulb_yellow };
                     rect!(x=bx-2, y=by+j, w=2, h=2, color=col);
                     rect!(x=bx+w, y=by+j, w=2, h=2, color=col);
                 }

                 // Corner Snowflakes
                 rect!(x=bx-2, y=by-2, w=3, h=3, color=0xE0F7FAFF); // TL
                 rect!(x=bx+w-1, y=by-2, w=3, h=3, color=0xE0F7FAFF); // TR
                 rect!(x=bx-2, y=by+h-1, w=3, h=3, color=0xE0F7FAFF); // BL
                 rect!(x=bx+w-1, y=by+h-1, w=3, h=3, color=0xE0F7FAFF); // BR
            }
        }
        
        // Draw Floating Score Pops
        for t in &self.floating_texts {
             // Shadow
             text!(&t.text, x=t.x as i32 - 9, y=t.y as i32 + 1, font="medium", color=0x000000AA);
             text!(&t.text, x=t.x as i32 - 10, y=t.y as i32, font="medium", color=t.color);
        }
        
        // Particles
        for p in &self.particles {
            rect!(x = p.x as i32, y = p.y as i32, w = 2, h = 2, color = p.color);
        }
        
        // Powerups (Procedural Gift Box)
        for pu in &self.powerups {
             let px = pu.x as i32;
             let py = pu.y as i32;
             
             if pu.kind == 0 { // Gift Box (New Design)
                 // Bobbing animation
                 let bob = ((self.frame_count / 15) % 2) as i32;
                 let py_anim = py - bob;

                 // 1. Glow (Back)
                 circ!(x=px, y=py_anim, d=20, color=0xFFEB3B44); // Yellow Glow
                 
                 // Dimensions
                 let w = 14;
                 let h_front = 12;
                 let h_top = 5;
                 
                 // Top Left of the whole shape
                 let bx = px - w/2; 
                 let by = py_anim - h_front/2; 
                 
                 // 2. Box Base (Front)
                 rect!(x=bx, y=by, w=w as u32, h=h_front as u32, color=0xD32F2FFF); // Red Body
                 
                 // 3. Box Top (Perspective)
                 rect!(x=bx, y=by - h_top + 1, w=w as u32, h=h_top as u32, color=0xE53935FF); // Lighter Red Top
                 
                 // 4. Ribbon (Green)
                 // Vertical Front
                 rect!(x=px-2, y=by, w=4, h=h_front as u32, color=0x43A047FF);
                 // Vertical Top
                 rect!(x=px-2, y=by - h_top + 1, w=4, h=h_top as u32, color=0x43A047FF);
                 // Horizontal Top (Cross)
                 rect!(x=bx, y=by - h_top + 3, w=w as u32, h=2, color=0x43A047FF);

                 // 5. Bow (Loops)
                 rect!(x=px-5, y=by-h_top-1, w=4, h=3, color=0x66BB6AFF); // Left Loop
                 rect!(x=px+1, y=by-h_top-1, w=4, h=3, color=0x66BB6AFF); // Right Loop
                 rect!(x=px-1, y=by-h_top, w=2, h=2, color=0x388E3CFF); // Knot center

                 // 6. Sparkles
                 if self.frame_count % 30 < 15 {
                     rect!(x=px-9, y=py_anim-9, w=1, h=1, color=0xFFFFFFFF);
                     rect!(x=px+9, y=py_anim+5, w=1, h=1, color=0xFFFFFFFF);
                 } else {
                     rect!(x=px+9, y=py_anim-9, w=1, h=1, color=0xFFFFFFFF);
                     rect!(x=px-9, y=py_anim+5, w=1, h=1, color=0xFFFFFFFF);
                 }

             } else { 
                 // Speed Boost (Winged Electric Boot)
                 let bob = ((self.frame_count / 15) % 2) as i32;
                 let py_anim = py - bob;
                 
                 // 1. Electric Aura (Cyan Glow)
                 circ!(x=px, y=py_anim, d=26, color=0x00E5FF33); // Cyan Aura

                 // Helper vars
                 let hx = px;
                 let hy = py_anim;
                 
                 // 2. Electric Trail (Blue/Cyan Zig-Zags)
                 let trail_offset = (self.frame_count % 4) as i32;
                 let col_trail = 0x4FC3F7FF; // Light Blue
                 
                 // Top Trail
                 rect!(x=hx+6, y=hy-5, w=6, h=1, color=col_trail);
                 rect!(x=hx+10 + trail_offset, y=hy-7, w=4, h=1, color=col_trail);
                 
                 // Mid Trail
                 rect!(x=hx+8, y=hy+1, w=8, h=2, color=0x00B0FFFF); // Darker Cyan
                 rect!(x=hx+14 + trail_offset, y=hy, w=3, h=1, color=0xFFFFFFFF); // White Spark
                 
                 // Bot Trail
                 rect!(x=hx+7, y=hy+6, w=5, h=1, color=col_trail);
                 
                 // 3. Boot Body (Brown)
                 let col_boot = 0x5D4037FF;
                 let col_sole = 0x3E2723FF;
                 
                 // Leg
                 rect!(x=hx-4, y=hy-6, w=7, h=6, color=col_boot);
                 // Foot
                 rect!(x=hx-6, y=hy, w=11, h=5, color=col_boot);
                 // Sole/Heel
                 rect!(x=hx-6, y=hy+5, w=11, h=2, color=col_sole);
                 
                 // Red Laces detail
                 rect!(x=hx+2, y=hy-2, w=2, h=1, color=0xD32F2FFF);
                 rect!(x=hx+1, y=hy+1, w=2, h=1, color=0xD32F2FFF);

                 // 4. Fur Trim (White)
                 rect!(x=hx-5, y=hy-8, w=9, h=3, color=0xFFFFFFFF); // Top fluff
                 
                 // 5. Wing (Angel White) - Extending from heel/ankle
                 let wx = hx - 2;
                 let wy = hy - 6;
                 let wing_flap = if (self.frame_count / 10) % 2 == 0 { -1 } else { 0 };
                 
                 // Wing Base
                 rect!(x=wx, y=wy + wing_flap, w=6, h=3, color=0xFFFFFFFF);
                 // Wing Tip Up
                 rect!(x=wx+2, y=wy-3 + wing_flap, w=2, h=3, color=0xFFFFFFFF);
                 rect!(x=wx+4, y=wy-5 + wing_flap, w=2, h=5, color=0xFFFFFFFF);
                 // Feathers (Cyan tint)
                 rect!(x=wx+3, y=wy-2 + wing_flap, w=2, h=2, color=0xE0F7FAFF);
                 
                 // 6. Sparkles
                 if self.frame_count % 20 < 10 {
                    rect!(x=hx-9, y=hy-9, w=2, h=2, color=0xFFFFFFFF);
                 }
             }
        }

        // Players
        // Players
        for p in &self.players {
            let x = p.x as i32;
            let y = p.y as i32;
            let is_santa = p.id == 1;
            
            // Animation Bob
            let bob = ((self.frame_count / 10) % 2) as i32;
            
            // Colors
            let suit_color = if is_santa { 0xD32F2FFF } else { 0x1976D2FF }; // Red vs Blue
            let trim_color = 0xFFFFFFFF; // White
            let skin_color = 0xFFCC80FF; // Peach
            let boot_color = 0x212121FF; // Black
            let belt_color = 0x212121FF; 
            let gold_color = 0xFFD700FF;
            
            // Size
            let w = 20;
            let h = 28;
            
            // Draw relative to center (x,y)
            let lx = x - w/2; // Left X
            let ty = y - h/2 - bob; // Top Y (bobbing)
            
            // 1. Legs/Boots
            rect!(x=lx+2, y=ty+22, w=6, h=6, color=boot_color);
            rect!(x=lx+12, y=ty+22, w=6, h=6, color=boot_color);
            
            // 2. Body (Suit)
            rect!(x=lx, y=ty+10, w=20, h=14, color=suit_color);
            
            // 3. Vertical White Trim (Coat)
            rect!(x=lx+8, y=ty+10, w=4, h=14, color=trim_color);
            
            // 4. Belt
            rect!(x=lx, y=ty+16, w=20, h=4, color=belt_color);
            rect!(x=lx+8, y=ty+16, w=4, h=4, color=gold_color); // Buckle
            
            // 5. Head (Face)
            rect!(x=lx+2, y=ty, w=16, h=12, color=skin_color);
            
            // 6. Beard
            rect!(x=lx+2, y=ty+8, w=16, h=6, color=trim_color);
            rect!(x=lx+4, y=ty+12, w=12, h=2, color=trim_color); // Taper
            
            // 7. Eyes
            rect!(x=lx+5, y=ty+4, w=2, h=2, color=boot_color);
            rect!(x=lx+13, y=ty+4, w=2, h=2, color=boot_color);
            
            // 8. Hat
            rect!(x=lx, y=ty-4, w=20, h=6, color=trim_color); // Brim
            rect!(x=lx+2, y=ty-10, w=16, h=6, color=suit_color); // Cap
            rect!(x=lx+18, y=ty-6, w=4, h=4, color=trim_color); // PomPom
            
            // Label (Optional, maybe remove if too cluttered, or keep small)
            // text!(if is_santa{"P1"}else{"P2"}, x=x-6, y=ty-20, font="small", color=0xFFFFFFFF);
        }
        
        // Level 2: Bomb Obstacles
        for o in &self.obstacles {
            if o.respawn_timer > 0 { continue; } // Hidden
            
            let x = o.x as i32;
            let y = o.y as i32;
            let w = o.w as i32;
            
            // Bomb Body (Black Circle)
            // d=w+2 (26). Box is 24x24. Center is x+12, y+12.
            // Top-left of circle should be x-1, y-1.
            let bx = x - 1;
            let by = y - 1;
            
            circ!(x=bx, y=by, d=w+2, color=0x212121FF); // Black body
            
            // Shine (Offset from top-left of circle)
            circ!(x=bx+6, y=by+6, d=8, color=0x424242FF); // Grey highlight
            
            // Fuse Cap (Connects to top)
            // Top of circle is 'by'. Center X is 'bx + 13' = 'x + 12'.
            let cx = x + 12;
            
            rect!(x=cx-3, y=by-2, w=6, h=4, color=0x9E9E9EFF); // Cap overlaps top slightly
            
            // Fuse Line
            rect!(x=cx-1, y=by-6, w=2, h=4, color=0x8D6E63FF);
            
            // Spark (Flickering)
            if self.frame_count % 10 < 5 {
                rect!(x=cx-2, y=by-9, w=4, h=4, color=0xFFC107FF); // Yellow
                rect!(x=cx-1, y=by-8, w=2, h=2, color=0xFFFFFFFF); // White center
            } else {
                 rect!(x=cx-2, y=by-9, w=4, h=4, color=0xFF5722FF); // Orange
            }
        }

        // HUD (Dark Text for Light BG)
        let p1_text = format!("{}: {}", self.players[0].name, self.players[0].score);
        text!(&p1_text, x=10, y=10, color=0xB71C1CFF); // Dark Red
        
        let p2_text = format!("{}: {}", self.players[1].name, self.players[1].score);
        let w = (p2_text.len() * 8) as i32;
        text!(&p2_text, x=502 - w, y=10, color=0x0D47A1FF); // Dark Blue
        
        let mins = self.timer / 60;
        let secs = self.timer % 60;
        let time_text = format!("{:02}:{:02}", mins, secs);
        text!(&time_text, x=240, y=10, color=0xF57F17FF); // Dark Orange/Gold
        
        if self.game_over {
             // Overlay
             rect!(w=512, h=288, color=0xFFFFFFAA); // Light Overlay
             text!(&self.winner_text, x=200, y=130, font="large", color=0x000000FF); // Black Text
             text!("Press START to Restart", x=180, y=160, color=0x333333FF);
             text!("Press X to Exit", x=200, y=180, color=0x333333FF);
        }
        
        // Vignette (Light edges/Frost?) 
        // Let's remove dark vignette for light theme or make it white frost
        rect!(w=512, h=288, color=0xFFFFFF22); // Subtle Frost overlay
    }
}
