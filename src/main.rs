use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

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

    let _html = page.wait_for_navigation().await?.content().await?;

    handle.await;
    Ok(())
}