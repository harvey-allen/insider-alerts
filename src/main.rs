mod authorities;
mod helpers;

use redis::AsyncCommands;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashSet;
use std::env;
use tokio::time::{sleep, Duration};

use authorities::sec::SecForm4Monitor;

#[derive(Debug, Serialize)]
pub struct InsiderTransaction {
    pub authority: String,
    pub issuer_name: String,
    pub issuer_ticker: String,
    pub insider_name: String,
    pub insider_cik: String,
    pub transaction_type: String,
    pub security_title: String,
    pub transaction_date: String,
    pub shares: i64,
    pub price: f64,
    pub shares_owned_following: i64,
    pub transaction_code: String,
    pub acquired_or_disposed: String,
    pub filing_date: String,
}

fn parse_selected_monitors() -> HashSet<String> {
    let raw = env::var("MONITORS").unwrap_or_else(|_| "SEC".to_string());
    raw.split(',')
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
r#"██╗███╗   ██╗███████╗██╗██████╗ ███████╗██████╗ 
██║████╗  ██║██╔════╝██║██╔══██╗██╔════╝██╔══██╗
██║██╔██╗ ██║███████╗██║██║  ██║█████╗  ██████╔╝
██║██║╚██╗██║╚════██║██║██║  ██║██╔══╝  ██╔══██╗
██║██║ ╚████║███████║██║██████╔╝███████╗██║  ██║
╚═╝╚═╝  ╚═══╝╚══════╝╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝

█████╗ ██╗     ███████╗██████╗ ████████╗
██╔══██╗██║     ██╔════╝██╔══██╗╚══██╔══╝
███████║██║     █████╗  ██████╔╝   ██║   
██╔══██║██║     ██╔══╝  ██╔══██╗   ██║   
██║  ██║███████╗███████╗██║  ██║   ██║   
╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝"#
    );

    let client = Client::builder()
        .user_agent("HarveyAllen research@example.com")
        .build()?;
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = redis_client.get_async_connection().await?;

    let selected_monitors = parse_selected_monitors();
    let run_sec = selected_monitors.contains("SEC");
    

    if !run_sec {
        return Err("No valid monitors selected. Use MONITORS=SEC".into());
    }

    let mut monitor_list: Vec<_> = selected_monitors.iter().cloned().collect();
    monitor_list.sort();
    println!("Running monitors: {}", monitor_list.join(", "));

    let mut redis_keys_to_clear = vec!["monitor:all:transactions".to_string()];
    if run_sec {
        redis_keys_to_clear.push("monitor:SEC:transactions".to_string());
    }

    let _: () = redis_conn.del(redis_keys_to_clear).await?;

    let mut sec_monitor = if run_sec {
        Some(SecForm4Monitor::new(client.clone(), "SEC")?)
    } else {
        None
    };

    loop {
        let mut transactions = Vec::new();

        if let Some(monitor) = sec_monitor.as_mut() {
            transactions.extend(monitor.fetch_and_parse_feed().await?);
        }

        for transaction in transactions {
            let payload = serde_json::to_string(&transaction)?;
            let authority_key = format!("monitor:{}:transactions", transaction.authority);
            let _: () = redis_conn
                .rpush("monitor:all:transactions", &payload)
                .await?;
            let _: () = redis_conn
                .rpush(authority_key, payload)
                .await?;
        }

        sleep(Duration::from_secs(10)).await;
    }
}