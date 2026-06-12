import type { Transaction } from '../types';
import { formatCurrency } from '../utils/formatCurrency';
import { formatDateTime } from '../utils/formatDateTime';
import { TransactionCategoryBadge } from './TransactionCategoryBadge';
import { TransactionStatusBadge } from './TransactionStatusBadge';

type TransactionListProps = {
    transactions: Transaction[];
    onDeleteTransaction: (id: string) => Promise<void>;
    onStartEditTransaction: (transaction: Transaction) => void;
    onConfirmTransaction: (transaction: Transaction) => Promise<void>;
    deletingTransactionId: string | null;
    confirmingTransactionId: string | null;
};

const formatTransactionDate = (value: string): string => {
    const date = new Date(`${value}T00:00:00`);

    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return new Intl.DateTimeFormat('ja-JP', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        weekday: 'short',
        timeZone: 'Asia/Tokyo',
    }).format(date);
};

export const TransactionList = ({
    transactions,
    onDeleteTransaction,
    onStartEditTransaction,
    onConfirmTransaction,
    deletingTransactionId,
    confirmingTransactionId,
}: TransactionListProps) => {
    if (transactions.length === 0) {
        return <p className="empty-message">取引がありません。</p>;
    }

    return (
        <section className="transaction-list-section">
            <div className="section-heading">
                <h2>取引一覧</h2>
                <span>{transactions.length}件</span>
            </div>

            <ul className="transaction-list">
                {transactions.map((transaction) => {
                    const isIncome = transaction.type === 'income';
                    const isDeleting = deletingTransactionId === transaction.id;
                    const isConfirming = confirmingTransactionId === transaction.id;
                    const isAnyTransactionDeleting = deletingTransactionId !== null;

                    return (
                        <li key={transaction.id} className="transaction-card">
                            <div className="transaction-card-header">
                                <div className="transaction-date-block">
                                    <span className="transaction-date-label">利用日</span>
                                    <span className="transaction-date">
                                        {formatTransactionDate(transaction.date)}
                                    </span>
                                </div>

                                <div
                                    className={
                                        isIncome
                                            ? 'transaction-amount income'
                                            : 'transaction-amount expense'
                                    }
                                >
                                    {isIncome ? '+' : '-'}
                                    {formatCurrency(transaction.amount)}
                                </div>
                            </div>

                            <div className="transaction-card-body">
                                <div className="transaction-main">
                                    <TransactionStatusBadge status={transaction.status} />
                                    <TransactionCategoryBadge category={transaction.category} />
                                    <span className="transaction-memo">{transaction.memo}</span>
                                </div>

                                <div className="transaction-timestamps">
                                    <span>登録: {formatDateTime(transaction.createdAt)}</span>
                                    <span>更新: {formatDateTime(transaction.updatedAt)}</span>
                                </div>
                            </div>

                            <div className="transaction-card-footer">
                                {transaction.status === 'planned' && (
                                    <button
                                        type="button"
                                        className="transaction-confirm-button"
                                        disabled={isConfirming}
                                        onClick={() => void onConfirmTransaction(transaction)}
                                    >
                                        {isConfirming ? '確定中...' : '確定する'}
                                    </button>
                                )}

                                <button
                                    type="button"
                                    className="transaction-edit-button"
                                    disabled={isAnyTransactionDeleting}
                                    onClick={() => onStartEditTransaction(transaction)}
                                >
                                    編集
                                </button>

                                <button
                                    type="button"
                                    className="transaction-delete-button"
                                    disabled={isAnyTransactionDeleting}
                                    onClick={() => void onDeleteTransaction(transaction.id)}
                                >
                                    {isDeleting ? '削除中...' : '削除'}
                                </button>
                            </div>
                        </li>
                    );
                })}
            </ul>
        </section>
    );
};
