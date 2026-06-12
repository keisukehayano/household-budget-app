import type { Transaction } from "../types";

export const getTransactionMonth = (transaction: Transaction): string => {
  return transaction.date.slice(0, 7);
};

export const filterTransactionsByMonth = (
  transactions: Transaction[],
  selectedMonth: string
): Transaction[] => {
  if (selectedMonth === "") {
    return transactions;
  }

  return transactions.filter(
    (transaction) => getTransactionMonth(transaction) === selectedMonth
  );
};