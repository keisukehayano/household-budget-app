use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    models::transaction::{
        TransactionCreateRequest, TransactionListFilter, TransactionListQuery,
        TransactionListResponse, TransactionResponse, TransactionSummaryResponse,
        TransactionUpdateRequest,
    },
    repositories::transaction as transaction_repository,
    validators::transaction::validate_and_build_transaction_input,
};

pub async fn list_transactions(
    db: &PgPool,
    user_id: Uuid,
    query: TransactionListQuery,
) -> Result<TransactionListResponse, ApiError> {
    let filter = TransactionListFilter::from_query(query).map_err(ApiError::bad_request)?;

    let total = transaction_repository::count_transactions(db, user_id, &filter)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to count transactions");
            ApiError::internal_server_error()
        })?;

    let items = transaction_repository::find_transactions(db, user_id, &filter)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to fetch transactions");
            ApiError::internal_server_error()
        })?;

    Ok(TransactionListResponse::new(
        items,
        filter.page,
        filter.limit,
        total,
    ))
}

pub async fn summarize_transactions(
    db: &PgPool,
    user_id: Uuid,
    query: TransactionListQuery,
) -> Result<TransactionSummaryResponse, ApiError> {
    let filter = TransactionListFilter::from_summary_query(query).map_err(ApiError::bad_request)?;

    transaction_repository::summarize_transactions(db, user_id, &filter)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to summarize transaction");
            ApiError::internal_server_error()
        })
}

pub async fn create_transaction(
    db: &PgPool,
    user_id: Uuid,
    payload: TransactionCreateRequest,
) -> Result<TransactionResponse, ApiError> {
    let input = validate_and_build_transaction_input(
        &payload.transaction_type,
        payload.date,
        &payload.category,
        payload.amount,
        &payload.memo,
        &payload.status,
    )
    .map_err(ApiError::bad_request)?;

    let id = Uuid::new_v4();

    transaction_repository::create_transaction(db, user_id, id, input)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to create transaction");
            ApiError::internal_server_error()
        })
}

pub async fn update_transaction(
    db: &PgPool,
    user_id: Uuid,
    id: Uuid,
    payload: TransactionUpdateRequest,
) -> Result<TransactionResponse, ApiError> {
    let input = validate_and_build_transaction_input(
        &payload.transaction_type,
        payload.date,
        &payload.category,
        payload.amount,
        &payload.memo,
        &payload.status,
    )
    .map_err(ApiError::bad_request)?;

    let transaction = transaction_repository::update_transaction(db, user_id, id, input)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to update transaction");
            ApiError::internal_server_error()
        })?;

    match transaction {
        Some(transaction) => Ok(transaction),
        None => Err(ApiError::not_found("指定された取引が見つかりません。")),
    }
}

pub async fn delete_transaction(db: &PgPool, user_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    let rows_affected = transaction_repository::delete_transaction(db, user_id, id)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to delete transaction");
            ApiError::internal_server_error()
        })?;

    if rows_affected == 0 {
        return Err(ApiError::not_found("指定された取引が見つかりません。"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        models::transaction::{
            TransactionCreateRequest, TransactionListQuery, TransactionUpdateRequest,
        },
        repositories::user as user_repository,
    };

    fn create_request(
        transaction_type: &str,
        date: &str,
        category: &str,
        amount: i32,
        memo: &str,
        status: &str,
    ) -> TransactionCreateRequest {
        TransactionCreateRequest {
            transaction_type: transaction_type.to_string(),
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            category: category.to_string(),
            amount,
            memo: memo.to_string(),
            status: status.to_string(),
        }
    }

    fn update_request(
        transaction_type: &str,
        date: &str,
        category: &str,
        amount: i32,
        memo: &str,
        status: &str,
    ) -> TransactionUpdateRequest {
        TransactionUpdateRequest {
            transaction_type: transaction_type.to_string(),
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            category: category.to_string(),
            amount,
            memo: memo.to_string(),
            status: status.to_string(),
        }
    }

    fn list_query(
        month: Option<&str>,
        q: Option<&str>,
        sort: Option<&str>,
        page: Option<&str>,
        limit: Option<&str>,
        status: Option<&str>,
    ) -> TransactionListQuery {
        TransactionListQuery {
            month: month.map(str::to_string),
            q: q.map(str::to_string),
            sort: sort.map(str::to_string),
            page: page.map(str::to_string),
            limit: limit.map(str::to_string),
            status: status.map(str::to_string),
        }
    }

    async fn create_test_user(pool: &PgPool, email: &str) -> Uuid {
        user_repository::create_user(pool, Uuid::new_v4(), email, "dummy-password-hash")
            .await
            .expect("test user should be created")
            .id
    }

    async fn seed_transaction(
        pool: &PgPool,
        user_id: Uuid,
        transaction_type: &str,
        date: &str,
        category: &str,
        amount: i32,
        memo: &str,
    ) -> Uuid {
        let transaction = create_transaction(
            pool,
            user_id,
            create_request(transaction_type, date, category, amount, memo, "confirmed"),
        )
        .await
        .expect("test transaction should be created");

        transaction.id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_transaction_accepts_valid_payload(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-create@example.com").await;
        let transaction = create_transaction(
            &pool,
            user_id,
            create_request("expense", "2024-06-11", "food", 1200, "昼食", "confirmed"),
        )
        .await
        .expect("valid transaction should be created");

        assert_eq!(transaction.transaction_type, "expense");
        assert_eq!(transaction.date.to_string(), "2024-06-11");
        assert_eq!(transaction.category, "food");
        assert_eq!(transaction.amount, 1200);
        assert_eq!(transaction.memo, "昼食");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_transaction_rejects_invalid_payload(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-invalid@example.com").await;
        let result = create_transaction(
            &pool,
            user_id,
            create_request("invalid", "2024-06-11", "unknown", 0, "", "confirmed"),
        )
        .await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn list_transactions_returns_paginated_response(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-list@example.com").await;
        let other_user_id = create_test_user(&pool, "service-list-other@example.com").await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-12",
            "transport",
            580,
            "電車代",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "income",
            "2024-06-25",
            "salary",
            250000,
            "給与",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-07-01",
            "food",
            900,
            "7月の昼食",
        )
        .await;
        seed_transaction(
            &pool,
            other_user_id,
            "expense",
            "2024-06-20",
            "food",
            777,
            "別ユーザー",
        )
        .await;

        let response = list_transactions(
            &pool,
            user_id,
            list_query(
                Some("2024-06"),
                Some("支出"),
                Some("date-desc"),
                Some("1"),
                Some("1"),
                None,
            ),
        )
        .await
        .expect("transactions should be fetched");

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].memo, "電車代");

        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 1);
        assert_eq!(response.pagination.total, 2);
        assert_eq!(response.pagination.total_pages, 2);
        assert!(response.pagination.has_next);
        assert!(!response.pagination.has_previous);

        let second_page_response = list_transactions(
            &pool,
            user_id,
            list_query(
                Some("2024-06"),
                Some("支出"),
                Some("date-desc"),
                Some("2"),
                Some("1"),
                None,
            ),
        )
        .await
        .expect("transactions should be fetched");

        assert_eq!(second_page_response.items.len(), 1);
        assert_eq!(second_page_response.items[0].memo, "昼食");

        assert_eq!(second_page_response.pagination.page, 2);
        assert_eq!(second_page_response.pagination.total, 2);
        assert!(!second_page_response.pagination.has_next);
        assert!(second_page_response.pagination.has_previous);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn list_transactions_rejects_invalid_query(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-query@example.com").await;
        let result = list_transactions(
            &pool,
            user_id,
            list_query(
                Some("2024/06"),
                None,
                Some("invalid"),
                Some("0"),
                Some("101"),
                None,
            ),
        )
        .await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn summarize_transactions_uses_all_matching_rows_not_current_page_only(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-summary@example.com").await;
        let other_user_id = create_test_user(&pool, "service-summary-other@example.com").await;
        seed_transaction(
            &pool,
            user_id,
            "income",
            "2024-06-25",
            "salary",
            250000,
            "給与",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;
        seed_transaction(&pool, user_id, "expense", "2024-06-12", "food", 800, "夕食").await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-13",
            "daily",
            980,
            "日用品",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-07-01",
            "food",
            900,
            "7月の昼食",
        )
        .await;
        seed_transaction(
            &pool,
            other_user_id,
            "expense",
            "2024-06-15",
            "food",
            999,
            "別ユーザー",
        )
        .await;

        let summary = summarize_transactions(
            &pool,
            user_id,
            list_query(
                Some("2024-06"),
                None,
                Some("date-desc"),
                Some("1"),
                Some("1"),
                None,
            ),
        )
        .await
        .expect("summary should be fetched");

        assert_eq!(summary.total_income, 250000);
        assert_eq!(summary.total_expense, 2980);
        assert_eq!(summary.balance, 247020);

        let food_summary = summary
            .category_summaries
            .iter()
            .find(|summary| summary.category == "food")
            .expect("food summary should exist");

        assert_eq!(food_summary.total, 2000);

        let daily_summary = summary
            .category_summaries
            .iter()
            .find(|summary| summary.category == "daily")
            .expect("daily summary should exist");

        assert_eq!(daily_summary.total, 980);

        assert!(
            summary
                .category_summaries
                .iter()
                .all(|summary| summary.category != "salary")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn summarize_transactions_applies_search_condition(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-search@example.com").await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-12",
            "transport",
            580,
            "電車代",
        )
        .await;
        seed_transaction(
            &pool,
            user_id,
            "income",
            "2024-06-25",
            "salary",
            250000,
            "給与",
        )
        .await;

        let summary = summarize_transactions(
            &pool,
            user_id,
            list_query(Some("2024-06"), Some("食費"), None, None, None, None),
        )
        .await
        .expect("summary should be fetched");

        assert_eq!(summary.total_income, 0);
        assert_eq!(summary.total_expense, 1200);
        assert_eq!(summary.balance, -1200);
        assert_eq!(summary.category_summaries.len(), 1);
        assert_eq!(summary.category_summaries[0].category, "food");
        assert_eq!(summary.category_summaries[0].total, 1200);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_transaction_updates_existing_transaction(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-update@example.com").await;
        let id = seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;

        let updated_transaction = update_transaction(
            &pool,
            user_id,
            id,
            update_request(
                "expense",
                "2024-06-12",
                "daily",
                2000,
                "更新後メモ",
                "confirmed",
            ),
        )
        .await
        .expect("transaction should be updated");

        assert_eq!(updated_transaction.id, id);
        assert_eq!(updated_transaction.transaction_type, "expense");
        assert_eq!(updated_transaction.date.to_string(), "2024-06-12");
        assert_eq!(updated_transaction.category, "daily");
        assert_eq!(updated_transaction.amount, 2000);
        assert_eq!(updated_transaction.memo, "更新後メモ");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_transaction_rejects_invalid_payload(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-update-invalid@example.com").await;
        let id = seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;

        let result = update_transaction(
            &pool,
            user_id,
            id,
            update_request("invalid", "2024-06-12", "unknown", 0, "", "confirmed"),
        )
        .await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_transaction_returns_error_when_transaction_does_not_exist(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-update-missing@example.com").await;
        let result = update_transaction(
            &pool,
            user_id,
            Uuid::new_v4(),
            update_request(
                "expense",
                "2024-06-12",
                "daily",
                2000,
                "更新後メモ",
                "confirmed",
            ),
        )
        .await;

        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transaction_deletes_existing_transaction(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-delete@example.com").await;
        let id = seed_transaction(
            &pool,
            user_id,
            "expense",
            "2024-06-11",
            "food",
            1200,
            "昼食",
        )
        .await;

        delete_transaction(&pool, user_id, id)
            .await
            .expect("transaction should be deleted");

        let response = list_transactions(
            &pool,
            user_id,
            list_query(None, None, Some("date-desc"), Some("1"), Some("10"), None),
        )
        .await
        .expect("transactions should be fetched");

        assert_eq!(response.pagination.total, 0);
        assert!(response.items.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transaction_returns_error_when_transaction_does_not_exist(pool: PgPool) {
        let user_id = create_test_user(&pool, "service-delete-missing@example.com").await;
        let result = delete_transaction(&pool, user_id, Uuid::new_v4()).await;

        assert!(result.is_err());
    }
}
