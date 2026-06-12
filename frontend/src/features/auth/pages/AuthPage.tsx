import { useState } from 'react';
import type { FormEvent } from 'react';
import { login, register } from '../api/authApi';
import type { AuthSession } from '../types';

type AuthMode = 'login' | 'register';

type AuthPageProps = {
    onAuthenticated: (session: AuthSession) => void;
};

export const AuthPage = ({ onAuthenticated }: AuthPageProps) => {
    const [mode, setMode] = useState<AuthMode>('login');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [passwordConfirmation, setPasswordConfirmation] = useState('');
    const [errorMessage, setErrorMessage] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    const isLoginMode = mode === 'login';

    const validateForm = (): string | null => {
        if (!email.trim()) {
            return 'メールアドレスを入力してください。';
        }

        if (!password) {
            return 'パスワードを入力してください。';
        }

        if (!isLoginMode && !passwordConfirmation) {
            return '確認用パスワードを入力してください。';
        }

        if (!isLoginMode && password !== passwordConfirmation) {
            return 'パスワードと確認用パスワードが一致しません。';
        }

        return null;
    };

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        const validationError = validateForm();

        if (validationError !== null) {
            setErrorMessage(validationError);
            return;
        }

        try {
            setIsSubmitting(true);
            setErrorMessage('');

            const session = isLoginMode
                ? await login({ email, password })
                : await register({ email, password });

            onAuthenticated(session);
        } catch (error) {
            console.error(error);
            setErrorMessage(
                error instanceof Error
                    ? error.message
                    : isLoginMode
                      ? 'ログインに失敗しました。'
                      : 'ユーザー登録に失敗しました。',
            );
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleToggleMode = () => {
        setMode((currentMode) => (currentMode === 'login' ? 'register' : 'login'));
        setPassword('');
        setPasswordConfirmation('');
        setErrorMessage('');
    };

    return (
        <section className="auth-page">
            <div className="auth-card">
                <h2>{isLoginMode ? 'ログイン' : 'ユーザー登録'}</h2>

                {errorMessage && <p className="page-error">{errorMessage}</p>}

                <form className="auth-form" onSubmit={handleSubmit}>
                    <div className="form-row">
                        <label htmlFor="auth-email">メールアドレス</label>
                        <input
                            id="auth-email"
                            type="email"
                            value={email}
                            autoComplete="email"
                            disabled={isSubmitting}
                            onChange={(event) => setEmail(event.target.value)}
                        />
                    </div>

                    <div className="form-row">
                        <label htmlFor="auth-password">パスワード</label>
                        <input
                            id="auth-password"
                            type="password"
                            value={password}
                            autoComplete={isLoginMode ? 'current-password' : 'new-password'}
                            disabled={isSubmitting}
                            onChange={(event) => setPassword(event.target.value)}
                        />
                    </div>

                    {!isLoginMode && (
                        <div className="form-row">
                            <label htmlFor="auth-password-confirmation">パスワード確認</label>
                            <input
                                id="auth-password-confirmation"
                                type="password"
                                value={passwordConfirmation}
                                autoComplete="new-password"
                                disabled={isSubmitting}
                                onChange={(event) =>
                                    setPasswordConfirmation(event.target.value)
                                }
                            />
                        </div>
                    )}

                    <button type="submit" disabled={isSubmitting}>
                        {isSubmitting
                            ? isLoginMode
                                ? 'ログイン中...'
                                : '登録中...'
                            : isLoginMode
                              ? 'ログイン'
                              : '登録する'}
                    </button>
                </form>

                <button
                    type="button"
                    className="auth-mode-switch-button"
                    onClick={handleToggleMode}
                    disabled={isSubmitting}
                >
                    {isLoginMode
                        ? 'アカウントを作成する'
                        : 'すでにアカウントがある場合はログイン'}
                </button>
            </div>
        </section>
    );
};
