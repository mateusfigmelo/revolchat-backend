use revolt_quark::{
    models::{Invite, User},
    perms, Db, Permission, Ref, Result,
};

use rocket::serde::json::Json;

#[post("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Invite>> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::InviteOthers)
        .await?;

    Invite::create(db, &user, &channel).await.map(Json)
}
