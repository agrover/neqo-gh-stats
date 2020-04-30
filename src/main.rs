use std::convert::TryInto;
use std::process::exit;

use chrono::prelude::*;
use chrono::{DateTime, Duration};
use github_stats::{Query, Search};
use serde_json::value::Value;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "neqo-gh-stats", about = "Stats on activity of Neqo project")]
struct Args {
    #[structopt(short = "d", long, default_value = "14")]
    /// Days of previous activity to summarize
    days: u16,
}

fn get_all_items(mut search: Search) -> Vec<Value> {
    let mut items = Vec::new();
    let mut results = match search.search() {
        Err(_) => {
            eprintln!("Search failed. Rate limited? Try again later");
            exit(1);
        }
        Ok(x) => x,
    };

    let mut remaining_results = results.total_count();
    while remaining_results > 0 {
        for item in results.items() {
            items.push(item.clone());
            remaining_results -= 1;
        }
        search.next_page();
        results = match search.search() {
            Err(_) => {
                eprintln!("subpage query failed (ratelimited?); incomplete results");
                break;
            }
            Ok(x) => x,
        };
    }
    items
}

fn main() {
    let args = Args::from_args();
    let days = args.days.try_into().unwrap();

    let now = Utc::now();
    let past = now - Duration::days(days);

    // Gets latest merged PR
    let search = Search::issues(&Query::new().repo("mozilla", "neqo").is("issue")).per_page(100);

    let issues = get_all_items(search);

    // Issues opened
    let mut opened = 0;
    for item in &issues {
        if let Some(opened_at) = item.get("created_at") {
            if opened_at.is_string() {
                let opened_at = DateTime::parse_from_rfc3339(opened_at.as_str().unwrap()).unwrap();
                if opened_at > past {
                    opened += 1;
                }
            }
        }
    }
    println!("Issues opened in past {} days: {}", days, opened);

    // Issues closed
    let mut closed = 0;
    for item in &issues {
        if let Some(closed_at) = item.get("closed_at") {
            if closed_at.is_string() {
                let closed_at = DateTime::parse_from_rfc3339(closed_at.as_str().unwrap()).unwrap();
                if closed_at > past {
                    closed += 1;
                }
            }
        }
    }
    println!("Issues closed in past {} days: {}", days, closed);

    // PRs merged
    let search =
        Search::issues(&Query::new().repo("mozilla", "neqo").is("pr").is("merged")).per_page(100);

    let prs = get_all_items(search);

    // Issues merged
    let mut merged = 0;
    for item in &prs {
        if let Some(merged_at) = item.get("closed_at") {
            if merged_at.is_string() {
                let merged_at = DateTime::parse_from_rfc3339(merged_at.as_str().unwrap()).unwrap();
                if merged_at > past {
                    merged += 1;
                }
            }
        }
    }
    println!("PRs merged in past {} days: {}", days, merged);
}
