use indexmap::{IndexMap, IndexSet};
use iso8601_timestamp::Timestamp;

use super::{Embed, File, MessageWebhook};

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
    },
    "PartialMessage"
);

auto_derived!(
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
    pub struct Masquerade {
        /// Replace the display name shown on this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// Replace the avatar shown on this message (URL to image file)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar: Option<String>,
        /// Replace the display role colour shown on this message
        ///
        /// Must have `ManageRole` permission to use
        #[serde(skip_serializing_if = "Option::is_none")]
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
    pub enum MessageSort {
        /// Sort by the most relevant messages
        #[default]
        Relevance,
        /// Sort by the newest messages first
        Latest,
        /// Sort by the oldest messages first
        Oldest,
    }
);

impl Interactions {
    /// Check if default initialisation of fields
    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}
