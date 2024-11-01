use clap::{command, Parser};
use headless_chrome::{Browser, LaunchOptions, Tab};
use scraper::{Html, Selector};

use std::collections::VecDeque;
use std::time::Duration;

use tokio::time::sleep;

const LONG_ABOUT: &str = r#"
long long about
"#;

const INKDROP_PLUGINS_URL: &str = "https://my.inkdrop.app/plugins";
// const INKDROP_PLUGINS: &str = "#app-container > div > div > div.ui.stackable.grid > div.ten.wide.column > div:nth-child(2) > div";
const INKDROP_PLUGIN_NAME: &str = "div.ui.segment.U2Wr > a > h2 > div";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = LONG_ABOUT)]
struct Args {
    #[arg(short, long)]
    list: bool,
}

async fn scroll_to_bottom(tab: &Tab) -> Result<usize, Box<dyn std::error::Error>> {
    tab.evaluate("window.scrollBy(0, window.innerHeight);", false)?;
    sleep(Duration::from_secs(10)).await;

    let height = tab.evaluate("document.body.scrollHeight", false)?.value.unwrap().as_u64().unwrap() as usize;
    Ok(height)
}

async fn check_new_plugins(tab: &Tab, _previous_plugin_count: usize) -> Result<usize, Box<dyn std::error::Error>> {
    let content = tab.get_content()?;
    let document = Html::parse_document(&content);
    let plugin_selector = Selector::parse(INKDROP_PLUGIN_NAME).unwrap();
    let plugin_count = document.select(&plugin_selector).count();
    Ok(plugin_count)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        let browser = Browser::new(LaunchOptions::default())?;
        let tab = browser.new_tab()?;
        tab.navigate_to(INKDROP_PLUGINS_URL)?
            .wait_until_navigated()?;

        let mut plugins: VecDeque<String> = VecDeque::new();
        let mut previous_plugin_count = 0;

        loop {
            let new_height = scroll_to_bottom(&tab).await?;
            sleep(Duration::from_secs(10)).await;
            let current_plugin_count = check_new_plugins(&tab, previous_plugin_count).await?;
            println!("current height: {}, Loaded plugins: {}", new_height, current_plugin_count);
            if current_plugin_count == previous_plugin_count {
                break;
            }
            previous_plugin_count = current_plugin_count;
        }
    
        let content = tab.get_content()?;
        let document = Html::parse_document(&content);

        let plugin_selector = Selector::parse(INKDROP_PLUGIN_NAME).unwrap();

        let plugin_elements = document.select(&plugin_selector);

        for plugin_element in plugin_elements {
            let plugin_name: String = plugin_element
                .children()
                .filter(|child| child.value().is_text())
                .map(|child| child.value().as_text().unwrap().trim())
                .collect::<Vec<_>>()
                .concat();

            plugins.push_back(plugin_name);
        }
        println!("plugins : {:?}", &plugins);
        println!("Result");
        for plugin in plugins {
            println!("- {}", plugin);
        }
    }
    Ok(())
}
