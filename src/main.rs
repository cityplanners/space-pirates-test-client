//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::{*, Resource};
use bevy::tasks::{Task, AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use chat::{Empty, Msg, Req};
use chat::chat_req_client::ChatReqClient;

use std::sync::{Arc, Mutex};
use tonic::transport::channel::Channel;
use tonic::Streaming;
use tokio::runtime::Runtime;
use futures::executor::block_on;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub mod chat {
    tonic::include_proto!("chat");
}

#[derive(Component)]
pub struct ChatStreamTask(pub Task<()>);

// #[derive(Resource)]
pub struct ChatClient {
    client: ChatReqClient<Channel>,
    stream: Option<Streaming<Msg>>,
    runtime: Runtime,
    chat: Arc<Mutex<Vec<String>>>
}

impl ChatClient {
    pub async fn connect_to_chat_server(&mut self, ui_state: &mut UiState) -> Result<(), Box<dyn std::error::Error>> {
        info!("connect_to_chat_server");
        let username = ui_state.username.as_mut().unwrap().clone();
        let request = Req {
            user_name: username,
        };

        let chat_vec = Arc::clone(&self.chat);

        match self.client.connect_server(request).await {
            Ok(mut response) => {

                let chat_copy = Arc::clone(&chat_vec);

                self.runtime.spawn(async move {
                    loop {
                        if let Some(next_message) = response.get_mut().message().await.ok() {
                            println!("{:?}", next_message);
                            let message = next_message.unwrap();
                            let line = message.user_name + ": " + &message.content;
                            // ui_state.chat.push(line);
                            chat_copy.lock().unwrap().push(line);
                        }
                    }
                });

            }
            Err(_) => {
            }
        }

        Ok(())
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chat_client = ChatClient {
        client: ChatReqClient::connect("http://127.0.0.1:50051").await?,
        stream: None,
        runtime: tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .unwrap(),
        chat: Arc::new(Mutex::new(Vec::<String>::new()))
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_non_send_resource(chat_client)
        .init_resource::<UiState>()
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_startup_system(configure_visuals)
        .add_system(ui_example)
        //.add_system(print_user_input)
        .run();

    Ok(())
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
pub struct UiState {
    user_input: String,
    chat: Vec<String>,
    username: Option<String>,
}

fn configure_visuals(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn ui_example(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut chat_client: NonSendMut<ChatClient>,
    mut chat_task: Query<(Entity, &mut ChatStreamTask)>
) {

    egui::SidePanel::left("side_panel")
        .exact_width(200.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.heading("Chat");
            ui.vertical(|ui| {
                for line in chat_client.chat.lock().unwrap().iter() {
                    ui.label(line);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Chat: ");
                ui.text_edit_singleline(&mut ui_state.user_input);
                if ui.button("Send").clicked() {
                    let username = ui_state.username.clone();
                    match username {
                        Some(value) => {
                            info!("Sending chat message");
                            let message = Msg {
                                content: ui_state.user_input.clone(),
                                user_name: value,
                            };
                            let future = send_chat_message(message, chat_client.as_mut());
                            block_on(future);
                        }
                        None => {
                            info!("Connecting to chat server");
                            ui_state.username = Some(ui_state.user_input.clone());
                            let future = chat_client.connect_to_chat_server(ui_state.as_mut());
                            info!("Blocking on connect_to_chat_server...");
                            block_on(future);
                            info!("done.");
                        }
                    } 

                    ui_state.user_input = String::from("");

                    send_request();
                }
            });
        });

}

fn chat_service(ui_state: &mut UiState) {

}

fn start_chat_service<'a>(commands: &mut Commands, ui_state: &'static mut UiState) {
    let thread_pool: &AsyncComputeTaskPool = AsyncComputeTaskPool::get();
    let mut ui = ui_state;

    let task: Task<()> = thread_pool.spawn(async move { chat_service(ui)});

    commands.spawn(ChatStreamTask(task));
}

fn print_user_input(ui_state: ResMut<UiState>) {
    println!("{}", ui_state.user_input);
}

async fn send_chat_message(message: Msg, chat_client: &mut ChatClient) -> Result<(), Box<dyn std::error::Error>> {

    chat_client.client.send_msg(message).await?;

    Ok(())
}

//#[tokio::main]
async fn send_request() -> Result<(), Box<dyn std::error::Error>> {
println!("Sending request");
/*
let mut greeter_client = GreeterClient::connect("http://[::1]:50051").await?;

let greeter_request = tonic::Request::new(HelloRequest {
name: "Tonic".into(),
});

let greeter_response = greeter_client.say_hello(greeter_request).await?;


println!("RESPONSE={:?}", greeter_response);

 */
Ok(())
}