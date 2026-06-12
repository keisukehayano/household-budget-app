import type { TransactionPagination as TransactionPaginationType } from '../types';

type TransactionPaginationProps = {
    pagination: TransactionPaginationType | null;
    isDisabled: boolean;
    onChangePage: (page: number) => void;
};

export const TransactionPagination = ({
    pagination,
    isDisabled,
    onChangePage,
}: TransactionPaginationProps) => {
    if (pagination === null || pagination.totalPages <= 1) {
        return null;
    }

    return (
        <nav className="transaction-pagination" aria-label="取引一覧ページネーション">
            <button
                type="button"
                disabled={!pagination.hasPrevious || isDisabled}
                onClick={() => onChangePage(pagination.page - 1)}
            >
                前へ
            </button>

            <span>
                {pagination.page} / {pagination.totalPages} ページ（全
                {pagination.total}件）
            </span>

            <button
                type="button"
                disabled={!pagination.hasNext || isDisabled}
                onClick={() => onChangePage(pagination.page + 1)}
            >
                次へ
            </button>
        </nav>
    );
};
