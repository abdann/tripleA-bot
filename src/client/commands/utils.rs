use rand::Rng;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;

use crate::client::commands::errors;
use crate::client::database::errors::MarkovFetchResultError;
use crate::client::database::interface::DbInterface;

pub async fn parse_member(
    ctx: &Context,
    guild_id: GuildId,
    string: &str,
) -> std::result::Result<Member, errors::MemberNotFoundError> {
    match Member::convert(ctx, Some(guild_id), None, string).await {
        Ok(member) => Ok(member),
        Err(_why) => Err(errors::MemberNotFoundError::MemberNotFound(format!(
            "The member specified by {} was not found!",
            string
        ))),
    }
}

pub async fn parse_channel(
    ctx: &Context,
    guild_id: GuildId,
    string: &str,
) -> std::result::Result<Channel, errors::ChannelNotFoundError> {
    match Channel::convert(ctx, Some(guild_id), None, string).await {
        Ok(channel) => Ok(channel),
        Err(_why) => Err(errors::ChannelNotFoundError::ChannelNotFound(format!(
            "The channel specified by {} was not found!",
            string
        ))),
    }
}
