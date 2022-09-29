use vk_method::Method;
use serde_json::Value;
use dotenv::dotenv;
use std::env;
use fast_vk::{Instance, InstancePool};
use vk_method::{PairsArray, Params};

use futures::future::join_all;

//TODO make vk mock server
fn get_users() -> Vec<Value> {
    return vec![
        serde_json::from_str(
            r#"{
                "id": 1,
                "first_name": "Pavel",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 2,
                "first_name": "Alexandra",
                "last_name": "Vladimirova",
                "is_closed": true,
                "can_access_closed": false
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 3,
                "first_name": "DELETED",
                "last_name": "",
                "deactivated": "deleted"
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 4,
                "first_name": "DELETED",
                "last_name": "",
                "deactivated": "deleted"
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 5,
                "first_name": "Ilya",
                "last_name": "Perekopsky",
                "is_closed": false,
                "can_access_closed": true
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 6,
                "first_name": "Nikolay",
                "last_name": "Durov",
                "is_closed": false,
                "can_access_closed": true
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 7,
                "first_name": "Alexey",
                "last_name": "Kobylyansky",
                "is_closed": true,
                "can_access_closed": false
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 8,
                "first_name": "Aki",
                "last_name": "Sepiashvili",
                "is_closed": false,
                "can_access_closed": true
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 9,
                "first_name": "Nastya",
                "last_name": "Vasilyeva",
                "is_closed": true,
                "can_access_closed": false
            }"#,
        )
        .unwrap(),
        serde_json::from_str(
            r#"{
                "id": 10,
                "first_name": "Alexander",
                "last_name": "Kuznetsov",
                "is_closed": true,
                "can_access_closed": false
            }"#,
        )
        .unwrap(),
    ];
}

#[tokio::test(flavor = "multi_thread")]
async fn ten_tasks_three_workers() {
    dotenv().unwrap();
    let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(3)).unwrap();

    let pool = InstancePool::new(instances);

    let mut vec = Vec::new();

    for i in 1..11 {
        let params = Params::try_from(PairsArray([("user_id", i)])).unwrap();

        vec.push(pool.run(Method::new("users.get", params)));
    }

    let responses = join_all(vec).await;

    for (index, res) in responses.into_iter().enumerate() {
        let res: Vec<Value> = serde_json::from_value(res.unwrap()).unwrap();
        assert_eq!(res[0], get_users()[index]);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn one_thousand_tasks_ten_workers() {
    dotenv().unwrap();
    
    let instances = Instance::from_tokens(env::var("tokens").unwrap().split(",").take(10)).unwrap();

    let pool = InstancePool::new(instances);

    let mut vec = Vec::new();

    for i in 1..1001 {
        let params = Params::try_from(PairsArray([("user_id", i)])).unwrap();

        vec.push(pool.run(Method::new("users.get", params)));
    }

    let responses = join_all(vec).await;

    for res in responses {
        println!("{:?}", res)
    }
}
