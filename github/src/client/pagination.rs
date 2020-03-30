use crate::client::HEADER_LINK;
use serde::Serialize;
use url::Url;

/// Represents `Pagination` information from a Github API request
#[derive(Debug, Default)]
pub struct Pagination {
    pub next_page: Option<usize>,
    pub prev_page: Option<usize>,
    pub first_page: Option<usize>,
    pub last_page: Option<usize>,

    pub next_page_token: Option<String>,
}

impl Pagination {
    pub(super) fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let mut pagination = Self::default();

        let links = if let Some(links) = headers.get(HEADER_LINK).and_then(|h| h.to_str().ok()) {
            links
        } else {
            return pagination;
        };

        for link in links.split(',') {
            let segments: Vec<&str> = link.split(';').map(str::trim).collect();

            // Skip if we don't at least have href and rel
            if segments.len() < 2 {
                continue;
            }

            // Check if href segment is well formed and a valid url format
            let url = if segments[0].starts_with('<') && segments[0].ends_with('>') {
                if let Ok(url) = Url::parse(&segments[0][1..segments[0].len() - 1]) {
                    url
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // and then pull out the page number
            let page = if let Some(page) =
                url.query_pairs()
                    .find_map(|(k, v)| if k == "page" { Some(v) } else { None })
            {
                page
            } else {
                continue;
            };

            for rel in &segments[1..] {
                match rel.trim() {
                    "rel=\"next\"" => {
                        if let Ok(n) = page.parse() {
                            pagination.next_page = Some(n);
                        } else {
                            pagination.next_page_token = Some(page.clone().into_owned());
                        }
                    }
                    "rel=\"prev\"" => {
                        pagination.prev_page = page.parse().ok();
                    }
                    "rel=\"first\"" => {
                        pagination.first_page = page.parse().ok();
                    }
                    "rel=\"last\"" => {
                        pagination.last_page = page.parse().ok();
                    }
                    _ => {}
                }
            }
        }

        pagination
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PaginationOptions {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Default, Serialize)]
pub struct PaginationCursorOptions {
    pub page: Option<String>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateFilter {
    Open,
    Closed,
    All,
}

impl Default for StateFilter {
    fn default() -> Self {
        StateFilter::Open
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortPages {
    Created,
    Updated,
    Comments,
}

impl Default for SortPages {
    fn default() -> Self {
        SortPages::Created
    }
}

#[derive(Debug, Serialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Descending
    }
}

#[cfg(test)]
mod test {
    use super::{Pagination, HEADER_LINK};
    use reqwest::header::HeaderMap;

    #[test]
    fn pagination() {
        let mut headers = HeaderMap::new();
        let link = r#"<https://api.github.com/user/repos?page=3&per_page=100>; rel="next", <https://api.github.com/user/repos?page=50&per_page=100>; rel="last""#;
        headers.insert(HEADER_LINK, link.parse().unwrap());

        let p = Pagination::from_headers(&headers);
        assert_eq!(p.next_page, Some(3));
        assert_eq!(p.last_page, Some(50));
    }
}
