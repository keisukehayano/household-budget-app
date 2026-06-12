import { useState } from 'react';
import type { FormEvent } from 'react';
import type {
    Transaction,
    TransactionCategory,
    TransactionCreateInput,
    TransactionStatus,
    TransactionType,
} from '../types';
import { transactionCategoryLabels, transactionStatusLabels } from '../types';
import {
    getTodayDateString,
    hasValidationErrors,
    validateTransactionForm,
    type TransactionFormErrors,
} from '../utils/transactionValidation';

type TransactionFormProps = {
    onAddTransaction: (transaction: TransactionCreateInput) => Promise<void>;
    editingTransaction: Transaction | null;
    onUpdateTransaction: (transaction: Transaction) => Promise<void>;
    onCancelEdit: () => void;
    isSubmitting: boolean;
};

type TransactionFormState = {
    type: TransactionType;
    date: string;
    category: TransactionCategory;
    amount: string;
    memo: string;
    status: TransactionStatus;
};

const createInitialFormState = (editingTransaction: Transaction | null): TransactionFormState => {
    if (editingTransaction !== null) {
        return {
            type: editingTransaction.type,
            date: editingTransaction.date,
            category: editingTransaction.category,
            amount: String(editingTransaction.amount),
            memo: editingTransaction.memo,
            status: editingTransaction.status,
        };
    }

    return {
        type: 'expense',
        date: '',
        category: 'food',
        amount: '',
        memo: '',
        status: 'confirmed',
    };
};

export const TransactionForm = ({
    onAddTransaction,
    editingTransaction,
    onUpdateTransaction,
    onCancelEdit,
    isSubmitting,
}: TransactionFormProps) => {
    const [formState, setFormState] = useState<TransactionFormState>(() =>
        createInitialFormState(editingTransaction),
    );

    const [errors, setErrors] = useState<TransactionFormErrors>({});

    const isEditMode = editingTransaction !== null;

    const resetForm = () => {
        setFormState(createInitialFormState(null));
        setErrors({});
    };

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        const validationErrors = validateTransactionForm(formState);

        if (hasValidationErrors(validationErrors)) {
            setErrors(validationErrors);
            return;
        }

        const parsedAmount = Number(formState.amount);

        try {
            if (editingTransaction !== null) {
                const updatedTransaction: Transaction = {
                    ...editingTransaction,
                    type: formState.type,
                    date: formState.date,
                    category: formState.category,
                    amount: parsedAmount,
                    memo: formState.memo.trim(),
                    status: formState.status,
                };

                await onUpdateTransaction(updatedTransaction);
                resetForm();
                return;
            }

            const newTransaction: TransactionCreateInput = {
                type: formState.type,
                date: formState.date,
                category: formState.category,
                amount: parsedAmount,
                memo: formState.memo.trim(),
                status: formState.status,
            };

            await onAddTransaction(newTransaction);
            resetForm();
        } catch {
            // エラー表示は親コンポーネント側で行う。
            // 失敗時はフォーム内容を維持する。
        }
    };

    const handleCancelEdit = () => {
        resetForm();
        onCancelEdit();
    };

    return (
        <form onSubmit={handleSubmit} className="transaction-form" noValidate>
            <h2>{isEditMode ? '取引を編集' : '取引を登録'}</h2>

            <div className="form-row">
                <label htmlFor="status">状態</label>
                <select
                    id="status"
                    value={formState.status}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            status: event.target.value as TransactionStatus,
                        }))
                    }
                    aria-invalid={errors.status ? 'true' : 'false'}
                >
                    {Object.entries(transactionStatusLabels).map(([value, label]) => (
                        <option key={value} value={value}>
                            {label}
                        </option>
                    ))}
                </select>
                {errors.status && <p className="form-error">{errors.status}</p>}
            </div>

            <div className="form-row">
                <label htmlFor="type">種類</label>
                <select
                    id="type"
                    value={formState.type}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            type: event.target.value as TransactionType,
                        }))
                    }
                >
                    <option value="expense">支出</option>
                    <option value="income">収入</option>
                </select>
            </div>

            <div className="form-row">
                <label htmlFor="date">日付</label>
                <input
                    id="date"
                    type="date"
                    value={formState.date}
                    max={formState.status === 'confirmed' ? getTodayDateString() : undefined}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            date: event.target.value,
                        }))
                    }
                    aria-invalid={errors.date ? 'true' : 'false'}
                />
                {errors.date && <p className="form-error">{errors.date}</p>}
            </div>

            <div className="form-row">
                <label htmlFor="category">カテゴリ</label>
                <select
                    id="category"
                    value={formState.category}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            category: event.target.value as TransactionCategory,
                        }))
                    }
                >
                    {Object.entries(transactionCategoryLabels).map(([value, label]) => (
                        <option key={value} value={value}>
                            {label}
                        </option>
                    ))}
                </select>
            </div>

            <div className="form-row">
                <label htmlFor="amount">金額</label>
                <input
                    id="amount"
                    type="number"
                    min="1"
                    max="10000000"
                    value={formState.amount}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            amount: event.target.value,
                        }))
                    }
                    aria-invalid={errors.amount ? 'true' : 'false'}
                />
                {errors.amount && <p className="form-error">{errors.amount}</p>}
            </div>

            <div className="form-row">
                <label htmlFor="memo">メモ</label>
                <input
                    id="memo"
                    type="text"
                    value={formState.memo}
                    maxLength={50}
                    disabled={isSubmitting}
                    onChange={(event) =>
                        setFormState((currentFormState) => ({
                            ...currentFormState,
                            memo: event.target.value,
                        }))
                    }
                    placeholder="例：昼食"
                    aria-invalid={errors.memo ? 'true' : 'false'}
                />
                <div className="form-help">{formState.memo.length}/50文字</div>
                {errors.memo && <p className="form-error">{errors.memo}</p>}
            </div>

            <div className="form-actions">
                <button type="submit" disabled={isSubmitting}>
                    {isSubmitting
                        ? isEditMode
                            ? '更新中...'
                            : '登録中...'
                        : isEditMode
                          ? '更新する'
                          : '登録する'}
                </button>

                {isEditMode && (
                    <button type="button" onClick={handleCancelEdit} disabled={isSubmitting}>
                        キャンセル
                    </button>
                )}
            </div>
        </form>
    );
};
