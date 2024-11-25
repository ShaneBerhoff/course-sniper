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