use std::collections::HashMap;
use std::error::Error;
use zbus::{Connection, Proxy};
use zbus::zvariant::{ObjectPath, Structure, Value};
use portal::ScreenCastProxy;
use futures_util::stream::StreamExt;
use zbus::proxy::SignalStream;
use std::{thread,time};

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
    let session_handle = ObjectPath::try_from(String::from("/org/freedesktop/portal/desktop/session/") + 
                sender_id + 
                "/" + 
                SESSION_HANDLE_TOKEN)? ;


// while let Some(msg) = response_stream.next().await {
//     let temp = msg.body();
//     let temp2: Dict = temp.deserialize::<Structure>()
//     .unwrap()
//     .into_fields()[1]
//     .clone()
//     .downcast()
//     .unwrap();
//     let temp3: HashMap<String, Value> = HashMap::<String, Value>::try_from(temp2).unwrap();
//     dbg!(temp3);
// }
    async fn listener_stream<'a>(connection: &'a Connection,sender_id: &str) -> Result<SignalStream<'a>,Box<dyn Error>> {
        let concatenated = String::from("/org/freedesktop/portal/desktop/request/")+sender_id+"/"+SESSION_HANDLE_TOKEN;
        let listener_proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            concatenated,
            "org.freedesktop.portal.Request",
        ).await?;
        let response_stream= listener_proxy.receive_all_signals().await?;

        Ok(response_stream)
    }
    let mut stream = listener_stream(&connection, &sender_id).await?;
    








  
    
    
    
    
    
    
    
    
    
                //ENUM PLEASE
    let args = HashMap::from([
            ("handle_token", Value::new("oguri")),
            // ("types", Value::new(1u32)),
            // ("multiple", Value::new(false)),
            // ("cursor_mode", Value::new(2u32)),
            // ("persist_mode", Value::new(1u32))
        ]);

    // returns objpath "/org/freedesktop/portal/desktop/request/{ID}/oguri"
    proxy.SelectSources(
        &session_handle,
        &args,
    ).await?;
   
    
    let args = HashMap::from([
            ("handle_token", Value::new("oguri"))
        ]);
    stream.next().await;
    proxy.Start(
        &session_handle,
        "",
        &args,
    ).await?;
    while let Some(msg) = stream.next().await {
        let body = msg.body();
        let parsed = body.deserialize::<Structure>().unwrap();
        dbg!(parsed);
    }

    // "/org/freedesktop/portal/desktop/request/{ID}/oguri"




   


    Ok(())
}

mod portal;