use chrono::prelude::*;
use chrono::{DateTime, Duration};
use github_stats::{Query, Search};
use serde_json::value::Value;

fn get_all_items(mut search: Search) -> Vec<Value> {
    let mut items = Vec::new();
    let mut results = search.search().unwrap();
    let mut remaining_results = results.total_count();
    println!("results = {}", remaining_results);
    while remaining_results > 0 {
        for item in results.items() {
            items.push(item.clone());
            remaining_results -= 1;
        }
        search.next_page();
        results = search.search().unwrap();
    }
    items
}

fn main() {
    let now = Utc::now();
    let past = now - Duration::days(14);

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
    println!("Issues opened in past 2 weeks: {}", opened);

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
    println!("Issues closed in past 2 weeks: {}", closed);

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
    println!("PRs merged in past 2 weeks: {}", merged);
}
