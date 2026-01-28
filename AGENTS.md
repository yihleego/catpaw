# Cat Paw Project Context

## Project Overview
**Cat Paw** is a Rust-based desktop application built with the [Bevy game engine](https://bevyengine.org/) (v0.18.0). It creates a transparent, always-on-top overlay window where a stylized "cat paw" arm extends from the bottom of the screen to follow the user's mouse cursor. The paw reacts to mouse clicks with animations.

## Key Features
- **Mouse Tracking:** An arm extends from the bottom-center of the screen, with the palm following the global mouse cursor position.
- **Interactive Animations:**
  - **Left Click:** The paw "clenches" (fingers move inward).
  - **Right Click:** The paw "opens" (fingers spread outward).
- **Visuals:** Vector-style rendering using basic meshes (circles, rectangles) with a black outline and white fill.
- **Window Management:** Runs as a transparent, undecorated, click-through (partially, `hit_test: false` for cursor) window that covers the entire primary monitor.

## Architecture & Code Structure
The project is currently contained primarily within a single source file:

- **`src/main.rs`**: Contains the application entry point, window configuration, ECS (Entity Component System) setup, and all game logic.
    - **Resources:**
        - `GlobalDeviceState`: Wraps `device_query::DeviceState` to poll global mouse input (required since the window is transparent/click-through).
        - `PawAnimState`: Tracks the animation state (-1.0 to 1.0) for smooth transitions between clenched and open states.
    - **Components:**
        - `PawArm`, `PawPalm`, `PawBottom`, `PawFinger`: Tag components to identify parts of the paw hierarchy.
    - **Systems:**
        - `setup_primary_window`: Configures the window to match the monitor size and position.
        - `setup`: Spawns the camera and the initial 2D meshes for the paw.
        - `follow_mouse`: Calculates geometry to point the arm and palm at the mouse.
        - `animate_paw`: Interpolates animation state based on mouse button input and updates finger transforms.

## Dependencies
- **Bevy (0.18.0):** The core game engine used for rendering, windowing, and ECS.
- **device_query (2.1.0):** Used to query global mouse state (position and button presses) independently of the Bevy window, which is necessary because the window is configured to ignore hit tests.

## Building and Running

### Prerequisites
- Rust toolchain (stable)
- OS-specific dependencies for Bevy (e.g., development libraries for Linux, see Bevy docs).

## Development Conventions
- **Code Style:** Follows standard Rust formatting (`rustfmt`).
- **Bevy Patterns:** Uses standard Bevy ECS patterns (Resources for state, Components for entities, Systems for logic).
- **Coordinate System:**
    - The arm is anchored at the bottom center of the screen.
    - Logic handles conversion between global screen coordinates (from `device_query`) and Bevy's world space.
