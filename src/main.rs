// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license
use regex::RegexSet;
use std::time::Duration;
use nostr_sdk::prelude::*;
type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
const BECH32_SK: &str = "<nsec key>";

#[derive(Serialize, Deserialize, Debug)]
struct BoltCardUpdateResponse {
    status: String   
}
#[derive(Serialize, Deserialize, Debug)]
struct JSONResponse {
    json: HashMap<String, String>,
}

async fn process_bolt_card_req(msg: String) -> Result<String, Box<dyn std::error::Error>> {
    let hclient = reqwest::Client::new(); 
    let cmd_vec: Vec<&str> = msg.split(" ").collect();
    let cname = cmd_vec[1].to_string();
    let get_url_string = format!("http://{}:9001/getboltcard?", "localhost");                                        
    let req = hclient        
    .get(get_url_string).query(&[("card_name", &cname)]);
    let res = req.send().await?.text().await?;
    let v: Value = serde_json::from_str(&res).unwrap();    
    if v.get("reason") != None {
        let err_str = format!("BoltCard Service Bot: {} unavailable", cname);
        log::error!("{}", v["reason"]);                
        return Ok(err_str.to_string());
    }       
    let cmd = cmd_vec[0].to_string(); 
    let mut cstatus = "true".to_string();               
    let mut tx_max = v.get("tx_limit_sats").unwrap().to_string();
    let mut day_max = v.get("day_limit_sats").unwrap().to_string();
    if cmd.contains("get") {
        return Ok(v.to_string())
     }
    else if cmd.contains("freeze") {
       cstatus = "false".to_string()
    }    
    else if cmd.contains("tx_max") {
        tx_max = cmd_vec[2].to_string();       
    }
    else if cmd.contains("day_max") {
        day_max = cmd_vec[2].to_string();
    }
   
    let url_string = format!("http://{}:9001/updateboltcard?", "localhost");   
    let req = hclient        
    .get(url_string)   
    .query(&[("card_name", cname), ("enable", cstatus),
     ("tx_max", tx_max.replace("\"","")), ("day_max", day_max.replace("\"",""))]);        
    let res = req.send().await?.json::<BoltCardUpdateResponse>().await?;    
    let mut resp_str = String::new();
    if res.status == "OK" {
        if cmd.contains("freeze") {
              resp_str = format!("BoltCard Bot: {} - DISABLED. To enable send command: /enable {}",
               cmd_vec[1].to_string(), cmd_vec[1].to_string())
        }   
        else if cmd.contains("enable") {
            resp_str = format!("BoltCard Bot: {} - ENABLED. To disable send command: /freeze {}",
               cmd_vec[1].to_string(), cmd_vec[1].to_string())
        }
        else if cmd.contains("tx_max") {
            resp_str = format!("BoltCard Bot: {} - New transaction maximum set: {} satoshis",
               cmd_vec[1].to_string(), tx_max.to_string().replace("\"",""))
        }
        else if cmd.contains("day_max") {
            resp_str = format!("BoltCard Bot: {} - New daily maximum set: {} satoshis",
               cmd_vec[1].to_string(), day_max.to_string().replace("\"",""))
        }
    }
    else if res.status == "ERROR" {
        resp_str = format!("BoltCard Bot: {} - Command: {} failed to run",
               cmd_vec[1].to_string(), cmd_vec[0].to_string())
    }
    Ok(resp_str)
}

#[tokio::main]
async fn main() ->  Result<()> {    
    env_logger::init();
    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let keys = Keys::new(secret_key);
    let opts = Options::new().skip_disconnected_relays(true)
    .connection_timeout(Some(Duration::from_secs(10)))
    .send_timeout(Some(Duration::from_secs(5)));
    let client = Client::with_opts(&keys, opts);
    
    client.add_relay("wss://nostr.mom").await?;

    client.connect().await;

    println!("Bot public key: {}", keys.public_key().to_bech32()?);

    let metadata = Metadata::new()
        .name("BoltCardBot")
        .display_name("BoltCard Service Bot")
        .website(Url::parse("https://example.com")?);
    client.set_metadata(&metadata).await?;

    let subscription = Filter::new()
        .pubkey(keys.public_key())
        .kind(Kind::GiftWrap)
        .limit(0);

    client.subscribe(vec![subscription], None).await;

    client.handle_notifications(|notification| async {
            if let RelayPoolNotification::Event {event, .. } = notification {
                if event.kind == Kind::GiftWrap {
                    let mut content: String = String::from("Invalid command, send /help to see all commands.");
                    match UnwrappedGift::from_gift_wrap(&keys, &event) {                    
                            Ok(UnwrappedGift { rumor, sender }) => {
                                if rumor.kind == Kind::PrivateDirectMessage {
                            let re = RegexSet::new(&[r"/freeze (\w+)", 
                            r"/enable (\w+)",
                            r"/tx_max (\w+) (\d+)",
                            r"/day_max (\w+) (\d+)",
                            r"/get (\w+)",
                            r"/help"]).unwrap();                            
                            let matches = re.matches(rumor.content.as_str());
                            for i in 0..6 {
                                let matched = matches.matched(i);
                                if matched {
                                    match i {
                                     0..=4 =>   content = process_bolt_card_req(rumor.content.clone()).await?,                                     
                                     5 =>  content = help(),                                     
                                    _ => todo!()
                                    }                                    
                                }
                            }                                                                                   
                            
                            client.send_private_msg(sender, content, None).await?;
                          }
                        }
                        Err(e) => log::error!("Impossible to decrypt direct message: {e}"),                       
                    }
                }
            }
            Ok((false)) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}

fn help() -> String {
    let mut output = String::new();
    output.push_str("BoltCard Bot Commands:\n");
    output.push_str("/freeze <card_name> - Disables BoltCard for payments\n");
    output.push_str("/enable <card_name> - Enables BoltCard for payments\n");
    output.push_str("/get  <card_name> - Displays BoltCard details\n");
    output.push_str("/tx_max <card_name> <sats> - Sets maximum satoshis amount for individual transaction\n");
    output.push_str("/day_max <card_name> <sats> - Sets daily maximum satoshis amount\n");
    output.push_str("/help - Help");
    output
}
