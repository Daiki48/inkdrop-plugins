use clap::{command, Parser};
use serde::{Deserialize, Serialize};
use futures::stream::{FuturesUnordered, StreamExt};

use std::collections::VecDeque;
use std::time::Instant;

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

async fn get_total_pages() -> Result<u32, reqwest::Error> {
    let mut total_pages: u32 = 0;
    let futures = FuturesUnordered::new();

    loop {
        let url: String = format!("{}?page={}&sort=majority", INKDROP_PLUGINS_API_URL, total_pages);
        futures.push(tokio::task::spawn({
            let url = url.clone();
            async move {
                let response = reqwest::get(&url).await;
                match response {
                    Ok(res) => {
                        let packages: Result<Vec<Package>, reqwest::Error> = res.json().await;
                        Ok::<_, reqwest::Error>(packages)
                    },
                    Err(e) => {
                        eprintln!("Request error in get_total_pages: {:?}", e);
                        Err(e)
                    }
                }
            }
        }));
        total_pages += 1;
        // ここで50と決め打ちしてtotal_pagesを50にしているから7秒ぐらいで処理が完了する
        // ただし、無駄なページを抽出対象にしているため、実装を改めないといけない
        if futures.len() >= 50 {
            break;
        }
    }
    // while let Some(result) = futures.next().await {
    //     match result {
    //         Ok(response) => {
    //             if response.is_empty() {
    //                 total_pages -= 1;
    //                 break;
    //             }
    //         },
    //         Err(e) => eprintln!("Request error: {:?}", e),
    //     }
    // }
    Ok(total_pages)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        let start = Instant::now();

        let total_pages: u32 = get_total_pages().await?;
        println!("total pages: {}", &total_pages);

        let mut plugins: VecDeque<String> = VecDeque::new();
        let mut futures = FuturesUnordered::new();

        for page in 0..total_pages {
            let url: String = format!("{}?page={}&sort=majority", INKDROP_PLUGINS_API_URL, page);
            futures.push(tokio::task::spawn({

                let url = url.clone();
                async move {
                    let response: Vec<Package> = reqwest::get(&url).await?.json().await?;
                    Ok::<_, reqwest::Error>(response)
                }
            }));
        }

        while let Some(result) = futures.next().await {
            match result {
                Ok(response) => {
                    for package in response? {
                        plugins.push_back(package.name);
                    }
                },
                Err(e) => eprintln!("Request error: {:?}", e),
            }
        }

        println!("Result");
        for plugin in plugins {
            println!("- {}", plugin);
        }
        let duration = start.elapsed();
        println!("\nExecution time: {:?}", duration);
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
