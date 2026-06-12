import type { TransactionCategory } from "../types";
import { transactionCategoryLabels } from "../types";

type TransactionCategoryBadgeProps = {
    category: TransactionCategory;
};

export const TransactionCategoryBadge = ({
    category,
}: TransactionCategoryBadgeProps) => {
    return (
        <span className="transaction-category-badge">
            {transactionCategoryLabels[category]}
        </span>
    );
};