use reqwest::Client;
use rpassword;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::PartialEq;
use std::error::Error;
use std::io;

#[derive(PartialEq)]
enum Input {
    Text,
    Password,
}

fn get_input(input_type: Input) -> String {
    let mut input = String::new();

    if input_type == Input::Password {
        return rpassword::read_password_from_tty(Some("Password: ")).unwrap();
    }

    io::stdin()
        .read_line(&mut input)
        .expect("something went wrong");

    input.trim().to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct Auth {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u64,
    created_at: u64,
    scope: String,
}

async fn authenticate(username: String, password: String) -> Result<Auth, Box<dyn Error>> {
    let body = json!({
        "grant_type": "password",
        "username": username,
        "password": password,
    });

    let client = Client::new();

    let res = client
        .post("https://kitsu.io/api/oauth/token")
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?
        .text()
        .await?;

    let auth: Auth = serde_json::from_str(&res.to_string())?;

    Ok(auth)
}

#[derive(Deserialize, Debug)]
struct Data {
    data: Vec<UserId>,
}

#[derive(Deserialize, Debug)]
struct UserId {
    id: String,
}

async fn get_current_account(
    access_token: &String,
    client: Client,
) -> Result<Data, Box<dyn Error>> {
    let bearer = "Bearer ".to_string() + access_token;
    println!("{}", bearer);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Accept", "application/vnd.api+json".parse().unwrap());
    headers.insert("Content-Type", "application/vnd.api+json".parse().unwrap());
    headers.insert("Authorization", bearer.parse().unwrap());

    let res = client
        .get("https://kitsu.io/api/edge/users?filter[self]=true")
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let user_id: Data = serde_json::from_str(&res.to_string())?;

    Ok(user_id)
}

async fn add_favourite(token: &String, character_id: String, user_id: &String) {
    let client = Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Accept", "application/vnd.api+json".parse().unwrap());
    headers.insert("Content-Type", "application/vnd.api+json".parse().unwrap());
    headers.insert(
        "Authorization",
        ("Bearer ".to_string() + &token).parse().unwrap(),
    );

    let body = json!({
            "data": {
                "relationships": {
                    "item": {
                        "data": {
                            "id": character_id,
                            "type": "characters"
                        }
                    },
                    "user": {
                        "data": {
                            "id": user_id,
                            "type": "users"
                        }
                    }
                },
                "type": "favorites"
            }
    })
    .to_string();

    let _res = client
        .post("https://kitsu.io/api/edge/favorites")
        .headers(headers)
        .body(body)
        .send()
        .await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    println!("Input username (url) or email");
    let username = get_input(Input::Text);

    println!("Input password");
    let password = get_input(Input::Password);

    println!("Input the id of the character you want to add as favourite");
    let character_id = get_input(Input::Text);

    let auth = authenticate(username, password).await?;
    let access_token = &auth.access_token;

    let user_id_data = get_current_account(&access_token, client).await?;
    let user_id = &user_id_data.data[0].id;

    add_favourite(&access_token, character_id, &user_id).await;

    println!("Success!");
    Ok(())
}
