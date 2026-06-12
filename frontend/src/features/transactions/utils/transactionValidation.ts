import type { TransactionCategory, TransactionStatus, TransactionType } from '../types';

type TransactionFormValues = {
    type: TransactionType;
    date: string;
    category: TransactionCategory;
    amount: string;
    memo: string;
    status: TransactionStatus;
};

export type TransactionFormErrors = {
    date?: string;
    amount?: string;
    memo?: string;
    status?: string;
};

export const getTodayDateString = (): string => {
    const now = new Date();

    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');

    return `${year}-${month}-${day}`;
};

export const validateTransactionForm = (
    formValues: TransactionFormValues,
): TransactionFormErrors => {
    const errors: TransactionFormErrors = {};

    if (!formValues.date) {
        errors.date = '日付を入力してください。';
    }

    if (
        formValues.date &&
        formValues.status === 'confirmed' &&
        formValues.date > getTodayDateString()
    ) {
        errors.date = '確定済みの取引に未来日付は入力できません。';
    }

    const amount = Number(formValues.amount);

    if (!formValues.amount) {
        errors.amount = '金額を入力してください。';
    } else if (!Number.isInteger(amount)) {
        errors.amount = '金額は整数で入力してください。';
    } else if (amount <= 0) {
        errors.amount = '金額は1円以上で入力してください。';
    } else if (amount > 10_000_000) {
        errors.amount = '金額は10,000,000円以下で入力してください。';
    }

    if (!formValues.memo.trim()) {
        errors.memo = 'メモを入力してください。';
    } else if (formValues.memo.trim().length > 50) {
        errors.memo = 'メモは50文字以内で入力してください。';
    }

    if (!['confirmed', 'planned'].includes(formValues.status)) {
        errors.status = '状態を選択してください。';
    }

    return errors;
};

export const hasValidationErrors = (errors: TransactionFormErrors): boolean => {
    return Object.values(errors).some(Boolean);
};
