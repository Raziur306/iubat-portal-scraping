use anyhow::{Context, Ok, Result};
use dotenv::dotenv;
use headless_chrome::{protocol::cdp::Page, Browser, Tab};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub struct BrowserTab {
    pub tab: Arc<Tab>,
}

impl BrowserTab {
    pub async fn new() -> Result<Self> {
        let browser = Browser::default()?;
        let browser = Arc::new(browser);
        let tab = browser.new_tab()?;
        let base_url = std::env::var("BASE_URL").expect("BASE_URL must be set");
        tab.navigate_to(&base_url)?;
        tab.wait_until_navigated()?;

        Ok(Self { tab })
    }

    pub async fn login(&self, student_login: &str, password: &str) -> Result<(String, bool)> {
        println!("Trying to login...");
        let dashboard_url = std::env::var("DASHBOARD_URL").expect("DASHBOARD_URL must be set");

        if self.tab.get_url() != dashboard_url {
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

        let mut retries = 10;
        while retries > 0 {
            sleep(Duration::from_secs(1)).await;
            if self.tab.get_url() == dashboard_url {
                return Ok(("Successfully logged in".to_string(), true));
            }
            retries -= 1;
        }

        Ok(("Failed to login".to_string(), false))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let browser_tab = BrowserTab::new()
        .await
        .context("Failed to create BrowserTab instance")?;
    let login_response = browser_tab.login("test", "test").await?;



    Ok(())
}
