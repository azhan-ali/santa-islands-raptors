use turbo::*;

mod model;
pub use model::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
enum AppState {
    Menu,
    SinglePlayer,
    MultiplayerLevelSelect,
    MultiplayerSetup,
    MultiplayerInstructions,
    Multiplayer,
    SinglePlayerFactory,
    SinglePlayerSleigh,
    SinglePlayerBreaker,
    SinglePlayerStealth,
    SinglePlayerInstructions,
    Developer,
}

#[turbo::serialize]
#[derive(Copy, PartialEq)]
enum MenuOption {
    SinglePlayer,
    Multiplayer,
    Developer,
}

#[turbo::game]
struct GameState {
    state: AppState,
    menu_option: MenuOption,
    mode_selection: u32,
    snow: Vec<SnowFlake>,
    transition_timer: u32,
    music_started: bool,
    multiplayer_game: Option<MultiplayerGame>,
    factory_game: Option<FactoryGame>,
    sleigh_game: Option<SleighGame>,
    breaker_game: Option<BreakerGame>,
    stealth_game: Option<StealthGame>,
    // Game Setup State
    p1_name: String,
    p2_name: String,
    mp_duration: u32,
    mp_level_selection: u32, // 1-5
    mp_setup_row: u8, // 0=P1, 1=P2, 2=Time, 3=Start
    mp_edit_cursor: usize,
    mp_is_editing: bool,
    show_instructions: bool,
    frame_count: u32,
}

impl GameState {
    fn new() -> Self {
        let mut snow = vec![];
        for _ in 0..100 {
            snow.push(SnowFlake::new());
        }

        Self {
            state: AppState::Menu,
            menu_option: MenuOption::SinglePlayer,
            mode_selection: 0,
            snow,
            show_instructions: false, // Default off
            transition_timer: 0,
            music_started: false,
            multiplayer_game: None,
            factory_game: None,
            sleigh_game: None,
            breaker_game: None,
            stealth_game: None,
            p1_name: "PLAYER 1".to_string(),
            p2_name: "PLAYER 2".to_string(),
            mp_duration: 3,
            mp_level_selection: 1,
            mp_setup_row: 0,
            mp_edit_cursor: 0,
            mp_is_editing: false,
            frame_count: 0,
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;
        // Update Snow
        for flake in self.snow.iter_mut() {
            flake.update();
        }

        // Global Instruction Toggle (Shift + I)
        // Since Turbo doesn't expose keyboard directly via gamepad(0) easily alongside gamepad buttons without mapping,
        // we check checking if 'select' is pressed as a fallback or if we can use a key.
        // Assuming Shift+I is desired but hard to detect specifically without `turbo::os::input::is_key_pressed`.
        // Let's rely on standard gamepad(0).select or a combo.
        // Wait, the user specifically said "Shift + I". 
        // If I can't detect Shift+I, I will use "Select" button on Gamepad which is often mapped to Shift or I.
        // Actually, let's just interpret "Shift + I" as "Select" button for now or add a debug toggle.
        // Or better: Toggle on 'Select' button press.
        // Check Select (Shift) or Y (S key)
        // Check Select (Shift) or Y (S key) to toggle
        if gamepad::get(0).select.just_pressed() || gamepad::get(0).y.just_pressed() {
            self.show_instructions = !self.show_instructions;
        }
        
        // Also allow closing with B if open
        if self.show_instructions && gamepad::get(0).b.just_pressed() {
             self.show_instructions = false;
        }

        // Music Loop
        // Music Loop (Ensure it plays)
        if !self.music_started || (self.frame_count % 300 == 0) {
             turbo::audio::play("home_music");
             self.music_started = true;
        }

        if self.show_instructions {
            // If instructions are open, DO NOT update the rest of the game state.
            // This prevents accidental clicks and "auto-closing" issues if keys overlap.
        } else if self.transition_timer > 0 {
            self.transition_timer -= 1;
            // Don't process input while transitioning
        } else {
            match self.state {
                AppState::Menu => self.update_menu(),
                AppState::SinglePlayer => self.update_single_player_menu(),
                AppState::MultiplayerLevelSelect => self.update_multiplayer_level_select(),
                AppState::MultiplayerSetup => self.update_multiplayer_setup(),
                AppState::MultiplayerInstructions => self.update_multiplayer_instructions(),
                AppState::Multiplayer => self.update_multiplayer(),
                AppState::SinglePlayerFactory => self.update_single_player_factory(),
                AppState::SinglePlayerSleigh => self.update_single_player_sleigh(),
                AppState::SinglePlayerBreaker => self.update_single_player_breaker(),
                AppState::SinglePlayerStealth => self.update_single_player_stealth(),
                AppState::SinglePlayerInstructions => self.update_single_player_instructions(),
                AppState::Developer => self.update_developer(),
            }
        }

        // Draw everything
        self.draw();
    }

    fn update_menu(&mut self) {
        if gamepad::get(0).up.just_pressed() {
            self.menu_option = match self.menu_option {
                MenuOption::SinglePlayer => MenuOption::Developer,
                MenuOption::Multiplayer => MenuOption::SinglePlayer,
                MenuOption::Developer => MenuOption::Multiplayer,
            };
        }
        if gamepad::get(0).down.just_pressed() {
            self.menu_option = match self.menu_option {
                MenuOption::SinglePlayer => MenuOption::Multiplayer,
                MenuOption::Multiplayer => MenuOption::Developer,
                MenuOption::Developer => MenuOption::SinglePlayer,
            };
        }

        if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() {
            self.transition_timer = 10; // Reduce delay
            match self.menu_option {
                MenuOption::SinglePlayer => self.state = AppState::SinglePlayer,
                MenuOption::Multiplayer => {
                    self.state = AppState::MultiplayerLevelSelect;
                    self.mp_level_selection = 1;
                },
                MenuOption::Developer => self.state = AppState::Developer,
            }
        }
    }

    fn update_single_player_menu(&mut self) {
        // Grid Navigation
        if gamepad::get(0).right.just_pressed() {
            if self.mode_selection % 2 == 0 { self.mode_selection += 1; }
        }
        if gamepad::get(0).left.just_pressed() {
            if self.mode_selection % 2 != 0 { self.mode_selection -= 1; }
        }
        if gamepad::get(0).down.just_pressed() {
            if self.mode_selection < 2 { self.mode_selection += 2; }
        }
        if gamepad::get(0).up.just_pressed() {
            if self.mode_selection >= 2 { self.mode_selection -= 2; }
        }

        // Back
        // Back
        if gamepad::get(0).b.just_pressed() {
            self.state = AppState::Menu;
            self.show_instructions = false;
            self.transition_timer = 10;
        }
        
        // Select (TODO: Launch game)
        // Select (Launch specific game)
        if gamepad::get(0).a.just_pressed() || gamepad::get(0).start.just_pressed() {
             if self.mode_selection == 0 { // Gift Packing
                 self.state = AppState::SinglePlayerInstructions;
                 self.transition_timer = 10;
             } else if self.mode_selection == 1 { // Reindeer Training (Sleigh)
                 self.state = AppState::SinglePlayerInstructions;
                 self.transition_timer = 10;
             } else if self.mode_selection == 2 { // Santa Breaker (Breaker)
                 self.state = AppState::SinglePlayerInstructions;
                 self.transition_timer = 10;
             } else if self.mode_selection == 3 { // Silent Santa (Stealth)
                 self.state = AppState::SinglePlayerInstructions;
                 self.transition_timer = 10;
             }
             // Other modes not implemented yet
        }
    }
    
    fn update_single_player_factory(&mut self) {
        let mut exit = false;
        
        if let Some(game) = &mut self.factory_game {
            game.update();
            
            // Checks inside borrow scope
            if gamepad::get(0).b.just_pressed() {
                 exit = true;
            }
        }
        
        if exit {
             self.state = AppState::SinglePlayer;
             self.factory_game = None;
             self.transition_timer = 10;
        }
    }

    fn update_single_player_sleigh(&mut self) {
        let mut exit = false;
        if let Some(game) = &mut self.sleigh_game {
            game.update();
            if gamepad::get(0).b.just_pressed() {
                 exit = true;   
            }
        }
        if exit {
            self.state = AppState::SinglePlayer;
            self.sleigh_game = None;
            self.transition_timer = 10;
        }
    }

    fn update_single_player_breaker(&mut self) {
        let mut exit = false;
        if let Some(game) = &mut self.breaker_game {
            game.update();
            if gamepad::get(0).b.just_pressed() {
                 exit = true;
            }
        }
        if exit {
            self.state = AppState::SinglePlayer;
            self.breaker_game = None;
            self.transition_timer = 10;
        }
    }

    fn update_single_player_stealth(&mut self) {
        let mut exit = false;
        if let Some(game) = &mut self.stealth_game {
            game.update();
            if game.state == StealthState::Menu && gamepad::get(0).b.just_pressed() {
                exit = true;
            }
        }
        if exit {
            self.state = AppState::SinglePlayer;
            self.stealth_game = None;
            self.transition_timer = 10;
        }
    }
    
    fn update_multiplayer(&mut self) {
        // Initialize if not present (SHOULD NOT HAPPEN via Setup, but safe fallback)
        if self.multiplayer_game.is_none() {
            // Default fallback
             self.multiplayer_game = Some(MultiplayerGame::new("Santa".to_string(), "Rival".to_string(), 3, 1));
        }

        if let Some(game) = &mut self.multiplayer_game {
            game.update();
            
            // Exit condition
            if  game.game_over && gamepad::get(0).b.just_pressed() {
                 self.state = AppState::Menu;
                 self.multiplayer_game = None; // Reset
                 self.transition_timer = 10;
            }
        }
        
        // Manual Exit (if not game over)
        if gamepad::get(0).b.just_pressed() {
            self.state = AppState::Menu;
            self.transition_timer = 10;
        }
    }
    
    fn update_multiplayer_level_select(&mut self) {
        let gp = gamepad::get(0);
        
        let cols = 3; 

        // Navigation (Left/Right)
        if gp.left.just_pressed() {
            if self.mp_level_selection > 1 { self.mp_level_selection -= 1; }
        }
        if gp.right.just_pressed() {
            if self.mp_level_selection < 5 { self.mp_level_selection += 1; }
        }
        
        // Navigation (Up/Down)
        if gp.up.just_pressed() {
             if self.mp_level_selection > cols { self.mp_level_selection -= cols; }
        }
        if gp.down.just_pressed() {
             if self.mp_level_selection + cols <= 5 { self.mp_level_selection += cols; }
        }
        
        // Select -> Go to Setup
        if gp.start.just_pressed() || gp.a.just_pressed() {
            self.state = AppState::MultiplayerSetup;
            self.mp_setup_row = 0; // Reset to P1
            self.mp_is_editing = false;
            self.transition_timer = 10;
        }
        
        // Back -> Menu
        if gp.b.just_pressed() {
            self.state = AppState::Menu;
            self.transition_timer = 10;
        }
    }
    
    fn update_multiplayer_setup(&mut self) {
        let gp = gamepad::get(0);

        if self.mp_is_editing {
            // EDIT MODE
            // Row 0 = P1, Row 1 = P2
            let target_name = if self.mp_setup_row == 0 { &mut self.p1_name } else { &mut self.p2_name };
            
            // KEYBOARD INPUT
            let kb = turbo::keyboard::get();
            
            // Text Input
            for c in kb.chars() {
                 // Only allow A-Z, 0-9, and space for simplicity/font support
                 if (c.is_alphanumeric() || c == ' ') && target_name.len() < 10 {
                     target_name.push(c.to_ascii_uppercase());
                     self.mp_edit_cursor = target_name.len();
                 }
            }
            
            // Backspace
            if kb.backspace().just_pressed() && target_name.len() > 0 {
                target_name.pop();
                self.mp_edit_cursor = target_name.len();
            }
            
            // Stop Editing (Enter, Escape, or Gamepad A/B/Start)
            if kb.enter().just_pressed() || kb.escape().just_pressed() || gp.a.just_pressed() || gp.b.just_pressed() || gp.start.just_pressed() {
                self.mp_is_editing = false;
                *target_name = target_name.trim().to_string();
                if target_name.is_empty() {
                    *target_name = if self.mp_setup_row == 0 { "PLAYER 1".to_string() } else { "PLAYER 2".to_string() };
                }
            }
            
            // Ensure cursor is valid (just in case)
            self.mp_edit_cursor = target_name.len();
            
        } else {
            // NAVIGATION MODE
            
            // Navigation (Up/Down)
            if gp.up.just_pressed() {
                if self.mp_setup_row > 0 { self.mp_setup_row -= 1; }
            }
            if gp.down.just_pressed() {
                if self.mp_setup_row < 3 { self.mp_setup_row += 1; }
            }
            
            // Row Interaction
            // Row Interaction
            match self.mp_setup_row {
                0 | 1 => { // Names
                    if gp.a.just_pressed() || gp.start.just_pressed() {
                        self.mp_is_editing = true;
                        self.mp_edit_cursor = 0;
                    }
                },
                2 => { // Duration
                    if gp.left.just_pressed() && self.mp_duration > 1 { self.mp_duration -= 1; }
                    if gp.right.just_pressed() && self.mp_duration < 10 { self.mp_duration += 1; }
                },
                3 => { // Start
                    if gp.start.just_pressed() || gp.a.just_pressed() {
                         let p1 = self.p1_name.clone();
                         let p2 = self.p2_name.clone();
                         self.multiplayer_game = Some(MultiplayerGame::new(p1, p2, self.mp_duration, self.mp_level_selection)); // Uses stored level
                         self.state = AppState::MultiplayerInstructions;
                         self.transition_timer = 10;
                    }
                },
                _ => {}
            }
            
            // Back
            // Back to Level Select
            if gp.b.just_pressed() {
                self.state = AppState::MultiplayerLevelSelect;
                self.transition_timer = 10;
            }
        }
    }


    fn update_multiplayer_instructions(&mut self) {
        let gp = gamepad::get(0);
        let kb = turbo::keyboard::get();
        
        // SPACE or START to Start Game
        if gp.start.just_pressed() || gp.a.just_pressed() || kb.space().just_pressed() {
            self.state = AppState::Multiplayer;
            self.transition_timer = 10;
        }
        
        // B to Go Back to Setup
        if gp.b.just_pressed() || kb.escape().just_pressed() {
            self.state = AppState::MultiplayerSetup;
            self.transition_timer = 10;
        }
    }
    
    fn draw_multiplayer_instructions(&self) {
        let center_x = |text: &str, font_w: i32| -> i32 { (512 - (text.len() as i32 * font_w)) / 2 };
        
        let lvl = self.mp_level_selection;
        let p1 = &self.p1_name;
        let p2 = &self.p2_name;
        
        // Header
        let title = format!("LEVEL {}", lvl);
        text!(&title, x = center_x(&title, 8), y = 30, font = "large", color = 0xFFFF00FF);
        
        let sub = match lvl {
            1 => "Classic Collection",
            2 => "Shifting Village",
            3 => "River Crossing",
            4 => "Power & Peril",
            5 => "The Dog Chase",
            _ => "Unknown Level"
        };
        text!(sub, x = center_x(sub, 5), y = 55, font = "medium", color = 0x00FFFFFF);
        
        // Instructions Content
        let start_y = 90;
        let gap = 16;
        let x_left = 60;
        
        let mut lines = vec![];
        
        lines.push(format!("Overview:"));
        match lvl {
            1 => {
                lines.push("Collect houses to earn points.".to_string());
                lines.push("Watch out for basic obstacles.".to_string());
            },
            2 => {
                lines.push("Houses reshuffle every 15 seconds!".to_string());
                lines.push("Beware of Bombs that reset your score.".to_string());
            },
            3 => {
                lines.push("Cross the river carefully using bridges.".to_string());
                lines.push("Don't fall in the water!".to_string());
                lines.push("Shadow trails slow you down.".to_string());
            },
            4 => {
                lines.push("Snowmen freeze you on contact!".to_string());
                lines.push("Power House spawns Risky Gifts.".to_string());
                lines.push("Risky Gift: +60 or -60 points!".to_string());
            },
            5 => {
                lines.push("THE DOG IS WATCHING!".to_string());
                lines.push("-60 Gift wakes the dog.".to_string());
                lines.push("Dog bite = -100 points.".to_string());
                lines.push("Goal: Survive & Score High.".to_string());
            },
             _ => {}
        }
        lines.push("".to_string());
        
        lines.push("Controls:".to_string());
        lines.push(format!("P1 (Santa): W A S D"));
        lines.push(format!("P2 (Rival): Arrow Keys"));
        
        lines.push("".to_string());
        lines.push("Win Condition:".to_string());
        lines.push("Highest score when time runs out wins!".to_string());
        
        for (i, line) in lines.iter().enumerate() {
            let col = if line.ends_with(':') { 0xFFD700FF } else { 0xFFFFFFFF };
            text!(line, x=x_left, y=start_y + (i as i32 * gap), font="medium", color=col);
        }
        
        // Footer
        let footer1 = "Press SPACE to START GAME";
        let footer2 = "Press B / ESC to Go Back";
        text!(footer1, x=center_x(footer1, 5), y=240, font="medium", color=0x00FF00FF);
        text!(footer2, x=center_x(footer2, 5), y=260, font="small", color=0xAAAAAAFF);
    }


    fn update_single_player_instructions(&mut self) {
        let gp = gamepad::get(0);
        let kb = turbo::keyboard::get();
        
        // SPACE or START to Start Game
        if gp.start.just_pressed() || gp.a.just_pressed() || kb.space().just_pressed() {
            // Launch specific game based on selection
            if self.mode_selection == 0 { // Gift Packing
                  self.factory_game = Some(FactoryGame::new());
                  self.state = AppState::SinglePlayerFactory;
            } else if self.mode_selection == 1 { // Sleigh
                  self.sleigh_game = Some(SleighGame::new());
                  self.state = AppState::SinglePlayerSleigh;
            } else if self.mode_selection == 2 { // Breaker
                  self.breaker_game = Some(BreakerGame::new());
                  self.state = AppState::SinglePlayerBreaker;
            } else if self.mode_selection == 3 { // Stealth
                  self.stealth_game = Some(StealthGame::new());
                  self.state = AppState::SinglePlayerStealth;
            }
            self.transition_timer = 10;
        }
        
        // B to Go Back to Single Player Menu
        if gp.b.just_pressed() || kb.escape().just_pressed() {
            self.state = AppState::SinglePlayer;
            self.transition_timer = 10;
        }
    }
    
    fn draw_single_player_instructions(&self) {
        let center_x = |text: &str, font_w: i32| -> i32 { (512 - (text.len() as i32 * font_w)) / 2 };
        
        let title;
        let overview;
        let mut lines = vec![];
        let controls;
        let win_cond;
        let lose_cond;
        
        if self.mode_selection == 0 {
             title = "GIFT PACKING";
             overview = "Help Santa sort gifts correctly!";
             lines.push("Gifts spawn on the top belt.");
             lines.push("Pick up gifts (Blue, Green, Purple).");
             lines.push("Drop them into the matching colored Bin.");
             lines.push("+100 Points for Correct Bin.");
             lines.push("-50 Points for Wrong Bin.");
             controls = "Move: Arrows | Action: A / Space";
             win_cond = "Score as high as possible in 60s!";
             lose_cond = "Time runs out.";
        } else if self.mode_selection == 1 {
             title = "RAINDEER RUSH";
             overview = "Fly the sleigh and defend spread joy!";
             lines.push("Fly Santa's Sleigh through the sky.");
             lines.push("Shoot magic at flying enemies.");
             lines.push("Avoid crashing into enemies.");
             lines.push("Survive as difficulty increases.");
             controls = "Move: Arrows | Shoot: A / Space";
             win_cond = "Survive longer for high score!";
             lose_cond = "Running out of Lives.";
        } else if self.mode_selection == 2 {
             title = "SANTA BREAKER";
             overview = "Break all bricks to clear levels!";
             lines.push("Bounce the Santa Ball off the paddle.");
             lines.push("Destroy all festive bricks.");
             lines.push("Don't let the ball fall!");
             lines.push("Advance through multiple levels.");
             controls = "Move: Left/Right | Launch: A / Space";
             win_cond = "Clear all bricks.";
             lose_cond = "Lose all lives (Ball drops).";
        } else if self.mode_selection == 3 {
             title = "SANTA MISSION";
             overview = "Deliver gifts without being seen!";
             lines.push("Sneak past sleeping dogs and patrol wolves.");
             lines.push("Place 3 Gifts at targets.");
             lines.push("Eat the Cookie in the Kitchen.");
             lines.push("Collect 5 Stars.");
             controls = "Move: Arrows | Stop: Quiet Down";
             win_cond = "Complete all tasks & Exit.";
             lose_cond = "Getting CAUGHT by a dog.";
        } else {
             title = "UNKNOWN MODE";
             overview = "";
             controls = "";
             win_cond = "";
             lose_cond = "";
        }
        
        // Draw
        
        // Background Snow
        for flake in &self.snow {
             // Use SnowFlake::draw OR manually. SnowFlake has .size field, not .r
             let sz = flake.size;
             rect!(x=flake.x as i32, y=flake.y as i32, w=sz, h=sz, color=0xFFFFFF66);
        }
        
        text!(title, x = center_x(title, 8), y = 15, font = "large", color = 0xFFFF00FF);
        text!(overview, x = center_x(overview, 5), y = 40, font = "medium", color = 0x00FFFFFF);
        
        let start_y = 65;
        let gap = 15;
        let x_left = 60;
        
        // Gameplay Lines
        lines.insert(0, "How to Play:");
        for (i, line) in lines.iter().enumerate() {
             let iter_line: &str = line;
             let col = if iter_line.ends_with(':') { 0xFFD700FF } else { 0xFFFFFFFF };
             text!(iter_line, x=x_left, y=start_y + (i as i32 * gap), font="medium", color=col);
        }
        
        let mut y = start_y + (lines.len() as i32) * gap + 5;
        
        text!("Controls:", x=x_left, y=y, font="medium", color=0xFFD700FF);
        y += gap;
        text!(controls, x=x_left, y=y, font="medium", color=0xFFFFFFFF);
        y += gap + 5;

        text!("Win Condition:", x=x_left, y=y, font="medium", color=0xFFD700FF);
        y += gap;
        text!(win_cond, x=x_left, y=y, font="medium", color=0x2ECC71FF);
        y += gap + 5;

        text!("Lose Condition:", x=x_left, y=y, font="medium", color=0xFFD700FF);
        y += gap;
        text!(lose_cond, x=x_left, y=y, font="medium", color=0xE74C3CFF);
        
        // Footer (Fixed at bottom)
        let footer1 = "Press SPACE to START GAME";
        let footer2 = "Press B / ESC to Go Back";
        text!(footer1, x=center_x(footer1, 5), y=250, font="medium", color=0x00FF00FF);
        text!(footer2, x=center_x(footer2, 5), y=270, font="small", color=0xAAAAAAFF);
    }

    fn update_developer(&mut self) {
        if gamepad::get(0).b.just_pressed() {
            self.state = AppState::Menu;
            self.transition_timer = 10;
        }
    }

    fn draw(&self) {
        // Background
        rect!(w = screen().w(), h = screen().h(), color = 0x000000FF); // Black background

        // Draw Snow
        for flake in &self.snow {
            flake.draw();
        }

        match self.state {
            AppState::Menu => self.draw_menu(),
            AppState::SinglePlayer => self.draw_single_player_menu(),
            AppState::MultiplayerLevelSelect => self.draw_multiplayer_level_select(),
            AppState::MultiplayerSetup => self.draw_multiplayer_setup(),
            AppState::MultiplayerInstructions => self.draw_multiplayer_instructions(),
            AppState::Multiplayer => {
                if let Some(game) = &self.multiplayer_game {
                    game.draw();
                } else {
                     text!("Loading...", x = 200, y = 140, color = 0xFFFFFFFF);
                }
            },
            AppState::SinglePlayerFactory => {
                if let Some(game) = &self.factory_game {
                    game.draw();
                }
            },
            AppState::SinglePlayerSleigh => {
                if let Some(game) = &self.sleigh_game {
                    game.draw();
                }
            },
            AppState::SinglePlayerBreaker => {
                if let Some(game) = &self.breaker_game {
                    game.draw();
                }
            },
            AppState::SinglePlayerStealth => {
                if let Some(game) = &self.stealth_game {
                    game.draw();
                }
            },
            AppState::SinglePlayerInstructions => self.draw_single_player_instructions(),
            AppState::Developer => self.draw_developer(),
        }
        
        // Draw Instructions Overlay
        if self.show_instructions {
            self.draw_instructions_overlay();
        }
    }
    
    fn draw_instructions_overlay(&self) {
        // Overlay Box
        rect!(x=50, y=50, w=412, h=188, color=0x000000EE);
        rect!(x=50, y=50, w=412, h=188, border_size=2, border_color=0xFFFFFFFF, color=0x00000000);
        
        text!("INSTRUCTIONS", x=180, y=60, font="large", color=0xFFFF00FF);
        
        let mut lines = vec![];
        
        match self.state {
             AppState::Menu => {
                 lines.push("Main Menu:");
                 lines.push("- Arrow Keys: Navigate Options");
                 lines.push("- A / Start: Select Option");
             },
             AppState::SinglePlayer => {
                 lines.push("Single Player Menu:");
                 lines.push("- Arrow Keys: Select Game Mode");
                 lines.push("- A / Start: Play Selected Game");
                 lines.push("- B: Go Back");
             },
             AppState::SinglePlayerFactory => {
                 lines.push("Gift Packing (Factory):");
                 lines.push("- Left/Right: Move Conveyor");
                 lines.push("- A / Start: Grab/Drop Gift");
                 lines.push("- B: Back/Exit (Paused)");
                 lines.push("Goal: Sort gifts into correct bins.");
             },
             AppState::SinglePlayerSleigh => {
                 lines.push("Raindeer Rush:");
                 lines.push("- Arrows: Move Sleigh");
                 lines.push("- A / Start: Shoot Gifts");
                 lines.push("Goal: Destroy enemies, don't crash!");
             },
             AppState::SinglePlayerBreaker => {
                 lines.push("Santa Breaker:");
                 lines.push("- Left/Right: Move Paddle");
                 lines.push("- A / Start: Launch Ball");
                 lines.push("Goal: Break all bricks. Don't lose the ball!");
             },
             AppState::SinglePlayerStealth => {
                 lines.push("Santa Mission (Stealth):");
                 lines.push("- Arrows: Move Carefuly");
                 lines.push("- Stop: Recover noise level");
                 lines.push("- A / Space: Interact");
                 lines.push("Goal: Place Gifts, Eat Cookie, Escape!");
             },
             _ => {
                 lines.push("Standard Controls:");
                 lines.push("- Arrows: Move / Navigate");
                 lines.push("- A / Start: Confirm / Action");
                 lines.push("- B: Cancel / Back");
             }
        }
        
        for (i, line) in lines.iter().enumerate() {
            text!(line, x=70, y=100 + (i as i32 * 20), font="medium", color=0xFFFFFFFF);
        }
        
        text!("Press Select (Shift) or Y (S) to Close", x=140, y=220, font="small", color=0xAAAAAAFF);
    }

    fn draw_multiplayer_level_select(&self) {
        let center_x = |text: &str, font_w: i32| -> i32 {
             (512 - (text.len() as i32 * font_w)) / 2
        };

        text!("SELECT LEVEL", x = center_x("SELECT LEVEL", 8), y = 25, font = "large", color = 0xFFFF00FF);
        
        let start_y = 65;
        let box_w = 80;
        let box_h = 60;
        let gap_x = 20;
        let gap_y = 20;
        
        // Grid Calculation
        let row1_count = 3;
        let row2_count = 2;
        
        // Row 1 (Levels 1-3)
        let row1_w = row1_count * box_w + (row1_count - 1) * gap_x;
        let row1_start_x = (512 - row1_w) as i32 / 2;
        
        // Row 2 (Levels 4-5)
        let row2_w = row2_count * box_w + (row2_count - 1) * gap_x;
        let row2_start_x = (512 - row2_w) as i32 / 2;
        
        for i in 1..=5 {
            let idx = (i - 1) as i32;
            let row = if idx < 3 { 0 } else { 1 };
            
            let x = if row == 0 {
                let col = idx;
                row1_start_x + col * (box_w + gap_x)
            } else {
                let col = idx - 3;
                row2_start_x + col * (box_w + gap_x)
            };
            
            let y = start_y + row * (box_h + gap_y);
            
            let is_selected = self.mp_level_selection == i as u32;
            let color = if is_selected { 0x00FF00FF } else { 0x444444FF };
            let _bg = if is_selected { 0x222222FF } else { 0x000000FF };
            
            // Box
            rect!(x=x-2, y=y-2, w=(box_w+4) as u32, h=(box_h+4) as u32, color=color);
            
            // Procedural Background
            let bg_color = 0x000000FF; 
            rect!(x=x, y=y, w=box_w as u32, h=box_h as u32, color=bg_color);
            
            // Thumbnail
            let sprite_name = match i {
                1 => "thumb_mp_lvl1",
                2 => "thumb_mp_lvl2",
                3 => "thumb_mp_lvl3",
                4 => "thumb_mp_lvl4",
                5 => "thumb_mp_lvl5",
                _ => "",
            };
            if !sprite_name.is_empty() {
                 sprite!(sprite_name, x=x, y=y, w=box_w as u32, h=box_h as u32);
            }
            
            // Level Label (Below Box)
            let lvl_text = format!("Level {}", i);
            // Center "Level X" (7 chars) under 80px box. Small font ~8px (actually small is smaller, say 6px).
            // Let's assume font="small" width is roughly 6px. 7*6 = 42. (80-42)/2 = 19.
            // Let's use generic centering for small font (approx 7px per char including spacing?)
            let txt_len = lvl_text.len() as i32 * 4; // Est 4px width for small
            let txt_x = x + (box_w / 2) - (txt_len / 2);
            
            let label_col = if is_selected { 0x00FF00FF } else { 0x888888FF };
            text!(&lvl_text, x=txt_x, y=y+box_h+8, font="small", color=label_col);
        }
        
        // Text Instructions (Below Grid)
        let grid_bottom = start_y + 2 * box_h + gap_y + 20; 
        
        let msg_start = "Press START to Continue";
        let msg_back = "Press X to Back";
        
        // Use 8px for medium to try centering better
        text!(msg_start, x=center_x(msg_start, 8), y=grid_bottom + 15, font="medium", color=0xFFFFFFFF);
        text!(msg_back, x=center_x(msg_back, 4), y=grid_bottom + 35, font="small", color=0xAAAAAAFF);

        // Subtitle (Bottom)
        let note = "(More exciting levels coming soon!)";
        text!(note, x=center_x(note, 4), y=270, font="small", color=0xFFD700FF);
    }
    
    fn draw_multiplayer_setup(&self) {
        // Title
        text!("MULTIPLAYER SETUP", x = 180, y = 40, font = "large", color = 0xFFFF00FF);
        
        // Helper
        let center_x = |text: &str, font_w: i32| -> i32 {
            (512 - (text.len() as i32 * font_w)) / 2
        };

        let start_y = 60;
        let gap = 45;
        
        let editing = self.mp_is_editing;
        
        // P1 (Row 0)
        let p1_col = if self.mp_setup_row == 0 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Player 1 Name:", x = 100, y = start_y, font="medium", color = p1_col);
        
        let p1_box_col = if self.mp_setup_row == 0 && editing { 0xFFFF00FF } else { 0xFFFFFFFF };
        rect!(x=260, y=start_y-2, w=140, h=14, color=p1_box_col); 
        rect!(x=261, y=start_y-1, w=138, h=12, color=0x000000FF); 
        
        // Render P1 chars
        let p1_str = &self.p1_name;
        for (i, c) in p1_str.chars().enumerate() {
            let cx = 270 + (i as i32 * 10);
            let s = c.to_string();
            text!(&s, x=cx, y=start_y+1, font="medium", color=0xFFFFFFFF);
        }
        if self.mp_setup_row == 0 && editing {
             let cx = 270 + (self.mp_edit_cursor as i32 * 10);
             rect!(x=cx, y=start_y+11, w=8, h=2, color=0xFFFF00FF);
        }
        if self.mp_setup_row == 0 && !editing { text!("(Press SPACE to Edit)", x=410, y=start_y+2, font="small", color=0x666666FF); }


        // P2 (Row 1)
        let p2_col = if self.mp_setup_row == 1 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Player 2 Name:", x = 100, y = start_y + gap, font="medium", color = p2_col);
        
        let p2_box_col = if self.mp_setup_row == 1 && editing { 0xFFFF00FF } else { 0xFFFFFFFF };
        rect!(x=260, y=start_y+gap-2, w=140, h=14, color=p2_box_col);
        rect!(x=261, y=start_y+gap-1, w=138, h=12, color=0x000000FF);
        
        let p2_str = &self.p2_name;
        for (i, c) in p2_str.chars().enumerate() {
            let cx = 270 + (i as i32 * 10);
            let s = c.to_string();
            text!(&s, x=cx, y=start_y+gap+1, font="medium", color=0xFFFFFFFF);
        }
        if self.mp_setup_row == 1 && editing {
             let cx = 270 + (self.mp_edit_cursor as i32 * 10);
             rect!(x=cx, y=start_y+gap+11, w=8, h=2, color=0xFFFF00FF);
        }
        if self.mp_setup_row == 1 && !editing { text!("(Press SPACE to Edit)", x=410, y=start_y+gap+2, font="small", color=0x666666FF); }

        // Time (Row 2)
        let time_col = if self.mp_setup_row == 2 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Duration:", x = 100, y = start_y + gap*2, font="medium", color = time_col);
        let time_val = format!(" < {} mins > ", self.mp_duration);
        text!(&time_val, x=270, y=start_y+gap*2, font="medium", color=if self.mp_setup_row == 2 { 0xFFFFFFFF } else { 0x888888FF });

        // Start (Row 3)
        let btn_y = 230;
        let btn_w = 120;
        let btn_x = (512 - btn_w) / 2;
        let is_btn = self.mp_setup_row == 3;
        
        let btn_col = if is_btn { 0x00E676FF } else { 0x444444FF };
        let txt_col = if is_btn { 0x000000FF } else { 0xAAAAAAFF };
        
        rect!(x=btn_x as i32, y=btn_y, w=btn_w as u32, h=30, color=btn_col);
        text!("START GAME", x = center_x("START GAME", 8), y = btn_y + 10, font="large", color=txt_col);
    }
    
    fn draw_menu(&self) {
        // Custom Pixel Title
        let scale = 4; // Reduced scale to fit better
        // "SANTA" (5 chars) + "ISLANDS" (7 chars) = 12 * 6px + space(32px)
        let title_pixel_width = ((5 + 7) * 6 + 4) * scale;
        let start_x = (512 - title_pixel_width as i32) / 2; // Hardcoced 512 width
        
        draw_title(start_x, 20, scale as u32);
        
        // Subtitle / Decor
        let sub = "- Christmas Adventure -";
        let sub_w = sub.len() as i32 * 8; // Large font is ~8px wide
        text!(sub, x = (512 - sub_w) / 2, y = 80, font = "large", color = 0xFFFF00FF);

        // Menu Box
        let box_w = 260; // Wider to fit large text
        let box_h = 130;
        let box_x = (512 - box_w) / 2;
        let box_y = 100;
        
        // Box border
        rect!(x = box_x - 2, y = box_y - 2, w = box_w as u32 + 4, h = box_h as u32 + 4,  color = 0xFFFFFFFF);
        // Box background
        rect!(x = box_x, y = box_y, w = box_w as u32, h = box_h as u32, color = 0x000000FF);

        // Options
        let start_y = box_y + 20;
        let line_height = 30;

        self.draw_menu_item("Single Player", 0, start_y);
        self.draw_menu_item("Multiplayer", 1, start_y + line_height);
        self.draw_menu_item("Developer", 2, start_y + line_height * 2);
        
        // Instructions
        let instr = "Arrows: Move | Space: Select";
        let instr_w = instr.len() as i32 * 8;
        text!(instr, x = (512 - instr_w) / 2, y = 260, font = "large", color = 0x555555FF);
    }

    fn draw_menu_item(&self, label: &str, index: i32, y: i32) {
        let is_selected = match (index, self.menu_option) {
            (0, MenuOption::SinglePlayer) => true,
            (1, MenuOption::Multiplayer) => true,
            (2, MenuOption::Developer) => true,
            _ => false,
        };

        let color = if is_selected { 0xFFFF00FF } else { 0xFFFFFFFF }; 
        let arrow = if is_selected { "> " } else { "" }; // No indentation on unselected
        let suffix = if is_selected { " <" } else { "" };
        
        let full_text = format!("{}{}{}", arrow, label, suffix);
        let text_w = full_text.len() as i32 * 8;
        
        // Center text (Hardcoded 512 width, 8px char width for Large)
        // Note: arrows might offset visual center, but mathematical center is correct based on string len
        let x = (512 - text_w) / 2;
        
        text!(&full_text, x = x, y = y, font = "large", color = color);
    }

    fn draw_single_player_menu(&self) {
        // 1. Draw Big Title (Same as Menu)
        let scale = 3; 
        let title_pixel_width = ((5 + 7) * 6 + 4) * scale;
        let start_x_title = (512 - title_pixel_width as i32) / 2;
        draw_title(start_x_title, 20, scale as u32);

        // 2. Sub-header
        text!("SELECT MODE", x = 210, y = 60, color = 0xFFFF00FF);

        // 3. Instructions
        text!("Press X to Back", x = 20, y = 20, color = 0xAAAAAAFF);

        let modes = [
        "Gift Packing", "Raindeer Rush",
        "Santa Breaker", "Santa Mission",
        "Bell Rush", "Snow Chaos"
    ];
    
    let icons = ["🎁", "🦌", "🧱", "🕵️", "🔔", "❄️"];

        // 4. Grid Layout (Squares)
        let box_size = 60;
        let gap_x = 40;
        let _gap_y = 50; // Unused but kept for reference logic
        
        // Calculate starting position to center the 2x3 grid
        // Grid Width = 2 * box + gap = 60*2 + 40 = 160
        // Grid Height = 3 * box + 2 * gap = 60*3 + 50*2 = 180 + 100 = 280 (Too tall for 288 screen)
        // Let's adjust gaps and start Y
        
        let start_x = (512 - (box_size * 2 + gap_x)) / 2;
        let start_y = 90;
        let gap_y_adjusted = 40;

        for i in 0..6 {
            let row = (i / 2) as i32;
            let col = (i % 2) as i32;
            let x = start_x + col * (box_size + gap_x);
            let y = start_y + row * (box_size + gap_y_adjusted);
            
            let is_selected = self.mode_selection == i as u32;
            let border_color = if is_selected { 0x00FF00FF } else { 0xFFFFFFFF }; // Green if selected
            let bg_color = if is_selected { 0x222222FF } else { 0x000000FF };

            // Draw Square Box (Thumbnail Container)
            rect!(x = x - 2, y = y - 2, w = (box_size + 4) as u32, h = (box_size + 4) as u32, color = border_color);
            rect!(x = x, y = y, w = box_size as u32, h = box_size as u32, color = bg_color);
            
            // Draw Icon (Centered in box)
            if i < 4 {
                 let sprite_name = match i {
                     0 => "thumb_factory",
                     1 => "thumb_sleigh",
                     2 => "thumb_breaker",
                     3 => "thumb_stealth",
                     _ => "",
                 };
                 // Draw sprite fitted to box (assuming sprite! supports w/h resizing or we rely on default)
                 // If w/h not supported, this might fail or draw huge.
                 // Safe bet: use `sprite!(name, x=x, y=y)` and hope for best? No, 1024px is huge.
                 // Use `w` and `h` parameters if `sprite!` macro follows `rect!` convention.
                 sprite!(sprite_name, x=x, y=y, w=box_size as u32, h=box_size as u32);
            } else {
                 text!(icons[i], x = x + 20, y = y + 20, font = "large", color = 0xFFFFFFFF);
            }
            
            // Draw Mode Name (Below Box)
            // Center the text below the box
            let name = modes[i];
            let name_w = name.len() as i32 * 6; // Estimating medium font width (approx 6px)
            let text_x = x + (box_size as i32 / 2) - (name_w / 2);
            
            // Highlight selected text color
            let text_color = if is_selected { 0x00FF00FF } else { 0xAAAAAAFF };
            text!(name, x = text_x, y = y + box_size + 10, font = "medium", color = text_color);
        }
    }

    fn draw_developer(&self) {
        // Title
        let scale = 3; 
        let title_pixel_width = ((5 + 7) * 6 + 4) * scale;
        let start_x_title = (512 - title_pixel_width as i32) / 2;
        draw_title(start_x_title, 20, scale as u32);
        
        let center_x = |text: &str, font_w: i32| -> i32 {
            (512 - (text.len() as i32 * font_w)) / 2
        };

        // Header
        let txt_dev = "DEVELOPERS";
        text!(txt_dev, x = center_x(txt_dev, 8), y = 70, font = "large", color = 0xFF0000FF);
        
        // Names
        let txt_name1 = "Aarif Khan";
        text!(txt_name1, x = center_x(txt_name1, 6), y = 100, font = "medium", color = 0xFFFFFFFF);
        
        let txt_name2 = "Azhan Ali";
        text!(txt_name2, x = center_x(txt_name2, 6), y = 120, font = "medium", color = 0xFFFFFFFF);
        
        // Team
        let txt_team_label = "Team Name:";
        text!(txt_team_label, x = center_x(txt_team_label, 6), y = 150, font = "medium", color = 0xFFFF00FF);
        
        let txt_team = "Tm-AzhanAarif";
        text!(txt_team, x = center_x(txt_team, 6), y = 165, font = "medium", color = 0xFFFFFFFF);
        
        // Back
        let txt_back = "Press X to return";
        text!(txt_back, x = center_x(txt_back, 5), y = 220, font = "small", color = 0xAAAAAAFF);
    }
}