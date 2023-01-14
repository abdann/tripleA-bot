use surreal_simple_client::SurrealClient;
#[tokio::test]
async fn test_surreal_db() {
    let mut client = SurrealClient::new("http://localhost:8000")
        .await
        .expect("Should be able to get client");
    client.signin("root", "root").await;
    client
        .use_namespace("test", "test")
        .await
        .expect("Should be able to use namespace");
}

#[test]
fn test_script() {
    use surreal_simple_querybuilder::prelude::*;

    let query = QueryBuilder::new()
        // 👇 edges can be referenced using an alias
        .select(user.friends.as_alias("friends"))
        .from(user)
        // 👇 but also in queries
        .filter(user.friends.filter(&user.age.greater_than("10")))
        .build();
    let query = QueryBuilder::new().select("");

    // SELECT ->likes->User AS friends FROM User WHERE ->likes->(User WHERE age > 10)
    println!("query: {query}");
}
