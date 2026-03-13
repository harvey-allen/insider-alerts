use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;

use crate::helpers::{log_info, log_error};
use crate::InsiderTransaction;

const SEC_FEED: &str =
    "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=4&output=atom";

pub struct SecForm4Monitor {
    authority: String,
    client: Client,
    seen_links: HashSet<String>,
    xml_regex: Regex,
}

impl SecForm4Monitor {
    pub fn new(client: Client, authority: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let xml_regex = Regex::new(r#"href="([^"]+\.xml)""#)?;
        Ok(SecForm4Monitor {
            authority: authority.to_string(),
            client,
            seen_links: HashSet::new(),
            xml_regex,
        })
    }

    pub async fn fetch_and_parse_feed(
        &mut self,
    ) -> Result<Vec<InsiderTransaction>, Box<dyn std::error::Error>> {
        let feed = self.client.get(SEC_FEED).send().await?.text().await?;
        let mut all_transactions = Vec::new();

        let links: Vec<String> = feed
            .lines()
            .filter(|l| l.contains("Archives/edgar/data"))
            .filter_map(|line| {
                line.split("href=\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .map(|s| {
                        if s.starts_with("http") {
                            s.to_string()
                        } else {
                            format!("https://www.sec.gov{}", s)
                        }
                    })
            })
            .collect();

        for link in links {
            if self.seen_links.contains(&link) {
                continue;
            }
            self.seen_links.insert(link.clone());

            // Fetch the filing index page
            let filing_page = self.client.get(&link).send().await?.text().await?;

            // Extract XML files
            for cap in self.xml_regex.captures_iter(&filing_page) {
                let xml_file = &cap[1];
                let xml_name = xml_file
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                // Skip XSL transforms and other presentation files
                if xml_file.contains("xsl") || !xml_name.ends_with("form4.xml") {
                    continue;
                }

                // Handle absolute vs relative path
                let xml_url = if xml_file.starts_with("/Archives") {
                    format!("https://www.sec.gov{}", xml_file)
                } else {
                    // append to the directory part of the index link
                    let base = link.rsplit_once('/').unwrap().0;
                    format!("{}/{}", base, xml_file)
                };

                log_info("SEC", format!("Fetching XML: {}", xml_url));

                let xml = self.client.get(&xml_url).send().await?.text().await?;

                // Verify it's actual Form 4 XML
                if xml.contains("<ownershipDocument") {

                    let transactions = self.parse_form4(&xml)?;
                    for tx in transactions {
                        log_info("SEC", format!(
                            "Parsed Form4: authority: {}, issuer: {}, insider: {}, type: {}, security: {}, date: {}, shares: {}, price: {}",
                            tx.authority,
                            tx.issuer_name,
                            tx.insider_name,
                            tx.transaction_code,
                            tx.security_title,
                            tx.transaction_date,
                            tx.shares,
                            tx.price
                        ));
                        all_transactions.push(tx);
                    }
                    break; // Stop after first valid XML
                }
            }
        }

        Ok(all_transactions)
    }

    fn read_nested_text(
        reader: &mut Reader<&[u8]>,
        end_tag: &[u8],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut value = String::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    if name.as_ref() == b"value" {
                        let nested_value = Self::read_nested_text(reader, b"value")?;
                        if !nested_value.is_empty() {
                            value = nested_value;
                        }
                    }
                }
                Event::Text(e) => {
                    let text = e.unescape()?.into_owned();
                    let text = text.trim();
                    if !text.is_empty() {
                        value = text.to_string();
                    }
                }
                Event::CData(e) => {
                    let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                    if !text.is_empty() {
                        value = text;
                    }
                }
                Event::End(e) => {
                    if e.name().as_ref() == end_tag {
                        break;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(value)
    }

    fn parse_form4_number(value: &str) -> i64 {
        value.replace(',', "").parse().unwrap_or(0)
    }

    fn parse_form4_price(value: &str) -> f64 {
        value.replace(',', "").parse().unwrap_or(0.0)
    }

    fn parse_form4(&self, xml: &str) -> Result<Vec<InsiderTransaction>, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut transactions = Vec::new();

        let mut issuer_name = String::new();
        let mut issuer_ticker = String::new();
        let mut reporter_name = String::new();
        let mut reporter_cik = String::new();
        let mut period_of_report = String::new();

        let mut transaction_code = String::new();
        let mut security_title = String::new();
        let mut transaction_date = String::new();
        let mut shares: i64 = 0;
        let mut price: f64 = 0.0;
        let mut shares_owned_following: i64 = 0;
        let mut acquired_or_disposed = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"issuerName" => {
                        issuer_name = Self::read_nested_text(&mut reader, b"issuerName")?;
                    }
                    b"issuerTradingSymbol" => {
                        issuer_ticker = Self::read_nested_text(&mut reader, b"issuerTradingSymbol")?;
                    }
                    b"rptOwnerName" => {
                        reporter_name = Self::read_nested_text(&mut reader, b"rptOwnerName")?;
                    }
                    b"rptOwnerCik" => {
                        reporter_cik = Self::read_nested_text(&mut reader, b"rptOwnerCik")?;
                    }
                    b"periodOfReport" => {
                        period_of_report = Self::read_nested_text(&mut reader, b"periodOfReport")?;
                    }
                    b"transactionCode" => {
                        transaction_code = Self::read_nested_text(&mut reader, b"transactionCode")?;
                    }
                    b"securityTitle" => {
                        security_title = Self::read_nested_text(&mut reader, b"securityTitle")?;
                    }
                    b"deemedExecutionDate" => {
                        transaction_date = Self::read_nested_text(&mut reader, b"deemedExecutionDate")?;
                    }
                    b"transactionDate" if transaction_date.is_empty() => {
                        transaction_date = Self::read_nested_text(&mut reader, b"transactionDate")?;
                    }
                    b"transactionShares" => {
                        let raw_shares = Self::read_nested_text(&mut reader, b"transactionShares")?;
                        shares = Self::parse_form4_number(&raw_shares);
                    }
                    b"transactionPricePerShare" => {
                        let raw_price = Self::read_nested_text(&mut reader, b"transactionPricePerShare")?;
                        price = Self::parse_form4_price(&raw_price);
                    }
                    b"sharesOwnedFollowingTransaction" => {
                        let raw_shares = Self::read_nested_text(&mut reader, b"sharesOwnedFollowingTransaction")?;
                        shares_owned_following = Self::parse_form4_number(&raw_shares);
                    }
                    b"transactionAcquiredDisposedCode" => {
                        acquired_or_disposed = Self::read_nested_text(&mut reader, b"transactionAcquiredDisposedCode")?;
                    }
                    _ => {}
                },
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"nonDerivativeTransaction" {
                        let effective_date = if transaction_date.is_empty() {
                            period_of_report.clone()
                        } else {
                            transaction_date.clone()
                        };

                        if !transaction_code.is_empty()
                            || !security_title.is_empty()
                            || !effective_date.is_empty()
                            || !reporter_name.is_empty()
                            || shares != 0
                        {
                            transactions.push(InsiderTransaction {
                                authority: self.authority.clone(),
                                issuer_name: issuer_name.clone(),
                                issuer_ticker: issuer_ticker.clone(),
                                insider_name: reporter_name.clone(),
                                insider_cik: reporter_cik.clone(),
                                transaction_type: transaction_code.clone(),
                                security_title: security_title.clone(),
                                transaction_date: effective_date.clone(),
                                shares,
                                price,
                                shares_owned_following,
                                transaction_code: transaction_code.clone(),
                                acquired_or_disposed: acquired_or_disposed.clone(),
                                filing_date: period_of_report.clone(),
                            });
                        }

                        issuer_name.clear();
                        issuer_ticker.clear();
                        reporter_name.clear();
                        reporter_cik.clear();
                        transaction_code.clear();
                        security_title.clear();
                        transaction_date.clear();
                        shares = 0;
                        price = 0.0;
                        shares_owned_following = 0;
                        acquired_or_disposed.clear();
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    log_error("SEC", format!("XML parse error: {:?}", e));
                    break;
                }
                _ => {}
            }
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::SecForm4Monitor;
    use reqwest::Client;

    fn build_monitor() -> SecForm4Monitor {
        SecForm4Monitor::new(Client::new(), "TEST").expect("monitor should construct")
    }

    #[test]
    fn parse_form4_extracts_expected_fields() {
        let monitor = build_monitor();
        let xml = r#"
        <ownershipDocument>
            <issuer>
                <issuerName>Example Corp</issuerName>
                <issuerTradingSymbol>EXM</issuerTradingSymbol>
            </issuer>
            <reportingOwner>
                <reportingOwnerId>
                    <rptOwnerCik>0001234567</rptOwnerCik>
                    <rptOwnerName>Jane Insider</rptOwnerName>
                </reportingOwnerId>
            </reportingOwner>
            <periodOfReport>2026-03-12</periodOfReport>
            <nonDerivativeTable>
                <nonDerivativeTransaction>
                    <securityTitle><value>Common Stock</value></securityTitle>
                    <transactionDate><value>2026-03-11</value></transactionDate>
                    <transactionCoding>
                        <transactionCode>P</transactionCode>
                    </transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>1,250</value></transactionShares>
                        <transactionPricePerShare><value>14.75</value></transactionPricePerShare>
                        <transactionAcquiredDisposedCode><value>A</value></transactionAcquiredDisposedCode>
                    </transactionAmounts>
                    <postTransactionAmounts>
                        <sharesOwnedFollowingTransaction><value>10,500</value></sharesOwnedFollowingTransaction>
                    </postTransactionAmounts>
                </nonDerivativeTransaction>
            </nonDerivativeTable>
        </ownershipDocument>
        "#;

        let transactions = monitor
            .parse_form4(xml)
            .expect("form4 parse should succeed");

        assert_eq!(transactions.len(), 1);
        let tx = &transactions[0];
        assert_eq!(tx.authority, "TEST");
        assert_eq!(tx.issuer_name, "Example Corp");
        assert_eq!(tx.issuer_ticker, "EXM");
        assert_eq!(tx.insider_name, "Jane Insider");
        assert_eq!(tx.insider_cik, "0001234567");
        assert_eq!(tx.transaction_code, "P");
        assert_eq!(tx.transaction_type, "P");
        assert_eq!(tx.security_title, "Common Stock");
        assert_eq!(tx.transaction_date, "2026-03-11");
        assert_eq!(tx.filing_date, "2026-03-12");
        assert_eq!(tx.shares, 1250);
        assert!((tx.price - 14.75).abs() < f64::EPSILON);
        assert_eq!(tx.shares_owned_following, 10500);
        assert_eq!(tx.acquired_or_disposed, "A");
    }

    #[test]
    fn parse_form4_uses_deemed_execution_date_and_falls_back_to_period() {
        let monitor = build_monitor();
        let xml = r#"
        <ownershipDocument>
            <issuer>
                <issuerName>Issuer One</issuerName>
                <issuerTradingSymbol>ONE</issuerTradingSymbol>
            </issuer>
            <reportingOwner>
                <reportingOwnerId>
                    <rptOwnerCik>0000000001</rptOwnerCik>
                    <rptOwnerName>Owner One</rptOwnerName>
                </reportingOwnerId>
            </reportingOwner>
            <periodOfReport>2026-02-20</periodOfReport>
            <nonDerivativeTable>
                <nonDerivativeTransaction>
                    <securityTitle><value>Class A</value></securityTitle>
                    <deemedExecutionDate><value>2026-02-18</value></deemedExecutionDate>
                    <transactionCoding><transactionCode>S</transactionCode></transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>100</value></transactionShares>
                    </transactionAmounts>
                </nonDerivativeTransaction>
                <nonDerivativeTransaction>
                    <securityTitle><value>Class A</value></securityTitle>
                    <transactionCoding><transactionCode>P</transactionCode></transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>25</value></transactionShares>
                    </transactionAmounts>
                </nonDerivativeTransaction>
            </nonDerivativeTable>
        </ownershipDocument>
        "#;

        let transactions = monitor
            .parse_form4(xml)
            .expect("form4 parse should succeed");

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].transaction_date, "2026-02-18");
        assert_eq!(transactions[1].transaction_date, "2026-02-20");
    }

    #[test]
    fn parse_form4_returns_empty_for_irrelevant_xml() {
        let monitor = build_monitor();
        let xml = r#"<root><note>no transactions here</note></root>"#;

        let transactions = monitor
            .parse_form4(xml)
            .expect("parser should not fail on irrelevant xml");

        assert!(transactions.is_empty());
    }

    #[test]
    fn parse_helpers_handle_invalid_numbers() {
        assert_eq!(SecForm4Monitor::parse_form4_number("1,234"), 1234);
        assert_eq!(SecForm4Monitor::parse_form4_number("N/A"), 0);
        assert!((SecForm4Monitor::parse_form4_price("12,345.67") - 12345.67).abs() < f64::EPSILON);
        assert_eq!(SecForm4Monitor::parse_form4_price("bad"), 0.0);
    }
}
