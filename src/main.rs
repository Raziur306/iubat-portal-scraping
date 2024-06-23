use anyhow::{bail, Context, Ok, Result};
use dotenv::dotenv;
use headless_chrome::{Browser, Tab};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub struct BrowserTab {
    pub tab: Arc<Tab>,
    pub browser: Arc<Browser>,
}

pub struct ProfileData {
    profile_img: String,
    address: String,
    student_info: Vec<String>,
}

impl BrowserTab {
    pub async fn new() -> Result<Self> {
        let browser = Browser::default()?;
        let browser = Arc::new(browser);
        let tab: Arc<Tab> = browser.new_tab()?;
        let base_url = std::env::var("BASE_URL").expect("BASE_URL must be set");
        tab.navigate_to(&base_url)?;
        tab.wait_until_navigated()?;

        Ok(Self { tab, browser })
    }

    pub async fn login(&self, student_login: &str, password: &str) -> Result<(String, bool)> {
        println!("Trying to login...");
        let dashboard_url = std::env::var("DASHBOARD_URL").expect("DASHBOARD_URL must be set");

        if self.tab.get_url() == dashboard_url {
            return Ok(("Already logged in".to_string(), true));
        }

        self.tab
            .wait_for_element("input[name='txtstudent_userid']")
            .context("Failed to find student login input")?
            .click()
            .context("Failed to click on student login input")?;

        self.tab
            .type_str(student_login)
            .context("Failed to type student login")?;

        self.tab
            .wait_for_element("input[name='txtstdpassword']")
            .context("Failed to find password input")?
            .click()
            .context("Failed to click on password input")?;

        self.tab
            .type_str(password)
            .context("Failed to type password")?;

        self.tab.wait_for_element("button")?.click()?;

        let mut retries = 15;
        while retries > 0 {
            sleep(Duration::from_secs(1)).await;
            if self.tab.get_url() == dashboard_url {
                return Ok(("Successfully logged in".to_string(), true));
            }
            retries -= 1;
        }

        Ok(("Failed to login".to_string(), false))
    }

    pub async fn get_profile_info(&self) -> Result<ProfileData> {
        let profile_url = std::env::var("STUDENT_PROFILE_URL").expect("PROFILE_URL must be set");
        let profile_tab = self.browser.new_tab()?;
        profile_tab.navigate_to(&profile_url)?;

        let mut retries = 10;
        while retries > 0 {
            sleep(Duration::from_secs(1)).await;
            if profile_tab.get_url() == profile_url {
                break;
            }
            retries -= 1;
        }

        if profile_tab.get_url() != profile_url {
            bail!("Failed to navigate to profile page");
        }

        let profile_img = profile_tab
            .find_element("img")?
            .get_attribute_value("src")?
            .unwrap_or_default();

        let address = profile_tab
            .find_element("div.panel-body")?
            .get_inner_text()?;

        let mut student_info: Vec<String> = Vec::new();
        let table_rows = profile_tab.find_elements("tr")?;
        for row in table_rows {
            let tds = row.wait_for_elements("td").unwrap();
            if let Some(last_td) = tds.last() {
                let text = last_td.get_inner_text()?;
                println!("{:?}", text);
                student_info.push(text.to_string());
            }
        }

        if student_info.is_empty() {
            bail!("Student info not found.")
        }

        return Ok(ProfileData {
            profile_img,
            address,
            student_info,
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let browser_tab = BrowserTab::new()
        .await
        .context("Failed to create BrowserTab instance")?;
    let login_response = browser_tab.login("test", "test").await?;
    print!("{:?}", login_response.0);
    let profile_data = browser_tab.get_profile_info().await?;
    println!("{:?}", profile_data.profile_img);
    println!("{:?}", profile_data.address);
    println!("{:?}", profile_data.student_info.get(0).unwrap());
    println!("{:?}", profile_data.student_info.get(1).unwrap());
    println!("{:?}", profile_data.student_info.get(2).unwrap());
    Ok(())
}
