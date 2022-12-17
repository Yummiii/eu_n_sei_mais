use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};
use pipewire::{
    properties,
    spa::Direction,
    stream::{ListenerBuilderT, Stream, StreamFlags},
    MainLoop,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;

    proxy
        .select_sources(
            &session,
            CursorMode::Hidden,
            SourceType::Monitor | SourceType::Window,
            false,
            None,
            PersistMode::ExplicitlyRevoked,
        )
        .await?;

    let (streams, _) = proxy.start(&session, &WindowIdentifier::default()).await?;
    let stream = streams.into_iter().next().unwrap();
    let fd = proxy.open_pipe_wire_remote(&session).await?;

    println!("node id: {}", stream.pipe_wire_node_id());
    println!("size: {:?}", stream.size());
    println!("position: {:?}", stream.position());

    let mainloop = MainLoop::new()?;


    let pw_stream = Stream::with_user_data(
        &mainloop,
        "fida",
        properties! {
            *pipewire::keys::MEDIA_TYPE => "Video",
            *pipewire::keys::MEDIA_CATEGORY => "Capture",
            *pipewire::keys::MEDIA_ROLE => "Screen",
        },
        (),
    )
    .process(|frame, _user_data| unsafe {
        let buffer = *(*frame.dequeue_raw_buffer()).buffer;
        let data = *buffer.datas;
        //agr me explica pq isso ta vindo vazio
        println!("{:?}", data);
    })
    .create()?;

    pw_stream.connect(
        Direction::Input,
        Some(stream.pipe_wire_node_id()),
        StreamFlags::AUTOCONNECT | StreamFlags::ALLOC_BUFFERS | StreamFlags::MAP_BUFFERS,
        &mut vec![],
    )?;

    mainloop.run();
    Ok(())
}
