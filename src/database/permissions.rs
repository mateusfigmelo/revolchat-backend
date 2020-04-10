use super::mutual::has_mutual_connection;
use crate::database::user::UserRelationship;
use crate::guards::auth::UserRef;
use crate::guards::channel::ChannelRef;
use crate::guards::guild::GuildRef;

use bson::doc;
use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Relationship {
    Friend = 0,
    Outgoing = 1,
    Incoming = 2,
    Blocked = 3,
    BlockedOther = 4,
    NONE = 5,
    SELF = 6,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum Permission {
    Access = 1,
    CreateInvite = 2,
    KickMembers = 4,
    BanMembers = 8,
    ReadMessages = 16,
    SendMessages = 32,
    ManageMessages = 64,
    ManageChannels = 128,
    ManageServer = 256,
    ManageRoles = 512,
}

bitfield! {
    pub struct MemberPermissions(MSB0 [u32]);
    u8;
    pub get_access, set_access: 31;
    pub get_create_invite, set_create_invite: 30;
    pub get_kick_members, set_kick_members: 29;
    pub get_ban_members, set_ban_members: 28;
    pub get_read_messages, set_read_messages: 27;
    pub get_send_messages, set_send_messages: 26;
    pub get_manage_messages, set_manage_messages: 25;
    pub get_manage_channels, set_manage_channels: 24;
    pub get_manage_server, set_manage_server: 23;
    pub get_manage_roles, set_manage_roles: 22;
}

pub fn get_relationship_internal(
    user_id: &str,
    target_id: &str,
    relationships: &Option<Vec<UserRelationship>>,
) -> Relationship {
    if user_id == target_id {
        return Relationship::SELF;
    }

    if let Some(arr) = &relationships {
        for entry in arr {
            if entry.id == target_id {
                match entry.status {
                    0 => return Relationship::Friend,
                    1 => return Relationship::Outgoing,
                    2 => return Relationship::Incoming,
                    3 => return Relationship::Blocked,
                    4 => return Relationship::BlockedOther,
                    _ => return Relationship::NONE,
                }
            }
        }
    }

    Relationship::NONE
}

pub fn get_relationship(a: &UserRef, b: &UserRef) -> Relationship {
    if a.id == b.id {
        return Relationship::SELF;
    }

    get_relationship_internal(&a.id, &b.id, &a.fetch_relationships())
}

pub struct PermissionCalculator {
    pub user: UserRef,
    pub channel: Option<ChannelRef>,
    pub guild: Option<GuildRef>,
}

impl PermissionCalculator {
    pub fn new(user: UserRef) -> PermissionCalculator {
        PermissionCalculator {
            user,
            channel: None,
            guild: None,
        }
    }

    pub fn channel(self, channel: ChannelRef) -> PermissionCalculator {
        PermissionCalculator {
            channel: Some(channel),
            ..self
        }
    }

    pub fn guild(self, guild: GuildRef) -> PermissionCalculator {
        PermissionCalculator {
            guild: Some(guild),
            ..self
        }
    }

    pub fn calculate(self) -> u32 {
        let guild = if let Some(value) = self.guild {
            Some(value)
        } else if let Some(channel) = &self.channel {
            match channel.channel_type {
                0..=1 => None,
                2 => {
                    if let Some(id) = &channel.guild {
                        GuildRef::from(id.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        let mut permissions: u32 = 0;
        if let Some(guild) = guild {
            if let Some(_data) = guild.fetch_data_given(
                doc! {
                    "members": {
                        "$elemMatch": {
                            "id": &self.user.id,
                        }
                    }
                },
                doc! {},
            ) {
                if guild.owner == self.user.id {
                    return u32::MAX;
                }

                permissions = guild.default_permissions as u32;
            }
        }

        if let Some(channel) = &self.channel {
            match channel.channel_type {
                0 => {
                    if let Some(arr) = &channel.recipients {
                        let mut other_user = "";
                        for item in arr {
                            if item == &self.user.id {
                                permissions = 177;
                            } else {
                                other_user = item;
                            }
                        }

                        let relationships = self.user.fetch_relationships();
                        let relationship =
                            get_relationship_internal(&self.user.id, &other_user, &relationships);

                        if relationship == Relationship::Blocked
                            || relationship == Relationship::BlockedOther
                        {
                            permissions = 1;
                        } else if has_mutual_connection(&self.user.id, other_user) {
                            permissions = 177;
                        }
                    }
                }
                1 => {
                    if let Some(id) = &channel.owner {
                        if &self.user.id == id {
                            return u32::MAX;
                        }
                    }

                    if let Some(arr) = &channel.recipients {
                        for item in arr {
                            if item == &self.user.id {
                                permissions = 177;
                                break;
                            }
                        }
                    }
                }
                2 => {
                    // nothing implemented yet
                }
                _ => {}
            }
        }

        permissions
    }

    pub fn as_permission(self) -> MemberPermissions<[u32; 1]> {
        MemberPermissions([self.calculate()])
    }
}
