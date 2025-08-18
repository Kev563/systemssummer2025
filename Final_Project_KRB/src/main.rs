// ---------------------------------------------------------
// Website Status Checker - Final Project (CSCI 3334)
// Kevin Bueno 
// ---------------------------------------------------------
// ---------------------------------------------------------

use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

#[derive(Debug, Clone)]
struct WebsiteStatus {
    url: String,
    status: Result<u16, String>,
    response_time: Duration,
    timestamp: DateTime<Utc>,
}

// ----- simple config -----
#[derive(Debug, Clone, Copy)]
struct Config {
    workers: usize,           // number of worker threads
    timeout_secs: u64,        // request timeout
    retries: u8,              // max retries per website
    period_secs: Option<u64>, // repeat every N seconds if Some
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workers: 50,
            timeout_secs: 5,
            retries: 1,
            period_secs: None,
        }
    }
}

//  (manual, student-level) 
fn parse_args() -> (Config, Option<String>, Vec<String>) {
    let mut cfg = Config::default();
    let mut urls: Vec<String> = Vec::new();
    let mut file_arg: Option<String> = None;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--workers" if i + 1 < args.len() => {
                cfg.workers = args[i + 1].parse().unwrap_or(cfg.workers);
                i += 2;
            }
            "--timeout" if i + 1 < args.len() => {
                cfg.timeout_secs = args[i + 1].parse().unwrap_or(cfg.timeout_secs);
                i += 2;
            }
            "--retries" if i + 1 < args.len() => {
                cfg.retries = args[i + 1].parse().unwrap_or(cfg.retries);
                i += 2;
            }
            "--period" if i + 1 < args.len() => {
                let p: u64 = args[i + 1].parse().unwrap_or(0);
                cfg.period_secs = if p > 0 { Some(p) } else { None };
                i += 2;
            }
            "--file" if i + 1 < args.len() => {
                file_arg = Some(args[i + 1].clone());
                i += 2;
            }
            s => {
                // treat anything else as a URL
                urls.push(s.to_string());
                i += 1;
            }
        }
    }
    (cfg, file_arg, urls)
}

fn load_urls_from_file(path: &str) -> Vec<String> {
    if let Ok(f) = File::open(path) {
        BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        eprintln!("Could not open file '{}', continuing without it.", path);
        Vec::new()
    }
}

//  request logic  
fn make_agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build()
}

// returns a WebsiteStatus for one URL (with retries)
fn check_once(agent: &ureq::Agent, url: &str, retries: u8) -> WebsiteStatus {
    let started = Instant::now();
    let mut last_err = String::new();

    for _attempt in 0..=retries {
        match agent
            .get(url)
            .set("User-Agent", "final-project-krb/0.1 (student)")
            .call()
        {
            Ok(resp) => {
                let code = resp.status();
                return WebsiteStatus {
                    url: url.to_string(),
                    status: Ok(code),
                    response_time: started.elapsed(),
                    timestamp: Utc::now(),
                };
            }
            Err(e) => {
                last_err = format!("{}", e);
                thread::sleep(Duration::from_millis(150));
            }
        }
    }

    WebsiteStatus {
        url: url.to_string(),
        status: Err(last_err),
        response_time: started.elapsed(),
        timestamp: Utc::now(),
    }
}

// ----- job type for workers -----
enum Job {
    Check(String),
    Quit,
}

// ----- run one round of checks with a worker pool -----
fn run_once(urls: &[String], cfg: Config) -> Vec<WebsiteStatus> {
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let job_rx = Arc::new(Mutex::new(job_rx)); // share receiver among workers

    let (res_tx, res_rx) = mpsc::channel::<WebsiteStatus>();

    // spawn workers
    let mut handles = Vec::new();
    for _ in 0..cfg.workers {
        let rx = Arc::clone(&job_rx);
        let tx = res_tx.clone();
        let agent = make_agent(Duration::from_secs(cfg.timeout_secs));
        let retries = cfg.retries;

        let h = thread::spawn(move || loop {
            // receive exactly one job while holding the lock,
            // then drop the lock before doing the HTTP work.
            let msg = {
                let r = rx.lock().unwrap();
                r.recv()
            };

            match msg {
                Ok(Job::Check(url)) => {
                    let status = check_once(&agent, &url, retries);
                    let _ = tx.send(status);
                }
                Ok(Job::Quit) | Err(_) => break,
            }
        });
        handles.push(h);
    }

    // queue jobs
    for u in urls {
        let _ = job_tx.send(Job::Check(u.clone()));
    }

    // collect results
    let mut results = Vec::with_capacity(urls.len());
    for _ in 0..urls.len() {
        if let Ok(s) = res_rx.recv() {
            results.push(s);
        }
    }

    // tell workers to quit and join
    for _ in 0..cfg.workers {
        let _ = job_tx.send(Job::Quit);
    }
    for h in handles {
        let _ = h.join();
    }

    results
}

//  simple printing + stats 
fn print_results(results: &[WebsiteStatus]) {
    println!(
        "{:<6}  {:<45}  {:<9}  {:<8}  {}",
        "TIME", "URL", "STATUS", "MS", "ERROR"
    );
    for r in results {
        let ms = r.response_time.as_millis();
        let time_s = r.timestamp.format("%H:%M:%S").to_string();
        match &r.status {
            Ok(code) => println!(
                "{:<6}  {:<45}  {:<9}  {:<8}  -",
                time_s,
                truncate(&r.url, 45),
                code,
                ms
            ),
            Err(e) => println!(
                "{:<6}  {:<45}  {:<9}  {:<8}  {}",
                time_s,
                truncate(&r.url, 45),
                "ERR",
                ms,
                truncate(e, 50)
            ),
        }
    }

    // stats: avg response time (successful only) + uptime %
    let mut ok_count = 0usize;
    let mut sum_ms = 0u128;
    for r in results {
        if let Ok(code) = r.status {
            if code < 400 {
                ok_count += 1;
                sum_ms += r.response_time.as_millis();
            }
        }
    }
    let total = results.len().max(1);
    let uptime = (ok_count as f64) * 100.0 / (total as f64);
    let avg_ms = if ok_count > 0 {
        (sum_ms / ok_count as u128) as u64
    } else {
        0
    };

    println!(
        "\nSummary: ok={}/{}  uptime={:.1}%  avg_ms={} (success only)",
        ok_count, total, uptime, avg_ms
    );
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n.saturating_sub(1)])
    }
}

// ----- main flow -----
fn main() {
    let (mut cfg, file_arg, mut urls) = parse_args();

    if let Some(path) = file_arg {
        let mut from_file = load_urls_from_file(&path);
        urls.append(&mut from_file);
    }

    if urls.is_empty() {
        urls = vec![
            "https://www.google.com/".to_string(),
            "https://www.rust-lang.org/".to_string(),
            "https://httpbin.org/status/200".to_string(),
            "https://httpbin.org/status/404".to_string(),
            "https://example.com/".to_string(),
        ];
        println!("No URLs provided; using small default list ({} urls).", urls.len());
    }

    if cfg.workers == 0 {
        cfg.workers = 1;
    }

    println!(
        "Starting Website Status Checker | workers={}, timeout={}s, retries={}, urls={}, periodic={}",
        cfg.workers,
        cfg.timeout_secs,
        cfg.retries,
        urls.len(),
        cfg.period_secs
            .map(|p| format!("every {}s", p))
            .unwrap_or_else(|| "off".to_string())
    );

    match cfg.period_secs {
        None => {
            let results = run_once(&urls, cfg);
            print_results(&results);
        }
        Some(period) => {
            // Graceful shutdown
            let stop = Arc::new(AtomicBool::new(false));
            let stop_clone = Arc::clone(&stop);
            thread::spawn(move || {
                let _ = std::io::stdin().read_line(&mut String::new());
                stop_clone.store(true, Ordering::SeqCst);
            });

            let mut round = 1u64;
            while !stop.load(Ordering::SeqCst) {
                println!("\n=== Round {} ===", round);
                let results = run_once(&urls, cfg);
                print_results(&results);
                round += 1;

                // Sleep up to period
                for _ in 0..period {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
            println!("Stopping (Enter pressed). Bye!");
        }
    }





}





// ---------------------------------------------------------
// Basic tests (simple)
// ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    // HTTP server that returns 200 OK once
    fn spawn_ok_server() -> (std::thread::JoinHandle<()>, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let h = std::thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK");
            }
        });
        (h, format!("http://{}", addr))
    }

    // tiny HTTP server that returns 404 once
    fn spawn_404_server() -> (std::thread::JoinHandle<()>, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let h = std::thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let _ = write!(s, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            }
        });
        (h, format!("http://{}", addr))
    }

    #[test]
    fn test_ok_status() {
        let (h, url) = spawn_ok_server();
        let agent = make_agent(Duration::from_secs(2));
        let r = check_once(&agent, &url, 0);
        h.join().unwrap();
        assert!(matches!(r.status, Ok(200)));
    }

    #[test]
    fn test_err_status_404() {
        let (h, url) = spawn_404_server();
        let agent = make_agent(Duration::from_secs(2));
        let r = check_once(&agent, &url, 0);
        h.join().unwrap();
        assert!(matches!(r.status, Err(_))); // ureq treats 404 as error
    }

    #[test]
    fn test_config_defaults() {
        let c = Config::default();
        assert_eq!(c.workers, 50);
        assert_eq!(c.timeout_secs, 5);
        assert_eq!(c.retries, 1);
        assert!(c.period_secs.is_none());
    }
}


