use futures_util::stream::StreamExt;
use portal::*;
use std::collections::HashMap;
use std::error::Error;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedFd, OwnedObjectPath, Structure, Value};
pub const SESSION_HANDLE_TOKEN: &str = "oguri";
pub struct ScreenCastSession {
    pub connection: Connection,
    pub sender_id: String,
}
impl ScreenCastSession {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let con: Connection = Connection::session().await?;
        let id: String = Self::create_session(&con).await?;
        Ok(Self {
            connection: con,
            sender_id: id,
        })
    }
    async fn create_session(connection: &Connection) -> Result<String, Box<dyn Error>> {
        let args = HashMap::<&str, Value>::from([(
            "session_handle_token",
            Value::new(SESSION_HANDLE_TOKEN),
        )]);
        let proxy = ScreenCastProxy::new(connection).await?;
        proxy.CreateSession(&args).await?;
        let n = proxy.CreateSession(&args).await?;
        let sender_id: &str = &n.as_str().split("/").nth(6).unwrap(); //probably could handle this option better.. but cant be arsed :3
        Ok(String::from(sender_id))
    }
}

pub async fn get_pipewire_fd() -> Result<OwnedFd, Box<dyn std::error::Error>> {
    let connection = Connection::session().await?;

    let args =
        HashMap::<&str, Value>::from([("session_handle_token", Value::new(SESSION_HANDLE_TOKEN))]);

    // returns objpath /org/freedesktop/portal/desktop/session/{ID}/oguri
    let proxy = ScreenCastProxy::new(&connection).await?;
    let n = proxy.CreateSession(&args).await?;

    let sender_id: &str = &n.as_str().split("/").nth(6).unwrap();
    let mut stream = listener_stream(&connection, &sender_id, SESSION_HANDLE_TOKEN).await?;

    //ENUM PLEASE
    let args = HashMap::from([
        ("handle_token", Value::new("oguri")),
        // ("types", Value::new(1u32)),
        // ("multiple", Value::new(false)),
        // ("cursor_mode", Value::new(2u32)),
        // ("persist_mode", Value::new(1u32))
    ]);

    let session_handle = ObjectPath::try_from(
        String::from("/org/freedesktop/portal/desktop/session/")
            + sender_id
            + "/"
            + SESSION_HANDLE_TOKEN,
    )?;
    // returns objpath "/org/freedesktop/portal/desktop/request/{ID}/oguri"
    proxy.SelectSources(&session_handle, &args).await?;

    stream.next().await;

    proxy.Start(&session_handle, "", &args).await?;

    let body = stream.next().await.unwrap().body();
    let parsed = body.deserialize::<Structure>().unwrap();
    // dbg!(parsed);

    let file_descriptor = proxy.OpenPipeWireRemote(&session_handle, &args).await?;

    Ok(file_descriptor)
}
pub mod portal;
