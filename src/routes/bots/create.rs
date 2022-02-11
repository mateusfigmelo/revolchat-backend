use crate::util::{regex::RE_USERNAME, variables::MAX_BOT_COUNT};

use nanoid::nanoid;
use revolt_quark::{
    models::{Bot, User, user::BotInformation},
    Db, Error, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    name: String,
}

#[post("/create", data = "<info>")]
pub async fn create_bot(db: &Db, user: User, info: Json<Data>) -> Result<Json<Bot>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if db.get_number_of_bots_by_user(&user.id).await? >= *MAX_BOT_COUNT {
        return Err(Error::ReachedMaximumBots);
    }

    if db.is_username_taken(&info.name).await? {
        return Err(Error::UsernameTaken);
    }

    let id = Ulid::new().to_string();
    let bot_user = User {
        id: id.clone(),
        username: info.name,
        bot: Some(BotInformation {
            owner: user.id.clone()
        }),
        ..Default::default()
    };

    let bot = Bot {
        id,
        owner: user.id,
        token: nanoid!(64),
        ..Default::default()
    };

    db.insert_user(&bot_user).await?;
    db.insert_bot(&bot).await?;
    Ok(Json(bot))
}
