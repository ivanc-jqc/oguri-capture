use std::{any, error::Error};
use screencast::ScreenCastSession;
use screencast::signals::listener_stream;
use zbus::{Connection, Message, message};
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
    let restore_token = stream.next().await.unwrap().body();
    sc.start().await?;
    let pw_streams = stream.next().await.unwrap();

    dbg!(restore_token.deserialize::<Structure>()?);
    Ok(())
}

// fn parse_message(message: Message) -> Vec<zbus::zvariant::Value<'static>>  {
//     let body = message.body().deserialize::<Structure>().unwrap().into_fields();
//     body
// }



mod screencast;
