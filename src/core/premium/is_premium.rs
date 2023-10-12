use crate::{client::bot::StarboardBot, errors::StarboardResult};

pub async fn is_guild_premium(
    bot: &StarboardBot,
    guild_id: i64,
    allow_cache: bool,
) -> StarboardResult<bool> {
    if allow_cache {
        let cached = bot.cache.guild_premium.with(&guild_id, |_, is_premium| {
            is_premium.as_ref().map(|v| *v.value())
        });
        if let Some(cached) = cached {
            return Ok(cached);
        };
    }

    let is_premium = true;

    bot.cache.guild_premium.insert(guild_id, is_premium);
    Ok(is_premium)
}
