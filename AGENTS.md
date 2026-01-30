# Cat Paw

## Project Overview

**Cat Paw** is a Rust-based desktop application utilizing the Bevy game engine (v0.18.0). It renders an interactive,
stylized 2D cat paw that overlays the screen, originating from the bottom center and following the mouse cursor.

The application is designed as a fun, interactive desktop pet/overlay.

## Key Features

* **Mouse Tracking:** The paw arm extends from the bottom of the screen and points towards the cursor.
* **Interactive Animations:**
    * **Left Click:** Paw clenches (grab animation).
    * **Right Click:** Paw opens wide (spread animation).
* **Rendering:** Procedural 2D mesh generation (Circle/Rectangle) with black outlines and white fill.
* **Windowing:** Transparent, click-through (hit-test disabled), always-on-top window.

## Tech Stack

* **Language:** Rust (Edition 2024)
* **Engine:** [Bevy](https://bevyengine.org/) (v0.18.0)
    * Features: `2d` (minimal set)
* **Input:** `device_query` (v4.0.1) for global mouse state polling.
