use chrono::{NaiveDate, Utc};

use crate::models::transaction::{
    TransactionCategory, TransactionInput, TransactionStatus, TransactionType,
};

pub fn validate_and_build_transaction_input(
    transaction_type: &str,
    date: NaiveDate,
    category: &str,
    amount: i32,
    memo: &str,
    status: &str,
) -> Result<TransactionInput, Vec<String>> {
    let mut errors = Vec::new();

    let parsed_transaction_type = match parse_transaction_type(transaction_type) {
        Ok(transaction_type) => Some(transaction_type),
        Err(message) => {
            errors.push(message);
            None
        }
    };

    let parsed_category = match parse_category(category) {
        Ok(category) => Some(category),
        Err(message) => {
            errors.push(message);
            None
        }
    };

    let parsed_status = match parse_status(status) {
        Ok(status) => Some(status),
        Err(message) => {
            errors.push(message);
            None
        }
    };

    if date > Utc::now().date_naive() && matches!(parsed_status, Some(TransactionStatus::Confirmed))
    {
        errors.push("確定済みの取引に未来日付は入力できません。".to_string());
    }

    if amount <= 0 {
        errors.push("金額は1円以上で入力してください。".to_string());
    }

    if amount > 10_000_000 {
        errors.push("金額は10,000,000円以下で入力してください。".to_string());
    }

    let trimmed_memo = memo.trim();

    if trimmed_memo.is_empty() {
        errors.push("メモを入力してください。".to_string());
    }

    if trimmed_memo.chars().count() > 50 {
        errors.push("メモは50文字以内で入力してください。".to_string());
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(TransactionInput {
        transaction_type: parsed_transaction_type.expect("transaction_type should be valid"),
        date,
        category: parsed_category.expect("category should be valid"),
        amount,
        memo: trimmed_memo.to_string(),
        status: parsed_status.expect("status should be valid"),
    })
}

fn parse_transaction_type(transaction_type: &str) -> Result<TransactionType, String> {
    match transaction_type {
        "income" => Ok(TransactionType::Income),
        "expense" => Ok(TransactionType::Expense),
        _ => Err("種類は income または expense を指定してください。".to_string()),
    }
}

fn parse_category(category: &str) -> Result<TransactionCategory, String> {
    match category {
        "food" => Ok(TransactionCategory::Food),
        "daily" => Ok(TransactionCategory::Daily),
        "transport" => Ok(TransactionCategory::Transport),
        "entertainment" => Ok(TransactionCategory::Entertainment),
        "salary" => Ok(TransactionCategory::Salary),
        "other" => Ok(TransactionCategory::Other),
        _ => Err("カテゴリが不正です。".to_string()),
    }
}

fn parse_status(status: &str) -> Result<TransactionStatus, String> {
    match status {
        "confirmed" => Ok(TransactionStatus::Confirmed),
        "planned" => Ok(TransactionStatus::Planned),
        _ => Err("状態は confirmed または planned を指定してください。".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn valid_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 11).unwrap()
    }

    #[test]
    fn validate_and_build_transaction_input_accepts_valid_expense() {
        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            1200,
            " 昼食 ",
            "confirmed",
        )
        .expect("valid input should be accepted");

        assert_eq!(result.transaction_type.as_str(), "expense");
        assert_eq!(result.date, valid_date());
        assert_eq!(result.category.as_str(), "food");
        assert_eq!(result.amount, 1200);
        assert_eq!(result.memo, "昼食");
    }

    #[test]
    fn validate_and_build_transaction_input_accepts_valid_income() {
        let result = validate_and_build_transaction_input(
            "income",
            valid_date(),
            "salary",
            250000,
            "給与",
            "confirmed",
        )
        .expect("valid input should be accepted");

        assert_eq!(result.transaction_type.as_str(), "income");
        assert_eq!(result.category.as_str(), "salary");
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_invalid_transaction_type() {
        let result = validate_and_build_transaction_input(
            "invalid",
            valid_date(),
            "food",
            1200,
            "昼食",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "種類は income または expense を指定してください。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_invalid_category() {
        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "unknown",
            1200,
            "昼食",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(errors.iter().any(|error| error == "カテゴリが不正です。"));
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_zero_amount() {
        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            0,
            "昼食",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "金額は1円以上で入力してください。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_too_large_amount() {
        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            10_000_001,
            "昼食",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "金額は10,000,000円以下で入力してください。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_empty_memo() {
        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            1200,
            "   ",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "メモを入力してください。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_memo_over_50_chars() {
        let long_memo = "あ".repeat(51);

        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            1200,
            &long_memo,
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "メモは50文字以内で入力してください。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_accepts_50_chars_memo() {
        let memo = "あ".repeat(50);

        let result = validate_and_build_transaction_input(
            "expense",
            valid_date(),
            "food",
            1200,
            &memo,
            "confirmed",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn validate_and_build_transaction_input_rejects_future_date() {
        let future_date = Utc::now().date_naive() + Duration::days(1);

        let result = validate_and_build_transaction_input(
            "expense",
            future_date,
            "food",
            1200,
            "昼食",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "確定済みの取引に未来日付は入力できません。")
        );
    }

    #[test]
    fn validate_and_build_transaction_input_returns_multiple_errors() {
        let result = validate_and_build_transaction_input(
            "invalid",
            valid_date(),
            "unknown",
            0,
            "",
            "confirmed",
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();

        assert!(
            errors
                .iter()
                .any(|error| error == "種類は income または expense を指定してください。")
        );
        assert!(errors.iter().any(|error| error == "カテゴリが不正です。"));
        assert!(
            errors
                .iter()
                .any(|error| error == "金額は1円以上で入力してください。")
        );
        assert!(
            errors
                .iter()
                .any(|error| error == "メモを入力してください。")
        );
    }
}
