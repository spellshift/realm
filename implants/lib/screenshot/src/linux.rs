use tokio_stream::StreamExt;
#[cfg(feature = "stdlib")]
use image::RgbaImage;

#[cfg(feature = "stdlib")]
pub fn capture_monitors() -> Result<Vec<RgbaImage>, String> {
    // Try ZBus native wayland/X11 screenshot via XDG Desktop Portal
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    match runtime.block_on(capture_xdg_portal()) {
        Ok(images) if !images.is_empty() => return Ok(images),
        _ => {}
    }

    Err("Failed to capture screenshot natively via XDG Portal (zbus).".into())
}

#[cfg(feature = "stdlib")]
async fn capture_xdg_portal() -> Result<Vec<RgbaImage>, zbus::Error> {
    use zbus::Connection;
    use std::collections::HashMap;

    let connection = Connection::session().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Screenshot"
    ).await?;

    let unique_name = connection.unique_name().ok_or_else(|| zbus::Error::Failure("No unique name".into()))?;
    let sender = unique_name.as_str().replace('.', "_").replace(':', "");
    let handle_token = format!("screenshot_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

    // Construct the expected request object path
    let request_path = format!("/org/freedesktop/portal/desktop/request/{}/{}", sender, handle_token);

    // Create a proxy to the expected Request object and subscribe to Response BEFORE calling Screenshot
    let request_proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        request_path,
        "org.freedesktop.portal.Request"
    ).await?;

    let mut stream = request_proxy.receive_signal("Response").await?;

    // Prepare options
    let mut options = HashMap::new();
    options.insert("handle_token", zbus::zvariant::Value::from(handle_token));
    options.insert("interactive", zbus::zvariant::Value::from(false));

    // Call Screenshot method (we don't wait for its response path since we already have it)
    let args = ("", options);
    let _ = proxy.call::<&str, (&str, HashMap<&str, zbus::zvariant::Value>), zbus::zvariant::OwnedValue>("Screenshot", &args).await?;

    // Wait for the signal
    let msg = stream.next().await.ok_or_else(|| zbus::Error::Failure("Stream closed before receiving signal".into()))?;

    let body = msg.body();
    let (response_code, results): (u32, std::collections::HashMap<String, zbus::zvariant::OwnedValue>) = body.deserialize()?;

    if response_code != 0 {
        return Err(zbus::Error::Failure(format!("Portal request failed with code {}", response_code)));
    }

    if let Some(uri_val) = results.get("uri") {
        if let Ok(uri_str) = <&str>::try_from(uri_val) {
            if let Ok(url) = url::Url::parse(uri_str) {
                if let Ok(file_path) = url.to_file_path() {
                    if let Ok(img) = image::open(&file_path) {
                        return Ok(vec![img.to_rgba8()]);
                    }
                }
            }
        }
    }

    Err(zbus::Error::Failure("Could not extract image from response".into()))
}

#[cfg(not(feature = "stdlib"))]
pub fn capture_monitors() -> Result<Vec<()>, String> {
    Err("capture_monitors requires the stdlib feature".into())
}
