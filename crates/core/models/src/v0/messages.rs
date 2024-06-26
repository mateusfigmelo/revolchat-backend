use std::time::SystemTime;

use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_config::config;

#[cfg(feature = "validator")]
use validator::Validate;

#[cfg(feature = "rocket")]
use rocket::{FromForm, FromFormField};

use iso8601_timestamp::Timestamp;

use super::{Embed, File, Member, MessageWebhook, User, Webhook, RE_COLOUR};

pub static RE_MENTION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap());

auto_derived_partial!(
    /// Message
    pub struct Message {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Unique value generated by client sending this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nonce: Option<String>,
        /// Id of the channel this message was sent in
        pub channel: String,
        /// Id of the user or webhook that sent this message
        pub author: String,
        /// The user that sent this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user: Option<User>,
        /// The member that sent this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub member: Option<Member>,
        /// The webhook that sent this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub webhook: Option<MessageWebhook>,
        /// Message content
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        /// System message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system: Option<SystemMessage>,
        /// Array of attachments
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<File>>,
        /// Time at which this message was last edited
        #[serde(skip_serializing_if = "Option::is_none")]
        pub edited: Option<Timestamp>,
        /// Attached embeds to this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
        /// Array of user ids mentioned in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mentions: Option<Vec<String>>,
        /// Array of message ids this message is replying to
        #[serde(skip_serializing_if = "Option::is_none")]
        pub replies: Option<Vec<String>>,
        /// Hashmap of emoji IDs to array of user IDs
        #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
        pub reactions: IndexMap<String, IndexSet<String>>,
        /// Information about how this message should be interacted with
        #[serde(skip_serializing_if = "Interactions::is_default", default)]
        pub interactions: Interactions,
        /// Name and / or avatar overrides for this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub masquerade: Option<Masquerade>,

        /// Bitfield of message flags
        ///
        /// https://docs.rs/revolt-models/latest/revolt_models/v0/enum.MessageFlags.html
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_zero_u32", default)
        )]
        pub flags: u32,
    },
    "PartialMessage"
);

auto_derived!(
    /// Bulk Message Response
    #[serde(untagged)]
    pub enum BulkMessageResponse {
        JustMessages(
            /// List of messages
            Vec<Message>,
        ),
        MessagesAndUsers {
            /// List of messages
            messages: Vec<Message>,
            /// List of users
            users: Vec<User>,
            /// List of members
            #[serde(skip_serializing_if = "Option::is_none")]
            members: Option<Vec<Member>>,
        },
    }

    /// System Event
    #[serde(tag = "type")]
    pub enum SystemMessage {
        #[serde(rename = "text")]
        Text { content: String },
        #[serde(rename = "user_added")]
        UserAdded { id: String, by: String },
        #[serde(rename = "user_remove")]
        UserRemove { id: String, by: String },
        #[serde(rename = "user_joined")]
        UserJoined { id: String },
        #[serde(rename = "user_left")]
        UserLeft { id: String },
        #[serde(rename = "user_kicked")]
        UserKicked { id: String },
        #[serde(rename = "user_banned")]
        UserBanned { id: String },
        #[serde(rename = "channel_renamed")]
        ChannelRenamed { name: String, by: String },
        #[serde(rename = "channel_description_changed")]
        ChannelDescriptionChanged { by: String },
        #[serde(rename = "channel_icon_changed")]
        ChannelIconChanged { by: String },
        #[serde(rename = "channel_ownership_changed")]
        ChannelOwnershipChanged { from: String, to: String },
    }

    /// Name and / or avatar override information
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct Masquerade {
        /// Replace the display name shown on this message
        #[serde(skip_serializing_if = "Option::is_none")]
        #[validate(length(min = 1, max = 32))]
        pub name: Option<String>,
        /// Replace the avatar shown on this message (URL to image file)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[validate(length(min = 1, max = 256))]
        pub avatar: Option<String>,
        /// Replace the display role colour shown on this message
        ///
        /// Must have `ManageRole` permission to use
        #[serde(skip_serializing_if = "Option::is_none")]
        #[validate(length(min = 1, max = 128), regex = "RE_COLOUR")]
        pub colour: Option<String>,
    }

    /// Information to guide interactions on this message
    #[derive(Default)]
    pub struct Interactions {
        /// Reactions which should always appear and be distinct
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub reactions: Option<IndexSet<String>>,
        /// Whether reactions should be restricted to the given list
        ///
        /// Can only be set to true if reactions list is of at least length 1
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub restrict_reactions: bool,
    }

    /// Appended Information
    pub struct AppendMessage {
        /// Additional embeds to include in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
    }

    /// Message Sort
    ///
    /// Sort used for retrieving messages
    #[derive(Default)]
    #[cfg_attr(feature = "rocket", derive(FromFormField))]
    pub enum MessageSort {
        /// Sort by the most relevant messages
        #[default]
        Relevance,
        /// Sort by the newest messages first
        Latest,
        /// Sort by the oldest messages first
        Oldest,
    }

    /// Push Notification
    pub struct PushNotification {
        /// Known author name
        pub author: String,
        /// URL to author avatar
        pub icon: String,
        /// URL to first matching attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image: Option<String>,
        /// Message content or system message information
        pub body: String,
        /// Unique tag, usually the channel ID
        pub tag: String,
        /// Timestamp at which this notification was created
        pub timestamp: u64,
        /// URL to open when clicking notification
        pub url: String,
    }

    /// Representation of a text embed before it is sent.
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct SendableEmbed {
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
        pub icon_url: Option<String>,
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 256)))]
        pub url: Option<String>,
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 100)))]
        pub title: Option<String>,
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 2000)))]
        pub description: Option<String>,
        pub media: Option<String>,
        #[cfg_attr(
            feature = "validator",
            validate(length(min = 1, max = 128), regex = "RE_COLOUR")
        )]
        pub colour: Option<String>,
    }

    /// What this message should reply to and how
    pub struct ReplyIntent {
        /// Message Id
        pub id: String,
        /// Whether this reply should mention the message's author
        pub mention: bool,
    }

    /// Message to send
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataMessageSend {
        /// Unique token to prevent duplicate message sending
        ///
        /// **This is deprecated and replaced by `Idempotency-Key`!**
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 64)))]
        pub nonce: Option<String>,

        /// Message content to send
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 2000)))]
        pub content: Option<String>,
        /// Attachments to include in message
        pub attachments: Option<Vec<String>>,
        /// Messages to reply to
        pub replies: Option<Vec<ReplyIntent>>,
        /// Embeds to include in message
        ///
        /// Text embed content contributes to the content length cap
        #[cfg_attr(feature = "validator", validate)]
        pub embeds: Option<Vec<SendableEmbed>>,
        /// Masquerade to apply to this message
        #[cfg_attr(feature = "validator", validate)]
        pub masquerade: Option<Masquerade>,
        /// Information about how this message should be interacted with
        pub interactions: Option<Interactions>,

        /// Bitfield of message flags
        ///
        /// https://docs.rs/revolt-models/latest/revolt_models/v0/enum.MessageFlags.html
        pub flags: Option<u32>,
    }

    /// Options for querying messages
    #[cfg_attr(feature = "validator", derive(Validate))]
    #[cfg_attr(feature = "rocket", derive(FromForm))]
    pub struct OptionsQueryMessages {
        /// Maximum number of messages to fetch
        ///
        /// For fetching nearby messages, this is \`(limit + 1)\`.
        #[cfg_attr(feature = "validator", validate(range(min = 1, max = 100)))]
        pub limit: Option<i64>,
        /// Message id before which messages should be fetched
        #[cfg_attr(feature = "validator", validate(length(min = 26, max = 26)))]
        pub before: Option<String>,
        /// Message id after which messages should be fetched
        #[cfg_attr(feature = "validator", validate(length(min = 26, max = 26)))]
        pub after: Option<String>,
        /// Message sort direction
        pub sort: Option<MessageSort>,
        /// Message id to search around
        ///
        /// Specifying 'nearby' ignores 'before', 'after' and 'sort'.
        /// It will also take half of limit rounded as the limits to each side.
        /// It also fetches the message ID specified.
        #[cfg_attr(feature = "validator", validate(length(min = 26, max = 26)))]
        pub nearby: Option<String>,
        /// Whether to include user (and member, if server channel) objects
        pub include_users: Option<bool>,
    }

    /// Options for searching for messages
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataMessageSearch {
        /// Full-text search query
        ///
        /// See [MongoDB documentation](https://docs.mongodb.com/manual/text-search/#-text-operator) for more information.
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 64)))]
        pub query: String,

        /// Maximum number of messages to fetch
        #[cfg_attr(feature = "validator", validate(range(min = 1, max = 100)))]
        pub limit: Option<i64>,
        /// Message id before which messages should be fetched
        #[cfg_attr(feature = "validator", validate(length(min = 26, max = 26)))]
        pub before: Option<String>,
        /// Message id after which messages should be fetched
        #[cfg_attr(feature = "validator", validate(length(min = 26, max = 26)))]
        pub after: Option<String>,
        /// Message sort direction
        ///
        /// By default, it will be sorted by latest.
        #[cfg_attr(feature = "serde", serde(default = "MessageSort::default"))]
        pub sort: MessageSort,
        /// Whether to include user (and member, if server channel) objects
        pub include_users: Option<bool>,
    }

    /// Changes to make to message
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataEditMessage {
        /// New message content
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 2000)))]
        pub content: Option<String>,
        /// Embeds to include in the message
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 10)))]
        pub embeds: Option<Vec<SendableEmbed>>,
    }

    /// Options for bulk deleting messages
    #[cfg_attr(
        feature = "validator",
        cfg_attr(feature = "validator", derive(Validate))
    )]
    pub struct OptionsBulkDelete {
        /// Message IDs
        #[validate(length(min = 1, max = 100))]
        pub ids: Vec<String>,
    }

    /// Options for removing reaction
    #[cfg_attr(feature = "rocket", derive(FromForm))]
    pub struct OptionsUnreact {
        /// Remove a specific user's reaction
        pub user_id: Option<String>,
        /// Remove all reactions
        pub remove_all: Option<bool>,
    }

    /// Message flag bitfield
    #[repr(u32)]
    pub enum MessageFlags {
        /// Message will not send push / desktop notifications
        SupressNotifications = 1,
    }
);

/// Message Author Abstraction
pub enum MessageAuthor<'a> {
    User(&'a User),
    Webhook(&'a Webhook),
    System {
        username: &'a str,
        avatar: Option<&'a str>,
    },
}

impl Interactions {
    /// Check if default initialisation of fields
    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}

impl<'a> MessageAuthor<'a> {
    pub fn id(&self) -> &str {
        match self {
            MessageAuthor::User(user) => &user.id,
            MessageAuthor::Webhook(webhook) => &webhook.id,
            MessageAuthor::System { .. } => "00000000000000000000000000",
        }
    }

    pub fn avatar(&self) -> Option<&str> {
        match self {
            MessageAuthor::User(user) => user.avatar.as_ref().map(|file| file.id.as_str()),
            MessageAuthor::Webhook(webhook) => webhook.avatar.as_ref().map(|file| file.id.as_str()),
            MessageAuthor::System { avatar, .. } => *avatar,
        }
    }

    pub fn username(&self) -> &str {
        match self {
            MessageAuthor::User(user) => &user.username,
            MessageAuthor::Webhook(webhook) => &webhook.name,
            MessageAuthor::System { username, .. } => username,
        }
    }
}

impl From<SystemMessage> for String {
    fn from(s: SystemMessage) -> String {
        match s {
            SystemMessage::Text { content } => content,
            SystemMessage::UserAdded { .. } => "User added to the channel.".to_string(),
            SystemMessage::UserRemove { .. } => "User removed from the channel.".to_string(),
            SystemMessage::UserJoined { .. } => "User joined the channel.".to_string(),
            SystemMessage::UserLeft { .. } => "User left the channel.".to_string(),
            SystemMessage::UserKicked { .. } => "User kicked from the channel.".to_string(),
            SystemMessage::UserBanned { .. } => "User banned from the channel.".to_string(),
            SystemMessage::ChannelRenamed { .. } => "Channel renamed.".to_string(),
            SystemMessage::ChannelDescriptionChanged { .. } => {
                "Channel description changed.".to_string()
            }
            SystemMessage::ChannelIconChanged { .. } => "Channel icon changed.".to_string(),
            SystemMessage::ChannelOwnershipChanged { .. } => {
                "Channel ownership changed.".to_string()
            }
        }
    }
}

impl PushNotification {
    /// Create a new notification from a given message, author and channel ID
    pub async fn from(msg: Message, author: Option<MessageAuthor<'_>>, channel_id: &str) -> Self {
        let config = config().await;

        let icon = if let Some(author) = &author {
            if let Some(avatar) = author.avatar() {
                format!("{}/avatars/{}", config.hosts.autumn, avatar)
            } else {
                format!("{}/users/{}/default_avatar", config.hosts.api, author.id())
            }
        } else {
            format!("{}/assets/logo.png", config.hosts.app)
        };

        let image = msg.attachments.and_then(|attachments| {
            attachments
                .first()
                .map(|v| format!("{}/attachments/{}", config.hosts.autumn, v.id))
        });

        let body = if let Some(sys) = msg.system {
            sys.into()
        } else if let Some(text) = msg.content {
            text
        } else {
            "Empty Message".to_string()
        };

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            author: author
                .map(|x| x.username().to_string())
                .unwrap_or_else(|| "Revolt".to_string()),
            icon,
            image,
            body,
            tag: channel_id.to_string(),
            timestamp,
            url: format!("{}/channel/{}/{}", config.hosts.app, channel_id, msg.id),
        }
    }
}
