use std::io;
use std::sync::Arc;
use std::path::Path;
use std::time::Duration;
use anyhow::{ anyhow, bail };
use serde_json::{ json, Map, Value };

use thirtyfour::prelude::*;
use crate::env::{ USER, PASS, HOMEPAGE, Open };
use crate::env::{ USER_CSS, PASS_CSS, REMEMBER_CSS, LOGIN_BTN_CSS, SIGN_IN_BTN_CSS };

type JMap = serde_json::Map<String, Value>;

pub async fn run(driver: &WebDriver) -> anyhow::Result<()> {
    let c = Competitions::new(driver);
    c.manage_login().await?;
    let comps = Value::from(c.get_competitions().await?);
    let comp_path = Path::new("./competitions.json");
    let _ = std::fs::write(comp_path, serde_json::to_string_pretty(&comps)?);
    Ok(())
}

#[derive(Debug, Clone)]
struct Competitions<'a> {
    driver: &'a WebDriver,
}

impl<'a> Competitions<'a> {
    fn new(driver: &'a WebDriver) -> Self {
        Competitions {
            driver,
        }
    }

    async fn manage_login(&self) -> anyhow::Result<()> {
        let is_logged_in =
            self.try_cookie_login().await.is_ok() || self.try_raw_login().await.is_ok();
        if !is_logged_in {
            println!("Login Failed");
            bail!("Login Failed");
        }
        println!("Login Successful");
        Ok(())
    }

    async fn is_logged_in(&self) -> anyhow::Result<()> {
        self.driver.goto(HOMEPAGE).await?;
        let login = self.query_login_btn().await?;
        login.click().await?;

        let driver = Arc::new(self.driver.clone_with_config(self.driver.config().clone()));
        let login = self.query_login_btn().await?;
        login
            .wait_until()
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .condition(
                Box::new(move |_| {
                    let driver = driver.clone();
                    Box::pin(async move {
                        if let Ok(elem) = driver.find(By::Css(LOGIN_BTN_CSS)).await {
                            if let Ok(text) = elem.text().await {
                                return Ok(text.to_lowercase() != "login");
                            }
                        }
                        Ok(false)
                    })
                })
            ).await?;

        // Below actions would only run if the condition is fulfilled
        // i.e. logged in successfully
        Ok(())
    }

    async fn try_cookie_login(&self) -> anyhow::Result<()> {
        self.driver.goto("https://www.worldcubeassociation.org/").await?;
        self.load_cookies().await?;
        self.is_logged_in().await?;
        Ok(())
    }

    async fn try_raw_login(&self) -> anyhow::Result<()> {
        let user = USER.open();
        let passw = PASS.open();

        self.driver.goto(HOMEPAGE).await?;
        self.driver
            .query(By::Css(LOGIN_BTN_CSS))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?
            .click().await?;

        self.driver
            .query(By::Css(USER_CSS))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?
            .send_keys(user).await?;

        self.driver
            .query(By::Css(PASS_CSS))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?
            .send_keys(passw).await?;

        self.driver
            .query(By::Css(REMEMBER_CSS))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?
            .click().await?;

        self.driver
            .query(By::Css(SIGN_IN_BTN_CSS))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?
            .click().await?;

        self.driver.goto("https://www.worldcubeassociation.org/").await?;
        let _ = self.save_cookies().await;
        self.is_logged_in().await?;
        Ok(())
    }

    async fn load_cookies(&self) -> anyhow::Result<()> {
        let cookie_path = Path::new("./cookies.json");
        let cookie_data = std::fs::read_to_string(cookie_path)?;
        let cookie_data: Value = serde_json::from_str(&cookie_data)?;
        let user_data = cookie_data["user_data"].as_str().unwrap_or("");

        if !self.check_if_last_user(user_data).map_or(false, |_| true) {
            bail!("stored cookies and user authenticating are both different");
        }

        let cookies = cookie_data["cookies"].as_array().ok_or(anyhow!("empty cookie array"))?;
        for cookie in cookies {
            let cookie: Cookie = serde_json::from_value(cookie.clone())?;
            self.driver.add_cookie(cookie).await?;
        }
        let _ = self.driver.refresh().await;
        Ok(())
    }

    async fn save_cookies(&self) -> anyhow::Result<()> {
        let user_data = hex::encode(format!("{}++==++{}", USER.open(), PASS.open()));
        let cookies = self.driver.get_all_cookies().await?;
        let cookie_path = Path::new("./cookies.json");
        let cookie_data =
            json!({
                "user_data": user_data,
                "cookies": cookies,
            });
        let json_str = serde_json::to_string_pretty(&cookie_data)?;
        let _ = std::fs::write(cookie_path, json_str);
        Ok(())
    }

    fn check_if_last_user(&self, user_data: &str) -> anyhow::Result<()> {
        let err = io::Error::from(io::ErrorKind::InvalidData);
        let decoded_data = hex::decode(user_data)?;
        let decoded_data = decoded_data.as_slice();
        let decoded_string = std::str::from_utf8(&decoded_data)?;
        if decoded_data.is_empty() {
            bail!(err);
        }
        let mut iter = decoded_string.split("++==++").into_iter();
        if let Some(user) = iter.next() {
            if let Some(passw) = iter.next() {
                if user == USER.open() && passw == PASS.open() {
                    return Ok(());
                }
            }
        }
        bail!(io::Error::new(io::ErrorKind::Other, "new user"))
    }

    async fn query_login_btn(&self) -> anyhow::Result<WebElement> {
        let login = self.driver
            .query(By::Css(LOGIN_BTN_CSS))
            .wait(Duration::from_secs(5), Duration::from_millis(500))
            .first().await?;
        Ok(login)
    }

    async fn get_competitions(&self) -> anyhow::Result<Vec<JMap>> {
        let upcoming_comps = self.driver
            .query(By::Css("#root > div > main > div > div > div.text-center.text-gray-500"))
            .wait(Duration::from_secs(5), Duration::from_millis(500))
            .first().await;
        if let Ok(upcoming_comps) = upcoming_comps {
            if let Ok(text) = upcoming_comps.text().await {
                println!("Your Upcoming competitions: {}", text);
                bail!(text);
            }
        }

        let boxx = self.driver.find(By::Css("#root > div > main > div > div > ul")).await?;
        boxx
            .wait_until()
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .displayed().await?;

        let mut competitions = Vec::new();

        for row in boxx.find_all(By::Css("a")).await? {
            let comp_page = row.attr("href").await?.map(|url| self.url_join(HOMEPAGE, &url));
            let mut competition_data: Map<String, Value> = Map::new();

            let _d_competition_name = row
                .find(By::Css("li > div.flex-1 > p.font-normal.leading-1")).await?
                .text().await?;
            let _d_city_and_date = row
                .find(By::Css("li > div.flex-1 > p.text-gray-600.text-sm.leading-1")).await?
                .text().await?;

            let _at_data = _d_city_and_date
                .split(" – ")
                .map(|s| s.trim())
                .collect::<Vec<_>>();
            let competition = _d_competition_name.trim();
            let [date, city] = _at_data.as_slice() else {
                bail!("unknown competition's case; not implemented yet!")
            };

            competition_data.insert("page".to_string(), Value::from(comp_page));
            competition_data.insert("competition".to_string(), Value::from(competition));
            competition_data.insert("date".to_string(), Value::from(*date));
            competition_data.insert("city".to_string(), Value::from(*city));
            competitions.push(competition_data);
        }
        Ok(self.crawl_and_get_assignments(competitions).await)
    }

    fn url_join<S: AsRef<str>>(&self, parent_url: S, child_url: S) -> String {
        let parent = parent_url.as_ref().trim_end_matches("/");
        let child = child_url.as_ref().trim_start_matches("/");
        format!("{parent}/{child}")
    }

    async fn crawl_and_get_assignments(&self, competitions: Vec<JMap>) -> Vec<JMap> {
        let mut updated_competitions = Vec::new();
        for mut comp in competitions {
            let competition_page = comp["page"].as_str().unwrap();
            let assigns = self
                .check_my_assignments(competition_page).await
                .map_err(|e|
                    println!("My assignments error: (page: {}) -> {:?}", competition_page, e)
                )
                .unwrap_or_default();
            comp.insert("assigns".to_string(), Value::from(assigns));
            updated_competitions.push(comp);
        }
        updated_competitions
    }

    async fn check_my_assignments<S: Into<String>>(
        &self,
        competition_page: S
    ) -> anyhow::Result<JMap> {
        self.driver.goto(competition_page).await?;
        let my_assigns = self.driver
            .query(
                By::Css(
                    "#root > div > main > div > div > div.flex.flex-col.w-full.lg\\:w-1\\/2.md\\:w-2\\/3.divide-y-2 > div > a"
                )
            )
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?;
        let assign_page = my_assigns
            .attr("href").await?
            .map(|url| self.url_join(HOMEPAGE, &url))
            .unwrap();

        // More detailed data
        // Not required for now
        // let assignments = self.get_my_assignments(assign_page.clone()).await?;
        let mut assign_data: Map<String, Value> = Map::new();
        assign_data.insert("assign_page".to_string(), Value::from(assign_page));
        // assign_data.insert("assignments".to_string(), Value::from(assignments));
        Ok(assign_data)
    }

    #[allow(dead_code)]
    async fn get_my_assignments(&self, assign_page: String) -> anyhow::Result<JMap> {
        self.driver.goto(assign_page).await?;
        self.driver
            .query(
                By::Css(
                    "#root > div > main > div > div > div.flex.flex-col.w-full.lg\\:w-1\\/2.md\\:w-2\\/3 > div > div.p-1 > div > div > h3"
                )
            )
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .first().await?;

        let assigns = self.driver.find(
            By::Css(
                "#root > div > main > div > div > div.flex.flex-col.w-full.lg\\:w-1\\/2.md\\:w-2\\/3 > div > div:nth-child(5)"
            )
        ).await?;
        assigns
            .wait_until()
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .displayed().await?;
        let assigns_text = assigns.text().await?;
        if assigns_text.to_lowercase().contains("no assignments") {
            bail!(assigns_text);
        }

        let tbody = self.driver.find(
            By::Css(
                "#root > div > main > div > div > div.flex.flex-col.w-full.lg\\:w-1\\/2.md\\:w-2\\/3 > div > div.shadow-md > table > tbody"
            )
        ).await?;
        tbody
            .wait_until()
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .displayed().await?;

        let mut assignments = Vec::new();
        let mut activities = Vec::new();
        let mut weekdays = Vec::new();

        for row in tbody.find_all(By::Css("tbody > *")).await? {
            if row.tag_name().await? == "tr" {
                let weekday = row.text().await?;
                let weekday = weekday.split_whitespace().last().unwrap().to_string();
                weekdays.push(weekday);
                continue;
            }
            let weekday = weekdays.last().unwrap().clone();
            let activity = row
                .query(By::Css("a > td.py-2.text-center.justify-center"))
                .wait(Duration::from_secs(0), Duration::from_millis(250))
                .first().await
                .unwrap_or_else(|_| activities.last().cloned().unwrap());

            activities.push(activity.clone());

            let _td_data = row.find_all(By::Css("a > td")).await?;
            let ([dtime, assignment, group, ..], _) = _td_data.split_at(4) else {
                bail!("unknown assignment's case; not implemented yet!")
            };

            let activity = activity.text().await?;
            let dtime = dtime.text().await?;
            let assignment = assignment.text().await?;
            let group = group.text().await?;

            let mut jdata = Map::new();
            jdata.insert("weekday".to_string(), Value::from(weekday));
            jdata.insert("activity".to_string(), Value::from(activity));
            jdata.insert("time".to_string(), Value::from(dtime));
            jdata.insert("assignment".to_string(), Value::from(assignment));
            jdata.insert("group".to_string(), Value::from(group));
            assignments.push(jdata);
        }
        let persons = self.driver.find(
            By::Css(
                "#root > div > main > div > div > div.flex.flex-col.w-full.lg\\:w-1\\/2.md\\:w-2\\/3 > div > div.p-1 > div > span"
            )
        ).await?;
        persons
            .wait_until()
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .displayed().await?;
        let persons = persons.text().await?;
        let mut competition_map = Map::new();
        competition_map.insert("persons".to_string(), Value::from(persons));
        competition_map.insert("assignments".to_string(), Value::from(assignments));
        Ok(competition_map)
    }
}
