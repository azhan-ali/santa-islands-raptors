# 🎄 Christmas Adventure

**Christmas Adventure** is a festive collection of retro-style mini-games designed to bring holiday cheer! Built with Rust and the Turbo game engine, it features a variety of challenges ranging from arcade shooters to stealth missions, all wrapped in pixel-perfect Christmas aesthetics.

![Game Screenshot](https://i.ibb.co/Z6P5GyYF/Untitled-design.png) *(Replace with actual screenshot)*

## 🎮 Game Modes

### 1. 🎁 Gift Packing (Factory)
Help Santa sort the chaotic conveyor belt!
*   **Objective:** Sort incoming gifts into their correctly colored bins (Blue, Green, Purple).
*   **Gameplay:** Gifts move down the belt. Santa must grab them and drop them into the matching bin before time runs out.
*   **Scoring:** +100 for correct sort, -50 for wrong sort.

### 2. 🦌 Raindeer Rush
Take to the skies in this side-scrolling shooter!
*   **Objective:** Fly Santa's sleigh, shoot down enemies, and avoid collisions.
*   **Gameplay:** Dodge incoming enemies and obstacles while firing gifts to destroy them. Survive as long as possible!
*   **Visuals:** Features a detailed animated Sleigh with running Reindeer.

### 3. 🧱 Santa Breaker
A holiday twist on the classic brick-breaker genre!
*   **Objective:** Smash all the festive bricks using Santa's head as the ball.
*   **Gameplay:** Control the paddle to keep Santa bouncing. Clear all bricks to advance to the next level.
*   **Difficulty:** Select from Easy (6 Lives) to Very Hard (1 Life).

### 4. 🕵️ Silent Santa (Stealth)
Sneak into a house to deliver joy... quietly!
*   **Objective:** Complete all mission tasks without waking the house pets.
    *   Place 3 Gifts (Near Dog 1, Dog 2, and the Tree).
    *   Eat the Cookie in the Kitchen.
    *   Collect 5 Stars.
    *   Escape!
*   **Mechanics:** Moving generates noise. If you move too fast near the dogs or the patrolling wolf, they will wake up! Stop moving to let your noise level drop.
*   **Enemies:** Sleeping Brown Dogs (stationary) and a Patrolling Grey Wolf.

### 5. ⚔️ Multiplayer (Santa vs. Rival)
Grab a friend for local 2-player chaos!
*   **Objective:** A "collection battle" where Player 1 (Santa) and Player 2 (Rival) race to claim houses.
*   **Gameplay:** Run to a house to claim it and earn points. Use power-ups to gain an edge.
*   **Power-ups:**
    *   **Gifts:** Bonus Points (+50).
    *   **Lightning:** Speed Boost for 5 seconds.
*   **Hazards:** Avoid Bombs and Obstacles!
*   **Setup:** Customize player names and match duration (1-10 mins) in the pre-game menu.

---

## 🕹️ Controls

| Action | Key / Button | Description |
| :--- | :--- | :--- |
| **Move** | **Arrow Keys** | Move Character / Navigate Menus |
| **Action** | **Space** / **Enter** / **Z** | Select / Interact / Shoot / Launch |
| **Back** | **X** / **Backspace** | Go Back / Cancel |
| **Instructions** | **Shift** | Toggle Instructions Overlay |

> **Note:** The game supports both Keyboard and Gamepad input.

---

## 🛠️ Technology Stack

*   **Language:** [Rust](https://www.rust-lang.org/) 🦀
*   **Engine:** [Turbo](https://turbo.computer/) 🚀
*   **Graphics:** Custom procedural pixel art (drawn via code).
*   **Platform:** Web (WASM) & Native.

## 🚀 How to Run

1.  **Install Turbo:**
    Follow the instructions at [turbo.computer](https://turbo.computer) to install the CLI.

2.  **Run Locally:**
    ```bash
    turbo run -w .
    ```

3.  **Build for Web:**
    ```bash
    turbo export
    ```
    This generates a `www` folder ready to be hosted on GitHub Pages or Vercel.

---

## ✨ features

*   **Detailed Pixel Art:** Characters, furniture, and enemies are drawn using custom rectangle-based sprites for a crisp retro look.
*   **Dynamic Animations:** Sleeping dogs breathe, wolves patrol, and Santa's sleigh has moving parts.
*   **Global Instructions:** Press Shift at any time to see context-sensitive help for the current game mode.
*   **Music & SFX:** Holiday themed background music and sound effects.

---

*Made with ❤️ by Aarif Khan & Azhan Ali*
