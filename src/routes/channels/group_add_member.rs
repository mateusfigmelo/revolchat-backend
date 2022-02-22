use revolt_quark::{
    models::{Channel, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

#[put("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc(db)
        .await
        .can_invite_others()
    {
        return Error::from_permission(Permission::InviteOthers);
    }

    match &channel {
        Channel::Group { .. } => {
            let member = member.as_user(db).await?;
            channel
                .add_user_to_group(db, &member.id, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
