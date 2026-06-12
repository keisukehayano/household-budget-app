use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum TransactionType {
    Income,
    Expense,
}

impl TransactionType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionCategory {
    Food,
    Daily,
    Transport,
    Entertainment,
    Salary,
    Other,
}

impl TransactionCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Daily => "daily",
            Self::Transport => "transport",
            Self::Entertainment => "entertainment",
            Self::Salary => "salary",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionStatus {
    Confirmed,
    Planned,
}

impl TransactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug)]
pub struct TransactionInput {
    pub transaction_type: TransactionType,
    pub date: NaiveDate,
    pub category: TransactionCategory,
    pub amount: i32,
    pub memo: String,
    pub status: TransactionStatus,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionResponse {
    pub id: Uuid,

    #[serde(rename = "type")]
    pub transaction_type: String,

    pub date: NaiveDate,
    pub category: String,
    pub amount: i32,
    pub memo: String,
    pub status: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TransactionListResponse {
    pub items: Vec<TransactionResponse>,
    pub pagination: PaginationResponse,
}

impl TransactionListResponse {
    pub fn new(items: Vec<TransactionResponse>, page: u32, limit: u32, total: i64) -> Self {
        let total_pages = if total == 0 {
            0
        } else {
            (total + i64::from(limit) - 1) / i64::from(limit)
        };

        Self {
            items,
            pagination: PaginationResponse {
                page,
                limit,
                total,
                total_pages,
                has_next: i64::from(page) < total_pages,
                has_previous: page > 1 && total_pages > 0,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    pub page: u32,
    pub limit: u32,
    pub total: i64,

    #[serde(rename = "totalPages")]
    pub total_pages: i64,

    #[serde(rename = "hasNext")]
    pub has_next: bool,

    #[serde(rename = "hasPrevious")]
    pub has_previous: bool,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummaryResponse {
    #[serde(rename = "totalIncome")]
    pub total_income: i64,

    #[serde(rename = "totalExpense")]
    pub total_expense: i64,

    pub balance: i64,

    #[serde(rename = "categorySummaries")]
    pub category_summaries: Vec<TransactionCategorySummaryResponse>,
}

impl TransactionSummaryResponse {
    pub fn new(
        total_income: i64,
        total_expense: i64,
        category_summaries: Vec<TransactionCategorySummaryResponse>,
    ) -> Self {
        Self {
            total_income,
            total_expense,
            balance: total_income - total_expense,
            category_summaries,
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionCategorySummaryResponse {
    pub category: String,
    pub total: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TransactionTotalSummaryRow {
    pub total_income: i64,
    pub total_expense: i64,
}

#[derive(Debug, Deserialize)]
pub struct TransactionListQuery {
    pub month: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub page: Option<String>,
    pub limit: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug)]
pub struct TransactionListFilter {
    pub month: Option<YearMonth>,
    pub search_query: Option<String>,
    pub sort_order: TransactionSortOrder,
    pub page: u32,
    pub limit: u32,
    pub status_filter: TransactionStatusFilter,
}

impl TransactionListFilter {
    pub fn from_query(query: TransactionListQuery) -> Result<Self, Vec<String>> {
        Self::from_query_with_default_status(query, TransactionStatusFilter::All)
    }

    pub fn from_summary_query(query: TransactionListQuery) -> Result<Self, Vec<String>> {
        Self::from_query_with_default_status(query, TransactionStatusFilter::Confirmed)
    }

    fn from_query_with_default_status(
        query: TransactionListQuery,
        default_status_filter: TransactionStatusFilter,
    ) -> Result<Self, Vec<String>> {
        let mut errors = Vec::new();

        let month = match query
            .month
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) => match YearMonth::parse(value) {
                Ok(month) => Some(month),
                Err(message) => {
                    errors.push(message);
                    None
                }
            },
            None => None,
        };

        let sort_order = match query
            .sort
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) => match TransactionSortOrder::parse(value) {
                Ok(sort_order) => sort_order,
                Err(message) => {
                    errors.push(message);
                    TransactionSortOrder::DateDesc
                }
            },
            None => TransactionSortOrder::DateDesc,
        };

        let page = match parse_positive_u32(query.page.as_deref(), "page") {
            Ok(page) => page.unwrap_or(1),
            Err(message) => {
                errors.push(message);
                1
            }
        };

        let limit = match parse_positive_u32(query.limit.as_deref(), "limit") {
            Ok(limit) => limit.unwrap_or(10),
            Err(message) => {
                errors.push(message);
                10
            }
        };

        if limit > 100 {
            errors.push("limit は100以下で指定してください。".to_string());
        }

        let status_filter = match query
            .status
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) => match TransactionStatusFilter::parse(value) {
                Ok(status_filter) => status_filter,
                Err(message) => {
                    errors.push(message);
                    default_status_filter
                }
            },
            None => default_status_filter,
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        let search_query = query
            .q
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            month,
            search_query,
            sort_order,
            page,
            limit,
            status_filter,
        })
    }

    pub fn offset(&self) -> i64 {
        i64::from(self.page.saturating_sub(1)) * i64::from(self.limit)
    }
}

fn parse_positive_u32(value: Option<&str>, name: &str) -> Result<Option<u32>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let parsed_value = value
        .parse::<u32>()
        .map_err(|_| format!("{name} は1以上の整数で指定してください。"))?;

    if parsed_value == 0 {
        return Err(format!("{name} は1以上の整数で指定してください。"));
    }

    Ok(Some(parsed_value))
}

#[derive(Debug, Clone, Copy)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

impl YearMonth {
    pub fn parse(value: &str) -> Result<Self, String> {
        let Some((year_text, month_text)) = value.split_once('-') else {
            return Err("month は YYYY-MM 形式で指定してください。".to_string());
        };

        if year_text.len() != 4 || month_text.len() != 2 {
            return Err("month は YYYY-MM 形式で指定してください。".to_string());
        }

        if !year_text
            .chars()
            .all(|character| character.is_ascii_digit())
            || !month_text
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return Err("month は YYYY-MM 形式で指定してください。".to_string());
        }

        let year = year_text
            .parse::<i32>()
            .map_err(|_| "month は YYYY-MM 形式で指定してください。".to_string())?;

        let month = month_text
            .parse::<u32>()
            .map_err(|_| "month は YYYY-MM 形式で指定してください。".to_string())?;

        if !(1..=12).contains(&month) {
            return Err("month の月は 01 から 12 の範囲で指定してください。".to_string());
        }

        if NaiveDate::from_ymd_opt(year, month, 1).is_none() {
            return Err("month の指定が不正です。".to_string());
        }

        Ok(Self { year, month })
    }

    pub fn date_range(self) -> Result<(NaiveDate, NaiveDate), String> {
        let start_date = NaiveDate::from_ymd_opt(self.year, self.month, 1)
            .ok_or_else(|| "month の指定が不正です。".to_string())?;

        let end_date = if self.month == 12 {
            NaiveDate::from_ymd_opt(self.year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(self.year, self.month + 1, 1)
        }
        .ok_or_else(|| "month の指定が不正です。".to_string())?;

        Ok((start_date, end_date))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionSortOrder {
    DateDesc,
    DateAsc,
    AmountDesc,
    AmountAsc,
}

impl TransactionSortOrder {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "date-desc" => Ok(Self::DateDesc),
            "date-asc" => Ok(Self::DateAsc),
            "amount-desc" => Ok(Self::AmountDesc),
            "amount-asc" => Ok(Self::AmountAsc),
            _ => Err(
                "sort は date-desc, date-asc, amount-desc, amount-asc のいずれかを指定してください。"
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionStatusFilter {
    Confirmed,
    Planned,
    All,
}

impl TransactionStatusFilter {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "confirmed" => Ok(Self::Confirmed),
            "planned" => Ok(Self::Planned),
            "all" => Ok(Self::All),
            _ => {
                Err("status は confirmed, planned, all のいずれかを指定してください。".to_string())
            }
        }
    }
}

fn default_transaction_status() -> String {
    "confirmed".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TransactionCreateRequest {
    #[serde(rename = "type")]
    pub transaction_type: String,

    pub date: NaiveDate,
    pub category: String,
    pub amount: i32,
    pub memo: String,

    #[serde(default = "default_transaction_status")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct TransactionUpdateRequest {
    #[serde(rename = "type")]
    pub transaction_type: String,

    pub date: NaiveDate,
    pub category: String,
    pub amount: i32,
    pub memo: String,

    #[serde(default = "default_transaction_status")]
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_month_parse_accepts_valid_value() {
        let result = YearMonth::parse("2026-06").expect("valid month should be parsed");

        assert_eq!(result.year, 2026);
        assert_eq!(result.month, 6);
    }

    #[test]
    fn year_month_parse_rejects_slash_format() {
        let result = YearMonth::parse("2026/06");

        assert!(result.is_err());
    }

    #[test]
    fn year_month_parse_rejects_non_zero_padded_month() {
        let result = YearMonth::parse("2026-6");

        assert!(result.is_err());
    }

    #[test]
    fn year_month_parse_rejects_invalid_month_number() {
        let result = YearMonth::parse("2026-13");

        assert!(result.is_err());
    }

    #[test]
    fn year_month_parse_rejects_non_numeric_value() {
        let result = YearMonth::parse("abcd-ef");

        assert!(result.is_err());
    }

    #[test]
    fn year_month_date_range_returns_start_and_next_month_start() {
        let month = YearMonth::parse("2026-06").expect("valid month should be parsed");

        let (start_date, end_date) = month.date_range().expect("date range should be valid");

        assert_eq!(start_date, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end_date, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
    }

    #[test]
    fn year_month_date_range_handles_december() {
        let month = YearMonth::parse("2026-12").expect("valid month should be parsed");

        let (start_date, end_date) = month.date_range().expect("date range should be valid");

        assert_eq!(start_date, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(end_date, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
    }

    #[test]
    fn transaction_sort_order_parse_accepts_valid_values() {
        assert!(matches!(
            TransactionSortOrder::parse("date-desc"),
            Ok(TransactionSortOrder::DateDesc)
        ));

        assert!(matches!(
            TransactionSortOrder::parse("date-asc"),
            Ok(TransactionSortOrder::DateAsc)
        ));

        assert!(matches!(
            TransactionSortOrder::parse("amount-desc"),
            Ok(TransactionSortOrder::AmountDesc)
        ));

        assert!(matches!(
            TransactionSortOrder::parse("amount-asc"),
            Ok(TransactionSortOrder::AmountAsc)
        ));
    }

    #[test]
    fn transaction_sort_order_parse_rejects_invalid_value() {
        let result = TransactionSortOrder::parse("invalid");

        assert!(result.is_err());
    }

    #[test]
    fn transaction_list_filter_uses_default_page_limit_and_sort() {
        let query = TransactionListQuery {
            month: None,
            q: None,
            sort: None,
            page: None,
            limit: None,
            status: None,
        };

        let filter = TransactionListFilter::from_query(query)
            .expect("empty query should use default values");

        assert_eq!(filter.page, 1);
        assert_eq!(filter.limit, 10);
        assert_eq!(filter.offset(), 0);
        assert!(matches!(filter.sort_order, TransactionSortOrder::DateDesc));
        assert!(filter.month.is_none());
        assert!(filter.search_query.is_none());
    }

    #[test]
    fn transaction_list_filter_parses_valid_query() {
        let query = TransactionListQuery {
            month: Some("2026-06".to_string()),
            q: Some(" 食費 ".to_string()),
            sort: Some("amount-desc".to_string()),
            page: Some("2".to_string()),
            limit: Some("20".to_string()),
            status: None,
        };

        let filter = TransactionListFilter::from_query(query)
            .expect("valid query should be converted into filter");

        assert_eq!(filter.page, 2);
        assert_eq!(filter.limit, 20);
        assert_eq!(filter.offset(), 20);
        assert_eq!(filter.search_query, Some("食費".to_string()));
        assert!(matches!(
            filter.sort_order,
            TransactionSortOrder::AmountDesc
        ));

        let month = filter.month.expect("month should exist");
        assert_eq!(month.year, 2026);
        assert_eq!(month.month, 6);
    }

    #[test]
    fn transaction_list_filter_rejects_page_zero() {
        let query = TransactionListQuery {
            month: None,
            q: None,
            sort: None,
            page: Some("0".to_string()),
            limit: None,
            status: None,
        };

        let result = TransactionListFilter::from_query(query);

        assert!(result.is_err());
    }

    #[test]
    fn transaction_list_filter_rejects_limit_over_100() {
        let query = TransactionListQuery {
            month: None,
            q: None,
            sort: None,
            page: None,
            limit: Some("101".to_string()),
            status: None,
        };

        let result = TransactionListFilter::from_query(query);

        assert!(result.is_err());
    }

    #[test]
    fn transaction_list_filter_rejects_invalid_month() {
        let query = TransactionListQuery {
            month: Some("2026/06".to_string()),
            q: None,
            sort: None,
            page: None,
            limit: None,
            status: None,
        };

        let result = TransactionListFilter::from_query(query);

        assert!(result.is_err());
    }

    #[test]
    fn transaction_list_filter_rejects_invalid_sort() {
        let query = TransactionListQuery {
            month: None,
            q: None,
            sort: Some("invalid".to_string()),
            page: None,
            limit: None,
            status: None,
        };

        let result = TransactionListFilter::from_query(query);

        assert!(result.is_err());
    }
}
