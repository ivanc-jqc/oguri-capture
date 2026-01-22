use std::{any, collections::HashMap, error::Error};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use zbus::{Connection, Message, message, zvariant::{Dict, OwnedValue, Value}};
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
    let lookup = String::from("streams");
    dbg!(parse_message(pw_streams));
    // let temp = &(parse_message(pw_streams)[&lookup]);
    Ok(())
}

fn parse_message(message: Message) -> HashMap<String, OwnedValue> {
    let body = message.body();
    let body_vec = body.deserialize::<Structure>().unwrap().into_fields();
    let body_dict: Dict = body_vec[1].clone().downcast().unwrap();
    let results_map = HashMap::<String, OwnedValue>::try_from(body_dict).unwrap();
    results_map
}   



mod screencast;
