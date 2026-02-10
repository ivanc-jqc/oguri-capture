use std::path::Path;
use std::{collections::HashMap, error::Error, os::fd::OwnedFd, sync::mpsc};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use std::thread;
use futures_util::stream::StreamExt;
use zbus::{Connection, Message};
use zbus::zvariant::{Structure,  Array, OwnedValue};
use std::time::{Duration, Instant};
use pipewire_capture::PipewireManager;
use pipewire_capture::TERMINATE;
use std::io::{self, Write};
use std::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let sc = ScreenCastSession::new(&connection).await?;
    let (pw_tx, pw_rx) = pipewire::channel::channel();

    
    sc.select_sources().await?;
    let mut sc_stream = listener_stream(&connection, &sc.sender_id).await?;
    sc_stream.next().await.unwrap();
    sc.start().await?;
    let pw_streams = sc_stream.next().await.unwrap();
    let pw_fd: OwnedFd = sc.get_pipewire_fd().await?;
    let node_id: u32 = parse_streams(&(parse_message(pw_streams)));

    let _pw_thread = thread::spawn(move|| {
        let mut pw_man = PipewireManager::new(pw_fd,node_id);
        pw_man.run(pw_rx);
        pw_man.video.dump();

        // let mut binding = pw_man.audio.audio_buffer.borrow_mut();
        // let vec = binding.dump().clone();
        // drop(binding);
        // let path = Path::new("dump");
        // let mut file = File::create(path).unwrap();
        // let start = Instant::now();
        // for x in vec.iter(){
        //     let x = match x {
        //         Some(x) => x,
        //         None => &(0f32),
        //     };
            
        //     let temp = x.to_le_bytes();
        //     file.write_all(&temp).unwrap();
    
        // }
        // let elapsed = start.elapsed();
        // dbg!(elapsed);
        // dbg!(vec.len());
    });
    
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer);
    // thread::sleep(Duration::from_secs(40));
    pw_tx.send(TERMINATE);
    // thread::sleep(Duration::from_secs(10));
    dbg!(buffer);
    _pw_thread.join().unwrap();
    
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
    let pw_node_id = fields[0].downcast_ref::<u32>().unwrap();
    pw_node_id
}

mod screencast;
mod pipewire_capture;
mod circular_buffer;
#[cfg(test)]
mod tests;