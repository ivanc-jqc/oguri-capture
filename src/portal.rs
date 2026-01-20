use std::collections::HashMap;
use std::error::Error;

use zbus::proxy::SignalStream;
use zbus::{Connection, Proxy, proxy};
use zbus::zvariant::{Value, ObjectPath, OwnedObjectPath, OwnedFd};

#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait ScreenCast {
    fn CreateSession (&self,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>; //ObjectPath doesnt support lifetimes
    fn SelectSources (&self,
        session_handle: &ObjectPath<'_>,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
    
    fn Start (&self,
        session_handle: &ObjectPath<'_>,
        parent_window: &str,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    fn OpenPipeWireRemote (&self,
        session_handle: &ObjectPath<'_>,
        parent_window: &str,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedFd>;

    #[zbus(property)]
    #[allow(non_snake_case)]
    fn AvailableSourceTypes(&self) -> zbus::Result<u32>;
}   
pub async fn listener_stream<'a>(connection: &'a Connection,sender_id: &str,handle_token: &str) -> Result<SignalStream<'a>,Box<dyn Error>> {
    let concatenated = String::from("/org/freedesktop/portal/desktop/request/")+sender_id+"/"+handle_token;
    let listener_proxy = Proxy::new(
        connection,
        "org.freedesktop.portal.Desktop",
        concatenated,
        "org.freedesktop.portal.Request",
    ).await?;
    let response_stream= listener_proxy.receive_all_signals().await?;

    Ok(response_stream)
}