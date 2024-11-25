use async_std::task::sleep;
use chromiumoxide::error::CdpError;
use chromiumoxide::{Browser, BrowserConfig, Element, Page};
use clap::Parser;
use elements::ToTable;
use futures::StreamExt;
use inquire::{MultiSelect, Password, PasswordDisplayMode, Select, Text};
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
    page.enable_stealth_mode().await?;

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

    // get course info
    wait_element_agressive_retry(&page, elements.course_row, TIMEOUT).await?;
    let courses = elements.get_cart_courses(&page).await?;
    println!("All Courses");
    println!("{}", courses.to_table());

    // pick courses
    let selected_courses = MultiSelect::new("Select courses", courses).prompt()?;
    println!("{:?}", selected_courses[0].checkbox_index);

    sleep(Duration::new(10, 0)).await;

    page.reload().await?.wait_for_navigation().await?;

    for (index, checkbox) in wait_elements_agressive_retry(&page, elements.checkboxes, TIMEOUT)
        .await?
        .into_iter()
        .enumerate()
    {
        if selected_courses
            .iter()
            .any(|course| course.checkbox_index == index as u8)
        {
            checkbox.click().await?;
        }
    }

    // validate
    wait_element_agressive_retry(&page, elements.validate_button, TIMEOUT)
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

async fn wait_elements_agressive_retry(
    page: &Page,
    selector: &str,
    wait_time: u64,
) -> Result<Vec<Element>, CdpError> {
    let start = Instant::now();
    let wait_time = Duration::new(wait_time, 0);
    loop {
        match page.find_elements(selector).await {
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
