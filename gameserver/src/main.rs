use crate::{
    network::client::handle_client,
    state::{AppState, ConnectionContext},
};
use ::config::configs;
use common::{config, excel_data_directory, game_port, host, init_config, init_tracing};
use database::{DatabaseSettings, connect_to, run_migrations};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;

mod error;
mod handlers;
mod network;
mod state;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let mut cfg = config::ServerConfig::load_or_create(&config_path)?;

    let config_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    cfg.resolve_paths(&config_dir)?;
    cfg.validate_paths()?;

    info!("Server configuration:");
    info!("  Host: {}:{}", cfg.server.host, cfg.server.game_port);

    init_config(cfg.clone());

    let db_settings = DatabaseSettings {
        db_name: config().database.path.to_string_lossy().to_string(),
    };

    let db = connect_to(&db_settings).await?;
    run_migrations(&db).await?;

    info!("Loading game data...");
    configs::init(excel_data_directory().to_str().unwrap())?;
    info!("Game data loaded");

    let state = Arc::new(AppState::new(db));
    let addr = format!("{}:{}", host(), game_port());
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on tcp://{}", &addr);

    loop {
        let (raw_socket, client) = listener.accept().await?;
        tracing::info!("New client connected: {:?}", client);

        let state = state.clone();
        let socket = Arc::new(Mutex::new(raw_socket));

        tokio::spawn(async move {
            let ctx = Arc::new(Mutex::new(ConnectionContext::new(
                socket.clone(),
                state.clone(),
            )));

            let result = handle_client(ctx.clone()).await;

            let conn = ctx.lock().await;
            if let Some(player_id) = conn.player_id {
                if let Err(e) = conn.save_current_player_state().await {
                    tracing::error!("Failed to save player state for {}: {}", player_id, e);
                }

                tracing::warn!("Player {} disconnected and saved progress", player_id);
                conn.state.unregister_session(player_id);
            }

            if let Err(e) = result {
                tracing::error!("Client handler error: {e}");
            }
        });
    }
}
