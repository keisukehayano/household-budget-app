import '../App.css';
import { TransactionsPage } from '../features/transactions';

function App() {
    return (
        <main className="app">
            <h1>家計簿アプリ</h1>
            <TransactionsPage />
        </main>
    );
}

export default App;
