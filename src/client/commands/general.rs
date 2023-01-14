use rand::Rng;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::ArgumentConvert;

use crate::client::commands::utils;
use crate::client::database::errors::{InsertResult, MarkovFetchResultError};
use crate::client::database::interface::DbInterface;

const MIN_SENTENCE_LENGTH: u8 = 4;
const MAX_SENTENCE_LENGTH: u8 = 20;

#[command]
#[description("Mimic the specified member.")]
#[min_args(1)]
#[max_args(1)]
pub async fn mimic(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Argument parsing here
    let member: Member;
    if args.len() == 0 {
        msg.reply(&ctx.http, "Please mention a member!").await;
        return Ok(());
    }
    // Fine to use unwrap here since we already checked if we have at least 1 arg
    let possible_member = args.current().unwrap();
    match Member::convert(ctx, msg.guild_id, Some(msg.channel_id), possible_member).await {
        Ok(member_found) => member = member_found,
        Err(why) => {
            msg.reply(
                &ctx.http,
                format!(
                    "The following error occurred while parsing the first argument: {}",
                    why
                ),
            )
            .await;
            return Ok(());
        }
    }
    // Obtain db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Db Interface is definitely here")
        .lock()
        .await;
    // Check that member is stored
    let internal_member_id: u32;
    match db_int
        .fetch_member(
            msg.guild_id.expect("Should be in a guild").0,
            member.user.id.0,
        )
        .await
    {
        Ok(possible_member) => {
            if let Some(member_found) = possible_member {
                internal_member_id = member_found;
            } else {
                msg.reply(&ctx.http, format!("{} is not a member that I know about! Help me learn about the musing the `trackmember` command!", member.mention())).await;
                return Ok(());
            }
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error has occurred: {}", why))
                .await;
            return Ok(());
        }
    }

    // Random sentence length
    let sentence_length = MAX_SENTENCE_LENGTH
        * (rand::thread_rng().gen_range(MIN_SENTENCE_LENGTH..=MAX_SENTENCE_LENGTH)
            / MAX_SENTENCE_LENGTH);
    // Now fetch words from db
    match db_int
        .fetch_random_member_words_into_sentence(internal_member_id, sentence_length)
        .await
    {
        Ok(sentence) => {
            msg.reply(&ctx.http, sentence).await;
            return Ok(());
        }
        Err(ref why) => match why {
            MarkovFetchResultError::NotEnoughWords(
                ref desired_sentence_length,
                ref generated_sentence_length,
            ) => {
                msg.reply(&ctx.http, format!("I didn't know enough words to generate the desired sentence length of {} words. (I was able to get {} words).", desired_sentence_length, generated_sentence_length)).await;
                return Ok(());
            }
            MarkovFetchResultError::SqlxError(ref sqlxerror) => {
                msg.reply(
                    &ctx.http,
                    format!("An SQLx error has occurred: {}", sqlxerror),
                )
                .await;
                return Ok(());
            }
        },
    };
}

#[command]
#[description("Begin learning about a member.")]
#[required_permissions("MANAGE_MESSAGES")]
#[min_args(1)]
#[max_args(1)]
pub async fn trackmember(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 0 {
        msg.reply(&ctx.http, "Please mention a member!").await;
        return Ok(());
    }
    //Argument parsing here
    let member: Member;
    match utils::parse_member(
        ctx,
        msg.guild_id.expect("Should be in a guild"),
        args.current().unwrap(),
    )
    .await
    {
        Ok(member_found) => member = member_found,
        Err(why) => {
            msg.reply(&ctx.http, why).await;
            return Ok(());
        }
    }
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;
    match db_int
        .add_tracked_member(
            msg.guild_id.expect("Should be in a guild").0,
            member.user.id.0,
        )
        .await
    {
        Ok(crate::client::database::errors::InsertResult::Added) => {
            msg.reply(
                &ctx.http,
                format!("Added {} to list of tracked members!", member.mention()),
            )
            .await;
            return Ok(());
        }
        Ok(crate::client::database::errors::InsertResult::AlreadyPresent) => {
            msg.reply(
                &ctx.http,
                format!("{} is already tracked!", member.mention()),
            )
            .await;
            return Ok(());
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why))
                .await;
            return Ok(());
        }
    }
}

#[command]
#[description("Stop learning about a member.")]
#[required_permissions("MANAGE_MESSAGES")]
#[min_args(1)]
#[max_args(1)]
pub async fn untrackmember(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Argument parsing
    if args.len() == 0 {
        msg.reply(&ctx.http, "Please mention a member!").await;
        return Ok(());
    }
    let member: Member;
    match utils::parse_member(
        ctx,
        msg.guild_id.expect("Should be in a guild"),
        args.current().unwrap(),
    )
    .await
    {
        Ok(member_found) => member = member_found,
        Err(why) => {
            msg.reply(&ctx.http, why).await;
            return Ok(());
        }
    }
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;

    match db_int
        .remove_tracked_member(
            msg.guild_id.expect("Should be in a guild").0,
            member.user.id.0,
        )
        .await
    {
        Ok(crate::client::database::errors::RemoveResult::Removed) => {
            msg.reply(
                &ctx.http,
                format!(
                    "Removed {} from the list of tracked members!",
                    member.mention()
                ),
            )
            .await;
            Ok(())
        }
        Ok(crate::client::database::errors::RemoveResult::NotPresent) => {
            msg.reply(
                &ctx.http,
                format!("{} is not a tracked member!", member.mention()),
            )
            .await;
            Ok(())
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why));
            return Ok(());
        }
    }
}

#[command]
#[description("List tracked members.")]
#[required_permissions("MANAGE_MESSAGES")]
#[min_args(0)]
#[max_args(0)]
pub async fn listtrackedmembers(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;

    match db_int
        .fetch_tracked_user_members(msg.guild_id.expect("Should be in a server").0)
        .await
    {
        Ok(possible_results) => {
            if let Some(tracked_members) = possible_results {
                let mut message = serenity::utils::MessageBuilder::new();
                message.push_bold_line("Current tracked members:");
                let mut former_members = String::new();
                for member in tracked_members.into_iter() {
                    match Member::convert(ctx, msg.guild_id, None, &member.to_string()).await {
                        Ok(current_member) => {
                            message.push_line(current_member.mention());
                        } // This assumes that any error in parsing the user ID into a member is indicative of the user no longer being a member of the server.
                        Err(why) => {
                            former_members.push_str(&format!("{}\n", member.to_string()));
                        }
                    }
                }
                message.push_bold_line("The following user IDs are tracked, but they are no longer members of this server. Please remove them using the `untrackmember` command:");
                message.push(former_members);
                msg.reply(&ctx.http, message.build()).await;
            } else {
                msg.reply(&ctx.http, "No members are being tracked.").await;
            }
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why));
        }
    }
    Ok(())
}

#[command]
#[description("Track messages in a channel")]
#[required_permissions("MANAGE_MESSAGES")]
pub async fn trackchannel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Argument parsing
    if args.len() == 0 {
        msg.reply(&ctx.http, "Please specify a channel!").await;
        return Ok(());
    }
    let channel: Channel;
    match utils::parse_channel(
        ctx,
        msg.guild_id.expect("Should be in a guild"),
        args.current().unwrap(),
    )
    .await
    {
        Ok(channel_found) => channel = channel_found,
        Err(why) => {
            msg.reply(&ctx.http, why).await;
            return Ok(());
        }
    }
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;

    match db_int
        .add_tracked_channel(
            msg.guild_id.expect("Should be in a server").0,
            msg.channel_id.0,
        )
        .await
    {
        Ok(crate::client::database::errors::InsertResult::Added) => {
            msg.reply(
                &ctx.http,
                format!("Added {} to list of tracked channels!", channel.mention()),
            )
            .await;
            return Ok(());
        }
        Ok(crate::client::database::errors::InsertResult::AlreadyPresent) => {
            msg.reply(
                &ctx.http,
                format!("{} is already tracked!", channel.mention()),
            )
            .await;
            return Ok(());
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why))
                .await;
            return Ok(());
        }
    }
}

#[command]
#[description("Stop tracking messages in a channel")]
#[required_permissions("MANAGE_MESSAGES")]
pub async fn untrackchannel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Argument parsing
    if args.len() == 0 {
        msg.reply(&ctx.http, "Please specify a channel!").await;
        return Ok(());
    }
    let channel: Channel;
    match utils::parse_channel(
        ctx,
        msg.guild_id.expect("Should be in a guild"),
        args.current().unwrap(),
    )
    .await
    {
        Ok(channel_found) => channel = channel_found,
        Err(why) => {
            msg.reply(&ctx.http, why).await;
            return Ok(());
        }
    }
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;

    match db_int
        .remove_tracked_channel(
            msg.guild_id.expect("Should be in a guild").0,
            msg.channel_id.0,
        )
        .await
    {
        Ok(crate::client::database::errors::RemoveResult::Removed) => {
            msg.reply(
                &ctx.http,
                format!(
                    "Removed {} from the list of tracked channels!",
                    channel.mention()
                ),
            )
            .await;
            Ok(())
        }
        Ok(crate::client::database::errors::RemoveResult::NotPresent) => {
            msg.reply(
                &ctx.http,
                format!("{} is not a tracked channel!", channel.mention()),
            )
            .await;
            Ok(())
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why));
            return Ok(());
        }
    }
}

#[command]
#[description("List tracked channels.")]
#[required_permissions("MANAGE_MESSAGES")]
pub async fn listtrackedchannels(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get db interface
    let map = ctx.data.read().await;
    let db_int = map
        .get::<DbInterface>()
        .expect("Should have DB here")
        .lock()
        .await;

    match db_int
        .fetch_tracked_channels(msg.guild_id.expect("Should be in a server").0)
        .await
    {
        Ok(possible_results) => {
            if let Some(tracked_channels) = possible_results {
                let mut message = serenity::utils::MessageBuilder::new();
                message.push_bold_line("Current tracked channels:");
                let mut former_channels = String::new();
                for channel in tracked_channels.into_iter() {
                    match Channel::convert(ctx, msg.guild_id, None, &channel.to_string()).await {
                        Ok(current_channel) => {
                            message.push_line(current_channel.mention());
                        } // This assumes that any error in parsing the channel ID into a channel is indicative of the channel no longer being visible to the bot due to permissions, or due to the channel no longer existing.
                        Err(why) => {
                            former_channels.push_str(&format!("{}\n", channel.to_string()));
                        }
                    }
                }
                message.push_bold_line("The following channel IDs are tracked, but they are either no longer visible to me or they no longer exist. Please check my permissions and/or remove them using the `untrackchannel` command:");
                message.push(former_channels);
                msg.reply(&ctx.http, message.build()).await;
            } else {
                msg.reply(&ctx.http, "No channels are being tracked.").await;
            }
        }
        Err(why) => {
            msg.reply(&ctx.http, format!("An SQLx error occurred: {}", why));
        }
    }
    Ok(())
}
