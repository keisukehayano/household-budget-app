use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::transaction::{
    TransactionCategorySummaryResponse, TransactionInput, TransactionListFilter,
    TransactionResponse, TransactionSortOrder, TransactionStatusFilter, TransactionSummaryResponse,
    TransactionTotalSummaryRow,
};

pub async fn find_transactions(
    db: &PgPool,
    filter: &TransactionListFilter,
) -> Result<Vec<TransactionResponse>, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        select
            id,
            transaction_type,
            date,
            category,
            amount,
            memo,
            status,
            created_at,
            updated_at
        from transactions
        "#,
    );

    let mut has_condition = false;
    push_filter_conditions(&mut builder, filter, &mut has_condition)?;

    push_sort_order(&mut builder, filter.sort_order);

    builder
        .push(" limit ")
        .push_bind(i64::from(filter.limit))
        .push(" offset ")
        .push_bind(filter.offset());

    builder
        .build_query_as::<TransactionResponse>()
        .fetch_all(db)
        .await
}

pub async fn count_transactions(
    db: &PgPool,
    filter: &TransactionListFilter,
) -> Result<i64, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        select count(*)
        from transactions
        "#,
    );

    let mut has_condition = false;
    push_filter_conditions(&mut builder, filter, &mut has_condition)?;

    builder.build_query_scalar::<i64>().fetch_one(db).await
}

pub async fn summarize_transactions(
    db: &PgPool,
    filter: &TransactionListFilter,
) -> Result<TransactionSummaryResponse, sqlx::Error> {
    let total_summary = find_total_summary(db, filter).await?;
    let category_summaries = find_category_summaries(db, filter).await?;

    Ok(TransactionSummaryResponse::new(
        total_summary.total_income,
        total_summary.total_expense,
        category_summaries,
    ))
}

async fn find_total_summary(
    db: &PgPool,
    filter: &TransactionListFilter,
) -> Result<TransactionTotalSummaryRow, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        select
            coalesce(
                sum(
                    case
                        when transaction_type = 'income' then amount
                        else 0
                    end
                ),
                0
            )::bigint as total_income,
            coalesce(
                sum(
                    case
                        when transaction_type = 'expense' then amount
                        else 0
                    end
                ),
                0
            )::bigint as total_expense
        from transactions
        "#,
    );

    let mut has_condition = false;
    push_filter_conditions(&mut builder, filter, &mut has_condition)?;

    builder
        .build_query_as::<TransactionTotalSummaryRow>()
        .fetch_one(db)
        .await
}

async fn find_category_summaries(
    db: &PgPool,
    filter: &TransactionListFilter,
) -> Result<Vec<TransactionCategorySummaryResponse>, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        select
            category,
            coalesce(sum(amount), 0)::bigint as total
        from transactions
        "#,
    );

    let mut has_condition = false;
    push_filter_conditions(&mut builder, filter, &mut has_condition)?;

    push_where_or_and(&mut builder, &mut has_condition);
    builder.push("transaction_type = 'expense'");

    builder.push(" group by category order by total desc, category asc");

    builder
        .build_query_as::<TransactionCategorySummaryResponse>()
        .fetch_all(db)
        .await
}

pub async fn create_transaction(
    db: &PgPool,
    id: Uuid,
    input: TransactionInput,
) -> Result<TransactionResponse, sqlx::Error> {
    sqlx::query_as::<_, TransactionResponse>(
        r#"
        insert into transactions (
            id,
            transaction_type,
            date,
            category,
            amount,
            memo,
            status
        )
        values ($1, $2, $3, $4, $5, $6, $7)
        returning
            id,
            transaction_type,
            date,
            category,
            amount,
            memo,
            status,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(input.transaction_type.as_str())
    .bind(input.date)
    .bind(input.category.as_str())
    .bind(input.amount)
    .bind(input.memo)
    .bind(input.status.as_str())
    .fetch_one(db)
    .await
}

pub async fn update_transaction(
    db: &PgPool,
    id: Uuid,
    input: TransactionInput,
) -> Result<Option<TransactionResponse>, sqlx::Error> {
    sqlx::query_as::<_, TransactionResponse>(
        r#"
        update transactions
        set
            transaction_type = $1,
            date = $2,
            category = $3,
            amount = $4,
            memo = $5,
            status = $6,
            updated_at = now()
        where id = $7
        returning
            id,
            transaction_type,
            date,
            category,
            amount,
            memo,
            status,
            created_at,
            updated_at
        "#,
    )
    .bind(input.transaction_type.as_str())
    .bind(input.date)
    .bind(input.category.as_str())
    .bind(input.amount)
    .bind(input.memo)
    .bind(input.status.as_str())
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn delete_transaction(db: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        delete from transactions
        where id = $1
        "#,
    )
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

fn push_filter_conditions(
    builder: &mut QueryBuilder<Postgres>,
    filter: &TransactionListFilter,
    has_condition: &mut bool,
) -> Result<(), sqlx::Error> {
    if let Some(month) = filter.month {
        let (start_date, end_date) = month.date_range().map_err(sqlx::Error::Protocol)?;

        push_where_or_and(builder, has_condition);
        builder
            .push("date >= ")
            .push_bind(start_date)
            .push(" and date < ")
            .push_bind(end_date);
    }

    if let Some(search_query) = filter.search_query.as_deref() {
        let pattern = format!("%{}%", search_query);

        push_where_or_and(builder, has_condition);

        builder
            .push("(")
            .push("date::text ilike ")
            .push_bind(pattern.clone())
            .push(" or memo ilike ")
            .push_bind(pattern.clone())
            .push(" or transaction_type ilike ")
            .push_bind(pattern.clone())
            .push(" or amount::text ilike ")
            .push_bind(pattern.clone())
            .push(" or category ilike ")
            .push_bind(pattern.clone())
            .push(" or status ilike ")
            .push_bind(pattern.clone())
            .push(
                r#"
                or case transaction_type
                    when 'income' then '収入'
                    when 'expense' then '支出'
                    else transaction_type
                end ilike
                "#,
            )
            .push_bind(pattern.clone())
            .push(
                r#"
                or case category
                    when 'food' then '食費'
                    when 'daily' then '日用品'
                    when 'transport' then '交通費'
                    when 'entertainment' then '娯楽'
                    when 'salary' then '給与'
                    when 'other' then 'その他'
                    else category
                end ilike
                "#,
            )
            .push_bind(pattern.clone())
            .push(
                r#"
                or case status
                    when 'confirmed' then '確定'
                    when 'planned' then '予定'
                    else status
                end ilike
                "#,
            )
            .push_bind(pattern)
            .push(")");
    }

    match filter.status_filter {
        TransactionStatusFilter::Confirmed => {
            push_where_or_and(builder, has_condition);
            builder.push("status = ").push_bind("confirmed");
        }
        TransactionStatusFilter::Planned => {
            push_where_or_and(builder, has_condition);
            builder.push("status = ").push_bind("planned");
        }
        TransactionStatusFilter::All => {}
    }

    Ok(())
}

fn push_sort_order(builder: &mut QueryBuilder<Postgres>, sort_order: TransactionSortOrder) {
    match sort_order {
        TransactionSortOrder::DateDesc => {
            builder.push(" order by date desc, created_at desc");
        }
        TransactionSortOrder::DateAsc => {
            builder.push(" order by date asc, created_at asc");
        }
        TransactionSortOrder::AmountDesc => {
            builder.push(" order by amount desc, date desc, created_at desc");
        }
        TransactionSortOrder::AmountAsc => {
            builder.push(" order by amount asc, date desc, created_at desc");
        }
    }
}

fn push_where_or_and(builder: &mut QueryBuilder<Postgres>, has_condition: &mut bool) {
    if *has_condition {
        builder.push(" and ");
    } else {
        builder.push(" where ");
        *has_condition = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::models::transaction::{
        TransactionCategory, TransactionInput, TransactionListFilter, TransactionSortOrder,
        TransactionStatus, TransactionStatusFilter, TransactionType, YearMonth,
    };

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn expense_input(
        date: NaiveDate,
        category: TransactionCategory,
        amount: i32,
        memo: &str,
    ) -> TransactionInput {
        TransactionInput {
            transaction_type: TransactionType::Expense,
            date,
            category,
            amount,
            memo: memo.to_string(),
            status: TransactionStatus::Confirmed,
        }
    }

    fn income_input(
        date: NaiveDate,
        category: TransactionCategory,
        amount: i32,
        memo: &str,
    ) -> TransactionInput {
        TransactionInput {
            transaction_type: TransactionType::Income,
            date,
            category,
            amount,
            memo: memo.to_string(),
            status: TransactionStatus::Confirmed,
        }
    }

    fn base_filter() -> TransactionListFilter {
        TransactionListFilter {
            month: None,
            search_query: None,
            sort_order: TransactionSortOrder::DateDesc,
            page: 1,
            limit: 10,
            status_filter: TransactionStatusFilter::All,
        }
    }

    async fn insert_test_transaction(
        pool: &PgPool,
        input: TransactionInput,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();

        create_transaction(pool, id, input).await?;

        Ok(id)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_transaction_inserts_row_and_returns_timestamps(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();

        let transaction = create_transaction(
            &pool,
            id,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        assert_eq!(transaction.id, id);
        assert_eq!(transaction.transaction_type, "expense");
        assert_eq!(transaction.date, date(2026, 6, 11));
        assert_eq!(transaction.category, "food");
        assert_eq!(transaction.amount, 1200);
        assert_eq!(transaction.memo, "昼食");
        assert!(transaction.created_at <= transaction.updated_at);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn find_transactions_applies_month_search_sort_and_pagination(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(
                date(2026, 6, 12),
                TransactionCategory::Transport,
                580,
                "電車代",
            ),
        )
        .await?;

        insert_test_transaction(
            &pool,
            income_input(
                date(2026, 6, 25),
                TransactionCategory::Salary,
                250000,
                "給与",
            ),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(
                date(2026, 7, 1),
                TransactionCategory::Food,
                900,
                "7月の昼食",
            ),
        )
        .await?;

        let filter = TransactionListFilter {
            month: Some(YearMonth {
                year: 2026,
                month: 6,
            }),
            search_query: Some("支出".to_string()),
            sort_order: TransactionSortOrder::DateDesc,
            page: 1,
            limit: 1,
            status_filter: TransactionStatusFilter::All,
        };

        let transactions = find_transactions(&pool, &filter).await?;

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].memo, "電車代");
        assert_eq!(transactions[0].date, date(2026, 6, 12));

        let second_page_filter = TransactionListFilter { page: 2, ..filter };

        let second_page_transactions = find_transactions(&pool, &second_page_filter).await?;

        assert_eq!(second_page_transactions.len(), 1);
        assert_eq!(second_page_transactions[0].memo, "昼食");
        assert_eq!(second_page_transactions[0].date, date(2026, 6, 11));

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn count_transactions_applies_filter_conditions(pool: PgPool) -> Result<(), sqlx::Error> {
        insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(
                date(2026, 6, 12),
                TransactionCategory::Transport,
                580,
                "電車代",
            ),
        )
        .await?;

        insert_test_transaction(
            &pool,
            income_input(
                date(2026, 6, 25),
                TransactionCategory::Salary,
                250000,
                "給与",
            ),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(
                date(2026, 7, 1),
                TransactionCategory::Food,
                900,
                "7月の昼食",
            ),
        )
        .await?;

        let filter = TransactionListFilter {
            month: Some(YearMonth {
                year: 2026,
                month: 6,
            }),
            search_query: Some("支出".to_string()),
            sort_order: TransactionSortOrder::DateDesc,
            page: 1,
            limit: 10,
            status_filter: TransactionStatusFilter::All,
        };

        let total = count_transactions(&pool, &filter).await?;

        assert_eq!(total, 2);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn summarize_transactions_returns_total_and_category_summary(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        insert_test_transaction(
            &pool,
            income_input(
                date(2026, 6, 25),
                TransactionCategory::Salary,
                250000,
                "給与",
            ),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 12), TransactionCategory::Food, 800, "夕食"),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 13), TransactionCategory::Daily, 980, "日用品"),
        )
        .await?;

        insert_test_transaction(
            &pool,
            expense_input(
                date(2026, 7, 1),
                TransactionCategory::Food,
                900,
                "7月の昼食",
            ),
        )
        .await?;

        let filter = TransactionListFilter {
            month: Some(YearMonth {
                year: 2026,
                month: 6,
            }),
            search_query: None,
            sort_order: TransactionSortOrder::DateDesc,
            page: 1,
            limit: 10,
            status_filter: TransactionStatusFilter::All,
        };

        let summary = summarize_transactions(&pool, &filter).await?;

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

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_transaction_updates_existing_row(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        let updated_transaction = update_transaction(
            &pool,
            id,
            expense_input(
                date(2026, 6, 12),
                TransactionCategory::Daily,
                2000,
                "更新後メモ",
            ),
        )
        .await?;

        let updated_transaction =
            updated_transaction.expect("updated transaction should be returned");

        assert_eq!(updated_transaction.id, id);
        assert_eq!(updated_transaction.transaction_type, "expense");
        assert_eq!(updated_transaction.date, date(2026, 6, 12));
        assert_eq!(updated_transaction.category, "daily");
        assert_eq!(updated_transaction.amount, 2000);
        assert_eq!(updated_transaction.memo, "更新後メモ");
        assert!(updated_transaction.created_at <= updated_transaction.updated_at);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_transaction_returns_none_when_row_does_not_exist(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let result = update_transaction(
            &pool,
            Uuid::new_v4(),
            expense_input(
                date(2026, 6, 12),
                TransactionCategory::Daily,
                2000,
                "更新後メモ",
            ),
        )
        .await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transaction_deletes_existing_row(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = insert_test_transaction(
            &pool,
            expense_input(date(2026, 6, 11), TransactionCategory::Food, 1200, "昼食"),
        )
        .await?;

        let rows_affected = delete_transaction(&pool, id).await?;

        assert_eq!(rows_affected, 1);

        let total = count_transactions(&pool, &base_filter()).await?;

        assert_eq!(total, 0);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_transaction_returns_zero_when_row_does_not_exist(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let rows_affected = delete_transaction(&pool, Uuid::new_v4()).await?;

        assert_eq!(rows_affected, 0);

        Ok(())
    }
}
