#![windows_subsystem = "windows"]

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, CursorOptions, PrimaryWindow, WindowLevel};
use bevy::winit::WINIT_WINDOWS;
use device_query::{DeviceQuery, DeviceState, MouseState};
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .add_systems(Startup, (setup_primary_window, setup_cat_paw))
        .add_systems(PreUpdate, poll_mouse_input)
        .add_systems(
            Update,
            (
                follow_mouse,
                update_inner_arm,
                animate_paw,
                handle_shortcuts,
            ),
        )
        // Make the background completely transparent
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(GlobalDeviceState(DeviceState::new()))
        .init_resource::<PawAnimState>()
        .init_resource::<GlobalMouseState>()
        .init_resource::<CursorControl>()
        .run();
}

// Visual constants
const OUTLINE_WIDTH: f32 = 10.0;
const ARM_WIDTH: f32 = 60.0;
const PALM_RADIUS: f32 = 40.0;
const FINGER_RADIUS: f32 = 25.0;
// Colors
const COLOR_FILL: Color = Color::WHITE;
const COLOR_OUTLINE: Color = Color::BLACK;

#[derive(Resource)]
struct GlobalDeviceState(DeviceState);

#[derive(Resource, Default)]
struct GlobalMouseState(MouseState);

#[derive(Component)]
struct PawArm;

#[derive(Component)]
struct PawPalm;

#[derive(Component)]
struct PawBottom;

#[derive(Component)]
struct PawFinger {
    base_pos: Vec3,
    index: usize,
}

#[derive(Resource, Default)]
struct PawAnimState {
    factor: f32, // -1.0 (Clench) to 1.0 (Open), 0.0 (Neutral)
}

#[derive(Resource, Default)]
struct CursorControl {
    is_hidden: bool,
    lr_press_start: Option<f64>,
}

fn window_plugin() -> WindowPlugin {
    let window = Window {
        title: "Cat Paw".into(),
        transparent: true,
        decorations: false, // No title bar or borders
        resizable: false,
        has_shadow: false,
        window_level: WindowLevel::AlwaysOnTop, // Keep the paw on top of other windows
        #[cfg(target_os = "macos")]
        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        #[cfg(target_os = "linux")]
        composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
        ..default()
    };

    let cursor_options = CursorOptions {
        // TODO: Cursor hiding is not working as expected
        visible: false,
        // Allow clicks to pass through to windows below
        hit_test: false,
        ..default()
    };

    WindowPlugin {
        primary_window: Some(window),
        primary_cursor_options: Some(cursor_options),
        ..default()
    }
}

// Sync Bevy window to match the monitor size for full-screen overlay
fn setup_primary_window(
    primary_window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    _non_send_marker: NonSendMarker,
) {
    let (entity, mut window) = primary_window.into_inner();
    WINIT_WINDOWS.with_borrow(|winit_windows| {
        let Some(winit_window) = winit_windows.get_window(entity) else {
            error!("Primary window not found: {:?}", entity);
            return;
        };
        let Some(current_monitor) = winit_window.current_monitor() else {
            error!("Current monitor not found: {:?}", entity);
            return;
        };

        let monitor_pos = current_monitor.position();
        let monitor_size = current_monitor.size();
        let scale_factor = current_monitor.scale_factor() as f32;

        let window_width = monitor_size.width as f32 / scale_factor;
        let window_height = monitor_size.height as f32 / scale_factor;
        let window_left = monitor_pos.x;
        let window_top = monitor_pos.y;

        debug!(
            "Current monitor: {:?}, Monitor size: {}x{}, Window new size: {}x{}, Window new pos: {}x{}",
            current_monitor.name(),
            monitor_size.width,
            monitor_size.height,
            window_width,
            window_height,
            window_left,
            window_top
        );

        window.resolution.set(window_width, window_height);
        window.position = WindowPosition::At(IVec2::new(window_left, window_top));

        // Set window icon
        let icon_buf = Cursor::new(include_bytes!("../build/macos/AppIcon.iconset/icon_256x256.png"));
        if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
                winit_window.set_window_icon(Some(icon));
            } else {
                error!("Failed to create icon");
            }
        } else {
            error!("Failed to open icon");
        }
    });
}

fn setup_cat_paw(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let mesh_circle = meshes.add(Circle::new(1.0));
    let mesh_rect = meshes.add(Rectangle::new(1.0, 1.0));

    let mat_white = materials.add(ColorMaterial::from(COLOR_FILL));
    let mat_black = materials.add(ColorMaterial::from(COLOR_OUTLINE));

    // Spawn Arm
    commands
        .spawn((
            Mesh2d(mesh_rect.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::default(),
            PawArm,
        ))
        .with_children(|parent| {
            // Inner white arm
            parent.spawn((
                Mesh2d(mesh_rect.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });

    // Spawn Arm Bottom (Semi-circle effect)
    commands
        .spawn((
            Mesh2d(mesh_circle.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::from_scale(Vec3::splat((ARM_WIDTH + OUTLINE_WIDTH) / 2.0)),
            PawBottom,
        ))
        .with_children(|parent| {
            // Inner white bottom
            parent.spawn((
                Mesh2d(mesh_circle.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(
                    (ARM_WIDTH - OUTLINE_WIDTH) / 2.0 / ((ARM_WIDTH + OUTLINE_WIDTH) / 2.0),
                )),
            ));
        });

    // Spawn Palm
    let palm_scale = PALM_RADIUS + OUTLINE_WIDTH;
    let finger_scale = FINGER_RADIUS + OUTLINE_WIDTH;

    commands
        .spawn((
            Mesh2d(mesh_circle.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::from_scale(Vec3::splat(palm_scale)),
            PawPalm,
        ))
        .with_children(|parent| {
            // Inner white palm
            parent.spawn((
                Mesh2d(mesh_circle.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 1.0)
                    .with_scale(Vec3::splat(PALM_RADIUS / palm_scale)),
            ));

            // Fingers
            let fingers_params: [(usize, f32, f32); 4] = [
                // Index, Angle (deg), Radius (absolute)
                (0, -56.0, 36.0),
                (1, -22.0, 43.0),
                (2, 22.0, 43.0),
                (3, 56.0, 36.0),
            ];

            for (i, angle_deg, dist) in fingers_params {
                let angle_rad = angle_deg.to_radians();
                let x_abs = dist * angle_rad.sin();
                let y_abs = dist * angle_rad.cos();

                // Normalized position relative to parent scale
                // Finger black circle stays behind the palm black circle
                let base_pos = Vec3::new(x_abs / palm_scale, y_abs / palm_scale, -0.1);

                parent
                    .spawn((
                        Mesh2d(mesh_circle.clone()),
                        MeshMaterial2d(mat_black.clone()),
                        Transform::from_translation(base_pos),
                        PawFinger { base_pos, index: i },
                    ))
                    .with_children(|f_parent| {
                        // Finger white circle moves in front of palm black circle but behind palm white circle
                        f_parent.spawn((
                            Mesh2d(mesh_circle.clone()),
                            MeshMaterial2d(mat_white.clone()),
                            Transform::from_xyz(0.0, 0.0, 1.0)
                                .with_scale(Vec3::splat(FINGER_RADIUS / finger_scale)),
                        ));
                    });
            }
        });
}

// Poll global mouse input using `device_query` since the window is non-interactive
fn poll_mouse_input(
    device_state: Res<GlobalDeviceState>,
    mut mouse_state: ResMut<GlobalMouseState>,
) {
    mouse_state.0 = device_state.0.get_mouse();
}

// Logic to make the paw follow the mouse cursor
fn follow_mouse(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_state: Res<GlobalMouseState>,
    mut arm_query: Query<&mut Transform, (With<PawArm>, Without<PawPalm>, Without<PawBottom>)>,
    mut palm_query: Query<&mut Transform, (With<PawPalm>, Without<PawArm>, Without<PawBottom>)>,
    mut bottom_query: Query<&mut Transform, (With<PawBottom>, Without<PawArm>, Without<PawPalm>)>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let mouse = &mouse_state.0;
    // Calculate window origin in screen coordinates
    let window_origin = match window.position {
        WindowPosition::At(pos) => Vec2::new(pos.x as f32, pos.y as f32),
        _ => Vec2::ZERO,
    };
    // Get cursor position relative to the Bevy window
    let cursor_pos = Vec2::new(mouse.coords.0 as f32, mouse.coords.1 as f32) - window_origin;

    if let Ok(mouse_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        let mouse_world_pos: Vec2 = mouse_world_pos;
        let start_pos = Vec2::new(0.0, -window.height() / 2.0); // Start from bottom center
        let diff = mouse_world_pos - start_pos;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x) - std::f32::consts::FRAC_PI_2;

        for mut transform in palm_query.iter_mut() {
            transform.translation = mouse_world_pos.extend(1.0);
            transform.rotation = Quat::from_rotation_z(angle);
        }

        let midpoint = (start_pos + mouse_world_pos) / 2.0;

        for mut transform in arm_query.iter_mut() {
            transform.translation = midpoint.extend(1.0);
            transform.rotation = Quat::from_rotation_z(angle);
            // Stretch arm to reach the mouse
            transform.scale = Vec3::new(ARM_WIDTH + OUTLINE_WIDTH, length, 1.0);
        }

        for mut transform in bottom_query.iter_mut() {
            transform.translation = start_pos.extend(1.0);
        }
    }
}

// Ensure the inner (white) part of the arm scales correctly with the outer (black) part
fn update_inner_arm(
    arm_query: Query<(&Transform, &Children), With<PawArm>>,
    mut inner_query: Query<&mut Transform, Without<PawArm>>,
) {
    for (parent_transform, children) in arm_query.iter() {
        let parent_scale = parent_transform.scale;
        let w_outer = parent_scale.x;
        let l_outer = parent_scale.y;

        if w_outer > 0.0 && l_outer > 0.0 {
            for child in children.iter() {
                if let Ok(mut child_transform) = inner_query.get_mut(child) {
                    let w_inner = (w_outer - 2.0 * OUTLINE_WIDTH).max(0.0);
                    let l_inner = l_outer.max(0.0);

                    child_transform.scale = Vec3::new(w_inner / w_outer, l_inner / l_outer, 1.0);
                }
            }
        }
    }
}

// Handle mouse clicks for paw animation (clench/open)
fn animate_paw(
    mouse_state: Res<GlobalMouseState>,
    mut anim_state: ResMut<PawAnimState>,
    mut palm_query: Query<&mut Transform, (With<PawPalm>, Without<PawFinger>)>,
    mut fingers: Query<(&mut Transform, &PawFinger), Without<PawPalm>>,
    time: Res<Time>,
) {
    let mouse = &mouse_state.0;
    let left_pressed = mouse.button_pressed.get(1).cloned().unwrap_or(false);
    let right_pressed = mouse.button_pressed.get(2).cloned().unwrap_or(false);

    let mut target = 0.0f32;
    if left_pressed {
        target = -1.0; // Clench
    } else if right_pressed {
        target = 1.0; // Open
    }

    // Interpolate
    let speed = 15.0; // Faster response
    anim_state.factor += (target - anim_state.factor) * speed * time.delta_secs();

    let factor = anim_state.factor;

    // Palm animation: slightly squash/stretch
    for mut palm_transform in palm_query.iter_mut() {
        let base_palm_scale = PALM_RADIUS + OUTLINE_WIDTH;
        let scale_y = base_palm_scale * (1.0 + factor * 0.05); // Reduced from 0.1
        let scale_x = base_palm_scale * (1.0 - factor * 0.025); // Reduced from 0.05
        palm_transform.scale = Vec3::new(scale_x, scale_y, 1.0);
    }

    for (mut transform, finger) in fingers.iter_mut() {
        let original_pos = finger.base_pos;
        let base_scale = (FINGER_RADIUS + OUTLINE_WIDTH) / (PALM_RADIUS + OUTLINE_WIDTH);

        if factor < 0.0 {
            // Clenching (factor 0 to -1)
            let t = -factor;
            let clench_offset = original_pos * -0.3 * t; // Reduced from -0.6 (Move inward)
            transform.translation = original_pos + clench_offset;
            transform.scale = Vec3::splat(base_scale * (1.0 - 0.1 * t)); // Reduced shrink from 0.2

            // Rotate fingers towards center
            let angle = match finger.index {
                0 => 0.2 * t,   // Reduced from 0.4
                1 => 0.08 * t,  // Reduced from 0.15
                2 => -0.08 * t, // Reduced from -0.15
                3 => -0.2 * t,  // Reduced from -0.4
                _ => 0.0,
            };
            transform.rotation = Quat::from_rotation_z(angle);
        } else {
            // Opening (factor 0 to 1)
            let t = factor;
            let mut open_offset = original_pos * 0.15 * t; // Reduced from 0.3 (Move outward)
            open_offset.x *= 1.3; // Slightly reduced spread multiplier from 1.6
            transform.translation = original_pos + open_offset;
            transform.scale = Vec3::splat(base_scale * (1.0 + 0.08 * t)); // Reduced grow from 0.15

            // Rotate fingers away from center
            let angle = match finger.index {
                0 => -0.12 * t, // Reduced from -0.25
                1 => -0.05 * t, // Reduced from -0.1
                2 => 0.05 * t,  // Reduced from 0.1
                3 => 0.12 * t,  // Reduced from 0.25
                _ => 0.0,
            };
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

// Handle global shortcuts (Left+Right click)
fn handle_shortcuts(
    mouse_state: Res<GlobalMouseState>,
    mut cursor_control: ResMut<CursorControl>,
    mut exit: MessageWriter<AppExit>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut paw_visibility: Query<&mut Visibility, Or<(With<PawArm>, With<PawPalm>, With<PawBottom>)>>,
    time: Res<Time>,
) {
    let mouse = &mouse_state.0;
    // 1=Left, 2=Right
    let left = mouse.button_pressed.get(1).cloned().unwrap_or(false);
    let right = mouse.button_pressed.get(2).cloned().unwrap_or(false);
    let both_pressed = left && right;

    let now = time.elapsed_secs_f64();

    if both_pressed {
        if cursor_control.lr_press_start.is_none() {
            cursor_control.lr_press_start = Some(now);
        } else {
            let start = cursor_control.lr_press_start.unwrap();
            // Long press (> 2s) to exit app
            if now - start > 2.0 {
                exit.write(AppExit::Success);
            }
        }
    } else {
        if let Some(start) = cursor_control.lr_press_start {
            // Released
            let duration = now - start;
            // Short press (< 2s) to toggle visibility
            if duration < 2.0 {
                // Toggle
                cursor_control.is_hidden = !cursor_control.is_hidden;

                // Toggle OS cursor visibility (Inverse of Paw visibility)
                if let Some(mut options) = cursor_options_query.iter_mut().next() {
                    options.visible = cursor_control.is_hidden;
                }

                // Toggle Paw visibility
                let new_vis = if cursor_control.is_hidden {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };

                for mut vis in paw_visibility.iter_mut() {
                    *vis = new_vis;
                }
            }
            cursor_control.lr_press_start = None;
        }
    }
}
