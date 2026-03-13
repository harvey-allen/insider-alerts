pub mod authorities;
pub mod helpers;

use serde::Serialize;
use std::collections::HashSet;
use std::env;

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

pub fn parse_selected_monitors_from(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn parse_selected_monitors_env() -> HashSet<String> {
    let raw = env::var("MONITORS").unwrap_or_else(|_| "SEC".to_string());
    parse_selected_monitors_from(&raw)
}