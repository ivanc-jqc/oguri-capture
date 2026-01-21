use std::error::Error;

use zbus::proxy::SignalStream;
use zbus::{Connection, Proxy};

use super::SESSION_HANDLE_TOKEN;


pub async fn listener_stream<'a>(
    connection: &'a Connection,
    sender_id: &str,
) -> Result<SignalStream<'a>, Box<dyn Error>> {
    let concatenated =
        String::from("/org/freedesktop/portal/desktop/request/") + sender_id + "/" + SESSION_HANDLE_TOKEN;
    let listener_proxy = Proxy::new(
        connection,
        "org.freedesktop.portal.Desktop",
        concatenated,
        "org.freedesktop.portal.Request",
    )
    .await?;
    let response_stream = listener_proxy.receive_all_signals().await?;

    Ok(response_stream)
}
