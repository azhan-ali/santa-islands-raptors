use turbo::*;

mod model;
pub use model::*;

#[turbo::serialize]
#[derive(Copy, PartialEq)]
enum AppState {
    Menu,
    SinglePlayer,
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
    // Add other game state fields here later
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
                AppState::Multiplayer => self.update_multiplayer(), // Placeholder
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
                MenuOption::Multiplayer => self.state = AppState::Multiplayer,
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
        // Initialize if not present
        if self.multiplayer_game.is_none() {
            self.multiplayer_game = Some(MultiplayerGame::new());
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
            let name_w = name.len() as i32 * 5; // Estimating small font width (approx 5px)
            let text_x = x + (box_size as i32 / 2) - (name_w / 2);
            
            // Highlight selected text color
            let text_color = if is_selected { 0x00FF00FF } else { 0xAAAAAAFF };
            text!(name, x = text_x, y = y + box_size + 8, font = "small", color = text_color);
        }
    }

    fn draw_developer(&self) {
        // Title
        let scale = 3; 
        let title_pixel_width = ((5 + 7) * 6 + 4) * scale;
        let start_x_title = (512 - title_pixel_width as i32) / 2;
        draw_title(start_x_title, 20, scale as u32);
        
        text!("DEVELOPERS", x = 216, y = 70, font = "large", color = 0xFF0000FF);
        
        text!("Aarif Khan", x = 216, y = 100, color = 0xFFFFFFFF);
        text!("Azhan Ali", x = 220, y = 120, color = 0xFFFFFFFF);
        
        text!("Team Name:", x = 216, y = 150, color = 0xFFFF00FF);
        text!("Tm-AzhanAarif", x = 204, y = 165, color = 0xFFFFFFFF);
        
        text!("Press X to return", x = 190, y = 220, color = 0xAAAAAAFF);
    }
}