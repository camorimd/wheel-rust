/**
    MIT License

    Copyright (c) 2020 Claudio Amorim

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
**/

use log::{info, debug, error};
use clap::{crate_authors, crate_name, clap_app, crate_version, crate_description};
use reqwest;
use std::fs;
use serde::Deserialize;
use rand::{thread_rng, seq::SliceRandom};
use std::collections::HashMap;
use itertools::Itertools;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Chatters{
    broadcaster: Vec<String>,
    vips: Vec<String>,
    moderators: Vec<String>,
    staff: Vec<String>,
    admins: Vec<String>,
    global_mods: Vec<String>,
    viewers: Vec<String>
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Chat {
    chatter_count: i32,
    chatters: Chatters,
}

#[derive(Deserialize)]
struct AccessToken {
    access_token: String
}

struct Token {
    access_token: String,
    client_id: String,
    client_secret: String,
    client_token: Option<String>
}

#[derive(Deserialize, Debug)]
struct Channel {
    id: String
}

#[derive(Deserialize, Debug, Clone)]
struct Followers {
    from_name: String
}

#[derive(Deserialize, Debug)]
struct FollowersData {
    data: Vec<Followers>,
    pagination: Pagination,
    total: i32
}
#[derive(Deserialize, Debug, Clone)]
struct Subscriptions{
    user_name: String
}
#[derive(Deserialize, Debug)]
struct SubscriptionData {
    data: Vec<Subscriptions>,
    pagination: Pagination
}

#[derive(Deserialize, Debug)]
struct Pagination {
    cursor: Option<String>
}

fn open(url: String) {
    use winapi::um::shellapi::ShellExecuteA;
    use std::ptr;
    use std::ffi::CString;

    unsafe {
        ShellExecuteA(
            ptr::null_mut(),
            CString::new("open").unwrap().as_ptr(),
            CString::new(url).unwrap().as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(), 
            winapi::um::winuser::SW_SHOWNORMAL
        );
    }
}

async fn authenticate_user(client: &reqwest::Client, token: &mut Token) {
    use std::net::{TcpListener};
    use std::io::prelude::*;
    use std::io::{BufReader, BufWriter};

    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    let mut code = String::from("");
    open(format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri=http://localhost&response_type=code&scope=channel:read:subscriptions", token.client_id));
    for stream in listener.incoming(){
        match stream {
            Ok(s) => {
                let mut reader = BufReader::new(&s);
                let mut writer = BufWriter::new(&s);
                let mut content = [0; 512];
                reader.read(&mut content).unwrap();
                let request = String::from_utf8_lossy(&content);
                writer.write(b"<html><head><title>Close Window</title></head><body><p>You can close this window now.</p></body></html>").unwrap();               
                code = {
                    use regex::Regex;
                    let re = Regex::new(r".*code=(.*)&.*").unwrap();

                    let a = &request.split("\n").map(|f| String::from(f)).collect::<Vec<String>>()[0];
                    let a = &a.split(" ").map(|f| String::from(f)).collect::<Vec<String>>()[1];
                    if re.is_match(a) {
                        let m = re.captures(a).unwrap().get(1).unwrap();
                        m.as_str().to_string()
                    }else{
                        a.clone()
                    }
                };

                break;                
            },
            Err(x) => {
                error!("Could not receive connection {}", x);
            }
        }
    }
    debug!("Code received {}", code);

    let r = client.post(
        &format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=http://localhost",
        token.client_id, token.client_secret, code)
    ).send().await.unwrap().json::<AccessToken>().await.unwrap();

    token.client_token = Some(r.access_token);
}

async fn get_subs(client: &reqwest::Client, token: &mut Token) -> Result<Vec<String>, reqwest::Error> {
    let mut result = Vec::<String>::new();

    if token.client_token.is_none() {
        authenticate_user(client, token).await;
    }

    let url = format!("https://api.twitch.tv/helix/users");
    let data = client.get(&url)
        .bearer_auth(token.client_token.as_deref().unwrap())
        .header("Client-Id", &token.client_id)
        .send().await?
        .json::<HashMap<String, Vec<Channel>>>().await?;

    let mut url = format!("https://api.twitch.tv/helix/subscriptions?broadcaster_id={}&first=20", data.get("data").unwrap()[0].id);
    let res = client.get(&url)
                .bearer_auth(token.client_token.as_deref().unwrap())
                .header("Client-Id", &token.client_id)
                .send().await?.json::<SubscriptionData>().await?;

    let mut cursor = match &res.pagination.cursor {
        Some(p) => p.clone(),
        None => String::from("")
    };
   

    result.extend(res.data.iter().map(|f| {f.user_name.clone()}).collect::<Vec<String>>());

    while cursor != "" && result.len() > 0{
        url = format!("https://api.twitch.tv/helix/subscriptions?broadcaster_id={}&after={}&first=20", data.get("data").unwrap()[0].id, cursor);
        let res = client.get(&url)
                    .bearer_auth(token.client_token.as_deref().unwrap())
                    .header("Client-Id", &token.client_id)
                    .send().await?.json::<SubscriptionData>().await?;
        
        cursor = match &res.pagination.cursor {
            Some(p) => p.clone(),
            None => String::from("")
        };

        result.extend(res.data.iter().map(|f| {f.user_name.clone()}).collect::<Vec<String>>());
    }
    
    Ok(result)
}

async fn authenticate(client: &reqwest::Client) -> Result<Token, reqwest::Error> {    
    info!("Authenticating app against twitch");
    let contents = fs::read_to_string("app").expect("Error reading app file");
    let app_file: Vec<&str> = contents.split(":").collect();
    
    let url = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", app_file[0], app_file[1]);
    let res = client.post(&url).send().await?;

    let token = res.json::<AccessToken>().await?;
    Ok(Token {
        access_token: token.access_token,
        client_id: String::from(app_file[0]),
        client_secret: String::from(app_file[1]),
        client_token: None
    })
}

async fn get_followers(channel: &str, client: &reqwest::Client, token: &Token) -> Result<Vec<String>, reqwest::Error> {
    info!("Getting followers for {}", channel);
    let mut url = format!("https://api.twitch.tv/helix/users?login={}", channel);
    let req_builder = client.get(&url).bearer_auth(&token.access_token).header("Client-ID", &token.client_id);
    debug!("rBuilder -> {:?}", req_builder);
    let res = req_builder.send().await?.json::<HashMap<String, Vec<Channel>>>().await?;
    let user_id = res.get("data").unwrap()[0].id.clone();
    url = format!("https://api.twitch.tv/helix/users/follows?to_id={}", user_id);

    let req_builder = client.get(&url).bearer_auth(&token.access_token).header("Client-ID", &token.client_id);
    let mut res = req_builder.send().await?.json::<FollowersData>().await?;

    let mut result: Vec<String> = Vec::with_capacity(res.total as usize);

    let mut cursor = match res.pagination.cursor {
        Some(p) => p,
        None => String::from("")
    };

    result.extend(res.data.iter().map(|f| {f.from_name.clone()}).collect::<Vec<String>>());

    while cursor != "" {
        url = format!("https://api.twitch.tv/helix/users/follows?to_id={}&after={}", user_id, cursor);
        let req_builder = client.get(&url).bearer_auth(&token.access_token).header("Client-ID", &token.client_id);
        res = req_builder.send().await?.json::<FollowersData>().await?;
        
        cursor = match &res.pagination.cursor {
            Some(p) => p.clone(),
            None => String::from("")
        };

        result.extend(res.data.iter().map(|f| {f.from_name.clone()}).collect::<Vec<String>>());
    }
    
    Ok(result)
}

fn pause () {
    use std::io;
    use std::io::Read;

    let mut sdtin = io::stdin();
    info!("Press any key to continue");

    let _ = sdtin.read(&mut [0u8]).unwrap();
}

#[tokio::main]
#[allow(deprecated)]
async fn main() -> Result<(), reqwest::Error>{
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let matches = clap_app!((crate_name!()) => 
        (version: crate_version!())
        (author: crate_authors!("\n"))
        (about: crate_description!())
        (@arg drop_moderators: --drop "Drop moderators from giveaway")
        (@group modality =>
            (@attributes +required ...)
            (@arg followers: --followers "Add followers to the giveaway")
            (@arg viewers: --viewers "Add viewers to the giveaway")
            (@arg subs: --subs "Add subs to the giveaway")
        )
        (@arg extra: -e --extra "Give viewers extra tickets")
        (@arg discarded_file: --discarded +takes_value "Set a custom discarded file, default is discarded.txt")
        (@arg channel: +required "User channel")
    ).get_matches();

    let channel = matches.value_of("channel").unwrap();
    let extra_tickets = matches.is_present("extra");
    let drop_moderators = matches.is_present("drop_moderators");
    let add_followers = matches.is_present("followers");
    let add_viewers = matches.is_present("viewers");
    let add_subs = matches.is_present("subs");
    let discard_file_name = matches.value_of("discarded_file").unwrap_or("discarded.txt");
    

    let client = reqwest::Client::new();
    let mut token = authenticate(&client).await?;
   

    let mut tickets = Vec::<String>::new();
    let mut viewers = Vec::<String>::new();
    let mut moderators = Vec::<String>::new();

    if extra_tickets || drop_moderators || add_viewers {
        info!("Downloading viewers and moderators");
        let body = reqwest::get(&format!("https://tmi.twitch.tv/group/user/{}/chatters", &channel))
            .await?
            .json::<Chat>()
            .await?;
        viewers.extend(body.chatters.viewers.iter().map(|f| f.trim_end().to_uppercase().to_string()));
        moderators.extend(body.chatters.moderators.iter().map(|f| f.trim_end().to_uppercase().to_string()));
    }
     

    if add_viewers {
        for v in &viewers {
            tickets.push(v.trim_end().to_uppercase().to_string());
        }
        for v in &moderators {
            tickets.push(v.trim_end().to_uppercase().to_string());
        }
    }
    
    if add_followers {
        for v in get_followers(channel, &client, &token).await? {
            tickets.push(String::from(v).trim_end().to_uppercase().to_string());
        }
    }

    //@TODO: add_subs
    if add_subs {
        get_subs(&client, &mut token).await?;
    }

    tickets = tickets.into_iter().unique().collect();

    if extra_tickets {
        info!("Adding extra tiquets for viewers");
        for v in viewers.iter().chain(moderators.iter()) {
            if tickets.contains(&v) {
                info!("{} extra ticket", &v);
                tickets.push(v.to_uppercase().to_string());
            }
        }
    }
    

    let content = match fs::read_to_string(discard_file_name) {
        Ok(s) => s,
        Err(_x) => {
            info!("No discarded file has been selected");
            String::from("")
        }
    };

    if drop_moderators { info!("Auto dropping moderators");}
    tickets.retain(|f| {
        let delete = {
            let mut found = false;
            for v in content.split("\n") {
                debug!("{} {} {}", v.trim_end().to_uppercase(), *f, v.trim_end().to_uppercase() == *f);
                if v.trim_end().to_uppercase() == *f {
                    found = true;
                    break;
                }
            }
            
            if !found && drop_moderators{
                found = false;
                for v in &moderators {
                    if v.trim_end().to_uppercase() == *f {
                        found = true;
                        break;
                    }
                }
            }
            found
        };
        if delete {
            info!("User {} will be deleted from tickets", f);
        }
        
        !delete
    });

    let mut rng = thread_rng();
    let grouped_tickets = tickets.iter().sorted().group_by(|&f| f);
    info!("{:?}", grouped_tickets.into_iter().map(|(key, group)| (key.clone(), group.count())).collect::<HashMap<String, usize>>());

    pause();

    if cfg!(feature = "distribution")
    {
        use std::collections::HashMap;
        let mut results = HashMap::new();

        for t in tickets.iter() {
            results.insert(t.to_string(), 0);
        }
        
        info!("Tickets {:?}", tickets);

        for i in 1..1000 {
            let s = tickets.choose(&mut rng);
            debug!("{} - {:?}", i, tickets.choose(&mut rng));
            let count = results.entry(s.unwrap().to_string()).or_insert(0);
            *count += 1;
        }
        info!("{:?}", results);
    }else{
        if tickets.len() > 0 {
            let s = tickets.choose(&mut rng);
            info!("Congrats {} you won the giveaway", s.unwrap());
        } else {
            info!("Tickets list are empty: select followers, viewers or subs");
        }
        
    }

    Ok(())
}