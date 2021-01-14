use std::{env, error::Error};
use twilight_http::{request::channel::allowed_mentions::AllowedMentionsBuilder, Client};
use twilight_model::id::{ChannelId, UserId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    //if we want to set the default for allowed mentions we need to use the builder, keep in mind these calls can't be chained!
    let client = Client::builder()
        //.proxy("localhost:8080", true)
        .token(env::var("DISCORD_TOKEN")?)
        //add an empty allowed mentions, this will prevent any and all pings
        .default_allowed_mentions(AllowedMentionsBuilder::new().build_solo())
        .build();
    let channel_id = ChannelId(480009549605240846);
    let user_id = UserId(356091260429402122);

    //here we want to warn a user about trying to ping everyone so we override to allow pinging them
    //but since we did not allow @everyone pings it will not ping everyone
    client
        .create_message(channel_id)
        .content(format!(
            "<@{}> you are not allowed to ping @everyone!",
            user_id.0
        ))?
        .allowed_mentions()
        .parse_specific_users(vec![user_id])
        .build()
        .await?;

    Ok(())
}
