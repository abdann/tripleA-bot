use crate::client::database::errors;
use crate::client::textgen::textprocessing;
use itertools::Itertools;
use serenity::prelude::{Mutex, TypeMapKey};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;

/** The DDL for the database schema */
const DB_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS servers (
	server_id BIGINT NOT NULL,
	PRIMARY KEY (server_id)
);
CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT NOT NULL,
    PRIMARY KEY (user_id)
);
CREATE TABLE IF NOT EXISTS members (
    member_id SERIAL NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
    PRIMARY KEY (member_id)
);
CREATE TABLE IF NOT EXISTS channels (
    channel_id BIGINT NOT NULL,
    server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id)
);
CREATE TABLE IF NOT EXISTS tracked_channels (
    channel_id BIGINT NOT NULL REFERENCES channels (channel_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS tracked_members (
    member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS words (
    word_id SERIAL NOT NULL,
    word TEXT NOT NULL,
    PRIMARY KEY (word_id),
    UNIQUE (word)    
);
CREATE TABLE IF NOT EXISTS member_words (
    word_id INTEGER NOT NULL REFERENCES words (word_id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE,
    next_word_id INTEGER NOT NULL REFERENCES words (word_id) ON DELETE CASCADE,
    frequency BIGINT NOT NULL
);";

/** SQL commands to delete everything. */
const SQL_REINIT_DB_SCHEMA: &str = "DELETE FROM servers;
DELETE FROM users;
DELETE FROM members;
DELETE FROM channels;
DELETE FROM tracked_channels;
DELETE FROM tracked_members;
DELETE FROM words;
DELETE FROM member_words;";

/** A wrapper and interface struct for a Postgres pool. This is intended to be made and reside in the data attribute of serenity::prelude::Context such that the pool can be accessed from the Context. */
pub struct DbInterface {
    db: Pool<Postgres>,
}

impl TypeMapKey for DbInterface {
    type Value = Arc<Mutex<DbInterface>>;
}

/** Represents the possible states for the database when emitting DDL */
#[derive(Debug)]
pub enum DbSchemaInitialState {
    AlreadyPresent,
    NotPresent,
}

impl DbInterface {
    /** Creates the pool wrapper. */
    pub async fn new(database_url: &str) -> DbInterface {
        DbInterface {
            db: PgPoolOptions::new()
                .max_connections(10)
                .connect(&database_url)
                .await
                .expect("Should be able to create pool for DbInterface"),
        }
    }

    /** Returns member_id if the user specified by the server_id and user_id is a tracked member*/
    pub async fn fetch_tracked_member(
        &self,
        server_id: u64,
        user_id: u64,
    ) -> Result<Option<u32>, sqlx::Error> {
        match sqlx::query!(
            "SELECT tracked_members.member_id FROM tracked_members INNER JOIN members ON tracked_members.member_id = members.member_id WHERE user_id = $1 AND server_id = $2;",
            user_id as i64,
            server_id as i64
        )
        .fetch_optional(&self.db.clone())
        .await {
            Ok(record_opt) => {
                if let Some(record) = record_opt {
                    Ok(Some(record.member_id as u32))
                } else {
                    Ok(None)
                }
            },
            Err(why) => {
                Err(why)
            }
        }
    }

    /** Adds a member to the list of tracked members */
    pub async fn add_tracked_member(
        &self,
        server_id: u64,
        user_id: u64,
    ) -> Result<errors::InsertResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO servers(server_id) VALUES ($1) ON CONFLICT DO NOTHING",
            server_id as i64
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "INSERT INTO users(user_id) VALUES ($1) ON CONFLICT DO NOTHING;",
            user_id as i64
        )
        .execute(&self.db.clone())
        .await?;
        if let Some(member_id) = self.fetch_member(server_id, user_id).await? {
            if let Some(_) = self.fetch_tracked_member(server_id, user_id).await? {
                Ok(errors::InsertResult::AlreadyPresent)
            } else {
                sqlx::query!(
                    "INSERT INTO tracked_members(member_id) VALUES ($1)",
                    member_id as i32
                )
                .execute(&self.db.clone())
                .await?;
                Ok(errors::InsertResult::Added)
            }
        } else {
            let new_row = sqlx::query!(
                "INSERT INTO members(user_id, server_id) VALUES ($1, $2) RETURNING *",
                user_id as i64,
                server_id as i64
            )
            .fetch_one(&self.db.clone())
            .await?;
            let member_id = new_row.member_id;
            sqlx::query!(
                "INSERT INTO tracked_members(member_id) VALUES ($1)",
                member_id
            )
            .execute(&self.db.clone())
            .await?;
            Ok(errors::InsertResult::Added)
        }
    }

    /** Removes a tracked member. */
    pub async fn remove_tracked_member(
        &self,
        server_id: u64,
        user_id: u64,
    ) -> Result<errors::RemoveResult, sqlx::Error> {
        //First check if the specified member is a tracked member
        if let Some(tracked_member_id) = self.fetch_tracked_member(server_id, user_id).await? {
            sqlx::query!(
                "DELETE FROM tracked_members WHERE member_id = $1",
                tracked_member_id as i32
            )
            .execute(&self.db.clone())
            .await?;
            Ok(errors::RemoveResult::Removed)
        } else {
            Ok(errors::RemoveResult::NotPresent)
        }
    }

    /** Returns a vector of member_id for tracked members for a server*/
    pub async fn fetch_tracked_members(
        &self,
        server_id: u64,
    ) -> Result<Option<Vec<u32>>, sqlx::Error> {
        match sqlx::query!(
            "SELECT tracked_members.member_id FROM tracked_members INNER JOIN members ON tracked_members.member_id = members.member_id WHERE server_id = $1;",
            server_id as i64
        )
        .fetch_all(&self.db.clone())
        .await {
            Ok(record_opt_vec) => {
                if record_opt_vec.len() == 0 {
                    Ok(None)
                } else {
                    Ok(Some(record_opt_vec.into_iter().map(|record|record.member_id as u32).collect()))
                }
            },
            Err(why) => {
                Err(why)
            }
        }
    }

    /** Fetches discord user_ids for tracked members in a server */
    pub async fn fetch_tracked_user_members(
        &self,
        server_id: u64,
    ) -> Result<Option<Vec<u64>>, sqlx::Error> {
        match sqlx::query!(
            "SELECT members.user_id FROM members INNER JOIN tracked_members ON tracked_members.member_id = members.member_id WHERE server_id = $1;",
            server_id as i64
        ).fetch_all(&self.db).await {
            Ok(rows) => {
                if rows.len() == 0 {
                    Ok(None)
                } else {
                    Ok(Some(rows.into_iter().map(|record|record.user_id as u64).collect()))
                }
            }
            Err(why) => {
                Err(why)
            }
        }
    }

    /** Returns member_id if the user specified by the server_id and user_id is a member*/
    pub async fn fetch_member(
        &self,
        server_id: u64,
        user_id: u64,
    ) -> Result<Option<u32>, sqlx::Error> {
        match sqlx::query!(
            "SELECT members.member_id FROM members WHERE user_id = $1 AND server_id = $2;",
            user_id as i64,
            server_id as i64
        )
        .fetch_optional(&self.db.clone())
        .await
        {
            Ok(record_opt) => {
                if let Some(record) = record_opt {
                    Ok(Some(record.member_id as u32))
                } else {
                    Ok(None)
                }
            }
            Err(why) => Err(why),
        }
    }

    /** Returns a vector of member_id for stored members for a server*/
    pub async fn fetch_members(&self, server_id: u64) -> Result<Option<Vec<u32>>, sqlx::Error> {
        match sqlx::query!(
            "SELECT members.member_id FROM members WHERE server_id = $1;",
            server_id as i64
        )
        .fetch_all(&self.db.clone())
        .await
        {
            Ok(record_opt_vec) => {
                if record_opt_vec.len() == 0 {
                    Ok(None)
                } else {
                    Ok(Some(
                        record_opt_vec
                            .into_iter()
                            .map(|record| record.member_id as u32)
                            .collect(),
                    ))
                }
            }
            Err(why) => Err(why),
        }
    }

    /** Checks if a channel is a tracked channel. */
    pub async fn is_tracked_channel(
        &self,
        server_id: u64,
        channel_id: u64,
    ) -> Result<bool, sqlx::Error> {
        if let Some(_) = sqlx::query!(
            "SELECT tracked_channels.channel_id FROM tracked_channels INNER JOIN channels ON tracked_channels.channel_id = channels.channel_id WHERE tracked_channels.channel_id = $1 AND server_id = $2;",
        channel_id as i64,
        server_id as i64
        ).fetch_optional(&self.db.clone()).await? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /** Returns a vector of tracked channels for a server */
    pub async fn fetch_tracked_channels(
        &self,
        server_id: u64,
    ) -> Result<Option<Vec<u64>>, sqlx::Error> {
        let records = sqlx::query!(
            "SELECT tracked_channels.channel_id FROM tracked_channels INNER JOIN channels ON tracked_channels.channel_id = channels.channel_id WHERE server_id = $1;",
            server_id as i64
        )
        .fetch_all(&self.db.clone())
        .await?;
        if records.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(
                records
                    .into_iter()
                    .map(|record| record.channel_id as u64)
                    .collect(),
            ))
        }
    }

    /** Adds a channel to a list of tracked channels */
    pub async fn add_tracked_channel(
        &self,
        server_id: u64,
        channel_id: u64,
    ) -> Result<errors::InsertResult, sqlx::Error> {
        if self.is_tracked_channel(server_id, channel_id).await? {
            Ok(errors::InsertResult::AlreadyPresent)
        } else {
            // Insert into lower relations prior to attempting to insert on tracked_channels
            sqlx::query!(
                "INSERT INTO servers(server_id) VALUES ($1) ON CONFLICT DO NOTHING",
                server_id as i64
            )
            .execute(&self.db.clone())
            .await?;
            sqlx::query!(
                "INSERT INTO channels(channel_id, server_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                channel_id as i64,
                server_id as i64
            ).execute(&self.db.clone()).await?;
            //Now insert into tracked_channels
            sqlx::query!(
                "INSERT INTO tracked_channels(channel_id) VALUES ($1)",
                channel_id as i64
            )
            .execute(&self.db.clone())
            .await?;
            Ok(errors::InsertResult::Added)
        }
    }

    /** Removes a channel from a list of tracked channels */
    pub async fn remove_tracked_channel(
        &self,
        server_id: u64,
        channel_id: u64,
    ) -> Result<errors::RemoveResult, sqlx::Error> {
        //First check if the specified member is a tracked member
        if self.is_tracked_channel(server_id, channel_id).await? {
            sqlx::query!(
                "DELETE FROM tracked_channels WHERE channel_id = $1",
                channel_id as i64
            )
            .execute(&self.db.clone())
            .await?;
            Ok(errors::RemoveResult::Removed)
        } else {
            Ok(errors::RemoveResult::NotPresent)
        }
    }

    // Fetches weighted random words for the member
    pub async fn fetch_random_member_words_into_sentence(
        &self,
        member_id: u32,
        desired_sentence_length: u8,
    ) -> Result<String, errors::MarkovFetchResultError> {
        // Initialize vector
        let mut word_ids: Vec<i32> = vec![];
        // Get sum of frequencies, so we can normalize and use rand function
        let freq_sum: sqlx::types::BigDecimal;
        if let Some(freq_sum_record) = sqlx::query!(
            "SELECT sum(frequency) FROM member_words WHERE member_id = $1",
            member_id as i64
        )
        .fetch_optional(&self.db.clone())
        .await?
        {
            freq_sum = freq_sum_record
                .sum
                .expect("Should be able to get value of sum");
        } else {
            return Err(errors::MarkovFetchResultError::NotEnoughWords(
                desired_sentence_length,
                0,
            ));
        }
        //Not optimal conversion, but it will have to do :<
        let freq_sum = freq_sum
            .to_string()
            .parse::<f64>()
            .expect("Should be able to convert BigDecimal to f64");
        // Choose random word id of a member. TODO: Make sure that this doesn't result in faulty word generation
        if let Some(word_id) = sqlx::query!(
            "SELECT word_id, next_word_id FROM member_words WHERE member_id = $1 ORDER BY random() LIMIT 1;",
            member_id as i32
        )
        .fetch_optional(&self.db.clone())
        .await?
        {
            word_ids.push(word_id.word_id as i32);
            word_ids.push(word_id.next_word_id as i32);
        } else {
            return Err(errors::MarkovFetchResultError::NotEnoughWords(
                desired_sentence_length,
                0,
            ));
        }
        for i in 0..desired_sentence_length - 2 {
            let current_word_id = &word_ids[i as usize];
            let next_word = sqlx::query!(
                "SELECT next_word_id FROM member_words WHERE member_id = $1 AND frequency > random()*$2 AND word_id = $3 ORDER BY random() LIMIT 1;",
                member_id as i32,
                freq_sum,
                *current_word_id as i32
            ).fetch_optional(&self.db).await?;
            if let Some(word) = next_word {
                word_ids.push(word.next_word_id as i32)
            } else {
                return Err(errors::MarkovFetchResultError::NotEnoughWords(
                    desired_sentence_length,
                    u32::try_from(word_ids.len())
                        .expect("Should be able to convert vector length to u32 size"),
                ));
            }
        }

        // Now get actual words from word_ids:
        match sqlx::query!(
            "SELECT word FROM words WHERE word_id = ANY($1)",
            &word_ids[..]
        )
        .fetch_all(&self.db)
        .await
        {
            Ok(word_vec) => Ok(word_vec.into_iter().map(|record| record.word).join(" ")),
            Err(why) => Err(errors::MarkovFetchResultError::SqlxError(why)),
        }
    }

    /** Adds a vector of words to known words. Expects words to be pre-processed. */
    pub async fn add_words(&self, words: Vec<String>) -> Result<Vec<i32>, sqlx::Error> {
        let results = sqlx::query!(
            "INSERT INTO words(word) SELECT * FROM UNNEST($1::text[]) ON CONFLICT DO NOTHING RETURNING word_id",
            &words[..]
        )
        .fetch_all(&self.db.clone())
        .await?;
        if results.len() != words.len() {
            // If we don't insert all the words, our query is empty; therefore we need to select the data and map it to the words to get the right order
            let found_words = sqlx::query!(
                "SELECT word_id, word FROM words WHERE word = ANY($1)",
                &words[..]
            )
            .fetch_all(&self.db)
            .await?;
            // Now make it into a dictionary for easy access to the word_id values:
            let mut found_words = HashMap::from_iter(
                found_words
                    .into_iter()
                    .map(|record| (record.word, record.word_id)),
            );
            let mut returnable = vec![];
            for word in words {
                returnable.push(found_words.get(&word).unwrap_or(&0)); // 0 for failure
            }
        }
        Ok(results.into_iter().map(|f| f.word_id).collect_vec())
    }

    // Adds member words
    pub async fn add_member_words(
        &self,
        member_id: u32,
        member_words: Vec<String>,
    ) -> Result<(), errors::AddMemberWordsError> {
        let member_word_ids = self.add_words(member_words).await?;
        println!("member_word_ids: {:?}", member_word_ids);
        let word_pairs = textprocessing::vec_word_split(member_word_ids);
        println!("word_pairs: {:?}", word_pairs);
        for insertable in word_pairs {
            sqlx::query!("INSERT INTO member_words (word_id, member_id, next_word_id, frequency) VALUES ($1, $2, $3, 1)
             ON CONFLICT (word_id, member_id, next_word_id) DO UPDATE SET frequency = member_words.frequency + 1;",
            &insertable[0],
            member_id as i32,
            &insertable[1]).execute(&self.db).await?;
            println!("word_id: {:?}", &insertable[0]);
            println!("member_id: {:?}", member_id);
            println!("next_word_id: {:?}", &insertable[1]);
        }
        Ok(())
    }

    /** deletes all data from all tables. For testing only, Do not use on production. */
    pub async fn reinit_all_data(&self) -> Result<bool, sqlx::Error> {
        //Don't want to have to split this up into multiple queries, but I guess I have to...
        sqlx::query!("DROP SCHEMA public CASCADE;")
            .execute(&self.db.clone())
            .await?;
        sqlx::query!("CREATE SCHEMA public;")
            .execute(&self.db.clone())
            .await?;
        sqlx::query!("GRANT ALL ON SCHEMA public TO postgres;")
            .execute(&self.db.clone())
            .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS servers (
            server_id BIGINT NOT NULL,
            PRIMARY KEY (server_id)
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS users (
            user_id BIGINT NOT NULL,
            PRIMARY KEY (user_id)
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS members (
            member_id SERIAL NOT NULL,
            user_id BIGINT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
            server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
            PRIMARY KEY (member_id)
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS channels (
            channel_id BIGINT NOT NULL,
            server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
            PRIMARY KEY (channel_id)
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS tracked_channels (
            channel_id BIGINT NOT NULL REFERENCES channels (channel_id) ON DELETE CASCADE
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS tracked_members (
            member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS words (
            word_id SERIAL NOT NULL,
            word TEXT NOT NULL,
            PRIMARY KEY (word_id),
            UNIQUE (word)    
        );"
        )
        .execute(&self.db.clone())
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS member_words (
            word_id INTEGER NOT NULL REFERENCES words (word_id) ON DELETE CASCADE,
            member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE,
            next_word_id INTEGER NOT NULL REFERENCES words (word_id) ON DELETE CASCADE,
            frequency BIGINT NOT NULL,
            PRIMARY KEY (word_id, member_id, next_word_id)
        );"
        )
        .execute(&self.db.clone())
        .await?;

        Ok(true)
    }
}
