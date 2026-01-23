use std::{any, collections::HashMap, error::Error, hash::Hash, os::fd::OwnedFd};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use zbus::{Connection, Message, message, zvariant::{ Array, Dict, OwnedValue, Value, DeserializeDict}};
use futures_util::stream::StreamExt;
use zbus::zvariant::Structure;
use pipewire::{context::ContextBox, main_loop::MainLoopBox, properties::PropertiesBox, registry::GlobalObject, stream::{Stream, StreamBox}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let sc = ScreenCastSession::new(&connection).await?;
    
    
    sc.select_sources().await?;
    let mut sc_stream = listener_stream(&connection, &sc.sender_id).await?;
    sc_stream.next().await.unwrap();
    sc.start().await?;
    let pw_streams = sc_stream.next().await.unwrap();
    let pw_fd: OwnedFd = sc.get_pipewire_fd().await?.into();

    // let fd = get_pipewire_fd().await?;
    let results = parse_message(pw_streams);
    let node_id = parse_streams(&results);


    let mainloop = MainLoopBox::new(None)?;
    let context = ContextBox::new(&mainloop.loop_(), None)?;

    let core = context.connect_fd(pw_fd,None)?;
    let registry = core.get_registry()?;
    let props = PropertiesBox::new();
    let _listener = registry
    .add_listener_local()
    .global(|global| {
        println!("object: id:{} type:{}/{}", global.id, global.type_, global.version);
    })
    .register();
    let pw_stream = StreamBox::new(&core, "oguri-capture", props)?;
    mainloop.run();
    
    Ok(())
}

fn parse_message(message: Message) -> HashMap<String, OwnedValue> {
    message.body().deserialize::<(u32, HashMap<String, OwnedValue> )>().unwrap().1
}   
//holy fuck.. this function is so bad
fn parse_streams<'a>(unparsed_streams: &'a HashMap<String,OwnedValue>) -> u32{
    let lookup = String::from("streams");
    let streams = unparsed_streams[&lookup].downcast_ref::<&Array>().unwrap();
    let main_stream = streams.get::<Structure>(0).unwrap().unwrap(); //HOLY SHIT THIS IS BAD!!!
    let fields = main_stream.fields();
    // dbg!(fields);
    let pw_node_id = fields[0].downcast_ref::<u32>().unwrap();
    pw_node_id
}

// struct StreamInfo {
    
//     persist_mode: u32,
//     streams: HashMap<String,OwnedValue>,
// }
mod screencast;
