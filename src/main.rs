use clap::{command, Parser};
use serde::{Deserialize, Serialize};

use std::collections::VecDeque;

const INKDROP_PLUGINS_API_URL: &str = "https://api.inkdrop.app/v1/packages";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Print inkdrop plugin list
    #[arg(short, long)]
    list: bool,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;

    #[tokio::test]
    async fn test_fetch_plugins() -> Result<(), reqwest::Error> {
        let server = MockServer::start();

        let plugins = vec![
            Package { name: "plugin1".to_string() },
            Package { name: "plugin2".to_string() },
        ];

        let plugins_json = serde_json::to_string(&plugins).unwrap();
        
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/packages")
                .query_param("page", "0")
                .query_param("sort", "majority");
            then.status(200)
                .header("content-type", "application/json")
                .body(plugins_json);
        });

        let url = format!("{}/v1/packages?page=0&sort=majority", server.base_url());
        let response : Vec<Package> = reqwest::get(&url).await?.json().await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].name, "plugin1");
        assert_eq!(response[1].name, "plugin2");

        mock.assert();
        Ok(())
    }
}
