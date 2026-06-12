import type { Transaction } from '../types';
import { transactionCategoryLabels } from '../types';

const transactionTypeLabels = {
    income: '収入',
    expense: '支出',
} as const;

export const filterTransactionsBySearchQuery = (
    transactions: Transaction[],
    searchQuery: string,
): Transaction[] => {
    const normalizedSearchQuery = searchQuery.trim().toLowerCase();

    if (normalizedSearchQuery === '') {
        return transactions;
    }

    return transactions.filter((transaction) => {
        const categoryLabel = transactionCategoryLabels[transaction.category];
        const typeLabel = transactionTypeLabels[transaction.type];

        const searchableText = [
            transaction.date,
            transaction.memo,
            categoryLabel,
            typeLabel,
            String(transaction.amount),
        ]
            .join(' ')
            .toLowerCase();

        return searchableText.includes(normalizedSearchQuery);
    });
};
