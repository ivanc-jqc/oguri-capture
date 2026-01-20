use std::collections::HashMap;
use std::error::Error;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, Structure, Value};
use portal::{ScreenCastProxy,listener_stream};
use futures_util::stream::StreamExt;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const SESSION_HANDLE_TOKEN: &str = "oguri";
    let connection = Connection::session().await?;

    let args = HashMap::<&str, Value>::from([
        ("session_handle_token", Value::new(SESSION_HANDLE_TOKEN))
    ]);
    
    // returns objpath /org/freedesktop/portal/desktop/session/{ID}/oguri
    let proxy = ScreenCastProxy::new(&connection).await?;
    let n = proxy.CreateSession(
        &args
    ).await?;

    let sender_id: &str = &n.as_str().split("/").nth(6).unwrap();
    let mut stream = listener_stream(&connection, 
                                        &sender_id, 
                                        SESSION_HANDLE_TOKEN).await?;
    
    //ENUM PLEASE
    let args = HashMap::from([
            ("handle_token", Value::new("oguri")),
            // ("types", Value::new(1u32)),
            // ("multiple", Value::new(false)),
            // ("cursor_mode", Value::new(2u32)),
            // ("persist_mode", Value::new(1u32))
        ]);


    let session_handle = ObjectPath::try_from(String::from("/org/freedesktop/portal/desktop/session/") + 
                sender_id + 
                "/" + 
                SESSION_HANDLE_TOKEN)? ;
    // returns objpath "/org/freedesktop/portal/desktop/request/{ID}/oguri"
    proxy.SelectSources(
        &session_handle,
        &args,
    ).await?;
   
    
    stream.next().await;


    proxy.Start(
        &session_handle,
        "",
        &args,
    ).await?;
  
    let body = stream.next().await.unwrap().body();
    let parsed = body.deserialize::<Structure>().unwrap();
    dbg!(parsed);
        
    


   


    Ok(())
}

mod portal;