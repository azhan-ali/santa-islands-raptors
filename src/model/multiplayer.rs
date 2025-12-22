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
    // Level 3
    pub shadow_trail: Vec<(f32, f32, u32)>, // x, y, life
    pub slow_timer: u32,
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
    pub is_high_value: bool, // Level 3: Unused. Level 4: True for Power House.
    pub team: u8, // 0 = Neutral, 1 = Red, 2 = Blue
    pub gift_timer: u32, // Level 4: Timer for spawning risky gifts
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub respawn_timer: u32,
    pub kind: u8, // 0 = Bomb, 1 = Wood, 2 = Snowman (Level 4)
}

#[turbo::serialize]
#[derive(PartialEq)]
pub struct PowerUp {
    pub x: f32,
    pub y: f32,
    pub kind: u8, // 0 = Gift, 1 = Speed, 2 = Risky Gift (Level 4)
    pub collected: bool,
    pub value: i32, // Level 4: +60 or -60
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
                shadow_trail: vec![],
                slow_timer: 0,
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
                shadow_trail: vec![],
                slow_timer: 0,
            },
        ];

        // Initialize Lists
        self.houses = vec![];
        self.obstacles = vec![];
        
        // 1. Static Level 3 Obstacles (Wood Barriers) - Spawn FIRST to reserve space
        if _level >= 3 {
             // 2 Bridges: Centered at y=100 and y=200. Width ~28.
             // Guards for Bridge 1 (y=100)
             self.obstacles.push(Obstacle { x: 210.0, y: 88.0, w: 24.0, h: 24.0, respawn_timer: 0, kind: 1 });
             self.obstacles.push(Obstacle { x: 280.0, y: 88.0, w: 24.0, h: 24.0, respawn_timer: 0, kind: 1 });
             
             // Guards for Bridge 2 (y=200)
             self.obstacles.push(Obstacle { x: 210.0, y: 188.0, w: 24.0, h: 24.0, respawn_timer: 0, kind: 1 });
             self.obstacles.push(Obstacle { x: 280.0, y: 188.0, w: 24.0, h: 24.0, respawn_timer: 0, kind: 1 });
        }
        
        // 2. Houses
        // 2. Houses
        if _level >= 3 {
             // Level 3 & 4: 5 Red (Team 1), 5 Blue (Team 2)
             // Level 4: +1 Power House (Team 0)
             
             // Manually generate to assign teams
             let mut rng = random::u32();
             let mut placed_count = 0;
             let target = if _level == 4 { 11 } else { 10 };
             let mut attempts = 0;
             
             while placed_count < target && attempts < 1000 {
                 attempts += 1;
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 let hx = 40.0 + (rng % 432) as f32;
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 let hy = 40.0 + (rng % 180) as f32; // Keep somewhat upper for start?
                 
                 // L3/L4 Checks (River)
                 if hx > 236.0 && hx < 276.0 { continue; } // River
                 
                 let mut safe = true;
                 for o in &self.obstacles {
                     if ((o.x - hx).powi(2) + (o.y - hy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                 }
                 if safe {
                     for h in &self.houses {
                         if ((h.x - hx).powi(2) + (h.y - hy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                     }
                 }
                 
                 if safe {
                     let mut team = 0;
                     let mut is_pow = false;
                     let mut g_timer = 0;
                     
                     if _level == 4 && placed_count == 10 {
                         // 11th House is Power House
                         team = 0; // Neutral? Or maybe make it uncapturable for points? 
                                   // Rules say "Power High Value House" releases gifts.
                                   // Does it belong to a team? Probably Neutral (0) so anyone can visit?
                                   // Or maybe it is just an object.
                                   // Let's make it Team 0 (Neutral) and Uncapturable logic later.
                         is_pow = true;
                         g_timer = 15 * 60; // 15s initial timer
                     } else {
                         team = if placed_count < 5 { 1 } else { 2 };
                     }

                     self.houses.push(House {
                        x: hx, y: hy, points: 5, cooldown: 0,
                        last_collected_by: None, last_collection_time: 0,
                        is_high_value: is_pow,
                        team: team,
                        gift_timer: g_timer,
                     });
                     placed_count += 1;
                 }
             }
             
        } else {
            // Level 1 & 2
            let count = if _level == 2 { 14 } else { 10 };
            self.generate_random_houses(count, false);
        }
        
        // 3. Random Bombs (Level 2 & 3)
        if _level >= 2 {
            // Level 2 & 3 have Bombs
            let bomb_count = if _level == 3 { 6 } else { 8 };
            
            // Level 4: No bombs mentioned in doc? "Existing bombs... remain". 
            // So Level 4 keeps L3 obstacles (Wood + Bombs?)
            // "obstacles: Existing bombs, trees, and wooden blocks remain" -> Yes.
            // Level 4 Bomb count = Same as L3 (6)? Or less to make room for Snowmen?
            // Let's assume 6 Bombs.
            
            let mut rng = random::u32();
            let mut attempts = 0;
            // Target Limit: L3 had ~9 obstacles (3 wood + 6 bombs)
            // L4 adds Snowmen later.
            let bomb_target = if _level >= 3 { 6 } else { 8 };
            
            let current_obs_count = self.obstacles.len();

            while (self.obstacles.len() - current_obs_count) < bomb_target && attempts < 100 {
                attempts += 1;
                rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                let ox = 60.0 + (rng % 392) as f32; 
                rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                let oy = 60.0 + (rng % 168) as f32;
                
                // Avoid overlap with houses
                let mut safe = true;
                for h in &self.houses {
                   if ((h.x - ox).powi(2) + (h.y - oy).powi(2)).sqrt() < 60.0 { safe = false; break; }
                }
                // Avoid level 3 water strip (ENTIRE STRIP banned for bombs)
                if _level >= 3 {
                     if ox > 236.0 && ox < 276.0 {
                         safe = false; // Block bridge too
                     }
                }
                
                // Avoid overlap with existing obstacles (Wood & Bombs)
                if safe {
                    for o in &self.obstacles {
                        if ((o.x - ox).powi(2) + (o.y - oy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                    }
                }
                
                if safe {
                    self.obstacles.push(Obstacle { x: ox, y: oy, w: 24.0, h: 24.0, respawn_timer: 0, kind: 0 }); // Kind 0 = Bomb
                }
            }
        }
        
        // 4. Snowmen (Level 4 Specific)
        if _level == 4 {
             let snowman_count = 5;
             let mut rng = random::u32();
             let mut attempts = 0;
             let start_count = self.obstacles.len();
             
             while (self.obstacles.len() - start_count) < snowman_count && attempts < 100 {
                 attempts += 1;
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 let sx = 40.0 + (rng % 432) as f32;
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 let sy = 40.0 + (rng % 208) as f32;
                 
                 // River Check
                 if sx > 236.0 && sx < 276.0 { continue; }
                 
                 let mut safe = true;
                 for h in &self.houses {
                     if ((h.x - sx).powi(2) + (h.y - sy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                 }
                 if safe {
                     for o in &self.obstacles {
                         if ((o.x - sx).powi(2) + (o.y - sy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                     }
                 }
                 
                 if safe {
                     // Kind 2 = Snowman
                     self.obstacles.push(Obstacle { x: sx, y: sy, w: 20.0, h: 30.0, respawn_timer: 0, kind: 2 });
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
        // Generate Decor (Trees and Piles)
        self.decors = vec![];
        let mut decor_attempts = 0;
        // Trees
        while self.decors.len() < 20 && decor_attempts < 500 {
            decor_attempts += 1;
            let dx = (random::u32() % 500 + 10) as f32;
            let dy = (random::u32() % 260 + 10) as f32;
            
            // Level 3 Check: No trees in River Strip
            if self.current_level >= 3 && dx > 236.0 && dx < 276.0 { continue; }
            
            let mut safe = true;
            // Check Houses
            for h in &self.houses {
                if ((h.x - dx).powi(2) + (h.y - dy).powi(2)).sqrt() < 40.0 { safe = false; break; }
            }
            // Check Obstacles
            if safe {
                for o in &self.obstacles {
                    if ((o.x - dx).powi(2) + (o.y - dy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                }
            }
            // Check Decors
            if safe {
                for d in &self.decors {
                    if ((d.x - dx).powi(2) + (d.y - dy).powi(2)).sqrt() < 30.0 { safe = false; break; }
                }
            }
            
            if safe {
                self.decors.push(Decor { x: dx, y: dy, kind: 0 });
            }
        }
        
        // Piles
        decor_attempts = 0;
        while self.decors.len() < 30 && decor_attempts < 200 { 
             decor_attempts += 1;
             if random::u32() % 2 == 0 {
                let dx = (random::u32() % 500 + 10) as f32;
                let dy = (random::u32() % 260 + 10) as f32;
                
                if self.current_level >= 3 && dx > 236.0 && dx < 276.0 { continue; }
                
                let mut safe = true;
                // Check Houses
                for h in &self.houses {
                    if ((h.x - dx).powi(2) + (h.y - dy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                }
                // Check Obstacles
                if safe {
                    for o in &self.obstacles {
                        if ((o.x - dx).powi(2) + (o.y - dy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                    }
                }
                // Check Decors
                if safe {
                    for d in &self.decors {
                        if ((d.x - dx).powi(2) + (d.y - dy).powi(2)).sqrt() < 30.0 { safe = false; break; }
                    }
                }
                
                if safe {
                    self.decors.push(Decor { x: dx, y: dy, kind: 1 });
                }
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
                 
                 while !safe && attempts < 50 {
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
                     
                     // Level 3 River Strip
                     if safe && self.current_level >= 3 {
                         if px > 236.0 && px < 276.0 { safe = false; }
                     }
                     
                     // 4. Not on Obstacles
                     if safe {
                         for o in &self.obstacles {
                             if ((o.x - px).powi(2) + (o.y - py).powi(2)).sqrt() < 45.0 { safe = false; break; }
                         }
                     }
                     
                     // 5. Not on Decors
                     if safe {
                         for d in &self.decors {
                             if ((d.x - px).powi(2) + (d.y - py).powi(2)).sqrt() < 40.0 { safe = false; break; }
                         }
                     }
                 }
                 
                 if safe {
                     self.powerups.push(PowerUp {
                         x: px,
                         y: py,
                         kind,
                         collected: false,
                         value: 0,
                     });
                 }
             }
        }

        // Players Update
        // Freeze movement if shuffling (Level 2 pause)
        if !self.is_shuffling {
            for i in 0..self.players.len() {
                let (dx, dy) = self.get_input(i, self.players[i].id);
            let mut speed = if self.players[i].boost_timer > 0 { 3.0 } else { 2.0 };
            
            // Level 3 Slow
            if self.players[i].slow_timer > 0 {
                speed *= 0.5;
            }
                
                if self.players[i].boost_timer > 0 {
                    self.players[i].boost_timer -= 1;
                }
                if self.players[i].invuln_timer > 0 {
                    self.players[i].invuln_timer -= 1;
                }

                let next_x = self.players[i].x + dx * speed;
                let next_y = self.players[i].y + dy * speed;

                // Level 3 Water Collision
                if self.current_level >= 3 && self.is_in_water(next_x, next_y) {
                    // Block movement - sliding logic?
                    // Simple: Don't update if in water. Check X and Y separately for slide.
                    if !self.is_in_water(next_x, self.players[i].y) {
                        self.players[i].x = next_x;
                    }
                    if !self.is_in_water(self.players[i].x, next_y) {
                        self.players[i].y = next_y;
                    }
                } else {
                    self.players[i].x = next_x;
                    self.players[i].y = next_y;
                }

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
        let mut house_reshuffle_indices = vec![];

        for (i, house) in self.houses.iter_mut().enumerate() {
            if house.cooldown > 0 { house.cooldown -= 1; }
            
            // Point Growth (delayed by 5s (300 ticks) after collection)
            if self.frame_count.saturating_sub(house.last_collection_time) > 300 
               && self.frame_count % 120 == 0 
               && house.points < 50 { 
                   house.points += 5; 
            }

            for player in self.players.iter_mut() {
                 let dist = ((player.x - house.x).powi(2) + (player.y - house.y).powi(2)).sqrt();
                 if dist < (player.radius + 12.0) && house.cooldown == 0 {
                        // Check Team (Level 3)
                        let is_wrong_team = self.current_level >= 3 && (
                            (house.team == 1 && player.id != 1) || 
                            (house.team == 2 && player.id != 2)
                        );
                        
                        if is_wrong_team {
                             // PENALTY
                             player.score = player.score.saturating_sub(20);
                             
                             // Penalty Pop
                             score_pops.push(FloatingText {
                                x: house.x,
                                y: house.y - 20.0,
                                text: "-20".to_string(),
                                color: 0xFF0000FF, // Red Warning
                                life: 60,
                             });
                             turbo::audio::play("coin"); 
                             
                             // Apply Cooldown (1s) instead of moving
                             house.cooldown = 60; 

                        } else {
                             // VALID CAPTURE
                             player.score += house.points;
                             
                             // Queue Pop
                             score_pops.push(FloatingText {
                                x: house.x,
                                y: house.y - 20.0,
                                text: format!("+{}", house.points),
                                color: if house.team == 1 { 0xFFCDD2FF } else if house.team == 2 { 0xBBDEFBFF } else { 0xE0F7FAFF }, // Team Color
                                life: 60,
                             });

                             if self.current_level >= 3 {
                                 house.cooldown = 300; // Disable first. Only enable if successfully moved.
                                 house_reshuffle_indices.push(i); 
                             } else {
                                 house.cooldown = 300; // L1/L2 Standard Cooldown
                             }
                             
                             // Reset Points logic
                             house.points = 5; // Always reset to 5 (User request) 
                             house.last_collected_by = Some(player.id);
                             house.last_collection_time = current_tick;
                             
                             turbo::audio::play("coin");
                             sparkle_reqs.push((house.x, house.y));
                        }
                 }
            }
        }
        
        // Execute Deferred Moves (Level 3)
        // Snapshot static obstacles
        let obstacle_pos: Vec<(f32, f32)> = self.obstacles.iter().map(|o| (o.x, o.y)).collect();
        // Snapshot current houses
        let mut house_pos: Vec<(f32, f32)> = self.houses.iter().map(|h| (h.x, h.y)).collect();
        // Snapshot decors
        let decor_pos: Vec<(f32, f32)> = self.decors.iter().map(|d| (d.x, d.y)).collect();
        
        for idx in house_reshuffle_indices {
             let mut rng = random::u32();
             let mut attempts = 0;
             let mut placed = false;
             
             let mut new_x = 0.0;
             let mut new_y = 0.0;
             
             // Determine Target Side (Cross the River)
             let current_x = self.houses[idx].x;
             let target_right = current_x < 256.0; // If currently left, go right
             
             while !placed && attempts < 1000 {
                 attempts += 1;
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 
                 if target_right {
                      // Target: 276..472
                      new_x = 280.0 + (rng % 190) as f32;
                 } else {
                      // Target: 40..236
                      new_x = 40.0 + (rng % 190) as f32;
                 }
                 
                 rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                 new_y = 40.0 + (rng % 180) as f32; 
                 
                 // Water Strip Check (Redundant if logic correct, but safe)
                 if new_x > 236.0 && new_x < 276.0 { continue; }
                 
                 let mut safe = true;
                 
                 // Check Obstacles
                 for (ox, oy) in &obstacle_pos {
                     if ((ox - new_x).powi(2) + (oy - new_y).powi(2)).sqrt() < 50.0 { safe = false; break; }
                 }
                 
                 // Check Decors
                 if safe {
                     for (dx, dy) in &decor_pos {
                         if ((dx - new_x).powi(2) + (dy - new_y).powi(2)).sqrt() < 45.0 { safe = false; break; }
                     }
                 }
                 
                 // Check other houses (including ones just moved)
                 if safe {
                     for (i, (hx, hy)) in house_pos.iter().enumerate() {
                         if i != idx { // Don't check self
                             if ((hx - new_x).powi(2) + (hy - new_y).powi(2)).sqrt() < 50.0 { safe = false; break; }
                         }
                     }
                 }
                 
                 if safe {
                     placed = true;
                 }
             }
             
             // End of RNG loop
             
             // GRID SEARCH FALLBACK (If RNG failed 1000 times)
             if !placed {
                 // Iterate grid points to find ANY safe spot
                 let start_x = if target_right { 280 } else { 40 };
                 let end_x = if target_right { 470 } else { 230 };
                 
                 'grid: for gx in (start_x..end_x).step_by(15) {
                     for gy in (40..250).step_by(15) {
                         let tx = gx as f32;
                         let ty = gy as f32;
                         
                         // Water Check
                         if tx > 236.0 && tx < 276.0 { continue; }
                         
                         let mut safe = true;
                         // Check Obstacles
                         for (ox, oy) in &obstacle_pos {
                             if ((ox - tx).powi(2) + (oy - ty).powi(2)).sqrt() < 50.0 { safe = false; break; }
                         }
                         // Check Decors - Reduced Radius 35.0 for Fallback
                         if safe {
                             for (dx, dy) in &decor_pos {
                                 if ((dx - tx).powi(2) + (dy - ty).powi(2)).sqrt() < 35.0 { safe = false; break; }
                             }
                         }
                         // Check Houses - Reduced Radius 40.0 for Fallback
                         if safe {
                             for (i, (hx, hy)) in house_pos.iter().enumerate() {
                                 if i != idx {
                                     if ((hx - tx).powi(2) + (hy - ty).powi(2)).sqrt() < 40.0 { safe = false; break; }
                                 }
                             }
                         }
                         
                         if safe {
                             new_x = tx;
                             new_y = ty;
                             placed = true;
                             break 'grid;
                         }
                     }
                 }
             }

             // Apply Position (If placed via RNG or Grid)
             if placed {
                 self.houses[idx].x = new_x;
                 self.houses[idx].y = new_y;
                 self.houses[idx].cooldown = 0; // Successfully moved -> Enable immediately
                 house_pos[idx] = (new_x, new_y); 
             } else {
                 // Grid Failed. Keep Cooldown=300 (Disabled).
                 // House stays in place but is inert for 5s. Prevents infinite loop.
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
        
        // Level 4: Snowman Collision & Power House Logic
        if self.current_level == 4 {
             for p in self.players.iter_mut() {
                 if p.invuln_timer > 0 { continue; }
                 
                 for o in &self.obstacles {
                     if o.kind == 2 { // Snowman
                         let dist = ((p.x - o.x).powi(2) + (p.y - o.y).powi(2)).sqrt();
                         if dist < (p.radius + 15.0) { // Snowman radius ~15
                             // Hit Snowman!
                             p.score = p.score.saturating_sub(10);
                             p.invuln_timer = 60; // 1s invuln
                             turbo::audio::play("hit");
                             
                             self.floating_texts.push(FloatingText {
                                x: p.x,
                                y: p.y - 20.0,
                                text: "-10".to_string(),
                                color: 0xFF0000FF,
                                life: 60,
                             });
                         }
                     }
                 }
             }
             
             // Power House Logic
             let mut new_house_pos = None;
             let mut house_idx = 0;
             
             for (i, h) in self.houses.iter_mut().enumerate() {
                 if h.is_high_value {
                     if h.gift_timer > 0 {
                         h.gift_timer -= 1;
                     } else {
                         // Spawn Risky Gift!
                         h.gift_timer = 15 * 60; // Reset 15s
                         
                         // Determine Effect (+60 or -60)
                         let is_good = (random::u32() % 2) == 0;
                         let val = if is_good { 60 } else { -60 };
                         
                         // Spawn Powerup near house
                         self.powerups.push(PowerUp {
                             x: h.x + 20.0,
                             y: h.y + 20.0, // Offset slightly
                             kind: 2, // 2 = Risky
                             collected: false,
                             value: val, 
                         });
                         
                         // Trigger Teleport
                         new_house_pos = Some((0.0, 0.0)); // Placeholder to trigger logic below
                         house_idx = i;
                     }
                 }
             }
             
             // Handle Teleportation outside the loop to avoid borrow checker issues
             if let Some(_) = new_house_pos {
                 let mut rng = random::u32();
                 let mut attempts = 0;
                 let mut placed = false;
                 
                 while !placed && attempts < 100 {
                     attempts += 1;
                     rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                     let nx = 40.0 + (rng % 432) as f32;
                     rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                     let ny = 40.0 + (rng % 208) as f32;
                     
                     // River Check
                     if nx > 236.0 && nx < 276.0 { continue; }
                     
                     let mut safe = true;
                     for o in &self.obstacles {
                         if ((o.x - nx).powi(2) + (o.y - ny).powi(2)).sqrt() < 50.0 { safe = false; break; }
                     }
                     if safe {
                         for (i, h) in self.houses.iter().enumerate() {
                             if i != house_idx {
                                 if ((h.x - nx).powi(2) + (h.y - ny).powi(2)).sqrt() < 50.0 { safe = false; break; }
                             }
                         }
                     }
                     
                     if safe {
                         self.houses[house_idx].x = nx;
                         self.houses[house_idx].y = ny;
                         placed = true;
                         
                         // Teleport Poof Particle
                         for _ in 0..10 {
                              self.particles.push(MParticle {
                                  x: nx, y: ny,
                                  vx: ((random::u32() % 10) as f32 - 5.0) * 0.5,
                                  vy: ((random::u32() % 10) as f32 - 5.0) * 0.5,
                                  life: 30,
                                  color: 0xFFD700FF,
                              });
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
                     } else if pu.kind == 1 {
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
                     } else {
                         // Risky Gift (Kind 2)
                         let val = pu.value;
                         let text_col;
                         let text_str;
                         
                         if val >= 0 {
                             player.score += val as u32;
                             text_col = 0xFFD700FF; // Gold
                             text_str = format!("+{}", val);
                         } else {
                             let pen = (-val) as u32;
                             player.score = player.score.saturating_sub(pen);
                             text_col = 0xFF0000FF; // Red
                             text_str = format!("{}", val);
                         }
                         
                         self.floating_texts.push(FloatingText {
                            x: pu.x,
                            y: pu.y - 10.0,
                            text: text_str,
                            color: text_col,
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
         // Level 3 Mechanics
         if self.current_level >= 3 {
             self.update_level3();
         }
    }
    
    fn generate_random_houses(&mut self, count: usize, _avoid_special_zone: bool) {
        // Only clear houses if we are doing a full reset (not implicit here, usage depends on context)
        // Actually, init_level clears houses. This function appends.
        // Wait, normally generate_houses replaced all. 
        // Let's assume this pushes new houses.
        
        let mut rng = random::u32();
        let mut attempts = 0;
        let target_len = self.houses.len() + count;
        
        while self.houses.len() < target_len && attempts < 200 {
            attempts += 1;
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hx = 40.0 + (rng % 432) as f32; 
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            let hy = 40.0 + (rng % 180) as f32; 
            
            // Avoid Water Strip (Level 3)
            if self.current_level == 3 {
                 // Vertical Strip 236-276
                 if hx > 236.0 && hx < 276.0 {
                     // Check Safe Bridge (130-158)
                     // But usually we don't put houses on bridge if it's narrow
                     continue; 
                 }
            }
            
            // Check overlap
            let mut overlap = false;
            for h in &self.houses {
                let d = ((h.x - hx).powi(2) + (h.y - hy).powi(2)).sqrt();
                if d < 60.0 { overlap = true; break; }
            }
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
                    is_high_value: false,
                    team: 0,
                    gift_timer: 0,
                });
            }
        }
    }
    
    #[allow(dead_code)]
    fn reshuffle_normal_houses(&mut self) {
        // Keep High Value, Remove Normal
        self.houses.retain(|h| h.is_high_value);
        // Add 8 Normal
        self.generate_random_houses(8, true);
        
         self.floating_texts.push(FloatingText {
            x: 256.0,
            y: 144.0,
            text: "HOUSES MOVED!".to_string(),
            color: 0x00E5FFFF,
            life: 60,
        });
    }

    fn update_level3(&mut self) {
        // 1. Shadow Trail Logic
        if self.frame_count % 5 == 0 {
            // Record Trail
             // We need to iterate indices to avoid borrow issues
             for i in 0..self.players.len() {
                 let px = self.players[i].x;
                 let py = self.players[i].y;
                 self.players[i].shadow_trail.push((px, py, 180)); // 3 seconds (60fps)
             }
        }
        
        // Update Trail Life
        for p in self.players.iter_mut() {
            for t in p.shadow_trail.iter_mut() {
                if t.2 > 0 { t.2 -= 1; }
            }
            p.shadow_trail.retain(|t| t.2 > 0);
            
            // Tick Slow
            if p.slow_timer > 0 { p.slow_timer -= 1; }
        }
        
        // Check Collision with Opponent Trail
        // Check Collision with Opponent Trail
        // Collect trails first to avoid borrow conflicts
        let trails: Vec<Vec<(f32, f32)>> = self.players.iter()
            .map(|p| p.shadow_trail.iter().map(|t| (t.0, t.1)).collect())
            .collect();
            
        for (i, p) in self.players.iter_mut().enumerate() {
            if p.invuln_timer == 0 {
                // Check against opponent trail usually (just other player for now)
                let opponent_idx = if i == 0 { 1 } else { 0 };
                if opponent_idx < trails.len() {
                    for (tx, ty) in &trails[opponent_idx] {
                         if ((p.x - tx).powi(2) + (p.y - ty).powi(2)).sqrt() < 10.0 {
                            p.slow_timer = 30; // 0.5s slow
                            break;
                        }
                    }
                }
            }
        }
        
        // 2. Obstacle Logic (Wood: Static, Bomb: Respawning)
        // Reuse Level 2 Bomb Logic for kind==0, Add Wood logic for kind==1
        let mut penalties = vec![];
        let mut explosions = vec![];
        
        // Snapshot
        let obstacle_positions: Vec<(f32, f32)> = self.obstacles.iter().map(|o| (o.x, o.y)).collect();
        let decor_pos: Vec<(f32, f32)> = self.decors.iter().map(|d| (d.x, d.y)).collect();
        
        for (i, o) in self.obstacles.iter_mut().enumerate() {
            if o.kind == 0 { // Bomb
                 // ... Copy Level 2 Bomb Logic ...
                 // (omitted duplicate code for brevity, will insert full logic if tool allows large block)
                 // Actually, reusing update_level2 is tricky because it assumes ALL are bombs.
                 // We should probably share the logic or duplicate it.
                 // Let's duplicate the Bomb logic here for clarity and safety.
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
                             // Level 3 Check: Water (Strict Ban on entire strip including bridges)
                             if self.current_level >= 3 {
                                 if ox > 236.0 && ox < 276.0 { safe = false; } // Ban entire strip
                             }
                             
                             if safe {
                                 for h in &self.houses { if ((h.x - ox).powi(2) + (h.y - oy).powi(2)).sqrt() < 60.0 { safe = false; break; } }
                             }
                             if safe {
                                 for (idx, (ex, ey)) in obstacle_positions.iter().enumerate() {
                                     if i != idx && ((ex - ox).powi(2) + (ey - oy).powi(2)).sqrt() < 50.0 { safe = false; break; }
                                 }
                             }
                             if safe {
                                 for (dx, dy) in &decor_pos {
                                     if ((dx - ox).powi(2) + (dy - oy).powi(2)).sqrt() < 40.0 { safe = false; break; }
                                 }
                             }
                             if safe { o.x = ox; o.y = oy; placed = true; }
                         }
                    }
                    continue;
                 }
            }
            
            // Collision
             for p in self.players.iter_mut() {
                if p.invuln_timer > 0 { continue; }
                
                // Hitbox
                let closest_x = p.x.clamp(o.x, o.x + o.w);
                let closest_y = p.y.clamp(o.y, o.y + o.h);
                let dist_sq = (p.x - closest_x).powi(2) + (p.y - closest_y).powi(2);
                
                if dist_sq < (p.radius * p.radius) {
                     p.invuln_timer = 60;
                     if o.kind == 0 { // Bomb
                         if p.score >= 20 { p.score -= 20; } else { p.score = 0; }
                         penalties.push((p.x, p.y, 20)); // -20
                         explosions.push((o.x + o.w/2.0, o.y + o.h/2.0));
                         o.respawn_timer = 30; 
                         o.x = -1000.0;
                     } else { // Wood
                         if p.score >= 10 { p.score -= 10; } else { p.score = 0; }
                         penalties.push((p.x, p.y, 10)); // -10
                         // No respawn for wood
                         turbo::audio::play("hit"); 
                     }
                }
             }
        }
        
        for (px, py, amount) in penalties {
            self.floating_texts.push(FloatingText { x: px, y: py - 20.0, text: format!("-{}", amount), color: 0xFF0000FF, life: 60 });
        }
        for (ex, ey) in explosions {
             turbo::audio::play("projectile_hit"); 
             self.spawn_explosion(ex, ey);
        }
    }
    
    fn is_in_water(&self, x: f32, y: f32) -> bool {
        if x > 236.0 && x < 276.0 {
            // Bridge 1 Safe Zone (y: 86..114)
            if y > 86.0 && y < 114.0 { return false; }
            // Bridge 2 Safe Zone (y: 186..214)
            if y > 186.0 && y < 214.0 { return false; }
            return true;
        }
        false
    }
    
    #[allow(dead_code)]
    fn is_valid_bomb_pos(&self, x: f32, y: f32) -> bool {
         if self.current_level == 3 && self.is_in_water(x, y) { return false; }
         true
    }

    fn update_level2(&mut self) {
        // Shuffle Logic
        if self.is_shuffling {
            if self.shuffle_pause_timer > 0 {
                self.shuffle_pause_timer -= 1;
            } else {
                // Time to shuffle!
                self.generate_random_houses(14, false);
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
                         
                         // Level 3 River Strip
                         if self.current_level == 3 && ox > 236.0 && ox < 276.0 { safe = false; }

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

        // 1b. Level 3 Terrain (Water/Bridges) - Draw FIRST
        if self.current_level >= 3 {
            // Vertical Water Strip
            rect!(x=236, y=0, w=40, h=288, color=0x29B6F6FF); 
            
            // Bridge 1 (Top)
            rect!(x=236, y=100, w=40, h=28, color=0x8D6E63FF); 
            rect!(x=236, y=98, w=40, h=2, color=0x5D4037FF); 
            rect!(x=236, y=128, w=40, h=2, color=0x5D4037FF);
            
            // Bridge 2 (Bot)
            rect!(x=236, y=200, w=40, h=28, color=0x8D6E63FF); 
            rect!(x=236, y=198, w=40, h=2, color=0x5D4037FF); 
            rect!(x=236, y=228, w=40, h=2, color=0x5D4037FF);
        }
        
         // Draw Shadows (Level 3)
        if self.current_level >= 3 {
             for (_i, p) in self.players.iter().enumerate() {
                 let col = if p.id == 1 { 0xD32F2F44 } else { 0x1976D244 }; // Transparent Red/Blue
                 for t in &p.shadow_trail {
                     circ!(x=t.0 as i32 - 4, y=t.1 as i32 - 4, d=8, color=col);
                 }
             }
        } 

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
                 let glow_col = if h.is_high_value { 0xFFD70044 } else { 0x00E5FF22 }; // Gold glow for Power House
                 circ!(x=x, y=y, d=32, color=glow_col); 
            }
            
            // ... (Procedural House Drawing Block - Same as existing, assume preserved by context match or manual re-insertion if needed. Wait, ReplaceFileContent replaces everything in range. I need to include the House Draw logic or use a generic "Draw House" comment if I don't want to rewrite it.
            // Actually, I should rewrite the House Draw logic to be safe, or select a smaller chunk range.
            // The previous chunk was huge. Let's target the "Floating Points" section specifically?)
            
            // RE-INSERTING PROCEDURAL HOUSE LOGIC (Compressed for tool limit):
            let w = 24; let h_body = 18;
            let (wall_c, roof_c) = if h.team == 1 {
                 (0xB71C1CFF, 0xFFEBEEFF) // Red Wall, White Roof
            } else if h.team == 2 {
                 (0x0277BDFF, 0xE1F5FEFF) // Blue Wall, Cyan Roof
            } else {
                 (0xB22222FF, 0xFFFFFFFF) // Neutral
            };

            let wall_color = if active { wall_c } else { 0x444444FF };
            let roof_color = if active { roof_c } else { 0x777777FF };
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
            
            // High Value Indicator
            if h.is_high_value {
                // Gold star or roof
                 rect!(x=rx-2, y=ry-4, w=(w+4) as u32, h=4, color=0xFFD700FF); // Gold Roof
                 // "100" text above if active
                 if active {
                     text!("100", x=x-8, y=y-35, font="small", color=0xFFD700FF);
                 }
            }

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
        
        // (Floating Texts moved to end)
        
        // Obstacles (Moved here: Before Powerups/Players)
        for o in &self.obstacles {
            if o.respawn_timer > 0 { continue; } 
            let x = o.x as i32; let y = o.y as i32; let w = o.w as i32; let h = o.h as i32;
            if o.kind == 0 { // Bomb
                let bx = x - 1; let by = y - 1;
                circ!(x=bx, y=by, d=w+2, color=0x212121FF); 
                circ!(x=bx+6, y=by+6, d=8, color=0x424242FF); 
                let cx = x + 12;
                rect!(x=cx-3, y=by-2, w=6, h=4, color=0x9E9E9EFF); 
                rect!(x=cx-1, y=by-6, w=2, h=4, color=0x8D6E63FF); 
                if self.frame_count % 10 < 5 {
                    rect!(x=cx-2, y=by-9, w=4, h=4, color=0xFFC107FF); 
                    rect!(x=cx-1, y=by-8, w=2, h=2, color=0xFFFFFFFF); 
                } else {
                     rect!(x=cx-2, y=by-9, w=4, h=4, color=0xFF5722FF); 
                }
            } else if o.kind == 1 { // Wood
                rect!(x=x, y=y, w=w as u32, h=h as u32, color=0xA1887FFF); 
                rect!(x=x, y=y, w=w as u32, h=2, color=0x5D4037FF); 
                rect!(x=x, y=y+h-2, w=w as u32, h=2, color=0x5D4037FF); 
                rect!(x=x, y=y, w=2, h=h as u32, color=0x5D4037FF); 
                rect!(x=x+w-2, y=y, w=2, h=h as u32, color=0x5D4037FF); 
                rect!(x=x+2, y=y+6, w=(w-4) as u32, h=1, color=0x8D6E63FF);
                rect!(x=x+2, y=y+12, w=(w-4) as u32, h=1, color=0x8D6E63FF);
                rect!(x=x+2, y=y+18, w=(w-4) as u32, h=1, color=0x8D6E63FF);
                for i in 2..22 { rect!(x=x+i, y=y+i, w=2, h=2, color=0x3E2723FF); }
                for i in 2..22 { rect!(x=x+w-2-i, y=y+i, w=2, h=2, color=0x3E2723FF); }
                rect!(x=x+2, y=y+2, w=2, h=2, color=0x8D6E63FF); 
                rect!(x=x+w-4, y=y+2, w=2, h=2, color=0x8D6E63FF);
                rect!(x=x+2, y=y+h-4, w=2, h=2, color=0x8D6E63FF);
                rect!(x=x+w-4, y=y+h-4, w=2, h=2, color=0x8D6E63FF);
            } else if o.kind == 2 { // Snowman (Level 4)
                let sx = x; 
                let sy = y;
                // Body (Bottom)
                circ!(x=sx, y=sy+14, d=20, color=0xFFFFFFFF);
                // Body (Mid)
                circ!(x=sx+2, y=sy+6, d=16, color=0xFFFFFFFF);
                // Head (Top)
                circ!(x=sx+5, y=sy-4, d=12, color=0xFFFFFFFF);
                
                // Face
                rect!(x=sx+8, y=sy-2, w=2, h=2, color=0x212121FF); // Eye L
                rect!(x=sx+12, y=sy-2, w=2, h=2, color=0x212121FF); // Eye R
                rect!(x=sx+11, y=sy, w=4, h=2, color=0xFF9800FF); // Carrot
                
                // Hat (Top Hat)
                rect!(x=sx+4, y=sy-6, w=14, h=2, color=0x212121FF); // Brim
                rect!(x=sx+6, y=sy-12, w=10, h=6, color=0x212121FF); // Cap
                
                // Arms
                rect!(x=sx-6, y=sy+8, w=8, h=2, color=0x5D4037FF); // L
                rect!(x=sx+18, y=sy+8, w=8, h=2, color=0x5D4037FF); // R
                
                // Scarf (Red)
                rect!(x=sx+6, y=sy+2, w=10, h=4, color=0xD32F2FFF);
                rect!(x=sx+12, y=sy+4, w=3, h=8, color=0xD32F2FFF);
            }
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

             } else if pu.kind == 1 { 
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
             } else {
                 // Risky Gift (Mystery Box - Purple)
                 let bob = ((self.frame_count / 12) % 2) as i32;
                 let py_anim = py - bob;
                 
                 // Purple Glow
                 circ!(x=px, y=py_anim, d=24, color=0x9C27B044);
                 
                 let w = 14;
                 let h_front = 12;
                 let h_top = 5;
                 let bx = px - w/2; 
                 let by = py_anim - h_front/2;
                 
                 // Box (Purple)
                 rect!(x=bx, y=by, w=w as u32, h=h_front as u32, color=0x7B1FA2FF);
                 rect!(x=bx, y=by - h_top + 1, w=w as u32, h=h_top as u32, color=0x9C27B0FF);
                 
                 // Ribbon (Yellow/Gold)
                 rect!(x=px-2, y=by, w=4, h=h_front as u32, color=0xFFD700FF);
                 rect!(x=px-2, y=by - h_top + 1, w=4, h=h_top as u32, color=0xFFD700FF);
                 rect!(x=bx, y=by - h_top + 3, w=w as u32, h=2, color=0xFFD700FF);
                 
                 // Question Mark
                 text!("?", x=bx+4, y=by+2, font="small", color=0xFFFFFFFF);
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
            
        // (Removed misplaced terrain/shadow code from here)

        // House
        let _house_w = 20;
        let _house_h = 16;
            
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
        
        // HUD Starts Here (Obstacles Loop Removed from here)

        // Draw Floating Score Pops (Moved here: After Players)
        for t in &self.floating_texts {
             // Shadow
             text!(&t.text, x=t.x as i32 - 9, y=t.y as i32 + 1, font="medium", color=0x000000AA);
             text!(&t.text, x=t.x as i32 - 10, y=t.y as i32, font="medium", color=t.color);
        }
        
        // HUD (Dark Text for Light BG)
        // Red Flash on Penalty
        let p1_col = if self.players[0].invuln_timer > 45 { 0xFF0000FF } else { 0xB71C1CFF };
        let p1_text = format!("{}: {}", self.players[0].name, self.players[0].score);
        text!(&p1_text, x=10, y=10, color=p1_col); // Dark Red
        
        let p2_col = if self.players[1].invuln_timer > 45 { 0xFF0000FF } else { 0x0D47A1FF };
        let p2_text = format!("{}: {}", self.players[1].name, self.players[1].score);
        let w = (p2_text.len() * 8) as i32;
        text!(&p2_text, x=502 - w, y=10, color=p2_col); // Dark Blue
        
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
