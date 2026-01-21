use calls::*;
use std::collections::HashMap;
use std::error::Error;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedFd, OwnedObjectPath, Value};


pub const SESSION_HANDLE_TOKEN: &str = "oguri";
pub const XDG_SESSION_PATH: &str = "/org/freedesktop/portal/desktop/session/";


pub struct ScreenCastSession<'oguri> {
    pub connection: &'oguri Connection,
    pub sender_id: String,
    pub session_handle: ObjectPath<'oguri>,
    proxy: ScreenCastProxy<'oguri>,
}
impl<'oguri> ScreenCastSession<'oguri> {
    pub async fn new(con: &'oguri Connection) -> Result<Self, Box<dyn Error>> {
        let prox = ScreenCastProxy::new(con).await?;
        let id: String = Self::create_session(&prox).await?;
        let session_handle = ObjectPath::try_from(
            String::from(XDG_SESSION_PATH) + &id + "/" + SESSION_HANDLE_TOKEN,
        )?;

        Ok(Self {
            connection: con,
            sender_id: id,
            session_handle: session_handle,
            proxy: prox,
        })
    }

    async fn create_session(proxy: &ScreenCastProxy<'_>) -> Result<String, Box<dyn Error>> {
        let args = HashMap::<&str, Value>::from([(
            "session_handle_token",
            Value::new(SESSION_HANDLE_TOKEN),
        )]);

        let n = proxy.CreateSession(&args).await?;
        let sender_id: &str = &n.as_str().split("/").nth(6).unwrap(); //probably could handle this option better.. but cant be arsed :3
        Ok(String::from(sender_id))
    }

    pub async fn select_sources(&self) -> Result<OwnedObjectPath, Box<dyn Error>> {
        let args = HashMap::from([
            ("handle_token", Value::new("oguri")),
            // ("types", Value::new(1u32)),
            // ("multiple", Value::new(false)),
            // ("cursor_mode", Value::new(2u32)),
            // ("persist_mode", Value::new(1u32))
        ]);
        
        Ok(self
            .proxy
            .SelectSources(&self.session_handle, &args)
            .await?)
    }
    pub async fn start(&self) -> Result<OwnedObjectPath, Box<dyn Error>> {
        let args = HashMap::from([("handle_token", Value::new("oguri"))]);

        Ok(self.proxy.Start(&self.session_handle, "", &args).await?)
    }

    pub async fn get_pipewire_fd(&self) -> Result<OwnedFd, Box<dyn std::error::Error>> {
        let args = HashMap::from([("handle_token", Value::new("oguri"))]);
        Ok(self
            .proxy
            .OpenPipeWireRemote(&self.session_handle, &args)
            .await?)
    }
}


pub mod calls;
pub mod signals;