import { useEffect, useState } from 'react';
import {
    createTransaction,
    deleteTransaction,
    fetchTransactionSummary,
    fetchTransactions,
    updateTransaction,
} from '../api/transactionsApi';
import { TransactionCategorySummary } from '../components/TransactionCategorySummary';
import { TransactionForm } from '../components/TransactionForm';
import { TransactionList } from '../components/TransactionList';
import { TransactionMonthFilter } from '../components/TransactionMonthFilter';
import { TransactionPagination } from '../components/TransactionPagination';
import { TransactionSearchInput } from '../components/TransactionSearchInput';
import { TransactionSortSelect } from '../components/TransactionSortSelect';
import { TransactionSummaryCards } from '../components/TransactionSummaryCards';
import { TransactionStatusFilter } from '../components/TransactionStatusFilter';
import type {
    Transaction,
    TransactionCreateInput,
    TransactionPagination as TransactionPaginationType,
    TransactionStatusFilter as TransactionStatusFilterType,
    TransactionSummary,
} from '../types';
import type { TransactionSortOrder } from '../utils/sortTransactions';
import { getTodayDateString } from '../utils/transactionValidation';
import { useDebouncedValue } from '../../../shared/hooks/useDebouncedValue';

const PAGE_SIZE = 10;

const emptyTransactionSummary: TransactionSummary = {
    totalIncome: 0,
    totalExpense: 0,
    balance: 0,
    categorySummaries: [],
};

export const TransactionsPage = () => {
    const [transactions, setTransactions] = useState<Transaction[]>([]);
    const [pagination, setPagination] = useState<TransactionPaginationType | null>(null);
    const [transactionSummary, setTransactionSummary] =
        useState<TransactionSummary>(emptyTransactionSummary);

    const [selectedMonth, setSelectedMonth] = useState('');
    const [searchQuery, setSearchQuery] = useState('');
    const [sortOrder, setSortOrder] = useState<TransactionSortOrder>('date-desc');
    const [currentPage, setCurrentPage] = useState(1);

    const [editingTransaction, setEditingTransaction] = useState<Transaction | null>(null);

    const [isLoading, setIsLoading] = useState(true);
    const [isFetching, setIsFetching] = useState(false);
    const [isSummaryFetching, setIsSummaryFetching] = useState(false);
    const [isFormSubmitting, setIsFormSubmitting] = useState(false);
    const [deletingTransactionId, setDeletingTransactionId] = useState<string | null>(null);
    const [confirmingTransactionId, setConfirmingTransactionId] = useState<string | null>(null);
    const [reloadKey, setReloadKey] = useState(0);
    const [errorMessage, setErrorMessage] = useState('');

    const [selectedStatus, setSelectedStatus] = useState<TransactionStatusFilterType>('all');

    const debouncedSearchQuery = useDebouncedValue(searchQuery, 500);
    const isSearchPending = searchQuery !== debouncedSearchQuery;

    useEffect(() => {
        let isActive = true;

        const loadTransactions = async () => {
            try {
                setIsFetching(true);
                setErrorMessage('');

                const response = await fetchTransactions({
                    month: selectedMonth,
                    q: debouncedSearchQuery,
                    sort: sortOrder,
                    page: currentPage,
                    limit: PAGE_SIZE,
                    status: selectedStatus,
                });

                if (!isActive) {
                    return;
                }

                if (response.pagination.totalPages === 0 && currentPage !== 1) {
                    setCurrentPage(1);
                    return;
                }

                if (
                    response.pagination.totalPages > 0 &&
                    currentPage > response.pagination.totalPages
                ) {
                    setCurrentPage(response.pagination.totalPages);
                    return;
                }

                setTransactions(response.items);
                setPagination(response.pagination);
            } catch (error) {
                console.error(error);

                if (!isActive) {
                    return;
                }

                setErrorMessage(
                    error instanceof Error ? error.message : '取引データの取得に失敗しました。',
                );
            } finally {
                if (isActive) {
                    setIsLoading(false);
                    setIsFetching(false);
                }
            }
        };

        void loadTransactions();

        return () => {
            isActive = false;
        };
    }, [selectedMonth, debouncedSearchQuery, sortOrder, currentPage, selectedStatus, reloadKey]);

    useEffect(() => {
        let isActive = true;

        const loadTransactionSummary = async () => {
            try {
                setIsSummaryFetching(true);
                setErrorMessage('');

                const summary = await fetchTransactionSummary({
                    month: selectedMonth,
                    q: debouncedSearchQuery,
                    status: selectedStatus,
                });

                if (!isActive) {
                    return;
                }

                setTransactionSummary(summary);
            } catch (error) {
                console.error(error);

                if (!isActive) {
                    return;
                }

                setErrorMessage(
                    error instanceof Error ? error.message : '集計データの取得に失敗しました。',
                );
            } finally {
                if (isActive) {
                    setIsSummaryFetching(false);
                }
            }
        };

        void loadTransactionSummary();

        return () => {
            isActive = false;
        };
    }, [selectedMonth, debouncedSearchQuery, selectedStatus, reloadKey]);

    const reloadTransactions = () => {
        setReloadKey((currentReloadKey) => currentReloadKey + 1);
    };

    const handleChangeMonth = (month: string) => {
        setSelectedMonth(month);
        setCurrentPage(1);
    };

    const handleChangeSearchQuery = (query: string) => {
        setSearchQuery(query);
        setCurrentPage(1);
    };

    const handleChangeSortOrder = (order: TransactionSortOrder) => {
        setSortOrder(order);
        setCurrentPage(1);
    };

    const handleChangeStatus = (status: TransactionStatusFilterType) => {
        setSelectedStatus(status);
        setCurrentPage(1);
    };

    const handleChangePage = (page: number) => {
        setCurrentPage(page);
        window.scrollTo({ top: 0, behavior: 'smooth' });
    };

    const handleAddTransaction = async (transaction: TransactionCreateInput): Promise<void> => {
        try {
            setIsFormSubmitting(true);
            setErrorMessage('');

            const createdTransaction = await createTransaction(transaction);

            setSelectedStatus(createdTransaction.status);
            setSelectedMonth(createdTransaction.date.slice(0, 7));
            setSearchQuery('');
            setSortOrder('date-desc');
            setCurrentPage(1);
            reloadTransactions();
        } catch (error) {
            console.error(error);
            setErrorMessage(error instanceof Error ? error.message : '取引の登録に失敗しました。');
            throw error;
        } finally {
            setIsFormSubmitting(false);
        }
    };

    const handleUpdateTransaction = async (updatedTransaction: Transaction): Promise<void> => {
        try {
            setIsFormSubmitting(true);
            setErrorMessage('');

            const savedTransaction = await updateTransaction(updatedTransaction);

            setEditingTransaction(null);
            setSelectedStatus(savedTransaction.status);
            setSelectedMonth(savedTransaction.date.slice(0, 7));
            setSearchQuery('');
            setCurrentPage(1);
            reloadTransactions();
        } catch (error) {
            console.error(error);
            setErrorMessage(error instanceof Error ? error.message : '取引の更新に失敗しました。');
            throw error;
        } finally {
            setIsFormSubmitting(false);
        }
    };

    const handleStartEditTransaction = (transaction: Transaction) => {
        setEditingTransaction(transaction);
        window.scrollTo({ top: 0, behavior: 'smooth' });
    };

    const handleCancelEdit = () => {
        setEditingTransaction(null);
    };

    const handleDeleteTransaction = async (id: string): Promise<void> => {
        const shouldDelete = window.confirm('この取引を削除しますか？');

        if (!shouldDelete) {
            return;
        }

        try {
            setDeletingTransactionId(id);
            setErrorMessage('');

            await deleteTransaction(id);

            setEditingTransaction((currentEditingTransaction) => {
                if (currentEditingTransaction?.id === id) {
                    return null;
                }

                return currentEditingTransaction;
            });

            reloadTransactions();
        } catch (error) {
            console.error(error);
            setErrorMessage(error instanceof Error ? error.message : '取引の削除に失敗しました。');
        } finally {
            setDeletingTransactionId(null);
        }
    };

    const handleConfirmTransaction = async (transaction: Transaction): Promise<void> => {
        if (transaction.status !== 'planned') {
            return;
        }

        const today = getTodayDateString();
        const isFuturePlannedTransaction = transaction.date > today;

        const shouldConfirm = isFuturePlannedTransaction
            ? window.confirm(
                  'この予定取引は未来日付です。\n日付を今日に変更して確定済みにしますか？',
              )
            : window.confirm('この予定取引を確定済みにしますか？');

        if (!shouldConfirm) {
            return;
        }

        try {
            setConfirmingTransactionId(transaction.id);
            setErrorMessage('');

            const confirmedTransaction = await updateTransaction({
                ...transaction,
                date: isFuturePlannedTransaction ? today : transaction.date,
                status: 'confirmed',
            });

            setEditingTransaction((currentEditingTransaction) => {
                if (currentEditingTransaction?.id === transaction.id) {
                    return null;
                }

                return currentEditingTransaction;
            });

            setSelectedStatus('confirmed');
            setSelectedMonth(confirmedTransaction.date.slice(0, 7));
            setCurrentPage(1);
            reloadTransactions();
        } catch (error) {
            console.error(error);
            setErrorMessage(
                error instanceof Error ? error.message : '予定取引の確定に失敗しました。',
            );
        } finally {
            setConfirmingTransactionId(null);
        }
    };

    if (isLoading) {
        return <p>読み込み中です...</p>;
    }

    return (
        <div className="transactions-page">
            {errorMessage && <p className="page-error">{errorMessage}</p>}

            {(isFetching || isSummaryFetching || isSearchPending) && (
                <p className="page-loading">
                    {isSearchPending ? '検索条件を反映中です...' : '取引データを更新中です...'}
                </p>
            )}

            <section className="transactions-controls">
                <TransactionStatusFilter
                    selectedStatus={selectedStatus}
                    onChangeStatus={handleChangeStatus}
                />

                <TransactionMonthFilter
                    selectedMonth={selectedMonth}
                    onChangeMonth={handleChangeMonth}
                />

                <TransactionSearchInput
                    searchQuery={searchQuery}
                    onChangeSearchQuery={handleChangeSearchQuery}
                />

                <TransactionSortSelect
                    sortOrder={sortOrder}
                    onChangeSortOrder={handleChangeSortOrder}
                />
            </section>

            <section className="transactions-dashboard">
                <TransactionSummaryCards summary={transactionSummary} />

                <TransactionCategorySummary
                    categorySummaries={transactionSummary.categorySummaries}
                />
            </section>

            <section className="transactions-main">
                <div className="transactions-form-area">
                    <TransactionForm
                        key={editingTransaction?.id ?? 'new-transaction'}
                        onAddTransaction={handleAddTransaction}
                        editingTransaction={editingTransaction}
                        onUpdateTransaction={handleUpdateTransaction}
                        onCancelEdit={handleCancelEdit}
                        isSubmitting={isFormSubmitting}
                    />
                </div>

                <div className="transactions-list-area">
                    <TransactionList
                        transactions={transactions}
                        onDeleteTransaction={handleDeleteTransaction}
                        onStartEditTransaction={handleStartEditTransaction}
                        onConfirmTransaction={handleConfirmTransaction}
                        deletingTransactionId={deletingTransactionId}
                        confirmingTransactionId={confirmingTransactionId}
                    />

                    <TransactionPagination
                        pagination={pagination}
                        isDisabled={isFetching || deletingTransactionId !== null}
                        onChangePage={handleChangePage}
                    />
                </div>
            </section>
        </div>
    );
};
