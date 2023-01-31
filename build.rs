fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut attributes = tonic_build::Attributes::default();
    attributes.push_struct("ChatReqClient", r#"#[derive(Resource)]"#);

    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/chat.proto"],
            &["proto".to_string()]
        )?;
    
    Ok(())
}