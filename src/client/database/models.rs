use crate::client::database::schema::*;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Insertable)]
#[diesel(table_name = servers)]
#[diesel(primary_key(server_id))]
pub struct Server {
    #[diesel(column_name = server_id)]
    pub id: i64,
}

#[derive(Queryable, Identifiable, Insertable)]
#[diesel(table_name = users)]
#[diesel(primary_key(user_id))]
pub struct User {
    #[diesel(column_name = user_id)]
    pub id: i64,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Server))]
#[diesel(table_name = members)]
#[diesel(primary_key(member_id))]
pub struct Member {
    #[diesel(column_name = member_id)]
    pub id: i32,
    pub user_id: i64,
    pub server_id: i64,
}

#[derive(Insertable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Server))]
#[diesel(table_name = members)]
pub struct NewMember {
    pub user_id: i64,
    pub server_id: i64,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Member))]
#[diesel(table_name = tracked_members)]
#[diesel(primary_key(member_id))]
pub struct TrackedMember {
    #[diesel(column_name = member_id)]
    pub id: i32,
}

#[derive(Insertable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Server))]
#[diesel(table_name = members)]
pub struct NewTrackedMember {
    pub user_id: i64,
    pub server_id: i64,
}

#[derive(Queryable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Server))]
#[diesel(primary_key(server_id))]
#[diesel(table_name = channels)]
pub struct Channel {
    #[diesel(column_name = channel_id)]
    pub id: i64,
    pub server_id: i64,
}
