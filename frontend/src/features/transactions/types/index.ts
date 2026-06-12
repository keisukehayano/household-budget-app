export type TransactionType = 'income' | 'expense';

export type TransactionCategory =
    | 'food'
    | 'daily'
    | 'transport'
    | 'entertainment'
    | 'salary'
    | 'other';

export type TransactionStatus = 'confirmed' | 'planned';

export type TransactionStatusFilter = 'all' | TransactionStatus;

export type Transaction = {
    id: string;
    type: TransactionType;
    date: string;
    category: TransactionCategory;
    amount: number;
    memo: string;
    status: TransactionStatus;
    createdAt: string;
    updatedAt: string;
};

export type TransactionCreateInput = Omit<Transaction, 'id' | 'createdAt' | 'updatedAt'>;

export type TransactionPagination = {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrevious: boolean;
};

export type TransactionListResponse = {
    items: Transaction[];
    pagination: TransactionPagination;
};

export type TransactionCategorySummaryItem = {
    category: TransactionCategory;
    total: number;
};

export type TransactionSummary = {
    totalIncome: number;
    totalExpense: number;
    balance: number;
    categorySummaries: TransactionCategorySummaryItem[];
};

export const transactionCategoryLabels: Record<TransactionCategory, string> = {
    food: '食費',
    daily: '日用品',
    transport: '交通費',
    entertainment: '娯楽',
    salary: '給与',
    other: 'その他',
};

export const transactionStatusLabels: Record<TransactionStatus, string> = {
    confirmed: '確定済み',
    planned: '予定',
};

export const transactionStatusFilterLabels: Record<TransactionStatusFilter, string> = {
    all: 'すべて',
    confirmed: '確定済み',
    planned: '予定',
};
