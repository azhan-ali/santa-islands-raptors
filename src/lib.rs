use turbo::*;

mod model;
pub use model::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
enum AppState {
    Menu,
    SinglePlayer,
    MultiplayerSetup,
    Multiplayer,
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
    // Game Setup State
    p1_name: String,
    p2_name: String,
    mp_duration: u32,
    mp_setup_row: u8, // 0=P1, 1=P2, 2=Time, 3=Start
    mp_edit_cursor: usize,
    mp_is_editing: bool,
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
            transition_timer: 0,
            music_started: false,
            multiplayer_game: None,
            p1_name: "PLAYER 1".to_string(),
            p2_name: "PLAYER 2".to_string(),
            mp_duration: 3,
            mp_setup_row: 0,
            mp_edit_cursor: 0,
            mp_is_editing: false,
        }
    }

    fn update(&mut self) {
        // Update Snow
        for flake in self.snow.iter_mut() {
            flake.update();
        }

        // Music Loop
        if !self.music_started {
             turbo::audio::play("home_music");
             self.music_started = true;
        }

        if self.transition_timer > 0 {
            self.transition_timer -= 1;
            // Don't process input while transitioning
        } else {
            match self.state {
                AppState::Menu => self.update_menu(),
                AppState::SinglePlayer => self.update_single_player_menu(),
                AppState::MultiplayerSetup => self.update_multiplayer_setup(),
                AppState::Multiplayer => self.update_multiplayer(),
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
                    self.state = AppState::MultiplayerSetup;
                    self.mp_setup_row = 0; // Reset cursor
                    self.mp_is_editing = false;
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
            if self.mode_selection < 4 { self.mode_selection += 2; }
        }
        if gamepad::get(0).up.just_pressed() {
            if self.mode_selection >= 2 { self.mode_selection -= 2; }
        }

        // Back
        if gamepad::get(0).b.just_pressed() {
            self.state = AppState::Menu;
            self.transition_timer = 10;
        }
        
        // Select (TODO: Launch game)
        if gamepad::get(0).a.just_pressed() || gamepad::get(0).start.just_pressed() {
             // For now just Log or do nothing, waiting for game implementation
        }
    }
    
    fn update_multiplayer(&mut self) {
        // Initialize if not present (SHOULD NOT HAPPEN via Setup, but safe fallback)
        if self.multiplayer_game.is_none() {
            // Default fallback
             self.multiplayer_game = Some(MultiplayerGame::new("Santa".to_string(), "Rival".to_string(), 3));
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
    
    fn update_multiplayer_setup(&mut self) {
        let gp = gamepad::get(0);

        if self.mp_is_editing {
            // EDIT MODE
            let target_name = if self.mp_setup_row == 0 { &mut self.p1_name } else { &mut self.p2_name };
            
            // Cursor Move
            if gp.left.just_pressed() {
                if self.mp_edit_cursor > 0 { self.mp_edit_cursor -= 1; }
            }
            if gp.right.just_pressed() {
                if self.mp_edit_cursor < 9 { // Max length 10
                    if self.mp_edit_cursor >= target_name.len() {
                         target_name.push(' '); // Auto extend
                    }
                    self.mp_edit_cursor += 1;
                }
            }
            
            // Be sure string is long enough
            while target_name.len() <= self.mp_edit_cursor {
                target_name.push(' ');
            }
            target_name.truncate(10); // Hard limit

            // Character Change
            if gp.up.just_pressed() || gp.down.just_pressed() {
                let mut chars: Vec<char> = target_name.chars().collect();
                let mut c = chars[self.mp_edit_cursor];
                // Cycle: Space -> A..Z -> Space
                // ASCII: Space=32, A=65, Z=90
                let delta = if gp.up.just_pressed() { 1 } else { -1 };
                
                let mut next_byte = c as i16 + delta;
                if next_byte < 32 { next_byte = 90; } // Wrap to Z
                else if next_byte > 90 { next_byte = 32; } // Wrap to Space
                else if next_byte > 32 && next_byte < 65 { 
                     // Skip non-alphas
                     if delta > 0 { next_byte = 65; } else { next_byte = 32; }
                }
                
                c = next_byte as u8 as char;
                chars[self.mp_edit_cursor] = c;
                *target_name = chars.into_iter().collect();
            }
            
            // Stop Editing
            if gp.b.just_pressed() || gp.start.just_pressed() || gp.a.just_pressed() {
                self.mp_is_editing = false;
                // Trim trailing spaces
                *target_name = target_name.trim().to_string();
                if target_name.is_empty() {
                    *target_name = if self.mp_setup_row == 0 { "PLAYER 1".to_string() } else { "PLAYER 2".to_string() };
                }
            }
            
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
                         self.multiplayer_game = Some(MultiplayerGame::new(p1, p2, self.mp_duration));
                         self.state = AppState::Multiplayer;
                         self.transition_timer = 10;
                    }
                },
                _ => {}
            }
            
            // Back
            if gp.b.just_pressed() {
                self.state = AppState::Menu;
                self.transition_timer = 10;
            }
        }
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
            AppState::MultiplayerSetup => self.draw_multiplayer_setup(),
            AppState::Multiplayer => {
                if let Some(game) = &self.multiplayer_game {
                    game.draw();
                } else {
                     text!("Loading...", x = 200, y = 140, color = 0xFFFFFFFF);
                }
            },
            AppState::Developer => self.draw_developer(),
        }
    }
    
    fn draw_multiplayer_setup(&self) {
        // Title
        text!("MULTIPLAYER SETUP", x = 180, y = 40, font = "large", color = 0xFFFF00FF);
        
        // Helper
        let center_x = |text: &str, font_w: i32| -> i32 {
            (512 - (text.len() as i32 * font_w)) / 2
        };

        let start_y = 90;
        let gap = 50;
        
        let editing = self.mp_is_editing;
        
        // P1
        let p1_col = if self.mp_setup_row == 0 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Player 1 Name:", x = 100, y = start_y, font="medium", color = p1_col);
        
        // Box P1
        let p1_box_col = if self.mp_setup_row == 0 && editing { 0xFFFF00FF } else { 0xFFFFFFFF };
        rect!(x=260, y=start_y-2, w=140, h=14, color=p1_box_col); 
        rect!(x=261, y=start_y-1, w=138, h=12, color=0x000000FF); 
        
        // Render P1 chars
        let p1_str = &self.p1_name;
        for (i, c) in p1_str.chars().enumerate() {
            let cx = 270 + (i as i32 * 10);
            let s = c.to_string();
            text!(&s, x=cx, y=start_y+1, font="medium", color=0xFFFFFFFF);
            // Cursor
            if self.mp_setup_row == 0 && editing && i == self.mp_edit_cursor {
                 rect!(x=cx, y=start_y+11, w=8, h=2, color=0xFFFF00FF);
            }
        }
        if self.mp_setup_row == 0 && !editing { text!("(Press SPACE to Edit)", x=410, y=start_y+2, font="small", color=0x666666FF); }


        // P2
        let p2_col = if self.mp_setup_row == 1 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Player 2 Name:", x = 100, y = start_y + gap, font="medium", color = p2_col);
        
        // Box P2
        let p2_box_col = if self.mp_setup_row == 1 && editing { 0xFFFF00FF } else { 0xFFFFFFFF };
        rect!(x=260, y=start_y+gap-2, w=140, h=14, color=p2_box_col);
        rect!(x=261, y=start_y+gap-1, w=138, h=12, color=0x000000FF);
        
        let p2_str = &self.p2_name;
        for (i, c) in p2_str.chars().enumerate() {
            let cx = 270 + (i as i32 * 10);
            let s = c.to_string();
            text!(&s, x=cx, y=start_y+gap+1, font="medium", color=0xFFFFFFFF);
            // Cursor
            if self.mp_setup_row == 1 && editing && i == self.mp_edit_cursor {
                 rect!(x=cx, y=start_y+gap+11, w=8, h=2, color=0xFFFF00FF);
            }
        }
        if self.mp_setup_row == 1 && !editing { text!("(Press SPACE to Edit)", x=410, y=start_y+gap+2, font="small", color=0x666666FF); }

        // Time
        let time_col = if self.mp_setup_row == 2 { 0x00FF00FF } else { 0xAAAAAAFF };
        text!("Duration:", x = 100, y = start_y + gap*2, font="medium", color = time_col);
        let time_val = format!(" < {} mins > ", self.mp_duration);
        text!(&time_val, x=270, y=start_y+gap*2, font="medium", color=if self.mp_setup_row == 2 { 0xFFFFFFFF } else { 0x888888FF });

        // Start
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
            "Gift Packing", "Reindeer Training",
            "Chimney Jump", "Cookie Madness",
            "Bell Rush", "Snow Chaos"
        ];
        
        let icons = ["🎁", "🦌", "🏠", "🍪", "🔔", "❄️"];

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
            // Icon is text, so we estimate position
            text!(icons[i], x = x + 20, y = y + 20, font = "large", color = 0xFFFFFFFF);
            
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