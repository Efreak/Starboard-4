use std::fmt::Debug;

use sqlx::PgPool;
use tokio::sync::RwLock;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_error::ErrorHandler;
use twilight_gateway::{
    cluster::{Cluster, Events, ShardScheme},
    Intents,
};
use twilight_http::client::{Client as HttpClient, InteractionClient};
use twilight_model::oauth::PartialApplication;

use crate::client::config::Config;

pub struct StarboardBot {
    pub cluster: Cluster,
    pub http: HttpClient,
    pub cache: RwLock<InMemoryCache>,
    pub application: RwLock<Option<PartialApplication>>,
    pub pool: PgPool,
    pub errors: ErrorHandler,
}

impl Debug for StarboardBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Starboard")
    }
}

impl StarboardBot {
    pub async fn new(config: Config) -> anyhow::Result<(Events, StarboardBot)> {
        // Setup gateway connection
        let scheme = ShardScheme::try_from((0..config.shards, config.shards))?;
        let intents = Intents::GUILDS
            | Intents::GUILD_EMOJIS_AND_STICKERS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_MESSAGE_REACTIONS;

        let (cluster, events) = Cluster::builder(config.token.clone(), intents)
            .shard_scheme(scheme)
            .build()
            .await?;

        // Setup HTTP connection
        let http = HttpClient::new(config.token.clone());

        // Setup cache
        let cache = InMemoryCache::builder()
            .resource_types(
                ResourceType::USER
                    | ResourceType::USER_CURRENT
                    | ResourceType::MEMBER
                    | ResourceType::MESSAGE
                    | ResourceType::GUILD
                    | ResourceType::CHANNEL
                    | ResourceType::ROLE
                    | ResourceType::EMOJI,
            )
            .message_cache_size(10_000)
            .build();

        // Setup database connection
        let pool = PgPool::connect(&config.db_url).await?;

        // Setup error handling
        let mut errors = ErrorHandler::new();
        if let Some(channel_id) = config.error_channel {
            errors.channel(channel_id.try_into().unwrap());
        }

        Ok((
            events,
            Self {
                cluster,
                http,
                cache: RwLock::new(cache),
                application: RwLock::new(None),
                pool,
                errors,
            },
        ))
    }

    pub async fn interaction_client<'a>(&'a self) -> anyhow::Result<InteractionClient<'a>> {
        match &*self.application.read().await {
            Some(info) => Ok(self.http.interaction(info.id)),
            None => Err(anyhow::anyhow!(
                "interaction_client called before bot was ready."
            )),
        }
    }
}
