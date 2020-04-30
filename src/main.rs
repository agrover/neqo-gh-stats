use std::cmp::min;
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

    #[structopt(short = "v", long)]
    verbose: bool,
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

    //let mut remaining_results = results.total_count();
    let mut remaining_results = min(results.total_count(), 100);
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

// prints stuff and returns how many things
fn print_things(
    args: &Args,
    things: &[Value],
    after: &DateTime<Utc>,
    date_field_filter: &str,
) -> usize {
    let mut matching = 0;
    for item in things {
        if let Some(date_item) = item.get(date_field_filter) {
            if date_item.is_string() {
                let date_item = DateTime::parse_from_rfc3339(date_item.as_str().unwrap()).unwrap();
                if date_item > *after {
                    matching += 1;

                    if args.verbose {
                        println!(
                            "* [{}]({})",
                            item.get("title").unwrap().as_str().unwrap(),
                            item.get("html_url").unwrap().as_str().unwrap()
                        );
                    }
                }
            }
        }
    }

    matching
}

fn main() {
    let args = Args::from_args();
    let days = args.days.try_into().unwrap();

    let now = Utc::now();
    let past = now - Duration::days(days);

    // Issues created/closed
    let search = Search::issues(&Query::new().repo("mozilla", "neqo").is("issue")).per_page(100);

    let issues = get_all_items(search);

    let found = print_things(&args, &issues, &past, "created_at");
    println!("Issues opened in past {} days: {}", days, found);

    let found = print_things(&args, &issues, &past, "closed_at");
    println!("Issues closed in past {} days: {}", days, found);

    // PRs merged
    let search =
        Search::issues(&Query::new().repo("mozilla", "neqo").is("pr").is("merged")).per_page(100);
    let prs = get_all_items(search);
    let found = print_things(&args, &prs, &past, "closed_at");
    println!("PRs merged in past {} days: {}", days, found);
}
