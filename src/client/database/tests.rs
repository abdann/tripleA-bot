#[cfg(test)]
mod tests {

    use tokio::time::sleep;

    use crate::client::database::interface::*;

    #[tokio::test]
    async fn test_fetch_tracked_user_members() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        db_int.add_tracked_member(123, 456).await;
        db_int.add_tracked_member(123, 789).await;
        db_int.add_tracked_member(456, 123).await;
        let result1 = {
            match db_int.fetch_tracked_user_members(123).await {
                Ok(results) => {
                    if let Some(list_of_members) = results {
                        list_of_members
                    } else {
                        vec![]
                    }
                }
                Err(_why) => {
                    vec![]
                }
            }
        };
        assert_eq!(result1, vec![456, 789]);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }
    #[tokio::test]
    async fn test_reinit_db() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        db_int
            .add_tracked_member(123, 456)
            .await
            .expect("Add member");

        db_int.reinit_all_data().await.expect("cleared data");
        let remaining_rows = sqlx::query!("SELECT * FROM tracked_members;")
            .fetch_all(&db_int.db.clone())
            .await
            .expect("Got selection")
            .len();

        assert_eq!(0, remaining_rows);
    }
    /** Tests inserting a new tracked member if the member is not already a tracked member and is not already a stored member and is not already a stored user */
    #[tokio::test]
    async fn test_insert_tracked_member_no_user_no_member() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        let first_result = db_int
            .add_tracked_member(1056949566718607391, 611962779762688020)
            .await.expect("Should be able to insert a new tracked member if the database does not have any information about that user.");
        assert_eq!(errors::InsertResult::Added, first_result);

        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_insert_tracked_member_with_user_already_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        //Add the user here
        sqlx::query!(
            "INSERT INTO users(user_id) VALUES ($1)",
            611962779762688020 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to place user prior to test");

        let first_result = db_int
            .add_tracked_member(1056949566718607391, 611962779762688020)
            .await
            .expect("Should be able to insert a new tracked member if the user already exists.");
        assert_eq!(errors::InsertResult::Added, first_result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_insert_tracked_member_with_user_already_present_with_member_already_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;
        sqlx::query!(
            "INSERT INTO users(user_id) VALUES ($1)",
            611962779762688020 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to add a user.");
        sqlx::query!(
            "INSERT INTO servers(server_id) VALUES ($1)",
            1056949566718607391 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to add a server.");
        sqlx::query!(
            "INSERT INTO members(server_id, user_id) VALUES ($1, $2)",
            1056949566718607391 as i64,
            611962779762688020 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Inserted row.");

        let first_result = db_int
            .add_tracked_member(1056949566718607391, 611962779762688020)
            .await
            .expect("Should be able to add tracked member.");
        assert_eq!(errors::InsertResult::Added, first_result);

        //Now rollback changes
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_insert_tracked_member_with_user_already_present_with_member_already_present_with_tracked_member_already_present(
    ) {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        sqlx::query!(
            "INSERT INTO users(user_id) VALUES ($1)",
            611962779762688020 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to add a user.");
        sqlx::query!(
            "INSERT INTO servers(server_id) VALUES ($1)",
            1056949566718607391 as i64
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to add a server.");
        let member_id = sqlx::query!(
            "INSERT INTO members(server_id, user_id) VALUES ($1, $2) RETURNING *;",
            1056949566718607391 as i64,
            611962779762688020 as i64
        )
        .fetch_one(&db_int.db.clone())
        .await
        .expect("Inserted row.")
        .member_id;
        sqlx::query!(
            "INSERT INTO tracked_members(member_id) VALUES ($1)",
            member_id
        )
        .execute(&db_int.db.clone())
        .await
        .expect("Should be able to add tracked member");

        let first_result = db_int
            .add_tracked_member(1056949566718607391, 611962779762688020)
            .await
            .expect("Should be able to add tracked member");
        assert_eq!(errors::InsertResult::AlreadyPresent, first_result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_add_tracked_channel_not_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");

        let result = db_int
            .add_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        assert_eq!(errors::InsertResult::Added, result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_add_tracked_channel_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        //Add channel beforehand
        let first_result = db_int
            .add_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        assert_eq!(errors::InsertResult::Added, first_result);
        let second_result = db_int
            .add_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        assert_eq!(errors::InsertResult::AlreadyPresent, second_result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_remove_tracked_channel_not_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        let result = db_int
            .remove_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        assert_eq!(errors::RemoveResult::NotPresent, result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_remove_tracked_channel_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;
        //Add channel beforehand
        db_int
            .add_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        let result = db_int
            .remove_tracked_channel(1056949566718607391, 1056949567561674808)
            .await
            .expect("Should be able to add tracked channel");
        assert_eq!(errors::RemoveResult::Removed, result);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_add_words_no_words_present() {
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        let sample_words = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];
        // Add words
        db_int.add_words(sample_words).await.expect("adding words.");
        assert!(true);
        db_int
            .reinit_all_data()
            .await
            .expect("Clear data after test");
    }

    #[tokio::test]
    async fn test_add_words_some_words_present() {
        // Tests if you can add words when some words are present. Note: this tests Non-overlapping words only.
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        let sample_words_1 = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ];

        let sample_words_2 = vec![
            "test4".to_string(),
            "test5".to_string(),
            "test6".to_string(),
        ];

        db_int
            .add_words(sample_words_1)
            .await
            .expect("Added first words");

        db_int
            .add_words(sample_words_2)
            .await
            .expect("Added second words");

        db_int.reinit_all_data().await.expect("reinit");
        assert!(true);
    }

    #[tokio::test]
    async fn test_add_words_some_words_present_overlap() {
        // Tests if you can add words when some words are present. Note: This *does* test overlap when trying to insert words that are already present
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;

        let sample_words_1 = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test4".to_string(),
        ];

        let sample_words_2 = vec![
            "test4".to_string(),
            "test5".to_string(),
            "test6".to_string(),
        ];

        db_int
            .add_words(sample_words_1)
            .await
            .expect("Added first words");

        db_int
            .add_words(sample_words_2)
            .await
            .expect("Added second words");

        db_int.reinit_all_data().await.expect("reinit");
        assert!(true);
    }

    #[tokio::test]
    async fn test_add_member_words_no_words_present() {
        use std::time::Duration;
        // Tests adding member words with member words not present already
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;
        db_int.reinit_all_data().await;

        sleep(Duration::from_secs(1)).await; //
        let test_words = vec!["I".to_owned(), "like".to_owned(), "pie".to_owned()];
        db_int.add_tracked_member(123, 456).await;
        let member_id = db_int
            .fetch_member(123, 456)
            .await
            .expect("Should get a result")
            .expect("Should have a member here");
        db_int
            .add_member_words(member_id, test_words)
            .await
            .expect("Should be able to add member words here");
        let expected_row1 = (1, member_id, 2, 1);
        let expected_row2 = (2, member_id, 3, 1);
        let found_row1 = sqlx::query!(
            "SELECT * from member_words where member_id = $1 and word_id = 1",
            member_id as i32
        )
        .fetch_one(&db_int.db)
        .await
        .expect("Shoudl be able to get row");
        let found_row2 = sqlx::query!(
            "SELECT * from member_words where member_id = $1 and word_id = 2",
            member_id as i32
        )
        .fetch_one(&db_int.db)
        .await
        .expect("Shoudl be able to get row");
        //Assertion fails here!
        assert_eq!(
            expected_row1,
            (
                found_row1.word_id,
                found_row1.member_id as u32,
                found_row1.next_word_id,
                found_row1.frequency
            )
        );
        assert_eq!(
            expected_row2,
            (
                found_row2.word_id,
                found_row2.member_id as u32,
                found_row2.next_word_id,
                found_row2.frequency
            )
        );
        //Re-init test db
        db_int.reinit_all_data().await;
    }

    #[tokio::test]
    async fn test_add_member_words_some_words_present() {
        use std::time::Duration;
        // Tests adding member words with some member words present already
        dotenv::dotenv().expect("Should be able to get .env file");
        let db_url: String =
            std::env::var("DATABASE_URL").expect("Should be able to get database url");
        let db_int = DbInterface::new(&db_url).await;
        // Initialize empty test db
        db_int.reinit_all_data().await;

        sleep(Duration::from_secs(1)).await;
        // words to be added
        let test_words = vec!["I".to_owned(), "like".to_owned(), "pie".to_owned()];
        // Add necessary record for foreign key constraint
        db_int
            .add_tracked_member(123, 456)
            .await
            .expect("Should be able to add tracked member");
        // Fetch the previously added member's ID
        let member_id = db_int
            .fetch_member(123, 456)
            .await
            .expect("Should get a result")
            .expect("Should have a member here");
        let test_words_clone = test_words.clone();

        db_int
            .add_member_words(member_id, test_words)
            .await
            .expect("Should be able to add member words here"); // Adding words in an empty db works fine
                                                                // No errors here

        sleep(Duration::from_secs(1)).await; // If this is necessary, the program becomes *really* unstable at runtime.
        db_int
            .add_member_words(member_id, test_words_clone)
            .await
            .expect("Should be able to add member words here");

        // These rows represent the expected results of the two previous insertions
        let expected_row1 = (1, member_id, 2, 2);
        let expected_row2 = (2, member_id, 3, 2);
        // Here I get the rows that were added
        let found_row1 = sqlx::query!(
            "SELECT * from member_words where member_id = $1 and word_id = 1",
            member_id as i32
        )
        .fetch_one(&db_int.db)
        .await
        .expect("Shoudl be able to get row");
        let found_row2 = sqlx::query!(
            "SELECT * from member_words where member_id = $1 and word_id = 2",
            member_id as i32
        )
        .fetch_one(&db_int.db)
        .await
        .expect("Shoudl be able to get row");
        // Assertion fails here! found_row1.frequency == 1, not the expected 2
        assert_eq!(
            expected_row1,
            (
                found_row1.word_id,
                found_row1.member_id as u32,
                found_row1.next_word_id,
                found_row1.frequency
            )
        );
        assert_eq!(
            expected_row2,
            (
                found_row2.word_id,
                found_row2.member_id as u32,
                found_row2.next_word_id,
                found_row2.frequency
            )
        );
    }
    // #[tokio::test]
    // async fn test_schema_output() {
    //     use std::env;
    //     dotenv::dotenv().expect("We should be able to load a .env file.");
    //     let db_url: String =
    //         env::var("DATABASE_URL").expect("We should have the database url in the .env file.");
    //     let mut db_interface = DbInterface::new(&db_url).await;
    //     match db_interface.check_db_schema().await {
    //         Ok(state) => {
    //             println!("Database found, the schema is {:?}", state);
    //             assert!(true);
    //         }
    //         Err(why) => {
    //             println!("SQLx error: {:?}", why);
    //             assert!(false);
    //         }
    //     }
    // }
}
