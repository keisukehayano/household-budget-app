import { useState } from 'react';
import type { FormEvent } from 'react';
import { forgotPassword, login, register } from '../api/authApi';
import type { AuthSession } from '../types';

type AuthMode = 'login' | 'register' | 'forgot-password';

type AuthPageProps = {
    onAuthenticated: (session: AuthSession) => void;
};

export const AuthPage = ({ onAuthenticated }: AuthPageProps) => {
    const [mode, setMode] = useState<AuthMode>('login');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [passwordConfirmation, setPasswordConfirmation] = useState('');
    const [errorMessage, setErrorMessage] = useState('');
    const [infoMessage, setInfoMessage] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    const isLoginMode = mode === 'login';
    const isRegisterMode = mode === 'register';
    const isForgotPasswordMode = mode === 'forgot-password';

    const validateForm = (): string | null => {
        if (!email.trim()) {
            return 'メールアドレスを入力してください。';
        }

        if (isForgotPasswordMode) {
            return null;
        }

        if (!password) {
            return 'パスワードを入力してください。';
        }

        if (isRegisterMode && !passwordConfirmation) {
            return '確認用パスワードを入力してください。';
        }

        if (isRegisterMode && password !== passwordConfirmation) {
            return 'パスワードと確認用パスワードが一致しません。';
        }

        return null;
    };

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        const validationError = validateForm();

        if (validationError !== null) {
            setErrorMessage(validationError);
            setInfoMessage('');
            return;
        }

        try {
            setIsSubmitting(true);
            setErrorMessage('');
            setInfoMessage('');

            if (isForgotPasswordMode) {
                const response = await forgotPassword({ email });
                setInfoMessage(response.message);
                return;
            }

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
                      : isRegisterMode
                        ? 'ユーザー登録に失敗しました。'
                        : 'パスワード再設定の申請に失敗しました。',
            );
        } finally {
            setIsSubmitting(false);
        }
    };

    const switchMode = (nextMode: AuthMode) => {
        setMode(nextMode);
        setPassword('');
        setPasswordConfirmation('');
        setErrorMessage('');
        setInfoMessage('');
    };

    return (
        <section className="auth-page">
            <div className="auth-card">
                <h2>
                    {isLoginMode
                        ? 'ログイン'
                        : isRegisterMode
                          ? 'ユーザー登録'
                          : 'パスワード再設定'}
                </h2>

                {errorMessage && <p className="page-error">{errorMessage}</p>}
                {infoMessage && <p className="page-loading">{infoMessage}</p>}

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

                    {!isForgotPasswordMode && (
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
                    )}

                    {isRegisterMode && (
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
                                : isRegisterMode
                                  ? '登録中...'
                                  : '送信中...'
                            : isLoginMode
                              ? 'ログイン'
                              : isRegisterMode
                                ? '登録する'
                                : '再設定案内を送信'}
                    </button>
                </form>

                <div className="auth-links">
                    {isLoginMode && (
                        <>
                            <button
                                type="button"
                                className="auth-mode-switch-button"
                                onClick={() => switchMode('register')}
                                disabled={isSubmitting}
                            >
                                アカウントを作成する
                            </button>

                            <button
                                type="button"
                                className="auth-link-button"
                                onClick={() => switchMode('forgot-password')}
                                disabled={isSubmitting}
                            >
                                パスワードを忘れた場合
                            </button>
                        </>
                    )}

                    {isRegisterMode && (
                        <button
                            type="button"
                            className="auth-mode-switch-button"
                            onClick={() => switchMode('login')}
                            disabled={isSubmitting}
                        >
                            すでにアカウントがある場合はログイン
                        </button>
                    )}

                    {isForgotPasswordMode && (
                        <button
                            type="button"
                            className="auth-mode-switch-button"
                            onClick={() => switchMode('login')}
                            disabled={isSubmitting}
                        >
                            ログイン画面に戻る
                        </button>
                    )}
                </div>
            </div>
        </section>
    );
};
