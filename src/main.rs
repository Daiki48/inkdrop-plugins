use clap::{command, Parser};
use serde::Deserialize;

use std::collections::VecDeque;

const INKDROP_PLUGINS_API_URL: &str = "https://api.inkdrop.app/v1/packages";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Print inkdrop plugin list
    #[arg(short, long)]
    list: bool,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        let mut plugins: VecDeque<String> = VecDeque::new();
        let mut page: u32 = 0;

        loop {
            let url: String = format!("{}?page={}&sort=majority", INKDROP_PLUGINS_API_URL, page);

            let response: Vec<Package> = reqwest::get(&url).await?.json().await?;

            if response.is_empty() {
                break;
            }

            for package in response {
                plugins.push_back(package.name);
            }
            page += 1;
        }
        println!("Result");
        for plugin in plugins {
            println!("- {}", plugin);
        }
    }
    Ok(())
}
