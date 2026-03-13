#[tokio::main]
async fn main() {
    let connection = zbus::Connection::session().await.unwrap();
    let _proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Screenshot"
    ).await.unwrap();
}
