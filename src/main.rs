use std::collections::HashMap;
use std::error::Error;
use futures_util::stream::StreamExt;
use zbus::{zvariant::Value, Connection, proxy, Proxy};
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;
#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenCast {
    fn CreateSession (&self,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>; //ObjectPath doesnt support lifetimes
    fn Start (&self,
        session_handle: OwnedObjectPath,
        app_id: &str,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<Vec<(u32, HashMap<String,OwnedValue>)>>;
    #[zbus(property)]
    #[allow(non_snake_case)]
    fn AvailableSourceTypes(&self) -> zbus::Result<u32>;
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let args = HashMap::<&str, Value>::from([
        ("session_handle_token", Value::new("oguri"))
    ]);

    let proxy = ScreenCastProxy::new(&connection).await?;
    let n = proxy.CreateSession(
        &args
    ).await?;
    let response_path: &str = &n.as_str();

    let listener_proxy = Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        response_path,
        "org.freedesktop.portal.Request",
    ).await?;

    println!("{}", response_path);
    let mut response_stream = listener_proxy.receive_all_signals().await?;
    while let Some(msg) = response_stream.next().await {
        println!("Received signal: {:?}", msg);
    }


    // let props = proxy.AvailableSourceTypes().await?;
    // let start = proxy.Start(
    //     n,
    //     "ble",
    //     &args

    // ).await?;



    Ok(())
}