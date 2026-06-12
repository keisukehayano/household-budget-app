type TransactionMonthFilterProps = {
    selectedMonth: string;
    onChangeMonth: (month: string) => void;
};

export const TransactionMonthFilter = ({
    selectedMonth,
    onChangeMonth,
}: TransactionMonthFilterProps) => {
    return (
        <section className="transaction-filter">
            <div className="form-row">
                <label htmlFor="transaction-month">表示する月</label>
                <input
                    id="transaction-month"
                    type="month"
                    value={selectedMonth}
                    onChange={(event) => onChangeMonth(event.target.value)}
                    />
            </div>

            <button type="button" onClick={() => onChangeMonth("")}>
                すべて表示
            </button>
        </section>
    );
};