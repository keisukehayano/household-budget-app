import type { Transaction } from "../types";

const TRANSACTION_STORAGE_KEY = "household-account-book:transactions";

export const loadTransactions = (): Transaction[] | null => {
    try {
        const storedTransactions = localStorage.getItem(TRANSACTION_STORAGE_KEY);

        if (storedTransactions === null) {
            return null;
        }

        const parsedTransactions: unknown = JSON.parse(storedTransactions);

        if (!Array.isArray(parsedTransactions)) {
            return null;
        }

        return parsedTransactions as Transaction[];
    } catch (error) {
        console.error("Failed to load transactions from localStorage.", error);
        return null;
    }
};

export const saveTransactions = (transactions: Transaction[]): void => {
    try {
        localStorage.setItem(
            TRANSACTION_STORAGE_KEY,
            JSON.stringify(transactions)
        );
    } catch (error) {
        console.error("Failed to save transactions to localStorage.", error);
    }
};