import type { TransactionStatus } from '../types';
import { transactionStatusLabels } from '../types';

type TransactionStatusBadgeProps = {
    status: TransactionStatus;
};

export const TransactionStatusBadge = ({ status }: TransactionStatusBadgeProps) => {
    return (
        <span className={`transaction-status-badge ${status}`}>
            {transactionStatusLabels[status]}
        </span>
    );
};
