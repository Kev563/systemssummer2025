// ----------------Kevin Bueno CSCI 3334-----------------------------------------
// Had some 409 error for etheurem that try to fix, but didnd find answer but output its good.
// ---------------------------------------------------------

use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

// ---------- Error type ----------
#[derive(Debug)]
enum FetchError {
    Network(String),
    Parse(String),
    File(String),
}

// ---------- Trait ----------
trait Pricing {
    fn name(&self) -> &str;
    fn fetch_price(&self) -> Result<f64, FetchError>;
    fn save_to_file(&self, price: f64) -> Result<(), FetchError> {
        let filename = format!("{}.csv", self.name().to_lowercase()); // bitcoin.csv
        let timestamp = Utc::now().to_rfc3339();

        // Create file if it doesn't exist, and add header if empty.
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&filename)
            .map_err(|e| FetchError::File(format!("open {}: {}", filename, e)))?;

        // Check if file is empty to write header
        let mut is_empty = false;
        {
            let mut f = File::open(&filename)
                .map_err(|e| FetchError::File(format!("open (read) {}: {}", filename, e)))?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)
                .map_err(|e| FetchError::File(format!("read {}: {}", filename, e)))?;
            is_empty = buf.trim().is_empty();
        }
        if is_empty {
            writeln!(file, "timestamp,price")
                .map_err(|e| FetchError::File(format!("write header {}: {}", filename, e)))?;
        }

        writeln!(file, "{},{}", timestamp, price)
            .map_err(|e| FetchError::File(format!("append {}: {}", filename, e)))?;

        Ok(())
    }
}

// ---------- BTC ----------
struct Bitcoin;

#[derive(Deserialize)]
struct SimplePriceBTC {
    bitcoin: Currency,
}
#[derive(Deserialize)]
struct Currency {
    usd: f64,
}

impl Pricing for Bitcoin {
    fn name(&self) -> &str {
        "Bitcoin"
    }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| FetchError::Network(format!("BTC request: {}", e)))?;
        let json: SimplePriceBTC = resp
            .into_json()
            .map_err(|e| FetchError::Parse(format!("BTC parse: {}", e)))?;
        Ok(json.bitcoin.usd)
    }
}

// ---------- ETH ----------
struct Ethereum;

#[derive(Deserialize)]
struct SimplePriceETH {
    ethereum: Currency,
}

impl Pricing for Ethereum {
    fn name(&self) -> &str {
        "Ethereum"
    }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| FetchError::Network(format!("ETH request: {}", e)))?;
        let json: SimplePriceETH = resp
            .into_json()
            .map_err(|e| FetchError::Parse(format!("ETH parse: {}", e)))?;
        Ok(json.ethereum.usd)
    }
}

// ---------- S&P 500 (Yahoo Finance) ----------
struct SP500;

impl Pricing for SP500 {
    fn name(&self) -> &str {
        "SP500"
    }
    fn fetch_price(&self) -> Result<f64, FetchError> {
        // Uses the endpoint provided by the professor.
        // We'll read meta.regularMarketPrice as the current price.
        let url = "https://query2.finance.yahoo.com/v8/finance/chart/%5EGSPC";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| FetchError::Network(format!("SP500 request: {}", e)))?;
        let v: Value = resp
            .into_json()
            .map_err(|e| FetchError::Parse(format!("SP500 parse: {}", e)))?;

        let price = v["chart"]["result"][0]["meta"]["regularMarketPrice"]
            .as_f64()
            .ok_or_else(|| FetchError::Parse("SP500: price not found".to_string()))?;
        Ok(price)
    }
}

// ---------- Main loop ----------
fn main() {
    let assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin),
        Box::new(Ethereum),
        Box::new(SP500),
    ];

    println!("Starting Financial Data Fetcher (every 10 seconds). Press Ctrl+C to stop.");

    loop {
        for asset in &assets {
            match asset.fetch_price() {
                Ok(p) => {
                    println!("{:<8} ${:.2}", asset.name(), p);
                    if let Err(e) = asset.save_to_file(p) {
                        eprintln!("Save error for {}: {:?}", asset.name(), e);
                    }
                }
                Err(e) => eprintln!("Fetch error for {}: {:?}", asset.name(), e),
            }
        }
        thread::sleep(Duration::from_secs(10));
    }
}
