use crate::config::RootConfig;
use crate::discord::send_to_discord;
use crate::minecraft::MinecraftMessage;
use serenity::{async_trait, model::id::GuildId, prelude::Context};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::debug;
use warp::Filter;

#[async_trait]
pub trait Listener {
    async fn listen(&self);
}

pub struct Webserver {
    ctx: Arc<Context>,
    cfg: Arc<RootConfig>,
    guild_id: Arc<GuildId>,
    // tx: Sender<MinecraftMessage>,
}

impl Webserver {
    pub fn new(
        ctx: Arc<Context>,
        cfg: Arc<RootConfig>,
        guild_id: Arc<GuildId>,
        // tx: Sender<MinecraftMessage>,
    ) -> Self {
        Webserver {
            ctx,
            cfg,
            guild_id,
            // tx,
        }
    }
}

#[async_trait]
impl Listener for Webserver {
    async fn listen(&self) {
        // let txc = tx.clone();
        // POST /message/:msg
        let messages = warp::post()
            .and(warp::path("message"))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .map(|message: MinecraftMessage| {
                debug!(
                    "dolphin:warp: received a message from a Minecraft instance: {:?}",
                    message
                );

                warp::reply()
            });
        // .and_then(move |msg: MinecraftMessage| async move {
        //     debug!(
        //         "dolphin:warp: received a message from a Minecraft instance: {:?}",
        //         msg
        //     );
        //     match txc.send(msg).await {
        //         Ok(()) => Ok(format!("post success")),
        //         Err(e) => Err(warp::reject::not_found()),
        //     }
        // });

        warp::serve(messages)
            .run(([127, 0, 0, 1], self.cfg.get_listener_port()))
            .await
    }
}
