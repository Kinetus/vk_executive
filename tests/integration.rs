mod common;

use vk_method::Method;
use serde_json::Value;
use dotenv::dotenv;
use std::env;
use fast_vk::{Instance, Client};
use vk_method::{PairsArray, Params};

use futures::future::join_all;
use common::USERS;

#[tokio::test(flavor = "multi_thread")]
async fn ten_tasks_three_workers() {
    dotenv().unwrap();
    let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(3)).unwrap();

    let pool = Client::from_instances(instances);

    let mut vec = Vec::new();

    for i in 1..11 {
        let params = Params::try_from(PairsArray([("user_id", i)])).unwrap();

        vec.push(pool.send(Method::new("users.get", params)));
    }

    let responses = join_all(vec).await;

    for (index, res) in responses.into_iter().enumerate() {
        let res: Vec<Value> = serde_json::from_value(res.unwrap()).unwrap();
        assert_eq!(res[0], USERS[index]);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn one_thousand_tasks_ten_workers() {
    dotenv().unwrap();
    
    let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(10)).unwrap();

    let pool = Client::from_instances(instances);

    let mut vec = Vec::new();

    for i in 1..1001 {
        let params = Params::try_from(PairsArray([("user_id", i)])).unwrap();

        vec.push(pool.send(Method::new("users.get", params)));
    }

    let _responses = join_all(vec).await;

    println!("done");
}
#[tokio::test(flavor = "multi_thread")]
async fn one_task_one_worker() {
    dotenv().unwrap();

    let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(1)).unwrap();
    let pool = Client::from_instances(instances);

    let mut params = Params::new();
    params.insert("user_id", 1);

    let response = pool.send(Method::new(
        "users.get",
        params
    )).await.unwrap();

    assert_eq!(
        response,
        serde_json::json!([
            {
                "id": 1,
                "first_name": "Pavel",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }
        ])
    )
}