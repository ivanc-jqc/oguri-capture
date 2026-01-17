use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, Connection};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let sesh = Value::new("sesh");
    let args = HashMap::<&str, &Value>::from([
        ("session_handle_token", &sesh)
    ]);
    let m = connection.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.ScreenCast"),
        "CreateSession",
        &(args),
    ).await?;

    //&("my-app", 0u32, "dialog-information", "A summary", "Some body",
    //vec![""; 0], HashMap::<&str, &Value>::new(), 5000),
    dbg!(m);
    Ok(())
}