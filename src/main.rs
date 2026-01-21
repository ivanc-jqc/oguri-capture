use std::error::Error;
use screencast::ScreenCastSession;
use zbus::Connection;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;
    let sc = ScreenCastSession::new(&connection).await?;
    // let fd = get_pipewire_fd().await?;
    sc.select_sources().await?;
    
    
    dbg!(sc.session_handle);
    
    Ok(())
}

mod screencast;
