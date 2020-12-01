use crate::minecraft::MinecraftMessage;
use serenity::async_trait;
use tokio::sync::mpsc::Sender;
use warp::Filter;

#[async_trait]
pub trait Listener {
    async fn listen(&self, tx: Sender<MinecraftMessage>);
}

pub struct Webserver {
    port: u16,
}

impl Webserver {
    pub fn new(port: u16) -> Self {
        Webserver { port }
    }
}

#[async_trait]
impl Listener for Webserver {
    async fn listen(&self, tx: Sender<MinecraftMessage>) {
        // POST /message/:msg
        let messages = warp::post()
            .and(warp::path("message"))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and_then(move |msg: MinecraftMessage| {
                let mut txc = tx.clone();
                async move {
                    match txc.send(msg).await {
                        Ok(()) => Ok(""),
                        Err(_) => Err(warp::reject::reject()),
                    }
                }
            });

        warp::serve(messages).run(([127, 0, 0, 1], self.port)).await
    }
}
