use crate::SearchError;

pub const DEFAULT_SEARCH_PAGE_SIZE: u32 = 20;
pub const MAX_SEARCH_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchPage {
    number: u32,
    size: u32,
}

impl SearchPage {
    pub fn first(size: u32) -> Result<Self, SearchError> {
        Self::new(1, size)
    }

    pub fn new(number: u32, size: u32) -> Result<Self, SearchError> {
        if number == 0 {
            return Err(SearchError::invalid_search_options(
                "page number must be at least 1".to_string(),
            ));
        }
        if size == 0 || size > MAX_SEARCH_PAGE_SIZE {
            return Err(SearchError::invalid_search_options(format!(
                "page size must be between 1 and {MAX_SEARCH_PAGE_SIZE}; got {size}"
            )));
        }
        Ok(Self { number, size })
    }

    pub const fn number(self) -> u32 {
        self.number
    }

    pub const fn size(self) -> u32 {
        self.size
    }

    pub(crate) fn offset(self) -> Result<u32, SearchError> {
        self.number
            .checked_sub(1)
            .and_then(|page_index| page_index.checked_mul(self.size))
            .ok_or_else(|| {
                SearchError::invalid_search_options(
                    "page request is too large for this search request".to_string(),
                )
            })
    }
}

impl Default for SearchPage {
    fn default() -> Self {
        Self {
            number: 1,
            size: DEFAULT_SEARCH_PAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchPageInfo {
    pub number: u32,
    pub size: u32,
    pub count: usize,
    pub total: u64,
    pub has_more: bool,
    pub next_page: Option<u32>,
}

impl SearchPageInfo {
    pub(crate) fn from_page(
        page: SearchPage,
        count: usize,
        total: u64,
    ) -> Result<Self, SearchError> {
        let consumed = u64::from(page.offset()?)
            .checked_add(count as u64)
            .ok_or_else(|| {
                SearchError::invalid_search_options(
                    "page traversal is too large for this search request".to_string(),
                )
            })?;
        let has_more = consumed < total;
        let next_page = if has_more {
            Some(page.number.checked_add(1).ok_or_else(|| {
                SearchError::invalid_search_options(
                    "next page number is too large for this search request".to_string(),
                )
            })?)
        } else {
            None
        };
        Ok(Self {
            number: page.number,
            size: page.size,
            count,
            total,
            has_more,
            next_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_page_derives_offset() {
        for (number, size, offset) in [(1, 1, 0), (3, 25, 50), (u32::MAX, 1, u32::MAX - 1)] {
            let page = SearchPage::new(number, size).unwrap();
            assert_eq!(page.offset().unwrap(), offset);
        }
        assert!(SearchPage::new(u32::MAX, 2).unwrap().offset().is_err());
    }

    #[test]
    fn search_page_rejects_zero_and_oversized_values() {
        assert!(SearchPage::new(0, 25).is_err());
        assert!(SearchPage::new(1, 0).is_err());
        assert!(SearchPage::new(1, MAX_SEARCH_PAGE_SIZE + 1).is_err());
        assert!(SearchPage::new(1, MAX_SEARCH_PAGE_SIZE).is_ok());
    }

    #[test]
    fn search_page_info_reports_next_page_from_consumed_results() {
        for (number, count, total, next_page) in [
            (1, 0, 0, None),
            (2, 10, 25, Some(3)),
            (3, 5, 25, None),
            (4, 0, 25, None),
        ] {
            let page = SearchPage::new(number, 10).unwrap();
            let info = SearchPageInfo::from_page(page, count, total).unwrap();
            assert_eq!(
                info,
                SearchPageInfo {
                    number,
                    size: 10,
                    count,
                    total,
                    has_more: next_page.is_some(),
                    next_page,
                }
            );
        }
        let last = SearchPage::new(u32::MAX, 1).unwrap();
        assert!(SearchPageInfo::from_page(last, 1, u64::from(u32::MAX) + 1).is_err());
    }
}
