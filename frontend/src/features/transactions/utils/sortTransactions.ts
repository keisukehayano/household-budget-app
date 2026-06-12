import type { Transaction } from '../types';

export type TransactionSortOrder = 'date-desc' | 'date-asc' | 'amount-desc' | 'amount-asc';

export const sortTransactions = (
    transaction: Transaction[],
    sortOrder: TransactionSortOrder,
): Transaction[] => {
    const copiedTransactions = [...transaction];

    switch (sortOrder) {
        case 'date-desc':
            return copiedTransactions.sort((a, b) => b.date.localeCompare(a.date));

        case 'amount-asc':
            return copiedTransactions.sort((a, b) => a.date.localeCompare(b.date));

        case 'amount-desc':
            return copiedTransactions.sort((a, b) => b.amount - a.amount);

        case 'date-asc':
            return copiedTransactions.sort((a, b) => a.amount - b.amount);

        default:
            return copiedTransactions;
    }
};
