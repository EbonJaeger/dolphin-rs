use crate::minecraft::{MessageParser, MinecraftMessage};
use linemux::MuxedLines;
use serenity::async_trait;
use tokio::{stream::StreamExt, sync::mpsc::Sender};
use tracing::{error, info};
use warp::Filter;

///
/// A Listener listens or watches for new messages from a Minecraft instance,
/// depending on the implementation. Listeners require a channel to pass messages
/// through so they can be sent to Discord.
///
#[async_trait]
pub trait Listener {
    ///
    /// Begin listening for messages from Minecraft. Usually you'll want to
    /// call this from an async thread so it doesn't block the rest of the
    /// program.
    ///
    async fn listen(&self, tx: Sender<MinecraftMessage>);
}

///
/// Registers a file event listener to watch for new lines to be added
/// to a file at a given path.
///
/// # Examples
///
/// ```rust
/// let (tx, mut rx) = mpsc::channel(100);
/// let log_tailer = LogTailer::new("/home/minecraft/server/logs/latest.log", Vec::new());
/// tokio::spawn(async move { log_tailer.listen(tx).await });
/// ```
///
pub struct LogTailer {
    path: String,
    custom_keywords: Vec<String>,
}

impl LogTailer {
    pub fn new(path: String, custom_keywords: Vec<String>) -> Self {
        LogTailer {
            path,
            custom_keywords,
        }
    }
}

#[async_trait]
impl Listener for LogTailer {
    async fn listen(&self, tx: Sender<MinecraftMessage>) {
        info!("log_tailer:listen: using log file at '{}'", self.path);
        let parser = MessageParser::new(self.custom_keywords.clone());

        // Create our log watcher
        let mut log_watcher = MuxedLines::new().unwrap();
        log_watcher
            .add_file(&self.path)
            .await
            .expect("Unable to add the Minecraft log file to tail");

        info!("log_tailer:listen: started watching the Minecraft log file");
        let mut txc = tx.clone();

        // Wait for the next line
        while let Some(Ok(line)) = log_watcher.next().await {
            let message = match parser.parse_line(line.line()) {
                Some(message) => message,
                None => continue,
            };

            // Send it down the pipe
            if let Err(e) = txc.send(message).await {
                error!(
                    "log_tailer:listen: unable to send message through channel: {}",
                    e
                );
            }
        }
    }
}

///
/// Binds to an IP address and port to listen for messages over a network.
/// It watches for messages at the `/message` endpoint.
///
/// # Examples
///
/// ```rust
/// let (tx, mut rx) = mpsc::channel(100);
/// let listener = Webserver::new(25585);
/// listener.listen(tx).await;
/// ```
///
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
            .and_then(move |message: MinecraftMessage| {
                let mut txc = tx.clone();
                async move {
                    match txc.send(message).await {
                        Ok(()) => Ok(""),
                        Err(_) => Err(warp::reject::reject()),
                    }
                }
            });

        // TODO: Maybe figure out how to bind to a configurable address?
        warp::serve(messages).run(([0, 0, 0, 0], self.port)).await
    }
}
