use futures::StreamExt;
use scraper::{Html, Selector};
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

static NEXT_DD: &str = "next dev diary";

#[derive(Debug)]
pub enum ScraperError {
    Reqwest(reqwest::Error),
    Scraper(String),
    FromUtf8(std::str::Utf8Error),
    Custom(String),
}

impl ScraperError {
    pub fn new(s: String) -> ScraperError {
        ScraperError::Custom(s)
    }
}

impl Display for ScraperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = "bla";
        write!(f, "{s}")
    }
}

impl Error for ScraperError {}

impl From<reqwest::Error> for ScraperError {
    fn from(e: reqwest::Error) -> Self {
        ScraperError::Reqwest(e)
    }
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(e: scraper::error::SelectorErrorKind) -> Self {
        ScraperError::Scraper(e.to_string())
    }
}

impl From<std::str::Utf8Error> for ScraperError {
    fn from(e: std::str::Utf8Error) -> Self {
        ScraperError::FromUtf8(e)
    }
}

pub async fn pdx_scraper(
    url: String,
    client: &reqwest::Client,
) -> Result<Option<String>, ScraperError> {
    let response = client.get(url).send().await?;
    let div_selector = Selector::parse("a.pagenav-jump--next")?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = &chunk?.to_ascii_lowercase();
        if chunk
            .windows(NEXT_DD.len())
            .all(|w| w != NEXT_DD.as_bytes())
        {
            continue;
        }

        let r = std::str::from_utf8(chunk)?;
        let doc = Html::parse_document(r);
        for elem in doc.select(&div_selector) {
            if !elem.inner_html().contains(NEXT_DD) {
                continue;
            }
            return Ok(Some(format!(
                "https://forum.paradoxplaza.com{}",
                elem.value()
                    .attr("href")
                    .ok_or_else(|| ScraperError::new("href error".to_string()))?
            )));
        }
    }

    Ok(None)
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
        let client = reqwest::Client::default();
        for link in ref_links {
            results.push(pdx_scraper(link.to_string(), &client).await);
        }
        let all_ok = results.iter().all(|res| res.is_ok());
        assert!(all_ok);
        let all_some = results.iter().all(|res| res.as_ref().unwrap().is_some());
        assert!(all_some);
        let res_links = results
            .iter()
            .map(|res| res.as_ref().unwrap().as_ref().unwrap())
            .collect::<Vec<&String>>();
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
