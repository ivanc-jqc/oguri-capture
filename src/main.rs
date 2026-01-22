use std::{any, collections::HashMap, error::Error, hash::Hash};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use zbus::{Connection, Message, message, zvariant::{ Array, Dict, OwnedValue, Value, DeserializeDict}};
use futures_util::stream::StreamExt;
use zbus::zvariant::Structure;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let sc = ScreenCastSession::new(&connection).await?;
    
    // let fd = get_pipewire_fd().await?;
    sc.select_sources().await?;
    let mut stream = listener_stream(&connection, &sc.sender_id).await?;
    //restore token
    let restore_token = stream.next().await.unwrap();
    sc.start().await?;
    let pw_streams = stream.next().await.unwrap();
    
    
    let results = parse_message(pw_streams);
    parse_streams(&results);
    // parse_streams(parse_message(pw_streams));
    // let temp = &(parse_message(pw_streams)[&lookup]);
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
