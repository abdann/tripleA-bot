use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub server_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Users {
    pub user_id: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub member_id: u32,
    pub user_id: u64,
    pub server_id: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub channel_id: u64,
    pub server_id: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedChannel {
    pub channel_id: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedMember {
    pub member_id: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    pub word_id: u32,
    pub word: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberWord {
    pub word_id: u32,
    pub member_id: u32,
    pub next_word_id: u32,
    pub frequency: u64,
}
