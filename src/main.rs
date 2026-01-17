use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, Connection, proxy};
use zbus::zvariant::OwnedObjectPath;
#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenCast {
    fn CreateSession (&self,
        options: &HashMap::<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>; 
}

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let args = HashMap::<&str, Value>::from([
        ("session_handle_token", Value::new("sesh"))
    ]);

    let proxy = ScreenCastProxy::new(&connection).await?;
    let n = proxy.CreateSession(
        &args
    ).await?;
    // let n = connection.call_method(
    //     Some("org.freedesktop.portal.Desktop"),
    //     "/org/freedesktop/portal/desktop",
    //     Some("org.freedesktop.portal.ScreenCast"),
    //     "CreateSession",
    //     &(args),
    // ).await?;
    // &("my-app", 0u32, "dialog-information", "A summary", "Some body",
    // vec![""; 0], HashMap::<&str, &Value>::new(), 5000),
    
    dbg!(&n.as_str());
    // let binding = n.body();
    // let deserialized_obj: ObjectPath = binding.deserialize::<ObjectPath>().unwrap();
    // let reply: &str = deserialized_obj.as_str();
    // dbg!(reply);
    Ok(())
}