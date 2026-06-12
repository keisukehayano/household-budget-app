import type { Transaction } from '../types';

export type TransactionSummary = {
    totalIncome: number;
    totalExpense: number;
    balance: number;
};

export const calculateTransactionSummary = (transactions: Transaction[]): TransactionSummary => {
    const summary = transactions.reduce(
        (currentSummary, transaction) => {
            if (transaction.type === 'income') {
                return {
                    ...currentSummary,
                    totalIncome: currentSummary.totalIncome + transaction.amount,
                };
            }

            return {
                ...currentSummary,
                totalExpense: currentSummary.totalExpense + transaction.amount,
            };
        },
        {
            totalIncome: 0,
            totalExpense: 0,
        },
    );

    return {
        totalIncome: summary.totalIncome,
        totalExpense: summary.totalExpense,
        balance: summary.totalIncome - summary.totalExpense,
    };
};
