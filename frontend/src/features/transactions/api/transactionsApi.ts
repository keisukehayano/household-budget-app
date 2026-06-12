import type {
    Transaction,
    TransactionCreateInput,
    TransactionListResponse,
    TransactionStatusFilter,
    TransactionSummary,
} from '../types';
import type { TransactionSortOrder } from '../utils/sortTransactions';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080';

type ApiErrorResponse = {
    message: string;
    details?: string[];
};

export type FetchTransactionsParams = {
    month?: string;
    q?: string;
    sort?: TransactionSortOrder;
    page?: number;
    limit?: number;
    status?: TransactionStatusFilter;
};

const createApiErrorMessage = (errorResponse: ApiErrorResponse): string => {
    if (errorResponse.details && errorResponse.details.length > 0) {
        return `${errorResponse.message}\n${errorResponse.details.join('\n')}`;
    }

    return errorResponse.message;
};

const handleResponse = async <T>(response: Response): Promise<T> => {
    if (!response.ok) {
        let errorMessage = `API request failed: ${response.status}`;

        try {
            const errorResponse = (await response.json()) as ApiErrorResponse;
            errorMessage = createApiErrorMessage(errorResponse);
        } catch {
            // JSON形式ではないエラーの場合は、デフォルトメッセージを使う。
        }

        throw new Error(errorMessage);
    }

    return response.json() as Promise<T>;
};

const buildTransactionsUrl = (params?: FetchTransactionsParams): string => {
    const url = new URL(`${API_BASE_URL}/api/transactions`);

    if (params?.month) {
        url.searchParams.set('month', params.month);
    }

    if (params?.q) {
        url.searchParams.set('q', params.q);
    }

    if (params?.sort) {
        url.searchParams.set('sort', params.sort);
    }

    if (params?.page !== undefined) {
        url.searchParams.set('page', String(params.page));
    }

    if (params?.limit !== undefined) {
        url.searchParams.set('limit', String(params.limit));
    }

    if (params?.status) {
        url.searchParams.set('status', params.status);
    }

    return url.toString();
};

export const fetchTransactions = async (
    params?: FetchTransactionsParams,
): Promise<TransactionListResponse> => {
    const response = await fetch(buildTransactionsUrl(params));

    return handleResponse<TransactionListResponse>(response);
};

export const createTransaction = async (
    transaction: TransactionCreateInput,
): Promise<Transaction> => {
    const response = await fetch(`${API_BASE_URL}/api/transactions`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(transaction),
    });

    return handleResponse<Transaction>(response);
};

export const updateTransaction = async (transaction: Transaction): Promise<Transaction> => {
    const response = await fetch(`${API_BASE_URL}/api/transactions/${transaction.id}`, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            type: transaction.type,
            date: transaction.date,
            category: transaction.category,
            amount: transaction.amount,
            memo: transaction.memo,
            status: transaction.status,
        }),
    });

    return handleResponse<Transaction>(response);
};

export const deleteTransaction = async (id: string): Promise<void> => {
    const response = await fetch(`${API_BASE_URL}/api/transactions/${id}`, {
        method: 'DELETE',
    });

    if (!response.ok) {
        let errorMessage = `API request failed: ${response.status}`;

        try {
            const errorResponse = (await response.json()) as ApiErrorResponse;
            errorMessage = createApiErrorMessage(errorResponse);
        } catch {
            // JSON形式ではないエラーの場合は、デフォルトメッセージを使う。
        }

        throw new Error(errorMessage);
    }
};

export type FetchTransactionSummaryParams = {
    month?: string;
    q?: string;
    status?: TransactionStatusFilter;
};

const buildTransactionSummaryUrl = (params?: FetchTransactionSummaryParams): string => {
    const url = new URL(`${API_BASE_URL}/api/transactions/summary`);

    if (params?.month) {
        url.searchParams.set('month', params.month);
    }

    if (params?.q) {
        url.searchParams.set('q', params.q);
    }

    if (params?.status) {
        url.searchParams.set('status', params.status);
    }

    return url.toString();
};

export const fetchTransactionSummary = async (
    params?: FetchTransactionSummaryParams,
): Promise<TransactionSummary> => {
    const response = await fetch(buildTransactionSummaryUrl(params));

    return handleResponse<TransactionSummary>(response);
};
