use std::collections::HashMap;
use std::error::Error;
use futures_util::stream::StreamExt;
use zbus::{zvariant::Value, Connection, proxy, Proxy};
use zbus::zvariant::ObjectPath;
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Structure;
use zbus::zvariant::Dict;
//fileio
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


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

    // let rule = MatchRule::builder()
    //         .interface("org.freedesktop.login1.Manager")?
    //         .build();
    // connection.add_match_rule(rule).await?;
    let listener_proxy = Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop/request/1_185/t",
        "org.freedesktop.portal.Request",
    ).await?;

    let mut response_stream = listener_proxy.receive_all_signals().await?;

    let args = HashMap::<&str, Value>::from([
        ("session_handle_token", Value::new("oguri"))
    ]);

    let proxy = ScreenCastProxy::new(&connection).await?;
    let n = proxy.CreateSession(
        &args
    ).await?;
    let response_path: &str = &n.as_str();
    dbg!(response_path);
   
    while let Some(msg) = response_stream.next().await {
        let temp = msg.body();
        let temp2: Dict = temp.deserialize::<Structure>()
        .unwrap()
        .into_fields()[1]
        .clone()
        .downcast()
        .unwrap();
        let temp3: HashMap<String, Value> = HashMap::<String, Value>::try_from(temp2).unwrap();
        dbg!(temp3);
    }


    // let props = proxy.AvailableSourceTypes().await?;
    // let start = proxy.Start(
    //     n,
    //     "ble",
    //     &args

    // ).await?;



    Ok(())
}