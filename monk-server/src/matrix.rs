use matrix_sdk::{
    async_trait,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent, SyncMessageEvent,
    },
    reqwest::Url,
    room::Room,
    Client, ClientConfig, EventHandler, SyncSettings,
};
use tracing::info;

pub struct MonkMatrixClient;

#[async_trait]
impl EventHandler for MonkMatrixClient {
    async fn on_room_message(&self, room: Room, event: &SyncMessageEvent<MessageEventContent>) {
        info!("On Room Message");

        if let Room::Joined(room) = room {
            if let Ok(members) = room.active_members_no_sync().await {
                for member in members {
                    info!(
                        "[room {}] [member {}]",
                        room.name().unwrap_or_default(),
                        member.name()
                    );
                }
            }
        }

        let msg_body = if let SyncMessageEvent {
            content:
                MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                    ..
                },
            ..
        } = event
        {
            msg_body
        } else {
            return;
        };

        info!("[message]: {}", msg_body);
    }
}

pub async fn login_and_sync(
    homeserver_url: String,
    username: String,
    password: String,
) -> Result<(), matrix_sdk::Error> {
    // the location for `JsonStore` to save files to
    // let mut home = dirs::home_dir().expect("no home directory found");
    // home.push("party_bot");

    let store_path = "./matrix_store";

    let client_config = ClientConfig::new()
        .store_path(store_path)
        .passphrase("monk".to_owned());

    let homeserver_url = Url::parse(&homeserver_url).expect("Couldn't parse the homeserver URL");
    // create a new Client with the given homeserver url and config
    let client = Client::new_with_config(homeserver_url, client_config).unwrap();

    client
        .login(&username, &password, None, Some("command bot"))
        .await?;

    info!("logged in as {}", username);

    // An initial sync to set up state and so our bot doesn't respond to old
    // messages. If the `StateStore` finds saved state in the location given the
    // initial sync will be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await.unwrap();

    // add our CommandBot to be notified of incoming messages, we do this after the
    // initial sync to avoid responding to messages before the bot was running.
    client.set_event_handler(Box::new(MonkMatrixClient)).await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
    // this keeps state from the server streaming in to CommandBot via the
    // EventHandler trait
    client.sync(settings).await;

    Ok(())
}
