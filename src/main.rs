use std::collections::HashMap;
use std::error::Error;
use zbus::{Connection, proxy};
use zbus::zvariant::{Value, ObjectPath, OwnedObjectPath, OwnedValue};
use std::{thread, time::Duration};

#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenCast {
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
    #[zbus(property)]
    #[allow(non_snake_case)]
    fn AvailableSourceTypes(&self) -> zbus::Result<u32>;
}


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
    //ENUM PLEASE
    let args = HashMap::from([
            ("handle_token", Value::new("oguri")),
            // ("types", Value::new(1u32)),
            // ("multiple", Value::new(false)),
            // ("cursor_mode", Value::new(2u32)),
            // ("persist_mode", Value::new(1u32))
        ]);

    // returns objpath "/org/freedesktop/portal/desktop/request/{ID}/oguri"
    let o = proxy.SelectSources(
        &session_handle,
        &args,
    ).await?;

    let args = HashMap::from([
            ("handle_token", Value::new("oguri"))
        ]);
    thread::sleep_ms(5000);
    // "/org/freedesktop/portal/desktop/request/{ID}/oguri"
    let p = proxy.Start(
        &session_handle,
        "",
        &args,
    ).await?;



    dbg!(p);
   
   
    // /org/freedesktop/portal/desktop/session/1_186/oguri

    // let props = proxy.AvailableSourceTypes().await?;
    // let start = proxy.Start(
    //     n,
    //     "ble",
    //     &args

    // ).await?;



    Ok(())
}


