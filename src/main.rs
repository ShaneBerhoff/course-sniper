use chromiumoxide::error::CdpError;
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::{Browser, BrowserConfig, Element, Page};
use chrono::{Local, Timelike};
use clap::Parser;
use core::fmt;
use elements::{EmoryPageElements, ToTable};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{MultiSelect, Password, PasswordDisplayMode, Select, Text};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod args;
use args::SniperArgs;

mod elements;

const TIMEOUT: u64 = 10;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get args
    let cli_args = SniperArgs::parse();

    let pb = get_progress_bar("Enabling browser...".to_string());

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
    pb.finish_with_message("Browser enabled.");

    // page elements
    let elements = elements::EmoryPageElements::default();

    let page = browser.new_page(elements.page_url).await?;
    page.enable_stealth_mode().await?;

    match run(&page, elements).await {
        Ok(_) => (),
        Err(e) => {
            page.save_screenshot(
                ScreenshotParams::builder().full_page(true).build(),
                format!(
                    "debug-{}.png",
                    Local::now().format("%H:%M:%S.%3f").to_string()
                ),
            )
            .await?;
            Err(e)?
        }
    }

    // cleanup
    browser.close().await?;
    browser.try_wait()?;
    running.store(false, Ordering::Relaxed);
    handle.await;
    Ok(())
}

async fn run(page: &Page, elements: EmoryPageElements) -> Result<(), Box<dyn std::error::Error>> {
    // login info
    let user_name = Text::new("Username: ").prompt()?;
    let user_pwd = Password::new("Password: ")
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt()?;

    let pb = get_progress_bar("Logging in with credentials...".to_string());

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
    match wait_element_agressive_retry(&page, elements.semester_cart, TIMEOUT).await {
        Ok(_) => pb.finish_with_message("Authenticated."),
        Err(e) => {
            pb.finish_with_message("Invalid credentials.");
            Err(e)?
        }
    }
    let carts = elements.get_shopping_carts(&page).await?;
    let selected_cart = Select::new("Select a cart", carts).prompt()?;
    selected_cart.element.click().await?;

    // get course info
    let pb = get_progress_bar("Fetching courses in cart...".to_string());
    wait_element_agressive_retry(&page, elements.course_row, TIMEOUT).await?;
    let courses = elements.get_cart_courses(&page).await?;
    pb.finish_with_message(format!("Found {} courses", courses.len()));
    println!("{}", courses.to_table());

    // pick courses
    let selected_courses = MultiSelect::new("Select courses", courses).prompt()?;

    //TODO improve registration time selection and implimentation
    let registration_times: Vec<RegistrationTime> = (1..=12)
        .flat_map(|hour| {
            (0..60).flat_map(move |minute| {
                [true, false]
                    .iter()
                    .map(move |&am| RegistrationTime(hour, minute, am))
            })
        })
        .collect();
    let registration_time = Select::new("Select registration time", registration_times).prompt()?;
    let pb = get_progress_bar(format!("Waiting for registration time: {registration_time}..."));
    let registration_hour = if registration_time.2 {
        registration_time.0
    } else {
        registration_time.0 + 12
    };
    loop {
        let now = Local::now();
        if now.hour() == registration_hour && now.minute() == registration_time.1 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    pb.finish_with_message("Reloading for registration.");

    page.reload().await?.wait_for_navigation().await?;

    let pb = get_progress_bar("Selecting courses...".to_string());
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
    pb.finish_with_message("Courses selected.");

    // validate
    wait_element_agressive_retry(&page, elements.validate_button, TIMEOUT)
        .await?
        .click()
        .await?;

    println!(
        "Validation clicked at {}",
        Local::now().format("%H:%M:%S.%3f").to_string()
    );
    // results
    let pb = get_progress_bar("Waiting for results...".to_string());
    wait_element_agressive_retry(&page, elements.results_rows, TIMEOUT).await?;
    let registration_results = elements.get_registration_results(&page).await?;
    pb.finish_with_message(format!("Found {} results", registration_results.len()));
    println!("{}", registration_results.to_table());

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

struct RegistrationTime(u32, u32, bool);

impl fmt::Display for RegistrationTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02} {}",
            self.0,
            self.1,
            if self.2 { "AM" } else { "PM" }
        )
    }
}

fn get_progress_bar(msg: String) -> ProgressBar{
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message(msg);
    pb
}