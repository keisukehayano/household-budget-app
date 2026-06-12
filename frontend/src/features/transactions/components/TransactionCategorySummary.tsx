import type { TransactionCategorySummaryItem } from '../types';
import { transactionCategoryLabels } from '../types';
import { formatCurrency } from '../utils/formatCurrency';

type TransactionCategorySummaryProps = {
    categorySummaries: TransactionCategorySummaryItem[];
};

export const TransactionCategorySummary = ({
    categorySummaries,
}: TransactionCategorySummaryProps) => {
    if (categorySummaries.length === 0) {
        return (
            <section className="category-summary">
                <h2>カテゴリ別支出</h2>
                <p>支出データがありません。</p>
            </section>
        );
    }

    return (
        <section className="category-summary">
            <h2>カテゴリ別支出</h2>

            <ul>
                {categorySummaries.map((summary) => (
                    <li key={summary.category}>
                        <span>{transactionCategoryLabels[summary.category]}</span>
                        <strong>{formatCurrency(summary.total)}</strong>
                    </li>
                ))}
            </ul>
        </section>
    );
};
