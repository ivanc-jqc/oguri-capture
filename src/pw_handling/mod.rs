use std::error::Error;
use std::os::fd::OwnedFd;
use pipewire::{context::ContextBox, main_loop::MainLoopBox, 
    stream::{Stream, StreamBox, StreamFlags}
    };
use pipewire::{properties::{properties}};
use pipewire::spa::{
    param::{self, ParamType, audio::{AudioFormat, AudioInfoRaw}},
    pod::{Object, Pod}, sys::spa_callbacks, utils::Direction};

pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 2;
pub const DEFAULT_VOLUME: f64 = 0.7;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();

pub fn pw_init(fd: OwnedFd) -> Result<(), Box<dyn Error>> {
     let mainloop = MainLoopBox::new(None)?;
    let context = ContextBox::new(&mainloop.loop_(), None)?;

    let core = context.connect_fd(fd,None)?;
    // let core = context.connect(None)?;
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
    .param_changed(|stream, user_data,id, param |{
        let Some(param) = param else {
            return;
        };
        dbg!(user_data);
        dbg!(id);


    })
    
    
    
    
    
    
    
    .process(|stream, acc| match stream.dequeue_buffer() {
            None => println!("No buffer received"),
            Some(mut buffer) => {
                let datas = buffer.datas_mut();
                let stride = CHAN_SIZE * DEFAULT_CHANNELS as usize;
                let data = &mut datas[0];
                let n_frames = if let Some(slice) = data.data() {
                    let n_frames = slice.len() / stride;
                    for i in 0..n_frames {
                        *acc += PI_2 * 440.0 / DEFAULT_RATE as f64;
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