use reqwest::{blocking::Client, Error, StatusCode};
use serde_json::Value;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
struct SMSResponse {
    account_sid: Option<String>,
    api_version: String,
    body: String,
    date_created: String,
    date_sent: String,
    date_updated: String,
    direction: String,
    error_code: String,
    error_message: String,
    from: String,
    messaging_service_sid: String,
    num_media: String,
    num_segments: String,
    price: String,
    price_unit: String,
    sid: String,
    status: String,
    subresource_uris: SubresourceUris,
    to: String,
    uri: String,
}

#[derive(Serialize, Deserialize)]
struct SubresourceUris {
    all_time: String,
    today: String,
    yesterday: String,
    this_month: String,
    last_month: String,
    daily: String,
    monthly: String,
    yearly: String,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    more_info: String,
    status: u16
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_sms_sent_time = Instant::now();
    let mut sms_sent_recently = false;
    let mut sms_counter = 0;

    loop {
        let graphql_query = serde_json::json!({
            "operationName": "GetRealmStatusData",
            "variables": {
                "input": {"compoundRegionGameVersionSlug": "classic1x-eu"}
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "b37e546366a58e211e922b8c96cd1ff74249f564a49029cc9737fef3300ff175"
                }
            }
        });

        let response = reqwest::blocking::Client::new()
            .post("https://worldofwarcraft.blizzard.com/graphql")
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            )
            .json(&graphql_query)
            .send();
        if response.is_err(){
            print!("Here");
            println!("{:?}",response);
            continue;
        }else{
            let response = response.unwrap();
            if response.status().is_success() {
                let json_response: serde_json::Value = response.json()?;
    
                if let Some(realms) = json_response["data"]["Realms"].as_array() {
                    for realm in realms.iter() {
                        let realm_name = realm["name"].as_str();
    
                        if realm_name.unwrap() == "Living Flame" {
                            if realm["realmLockStatus"]["isLockedForNewCharacters"] == "false"
                                || realm["realmLockStatus"] == Value::Null
                            {
                                if !sms_sent_recently {
                                    print!("HERE! SMS Counter: {}\n", sms_counter);
                                    send_sms()?;
                                    last_sms_sent_time = Instant::now();
                                    sms_sent_recently = true;
                                    sms_counter+= 1;
                                    if sms_counter >= 20 {
                                        return Ok(());
                                    }
                                }
                            }else{
                                print!("Realm: Living Flame, Status:{}\n",realm["realmLockStatus"])
                            }
                        }
                    }
                }
            }
    
            if sms_sent_recently && last_sms_sent_time.elapsed() >= Duration::from_secs(300) {
                sms_sent_recently = false;
            }
    
        }
        thread::sleep(Duration::from_secs(10));

        
        }
}

fn handle_error(body: String) {
    let error_response: ErrorResponse = serde_json::from_str(&body).expect("Unable to deserialise JSON error response.");
    println!("SMS was not able to be sent because: {:?}.", error_response.message);
}

fn handle_success(body: String) {
    let sms_response: SMSResponse = serde_json::from_str(&body).expect("Unable to deserialise JSON success response.");
    println!("Your SMS with the body \"{:?}\".", sms_response.body);
}

fn send_sms() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let twilio_account_sid =
        env::var("TWILIO_ACCOUNT_SID").expect("Twilio Account SID could not be retrieved.");
    let twilio_auth_token =
        env::var("TWILIO_AUTH_TOKEN").expect("Twilio Auth Token could not be retrieved.");
    let twilio_phone_number =
        env::var("TWILIO_PHONE_NUMBER").expect("The Twilio phone number could not be retrieved.");
    let recipient_phone_number = env::var("RECIPIENT_PHONE_NUMBER")
        .expect("The recipient's phone number could not be retrieved.");

    let sms_body = "MAKE ACCOUNT FAST IDIOT".to_string();

    let request_url =
        format!("https://api.twilio.com/2010-04-01/Accounts/{twilio_account_sid}/Messages.json");

    let client = Client::new();
    let request_params = [
        ("To", &recipient_phone_number),
        ("From", &twilio_phone_number),
        ("Body", &sms_body),
    ];
    let response = client
        .post(request_url)
        .basic_auth(twilio_account_sid, Some(twilio_auth_token))
        .form(&request_params)
        .send()?;

    let status = response.status();
    let body = match response.text() {
        Ok(result) => result,
        Err(error) => panic!(
            "Problem extracting the JSON body content. Reason: {:?}",
            error
        ),
    };

    match status {
        StatusCode::BAD_REQUEST => handle_error(body),
        StatusCode::OK => handle_success(body),
        _ => println!("Received status code: {}", status),
    }

    Ok(())
}