use url::Url;
use std::error::Error;
use linkify::{LinkFinder, LinkKind};
use serenity::cache::{Cache, Settings};
use std::sync::Arc;

static SITES: [&str; 4] = ["www.reddit.com", "discord.com", "www.youtube.com", "discord.gift"];

fn levenshtein_distance<T: Eq>(str1: &[T], str2: &[T]) -> usize {
    use std::cmp::min;
    let mut track: Vec<Vec<usize>> = vec![vec![0; str2.len() + 1]; str1.len() + 1];
    for i in 0..str1.len() {
        track[i][0] = i;
    }
    for i in 0..str2.len() {
        track[0][i] = i;
    }
    for j in 1..=str2.len() {
        for i in 1..=str1.len() {
            let sub_cost = if str1[i - 1] == str2[j - 1] {0} else {1};
            track[i][j] = min(
                min(
                    track[i    ][j - 1] + 1, // deletion
                    track[i - 1][j    ] + 1, // insertion
                ),
                track[i - 1][j - 1] + sub_cost, // substitution
            );
        }
    }
    return track[str1.len()][str2.len()];
}

#[test]
fn test_dist() {
    assert_eq!(levenshtein_distance(b"kitten", b"sitting"), 3);
    assert_eq!(levenshtein_distance(b"discord", b"dicsord"), 2)
}

fn check_pishing(url: &str) -> Option<String> {
    match Url::parse(url).ok() {
        Some(url) => {
            let host = match url.host() {
                Some(url::Host::Domain(x)) => x,
                _ => return None
            };
            for site in SITES {
                let dist = levenshtein_distance(host.as_bytes(),site.as_bytes());
                if dist != 0 && dist < 4 {
                    return Some(site.to_owned())
                }
            }
            None
        },
        None => None
    }
}

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use std::env;


struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let finder = LinkFinder::new();
        let links: Vec<_> = finder.links(&msg.content).collect();
        for link in links {
            match check_pishing(&link.as_str()) {
                Some(lookalike) => {
                    match msg.channel_id.say(&ctx.http, format!("Warining, {} is not {}", link.as_str(), lookalike)).await {
                        Ok(_) => (),
                        Err(e) => println!("ERROR: {:?}", e)
                    }
                }
                None => ()
            }
        }
        return;
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(&token).event_handler(Handler {}).await.expect("Err creating client");
    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {:?}", why);
    }
}
