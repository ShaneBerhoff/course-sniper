use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
mod args;
use args::SniperArgs;
use clap::Parser;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = SniperArgs::parse();
    println!("Args: {:?}", cli_args);

    let (browser, mut handler) = 
        if cli_args.detached {
            Browser::launch(BrowserConfig::builder().build()?).await?
        } else {
            Browser::launch(BrowserConfig::builder().with_head().build()?).await?
        };

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    let page = browser.new_page("https://wikipedia.org").await?;

    // type into the search field and hit `Enter`,
    // this triggers a navigation to the search result page
    page.find_element("input#searchInput")
            .await?
            .click()
            .await?
            .type_str("Rust programming language")
            .await?
            .press_key("Enter")
            .await?;

    let html = page.wait_for_navigation().await?.content().await?;
    println!("html contains rust: {}", html.contains("rust"));

    handle.await;
    Ok(())
}