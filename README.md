# Toon Dash

**Toon Dash** is a fast-paced, 3D endless runner game inspired by Tomb Runner. Built entirely in Rust using the lightweight [Macroquad](https://macroquad.rs/) engine!

## What is this game about?
You play as a brave explorer running endlessly along a 3-lane road, dodging obstacles, collecting gold coins, and trying to beat your high score!

It features:
- **Procedural 3D World:** Endless terrain generation that dynamically spawns roads, coins, and obstacles.
- **Multiple Characters:** 5 distinct characters to choose from (Oodi, Ooli, Oozi, Oopi, Oobi) from the Kenney Platformer Kit.
- **Custom UI:** Built entirely from scratch, rendering beautiful 2D overlays on top of 3D viewports.
- **Cross-Platform:** Native support for both Desktop and WebAssembly (WASM).

## 🎮 How to Play
Navigate the menus using the **Arrow Keys** (D-Pad) and press **ENTER** to select.

During gameplay:
- **Left / Right Arrays:** Switch between the 3 lanes.
- **Up Arrow:** Jump over low fences and hazards.
- **Down Arrow:** Slide under high, floating obstacles.
- **ESC / Back Space:** Pause the game.

Collect coins to massively boost your score and travel as far as you can!

## 🛠️ Technical Details
Toon Dash was constructed with performance and extreme portability in mind:
- **Engine:** [Macroquad](https://macroquad.rs/) — a fast, immediate-mode game engine for Rust.
- **Language:** 100% Rust.
- **Models & Assets:** GLTF/GLB models are parsed using the `gltf` crate. We use Macroquad's 3D rendering pipeline to draw the animated meshes manually to the screen. 
- **WASM Interop:** Designed specifically to compile straight to WebAssembly without relying on `wasm-bindgen`, utilizing Macroquad's custom `mq_js_bundle.js` for pure, optimal web delivery.

## 🧰 Prerequisites

Before you can build or run the game, you'll need the Rust toolchain installed on your machine.

1. **Install Rust & Cargo:**
   The easiest way is to use [rustup](https://rustup.rs/). Open your terminal and run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Add the WebAssembly Target (For Web Builds):**
   If you plan to compile the game for the browser, add the WASM target to your toolchain:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Install a Local HTTP Server (For Web Testing):**
   To serve the game locally after building it for the web, `basic-http-server` is highly recommended:
   ```bash
   cargo install basic-http-server
   ```

## 🚀 How to Run
### 1. Native Desktop (Mac/Linux/Windows)
The absolute easiest way to play and test the game is natively via Cargo:
```bash
cargo run --release
```

### 2. WebAssembly (Browser)
You can compile the game to WebAssembly and serve it locally!

First, build the game for the web target:
```bash
# Add the WebAssembly target if you haven't already
rustup target add wasm32-unknown-unknown

# Build the WASM binary
cargo build --release --target wasm32-unknown-unknown
```

Next, use the provided `build.sh` script to package the HTML, JavaScript, and compiled WASM binary into a clean `dist/` directory:
```bash
./build.sh package
```

Finally, serve the `dist/` directory using a basic HTTP server. A great Rust-based option is `basic-http-server`:
```bash
# Install the server globally via Cargo
cargo install basic-http-server

# Serve the dist/ directory on your local machine
basic-http-server dist/
```
Then simply open your browser and navigate to the provided local address (usually `http://127.0.0.1:4000`) to play Toon Dash completely in the browser!
