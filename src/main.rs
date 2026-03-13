use redis::AsyncCommands;
use reqwest::Client;
use std::env;
use tokio::time::{sleep, Duration};

use insider_alert_monitor::authorities::sec::SecForm4Monitor;
use insider_alert_monitor::parse_selected_monitors_env;

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
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    let selected_monitors = parse_selected_monitors_env();
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
        redis_keys_to_clear.push("monitor:SEC:transaction_ids".to_string());
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
            let dedupe_key = format!("monitor:{}:transaction_ids", transaction.authority);
            let is_new: usize = redis_conn.sadd(dedupe_key, &payload).await?;

            if is_new == 0 {
                continue;
            }

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