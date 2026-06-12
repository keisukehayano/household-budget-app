type TransactionSearchInputProps = {
    searchQuery: string;
    onChangeSearchQuery: (searchQuery: string) => void;
};

export const TransactionSearchInput = ({
    searchQuery,
    onChangeSearchQuery,
}: TransactionSearchInputProps) => {
    return (
        <section className="transaction-search">
            <div className="form-row">
                <label htmlFor="transaction-search">検索</label>
                <input
                    id="transaction-search"
                    type="search"
                    value={searchQuery}
                    onChange={(event) => onChangeSearchQuery(event.target.value)}
                    placeholder="メモ・カテゴリ・金額で検索"
                />
            </div>

            {searchQuery !== '' && (
                <button type="button" onClick={() => onChangeSearchQuery('')}>
                    クリア
                </button>
            )}
        </section>
    );
};
