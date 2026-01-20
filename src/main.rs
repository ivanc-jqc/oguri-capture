use std::collections::HashMap;
use std::error::Error;
use zbus::{Connection, proxy};
use zbus::zvariant::{Value, ObjectPath, OwnedObjectPath, OwnedValue};


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
    const SESSION_HANDLE_TOKEN: &str = "oguri";
    let connection = Connection::session().await?;

    let args = HashMap::<&str, Value>::from([
        ("session_handle_token", Value::new(SESSION_HANDLE_TOKEN))
    ]);

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
            ("handle_token", Value::new("bruh")),
            ("types", Value::new(1u32)),
            ("multiple", Value::new(false)),
            ("cursor_mode", Value::new(2u32)),
            ("persist_mode", Value::new(1u32))
        ]);

    // returns objpath "/org/freedesktop/portal/desktop/request/{ID}/bruh"
    let o = proxy.SelectSources(
        &session_handle,
        &args,
    ).await?;




    dbg!(o);
   
   
    // /org/freedesktop/portal/desktop/session/1_186/oguri

    // let props = proxy.AvailableSourceTypes().await?;
    // let start = proxy.Start(
    //     n,
    //     "ble",
    //     &args

    // ).await?;



    Ok(())
}


