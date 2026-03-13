use insider_alert_monitor::authorities::sec::SecForm4Monitor;
use insider_alert_monitor::{parse_selected_monitors_from, InsiderTransaction};
use reqwest::Client;

#[test]
fn parse_selected_monitors_normalizes_and_deduplicates() {
    let selected = parse_selected_monitors_from(" sec, fca ,SEC, ,rns ");

    assert_eq!(selected.len(), 3);
    assert!(selected.contains("SEC"));
    assert!(selected.contains("FCA"));
    assert!(selected.contains("RNS"));
}

#[test]
fn insider_transaction_serializes_to_json() {
    let transaction = InsiderTransaction {
        authority: "SEC".to_string(),
        issuer_name: "Example Corp".to_string(),
        issuer_ticker: "EXM".to_string(),
        insider_name: "Jane Insider".to_string(),
        insider_cik: "0001234567".to_string(),
        transaction_type: "P".to_string(),
        security_title: "Common Stock".to_string(),
        transaction_date: "2026-03-13".to_string(),
        shares: 100,
        price: 10.5,
        shares_owned_following: 200,
        transaction_code: "P".to_string(),
        acquired_or_disposed: "A".to_string(),
        filing_date: "2026-03-13".to_string(),
    };

    let payload = serde_json::to_value(&transaction).expect("serialization should succeed");

    assert_eq!(payload["authority"], "SEC");
    assert_eq!(payload["issuer_ticker"], "EXM");
    assert_eq!(payload["shares"], 100);
    assert_eq!(payload["transaction_code"], "P");
}

#[test]
fn sec_monitor_constructor_succeeds() {
    let monitor = SecForm4Monitor::new(Client::new(), "SEC");
    assert!(monitor.is_ok());
}