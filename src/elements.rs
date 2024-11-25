use chromiumoxide::{error::CdpError, Element, Page};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmoryPageElements {
    pub page_url: &'static str,
    pub username_input: &'static str,
    pub passwd_input: &'static str,
    pub validate_button: &'static str,
    pub enroll_button: &'static str,
    pub semester_cart: &'static str,
    pub course_row: &'static str,
    pub checkboxes: &'static str,
    pub availability: &'static str,
    pub description: &'static str,
    pub schedule: &'static str,
    pub room: &'static str,
    pub instructor: &'static str,
    pub credits: &'static str,
    pub seats: &'static str,
}

impl Default for EmoryPageElements {
    fn default() -> Self {
        Self {
            page_url: "https://saprod.emory.edu/psc/saprod_48/EMPLOYEE/SA/c/SSR_STUDENT_FL.SSR_SHOP_CART_FL.GBL",
            username_input: "input#userid",
            passwd_input: "input#pwd",
            validate_button: "a#DERIVED_SSR_FL_SSR_VALIDATE_FL",
            enroll_button: "a#DERIVED_SSR_FL_SSR_ENROLL_FL",
            semester_cart: r#"a[id^="SSR_CART_TRM_FL_TERM_DESCR30$"]"#,
            course_row: r#"tr[id^="SSR_REGFORM_VW$0_row_"]"#,
            checkboxes: r#"input[type="checkbox"][id^="DERIVED_REGFRM1_SSR_SELECT$"]"#,
            availability: r#"span[id^="DERIVED_SSR_FL_SSR_AVAIL_FL$"]"#,
            description: r#"span[id^="DERIVED_SSR_FL_SSR_DESCR80$"]"#,
            schedule: r#"span[id^="DERIVED_REGFRM1_SSR_MTG_SCHED_LONG$"]"#,
            room: r#"span[id^="DERIVED_REGFRM1_SSR_MTG_LOC_LONG$"]"#,
            instructor: r#"span[id^="DERIVED_REGFRM1_SSR_INSTR_LONG$"]"#,
            credits: r#"span[id^="DERIVED_SSR_FL_SSR_UNITS_LBL$"]"#,
            seats: r#"span[id^="DERIVED_SSR_FL_SSR_DESCR50$"]"#,
        }
    }
}

pub struct ShoppingCart {
    pub element: Element,
    pub text: String,
}

impl fmt::Display for ShoppingCart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CourseStatus {
    Waitlist,
    Open { available: u32, capacity: u32 },
    Closed,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Course {
    pub checkbox_element: Element,
    pub availability: CourseStatus,
    pub description: String,
    pub schedule: String,
    pub room: String,
    pub instructor: String,
    pub credits: String,
}

impl EmoryPageElements {
    pub async fn get_shopping_carts(&self, page: &Page) -> Result<Vec<ShoppingCart>, CdpError> {
        let semester_cart_elements = page.find_elements(self.semester_cart).await?;
        let semester_carts: Vec<ShoppingCart> =
            futures::future::join_all(semester_cart_elements.into_iter().map(|cart| async move {
                let text = cart.inner_text().await.unwrap().expect("test");
                ShoppingCart {
                    element: cart,
                    text,
                }
            }))
            .await;
        Ok(semester_carts)
    }

    pub async fn get_cart_courses(&self, page: &Page) -> Result<Vec<Course>, CdpError> {
        let course_row_elements = page.find_elements(self.course_row).await?;
        let courses: Vec<Course> =
            futures::future::try_join_all(course_row_elements.into_iter().map(|row| async move {
                let course_status = match row
                    .find_element(self.availability)
                    .await?
                    .inner_text()
                    .await?
                    .unwrap_or("".to_string())
                {
                    text if text.contains("Wait List") => CourseStatus::Waitlist,
                    text if text.contains("Closed") => CourseStatus::Closed,
                    text if text.contains("Open") => CourseStatus::Open {
                        available: 1,
                        capacity: 10,
                    },
                    _ => CourseStatus::Closed,
                };

                Ok::<Course, CdpError>(Course {
                    checkbox_element: row.find_element(self.checkboxes).await?,
                    availability: course_status,
                    description: row
                        .find_element(self.schedule)
                        .await?
                        .inner_text()
                        .await?
                        .unwrap_or("None".to_string()),
                    schedule: row
                        .find_element(self.schedule)
                        .await?
                        .inner_text()
                        .await?
                        .unwrap_or("None".to_string()),
                    instructor: row
                        .find_element(self.instructor)
                        .await?
                        .inner_text()
                        .await?
                        .unwrap_or("None".to_string()),
                    room: row
                        .find_element(self.room)
                        .await?
                        .inner_text()
                        .await?
                        .unwrap_or("None".to_string()),
                    credits: row
                        .find_element(self.credits)
                        .await?
                        .inner_text()
                        .await?
                        .unwrap_or("None".to_string()),
                })
            }))
            .await?;

        Ok(courses)
    }
}
