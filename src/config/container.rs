use std::{path::PathBuf, sync::Arc};

use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;

use super::RootConfig;

pub struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
    type Value = Arc<RwLock<RootConfig>>;
}

pub struct ConfigPathContainer;

impl TypeMapKey for ConfigPathContainer {
    type Value = Arc<PathBuf>;
}
