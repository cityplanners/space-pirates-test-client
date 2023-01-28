//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<UiState>()
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_startup_system(configure_visuals)
        .add_system(ui_example)
        .add_system(print_user_input)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Resource, Default)]
struct UiState {
    user_input: String,
    chat: Vec<String>
}

fn configure_visuals(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn ui_example(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>
) {

    egui::SidePanel::left("side_panel")
        .exact_width(200.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.heading("Chat");
            ui.vertical(|ui| {
                for line in ui_state.chat.iter() {
                    ui.label(line);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Chat: ");
                ui.text_edit_singleline(&mut ui_state.user_input);
                if ui.button("Send").clicked() {
                    let text = ui_state.user_input.clone();
                    ui_state.chat.push(text);
                    ui_state.user_input = String::from("");
                }
            });
        });

}

fn print_user_input(ui_state: ResMut<UiState>) {
    println!("{}", ui_state.user_input);
}