use async_std::task::sleep;
use chromiumoxide::error::CdpError;
use chromiumoxide::{Browser, BrowserConfig, Element, Page};
use clap::Parser;
use futures::StreamExt;
use inquire::{Password, PasswordDisplayMode, Select, Text};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod args;
use args::SniperArgs;

mod elements;

const TIMEOUT: u64 = 10;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get args
    let cli_args = SniperArgs::parse();

    // login info
    let user_name = Text::new("Username: ").prompt()?;
    let user_pwd = Password::new("Password: ")
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt()?;

    // setup browser
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

    browser.clear_cookies().await?;

    // page elements
    let elements = elements::EmoryPageElements::default();

    let page = browser.new_page(elements.page_url).await?;

    // login
    page.wait_for_navigation()
        .await?
        .find_element(elements.username_input)
        .await?
        .click()
        .await?
        .type_str(user_name)
        .await?;
    page.find_element(elements.passwd_input)
        .await?
        .click()
        .await?
        .type_str(user_pwd)
        .await?
        .press_key("Enter")
        .await?;

    // pick a shopping cart
    wait_element_agressive_retry(&page, elements.semester_cart, TIMEOUT).await?;
    let carts = elements.get_shopping_carts(&page).await?;
    let selected_cart = Select::new("Select a cart", carts).prompt()?;
    selected_cart.element.click().await?;

    // all checkboxes
    wait_element_agressive_retry(&page, elements.checkboxes, TIMEOUT).await?;
    let courses = elements.get_cart_courses(&page).await?;
    println!("{:?}", courses);

    for course in courses {
        course.checkbox_element.click().await?;
    }

    // validate
    wait_element_agressive_retry(&page, elements.validate_button, TIMEOUT).await?;
    page.find_element(elements.validate_button)
        .await?
        .click()
        .await?;

    sleep(Duration::new(10, 0)).await;
    // cleanup
    browser.close().await?;
    browser.try_wait()?;
    running.store(false, Ordering::Relaxed);
    handle.await;
    Ok(())
}

async fn wait_element_agressive_retry(
    page: &Page,
    selector: &str,
    wait_time: u64,
) -> Result<Element, CdpError> {
    let start = Instant::now();
    let wait_time = Duration::new(wait_time, 0);
    loop {
        match page.find_element(selector).await {
            Ok(element) => return Ok(element),
            Err(e) => {
                if start.elapsed() < wait_time {
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}
