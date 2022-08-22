use std::{fmt::format, io::Error};

use headless_chrome::Browser;
use reqwest;

pub async fn search(song: &str) -> Option<String> {
    if let Ok(browser) = Browser::default() {
        if let Ok(tab) = browser.wait_for_initial_tab() {
            match tab.navigate_to(&format!("https://www.youtube.com/results?search_query={}", song)) {
                Ok(_) => {
                    match tab.wait_until_navigated() {
                        Ok(_) => {
                            if let Ok(titles) = tab.find_elements("a#video-title") {
                                if let Some(first_result) = titles.first() {
                                    if let Ok(attributes_1) = first_result.get_attributes() {
                                        if let Some(attributes) = attributes_1 {
                                            if let Some(href) = attributes.get("href") {
                                                return Some(String::from(format!("https://www.youtube.com{}", href)));
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(_) => return None,
                    }
                },
                Err(_) => return None,
            }
        }
    }

    return None;
    
    // tab.navigate_to(&format!("https://www.youtube.com/results?search_query={}", song)).unwrap();
    // tab.wait_until_navigated().unwrap();
    // let titles = tab.find_elements("a#video-title").unwrap();
    // let attributes = titles.first().unwrap().get_attributes().unwrap().unwrap();
    // let href = attributes.get("href").unwrap();
    // return Some(String::from(format!("https://www.youtube.com{}", href)));
}