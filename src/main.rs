use chromiumoxide::{Browser, BrowserConfig};
use clap::Parser;
use futures::StreamExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod args;
use args::SniperArgs;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = SniperArgs::parse();
    println!("Args: {:?}", cli_args);

    let (mut browser, mut handler) = if cli_args.detached {
        Browser::launch(BrowserConfig::builder().build()?).await?
    } else {
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = async_std::task::spawn(async move {
        while running_clone.load(Ordering::Relaxed) {
            if let Some(event) = handler.next().await {
                let _ = event;
            }
        }
    });

    let page = browser.new_page("https://wikipedia.org").await?;

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

    // cleanup
    browser.close().await?;
    browser.try_wait()?;
    running.store(false, Ordering::Relaxed);
    handle.await;
    Ok(())
}
