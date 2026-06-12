import type { TransactionStatusFilter as TransactionStatusFilterType } from '../types';
import { transactionStatusFilterLabels } from '../types';

type TransactionStatusFilterProps = {
    selectedStatus: TransactionStatusFilterType;
    onChangeStatus: (status: TransactionStatusFilterType) => void;
};

export const TransactionStatusFilter = ({
    selectedStatus,
    onChangeStatus,
}: TransactionStatusFilterProps) => {
    return (
        <section className="transaction-status-filter">
            <div className="form-row">
                <label className="transaction-status-filter">状態</label>
                <select
                    id="transaction-status-filter"
                    value={selectedStatus}
                    onChange={(event) =>
                        onChangeStatus(event.target.value as TransactionStatusFilterType)
                    }
                >
                    {Object.entries(transactionStatusFilterLabels).map(([value, label]) => (
                        <option key={value} value={value}>
                            {label}
                        </option>
                    ))}
                </select>
            </div>
        </section>
    );
};
