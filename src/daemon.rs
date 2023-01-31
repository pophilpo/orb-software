pub mod client;
pub mod dbus;
pub mod logging;
pub mod remote_api;

use std::sync::Arc;

use eyre::{self, WrapErr};
use futures::{select, FutureExt};
use tokio::{sync::Notify, time::sleep};
use tracing::info;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();

    info!("Build Timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    info!("git sha: {}", env!("VERGEN_GIT_SHA"));
    #[cfg(feature = "prod")]
    info!("build for PROD backend");
    #[cfg(not(feature = "prod"))]
    info!("build for STAGING backend");

    let orb_id = std::env::var("ORB_ID").wrap_err("env variable `ORB_ID` should be set")?;

    let force_refresh_token = Arc::new(Notify::new());

    let iface_ref = setup_dbus(force_refresh_token.clone())
        .await
        .wrap_err("Initialization failed")?;
    run(&orb_id, iface_ref, force_refresh_token.clone())
        .await
        .wrap_err("mainloop failed")
}

#[tracing::instrument]
async fn setup_dbus(
    force_refresh_token: Arc<Notify>,
) -> eyre::Result<zbus::InterfaceRef<dbus::AuthTokenManager>> {
    let dbus = dbus::create_dbus_connection(force_refresh_token)
        .await
        .wrap_err("failed to create DBus connection")?;

    let object_server = dbus.object_server();
    let iface_ref = object_server
        .interface::<_, dbus::AuthTokenManager>("/org/worldcoin/AuthTokenManager")
        .await
        .wrap_err("failed to get reference to AuthTokenManager from object server")?;

    Ok(iface_ref)
}

async fn run(
    orb_id: &str,
    iface_ref: zbus::InterfaceRef<dbus::AuthTokenManager>,
    force_refresh_token: Arc<Notify>,
) -> eyre::Result<()> {
    loop {
        let token = remote_api::get_token(orb_id).await;
        let token_refresh_delay = token.get_best_refresh_time();
        // get_mut() blocks access to the iface_ref object. So we never bind its result to be safe.
        // https://docs.rs/zbus/3.7.0/zbus/struct.InterfaceRef.html#method.get_mut
        iface_ref.get_mut().await.update_token(&token.token);
        iface_ref
            .get_mut()
            .await
            .token_changed(iface_ref.signal_context())
            .await?;

        //  Wait for whatever happens first: token expires or a refresh is requested
        select! {
            () = sleep(token_refresh_delay).fuse() => {info!("token is about to expire, refreshing it");},
            _ = force_refresh_token.notified().fuse() => {info!("refresh was requested, refreshing the token");},
        };
    }
}
