use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, CursorOptions, PrimaryWindow, WindowLevel};
use bevy::winit::WINIT_WINDOWS;
use device_query::{DeviceQuery, DeviceState};

#[derive(Resource)]
struct GlobalDeviceState(DeviceState);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .insert_resource(ClearColor(Color::NONE))
        .init_resource::<PawAnimState>()
        .insert_resource(GlobalDeviceState(DeviceState::new()))
        .add_systems(Startup, (setup, setup_primary_window))
        .add_systems(Update, (follow_mouse, update_inner_arm, animate_paw))
        .run();
}

#[derive(Resource, Default)]
struct PawAnimState {
    factor: f32, // -1.0 (Clench) to 1.0 (Open), 0.0 (Neutral)
}

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

// Visual constants
const OUTLINE_WIDTH: f32 = 10.0;
const ARM_WIDTH: f32 = 60.0;
const PALM_RADIUS: f32 = 40.0;
const FINGER_RADIUS: f32 = 25.0;
// Colors
const COLOR_FILL: Color = Color::WHITE;
const COLOR_OUTLINE: Color = Color::BLACK;

fn window_plugin() -> WindowPlugin {
    let window = Window {
        title: "Cat Paw".into(),
        transparent: true,
        decorations: false,
        resizable: false,
        has_shadow: false,
        window_level: WindowLevel::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        #[cfg(target_os = "linux")]
        composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
        ..default()
    };

    let cursor_options = CursorOptions {
        visible: false,
        hit_test: false,
        ..default()
    };

    WindowPlugin {
        primary_window: Some(window),
        primary_cursor_options: Some(cursor_options),
        ..default()
    }
}

fn setup_primary_window(
    primary_window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    _non_send_marker: NonSendMarker,
) {
    let (entity, mut window) = primary_window.into_inner();
    WINIT_WINDOWS.with_borrow(|winit_windows| {
        let Some(winit_window) = winit_windows.get_window(entity) else {
            error!("Primary window找不到: {:?}", entity);
            return;
        };
        let Some(current_monitor) = winit_window.current_monitor() else {
            error!("当前显示器找不到: {:?}", entity);
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
            "当前屏幕: {:?}, 屏幕尺寸: {}x{}, 窗口新尺寸: {}x{}, 窗口新坐标: {}x{}",
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
    });
}

fn setup(
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
                Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(
                    PALM_RADIUS / palm_scale,
                )),
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
                            Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(
                                FINGER_RADIUS / finger_scale,
                            )),
                        ));
                    });
            }
        });
}

fn follow_mouse(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    device_state: Res<GlobalDeviceState>,
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

    let mouse = device_state.0.get_mouse();
    let window_origin = match window.position {
        WindowPosition::At(pos) => Vec2::new(pos.x as f32, pos.y as f32),
        _ => Vec2::ZERO,
    };
    let cursor_pos = Vec2::new(mouse.coords.0 as f32, mouse.coords.1 as f32) - window_origin;

    if let Ok(mouse_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        let mouse_world_pos: Vec2 = mouse_world_pos;
        let start_pos = Vec2::new(0.0, -window.height() / 2.0);
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
            transform.scale = Vec3::new(ARM_WIDTH + OUTLINE_WIDTH, length, 1.0);
        }

        for mut transform in bottom_query.iter_mut() {
            transform.translation = start_pos.extend(1.0);
        }
    }
}

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

fn animate_paw(
    device_state: Res<GlobalDeviceState>,
    mut anim_state: ResMut<PawAnimState>,
    mut palm_query: Query<&mut Transform, (With<PawPalm>, Without<PawFinger>)>,
    mut fingers: Query<(&mut Transform, &PawFinger), Without<PawPalm>>,
    time: Res<Time>,
) {
    let mouse = device_state.0.get_mouse();
    let left_pressed = mouse.button_pressed.get(1).cloned().unwrap_or(false);
    let right_pressed = mouse.button_pressed.get(2).cloned().unwrap_or(false);

    let mut target = 0.0f32;
    if left_pressed {
        target = -1.0;
    } else if right_pressed {
        target = 1.0;
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
