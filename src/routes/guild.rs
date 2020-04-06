use crate::database::{self, channel::Channel, guild::Guild, user::User};

use bson::{bson, doc, from_bson, Bson};
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::channel::ChannelType;

/// fetch your guilds
#[get("/@me")]
pub fn my_guilds(user: User) -> JsonValue {
    let col = database::get_collection("guilds");
    let guilds = col
        .find(
            doc! {
                "members": {
                    "$elemMatch": {
                        "id": user.id,
                    }
                }
            },
            None,
        )
        .unwrap();

    let mut parsed = vec![];
    for item in guilds {
        let doc = item.unwrap();
        parsed.push(json!({
            "id": doc.get_str("_id").unwrap(),
            "name": doc.get_str("name").unwrap(),
            "description": doc.get_str("description").unwrap(),
            "owner": doc.get_str("owner").unwrap(),
        }));
    }

    json!(parsed)
}

/// fetch a guild
#[get("/<target>")]
pub fn guild(user: User, target: Guild) -> JsonValue {
    let mut targets = vec![];
    for channel in target.channels {
        targets.push(Bson::String(channel));
    }

    let col = database::get_collection("channels");
    match col.find(
        doc! {
            "_id": {
                "$in": targets,
            }
        },
        None,
    ) {
        Ok(results) => {
            let mut channels = vec![];
            for item in results {
                let channel: Channel = from_bson(bson::Bson::Document(item.unwrap()))
                    .expect("Failed to unwrap channel.");

                channels.push(json!({
                    "_id": channel.id,
                    "last_message": channel.last_message,
                    "name": channel.name,
                    "description": channel.description,
                }));
            }

            json!({
                "id": target.id,
                "name": target.name,
                "description": target.description,
                "owner": target.owner,
                "channels": channels,
            })
        }
        Err(_) => json!({
            "success": false,
            "error": "Failed to fetch channels."
        }),
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGuild {
    name: String,
    description: Option<String>,
    nonce: String,
}

/// send a message to a channel
#[post("/create", data = "<info>")]
pub fn create_guild(user: User, info: Json<CreateGuild>) -> JsonValue {
    if !user.email_verification.verified {
        return json!({
            "success": false,
            "error": "Email not verified!",
        });
    }

    let name: String = info.name.chars().take(32).collect();
    let description: String = info
        .description
        .clone()
        .unwrap_or("No description.".to_string())
        .chars()
        .take(255)
        .collect();
    let nonce: String = info.nonce.chars().take(32).collect();

    let channels = database::get_collection("channels");
    let col = database::get_collection("guilds");
    if let Some(_) = col.find_one(doc! { "nonce": nonce.clone() }, None).unwrap() {
        return json!({
            "success": false,
            "error": "Guild already created!"
        });
    }

    let channel_id = Ulid::new().to_string();
    if let Err(_) = channels.insert_one(
        doc! {
            "_id": channel_id.clone(),
            "type": ChannelType::GUILDCHANNEL as u32,
            "name": "general",
        },
        None,
    ) {
        return json!({
            "success": false,
            "error": "Failed to create guild channel."
        });
    }

    let id = Ulid::new().to_string();
    if col
        .insert_one(
            doc! {
                "_id": id.clone(),
                "nonce": nonce,
                "name": name,
                "description": description,
                "owner": user.id.clone(),
                "channels": [
                    channel_id.clone()
                ],
                "members": [
                    {
                        "id": user.id,
                    }
                ],
                "invites": [],
            },
            None,
        )
        .is_ok()
    {
        json!({
            "success": true,
            "id": id,
        })
    } else {
        channels
            .delete_one(doc! { "_id": channel_id }, None)
            .expect("Failed to delete the channel we just made.");

        json!({
            "success": false,
            "error": "Failed to create guild."
        })
    }
}
