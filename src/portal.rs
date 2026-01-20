use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::{Value, ObjectPath, OwnedObjectPath};

#[proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait ScreenCast {
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
