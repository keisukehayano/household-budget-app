import type { Transaction, TransactionCategory } from '../types';
import { transactionCategoryLabels } from '../types';

export type CategorySummaryItem = {
    category: TransactionCategory;
    label: string;
    amount: number;
};

export const calculateCategorySummary = (transactions: Transaction[]): CategorySummaryItem[] => {
    const expenseTransactions = transactions.filter(
        (transaction) => transaction.type === 'expense',
    );

    const summaryMap = expenseTransactions.reduce<Partial<Record<TransactionCategory, number>>>(
        (currentSummary, transaction) => {
            const currentAmount = currentSummary[transaction.category] ?? 0;

            return {
                ...currentSummary,
                [transaction.category]: currentAmount + transaction.amount,
            };
        },
        {},
    );

    return Object.entries(summaryMap)
        .map(([category, amount]) => ({
            category: category as TransactionCategory,
            label: transactionCategoryLabels[category as TransactionCategory],
            amount,
        }))
        .sort((a, b) => b.amount - a.amount);
};
