import { useCallback, useEffect, useState } from 'react';
import '../App.css';
import {
    AuthPage,
    PasswordChangeForm,
    ResetPasswordPage,
    clearAuthSession,
    fetchMe,
    loadAuthSession,
    saveAuthSession,
    type AuthSession,
} from '../features/auth';
import { TransactionsPage } from '../features/transactions';

function App() {
    const [authSession, setAuthSession] = useState<AuthSession | null>(() => loadAuthSession());
    const [isCheckingAuth, setIsCheckingAuth] = useState(false);
    const [isPasswordChangeOpen, setIsPasswordChangeOpen] = useState(false);
    const [appMessage, setAppMessage] = useState('');
    const authToken = authSession?.token ?? null;
    const isResetPasswordRoute = window.location.pathname === '/reset-password';

    const handleAuthenticated = useCallback((session: AuthSession) => {
        saveAuthSession(session);
        setAuthSession(session);
        setAppMessage('');
    }, []);

    const handleLogout = useCallback(() => {
        clearAuthSession();
        setAuthSession(null);
        setIsPasswordChangeOpen(false);
        setAppMessage('');
    }, []);

    const handleGoToLogin = useCallback(() => {
        clearAuthSession();
        setAuthSession(null);
        setIsPasswordChangeOpen(false);
        setAppMessage('');
        window.history.replaceState(null, '', '/');
    }, []);

    useEffect(() => {
        if (authToken === null) {
            return;
        }

        let isActive = true;

        const checkAuth = async () => {
            try {
                setIsCheckingAuth(true);

                const user = await fetchMe(authToken);

                if (!isActive) {
                    return;
                }

                const refreshedSession: AuthSession = {
                    token: authToken,
                    user,
                };

                saveAuthSession(refreshedSession);
                setAuthSession(refreshedSession);
            } catch (error) {
                console.error(error);

                if (!isActive) {
                    return;
                }

                clearAuthSession();
                setAuthSession(null);
            } finally {
                if (isActive) {
                    setIsCheckingAuth(false);
                }
            }
        };

        void checkAuth();

        return () => {
            isActive = false;
        };
    }, [authToken]);

    if (isResetPasswordRoute) {
        return (
            <main className="app">
                <h1>家計簿アプリ</h1>
                <ResetPasswordPage onCompleted={handleGoToLogin} />
            </main>
        );
    }

    if (isCheckingAuth) {
        return (
            <main className="app">
                <h1>家計簿アプリ</h1>
                <p className="page-loading">ログイン状態を確認中です...</p>
            </main>
        );
    }

    if (authSession === null) {
        return (
            <main className="app">
                <h1>家計簿アプリ</h1>
                <AuthPage onAuthenticated={handleAuthenticated} />
            </main>
        );
    }

    return (
        <main className="app">
            <header className="app-header">
                <div>
                    <h1>家計簿アプリ</h1>
                    <p>{authSession.user.email}</p>
                </div>

                <div className="app-header-actions">
                    <button
                        type="button"
                        className="password-change-open-button"
                        onClick={() => {
                            setIsPasswordChangeOpen((currentValue) => !currentValue);
                            setAppMessage('');
                        }}
                    >
                        パスワード変更
                    </button>

                    <button type="button" className="logout-button" onClick={handleLogout}>
                        ログアウト
                    </button>
                </div>
            </header>

            {appMessage && <p className="page-loading">{appMessage}</p>}

            {isPasswordChangeOpen && (
                <PasswordChangeForm
                    token={authSession.token}
                    onUnauthorized={handleLogout}
                    onCancel={() => setIsPasswordChangeOpen(false)}
                    onChanged={(session) => {
                        saveAuthSession(session);
                        setAuthSession(session);
                        setIsPasswordChangeOpen(false);
                        setAppMessage(
                            'パスワードを変更しました。既存のログイン状態は失効されました。',
                        );
                    }}
                />
            )}

            <TransactionsPage token={authSession.token} onUnauthorized={handleLogout} />
        </main>
    );
}

export default App;
