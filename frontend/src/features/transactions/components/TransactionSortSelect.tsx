import type { TransactionSortOrder } from '../utils/sortTransactions';

type TransactionSortSelectProps = {
    sortOrder: TransactionSortOrder;
    onChangeSortOrder: (sortOrder: TransactionSortOrder) => void;
};

export const TransactionSortSelect = ({
    sortOrder,
    onChangeSortOrder,
}: TransactionSortSelectProps) => {
    return (
        <section className="transaction-sort">
            <div className="form-row">
                <label htmlFor="transaction-sort">並び替え</label>
                <select
                    className="transaction-sort"
                    value={sortOrder}
                    onChange={(event) =>
                        onChangeSortOrder(event.target.value as TransactionSortOrder)
                    }
                >
                    <option value="date-desc">日付が新しい順</option>
                    <option value="date-asc">日付が古い順</option>
                    <option value="amount-desc">金額が高い順</option>
                    <option value="amount-asc">金額が低い順</option>
                </select>
            </div>
        </section>
    );
};
