use std::{any, collections::HashMap, error::Error, hash::Hash, os::{fd::OwnedFd, unix::process}};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use zbus::{Connection, Message, message, zvariant::{ Array, Dict, OwnedValue, Value, DeserializeDict}};
use futures_util::stream::StreamExt;
use zbus::zvariant::Structure;
use pipewire::{context::ContextBox, main_loop::MainLoopBox, properties::PropertiesBox, registry::GlobalObject, spa::{param::{self, ParamType, audio::{AudioFormat, AudioInfoRaw}}, pod::{Object, Pod}, sys::spa_callbacks, utils::Direction}, stream::{Stream, StreamBox, StreamFlags}, sys::PW_KEY_MEDIA_TYPE};
use pipewire::{properties::{properties}};
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 2;
pub const DEFAULT_VOLUME: f64 = 0.7;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();


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

    // let core = context.connect_fd(pw_fd,None)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;
    

    let _listener = registry
    .add_listener_local()
    .global(|global| {
        println!("object: id:{} type:{}/{}", global.id, global.type_, global.version);
        if let Some(props) = &global.props {
                for (k, v) in props.iter() {
                    println!("  {} = {}", k, v);
                }
            }
    })
    .register();

    let data: f64 = 0.0;
    let props = properties!{
        "PW_KEY_MEDIA_TYPE" => "Audio",
        "PW_KEY_MEDIA_CATEGORY" => "Playback",
        "PW_KEY_MEDIA_ROLE" => "Music",
        "PW_KEY_AUDIO_CHANNELS" => "2"
    };
    let pw_stream = StreamBox::new(&core, "oguri-capture", props)?;
    let strem = pw_stream
    .add_local_listener_with_user_data(data)
    .process(|stream, acc| match stream.dequeue_buffer() {
            None => println!("No buffer received"),
            Some(mut buffer) => {
                let datas = buffer.datas_mut();
                let stride = CHAN_SIZE * DEFAULT_CHANNELS as usize;
                let data = &mut datas[0];
                let n_frames = if let Some(slice) = data.data() {
                    let n_frames = slice.len() / stride;
                    for i in 0..n_frames {
                        *acc += PI_2 * 220.0 / DEFAULT_RATE as f64;
                        if *acc >= PI_2 {
                            *acc -= PI_2
                        }
                        let val = (f64::sin(*acc) * DEFAULT_VOLUME * 16767.0) as i16;
                        for c in 0..DEFAULT_CHANNELS {
                            let start = i * stride + (c as usize * CHAN_SIZE);
                            let end = start + CHAN_SIZE;
                            let chan = &mut slice[start..end];
                            chan.copy_from_slice(&i16::to_le_bytes(val));
                        }
                    }
                    n_frames
                } else {
                    0
                };
                let chunk = data.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = stride as _;
                *chunk.size_mut() = (stride * n_frames) as _;
            }
        })
        .register()?;

    
    dbg!(&node_id);
    let mut audio_info = AudioInfoRaw::new();
    audio_info.set_format(AudioFormat::S16LE);
    audio_info.set_rate(DEFAULT_RATE);
    audio_info.set_channels(DEFAULT_CHANNELS);
    let mut position = [0; pipewire::spa::param::audio::MAX_CHANNELS];
    position[0] = libspa_sys::SPA_AUDIO_CHANNEL_FL;
    position[1] = libspa_sys::SPA_AUDIO_CHANNEL_FR;
    audio_info.set_position(position);
 
    let pod_object = Object {
        type_: pipewire::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        id: ParamType::EnumFormat.as_raw(),
        properties: audio_info.into(),
    };


     let values: Vec<u8> =
        pipewire::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pipewire::spa::pod::Value::Object(pod_object),
        )
        .ok().unwrap()
        .0
        .into_inner();



    let mut params = [Pod::from_bytes(&values).unwrap()];


    pw_stream.connect(Direction::Output, None, StreamFlags::AUTOCONNECT |StreamFlags::MAP_BUFFERS 
        | StreamFlags::RT_PROCESS
        ,&mut params)?;
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
