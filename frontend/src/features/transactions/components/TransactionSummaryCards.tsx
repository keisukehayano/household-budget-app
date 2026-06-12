import type { TransactionSummary } from '../types';
import { formatCurrency } from '../utils/formatCurrency';

type TransactionSummaryCardsProps = {
    summary: TransactionSummary;
};

export const TransactionSummaryCards = ({ summary }: TransactionSummaryCardsProps) => {
    return (
        <section className="summary-cards">
            <div className="summary-card income">
                <span>収入</span>
                <strong>{formatCurrency(summary.totalIncome)}</strong>
            </div>

            <div className="summary-card expense">
                <span>支出</span>
                <strong>{formatCurrency(summary.totalExpense)}</strong>
            </div>

            <div className="summary-card balance">
                <span>残高</span>
                <strong>{formatCurrency(summary.balance)}</strong>
            </div>
        </section>
    );
};
