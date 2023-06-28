use std::{error::Error, fmt::{Display, Formatter}};

use scraper::{Html, Selector};

static LINK_STARTS: &str = "https://forum.paradoxplaza.com/forum/developer-diary";

#[derive(Debug)]
pub enum ScraperError<'a> {
    Reqwest(reqwest::Error),
    Scraper(scraper::error::SelectorErrorKind<'a>),
    Custom(String),
}

impl<'a> ScraperError<'a> {
    pub fn new(s: String) -> ScraperError<'a> {
        ScraperError::Custom(s)
    }
}

impl<'a> Display for ScraperError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = "bla";
        write!(f, "{s}")
    }
}

impl<'a> Error for ScraperError<'a> {}

impl<'a> From<reqwest::Error> for ScraperError<'a> {
    fn from(e: reqwest::Error) -> Self {
        ScraperError::Reqwest(e)
    }
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError<'a> {
    fn from(e: scraper::error::SelectorErrorKind<'a>) -> Self {
        ScraperError::Scraper(e)
    }
}

pub async fn pdx_scraper<'a>(url: &str) -> Result<Option<String>, ScraperError<'a>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let div_selector = Selector::parse("div.buttonGroup a")?;

    if let Some(elem) = document.select(&div_selector).nth(1) {
        let link = format!(
            "https://forum.paradoxplaza.com{}",
            elem.value()
                .attr("href")
                .ok_or(ScraperError::new("href error".to_string()))?
        );
        if link.contains(LINK_STARTS) {
            Ok(Some(link))
        } else {
            Err(ScraperError::new(format!("mauvais lien : {link}")))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn pdx_dd_test() {
        let ref_links = &[
            "https://forum.paradoxplaza.com/forum/developer-diary/europa-universalis-iv-development-diary-23rd-of-may-2023-1-35-3-known-issues-and-the-road-to-1-35-4.1586331/",
            "https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-89-whats-next-after-1-3.1589178/",
            "https://forum.paradoxplaza.com/forum/developer-diary/developer-diary-historical-sweden.1589418/",
            "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-130-wards-and-wardens-the-vision.1590033/",
            "https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-304-3-8-4-released-whats-next.1589870/",
            "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-18-dragon-lords.1589296/"
        ];
        let mut results = Vec::new();
        for link in ref_links {
            results.push(pdx_scraper(link).await);
        }
        let all_ok = results.iter().all(|res| res.is_ok());
        assert!(all_ok);
        let all_some = results.iter().all(|res| res.as_ref().unwrap().is_some());
        assert!(all_some);
        let res_links = results.iter().map(|res| res.as_ref().unwrap().as_ref().unwrap()).collect::<Vec<&String>>();
        let expected_links = &[
            &"https://forum.paradoxplaza.com/forum/developer-diary/europa-universalis-iv-development-diary-20th-of-june-2023-1-35-4-release-history-lessons-dlc.1590980/".to_string(),
            &"https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-90-update-1-3-5-changelog.1591304/".to_string(),
            &"https://forum.paradoxplaza.com/forum/developer-diary/developer-diary-historical-norway.1590854/".to_string(),
            &"https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-131-ckiii-university-101.1590985/".to_string(),
            &"https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-305-midsummer-festivities.1591017/".to_string(),
            &"https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-19-dragon-dawn-lizardfolk-tomes-and-units.1590342/".to_string()
        ];
        assert_eq!(res_links, expected_links);
    }
}
