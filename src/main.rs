use std::error::Error;
use screencast::ScreenCastSession;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sc = ScreenCastSession::new().await?;
    // let fd = get_pipewire_fd().await?;
    dbg!(sc.sender_id);

    Ok(())
}

mod screencast;
