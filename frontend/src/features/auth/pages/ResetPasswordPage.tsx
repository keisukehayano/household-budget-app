import { useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { resetPassword } from '../api/authApi';

type ResetPasswordPageProps = {
    onCompleted: () => void;
};

export const ResetPasswordPage = ({ onCompleted }: ResetPasswordPageProps) => {
    const token = useMemo(() => {
        return new URLSearchParams(window.location.search).get('token') ?? '';
    }, []);

    const [newPassword, setNewPassword] = useState('');
    const [newPasswordConfirmation, setNewPasswordConfirmation] = useState('');
    const [errorMessage, setErrorMessage] = useState('');
    const [infoMessage, setInfoMessage] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [isCompleted, setIsCompleted] = useState(false);

    const validateForm = (): string | null => {
        if (!token) {
            return '再設定トークンがありません。再度パスワード再設定を申請してください。';
        }

        if (!newPassword) {
            return '新しいパスワードを入力してください。';
        }

        if (newPassword.length < 8) {
            return '新しいパスワードは8文字以上で入力してください。';
        }

        if (newPassword.length > 128) {
            return '新しいパスワードは128文字以内で入力してください。';
        }

        if (!newPasswordConfirmation) {
            return '新しいパスワード確認を入力してください。';
        }

        if (newPassword !== newPasswordConfirmation) {
            return '新しいパスワードと確認用パスワードが一致しません。';
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

            await resetPassword({
                token,
                newPassword,
            });

            setIsCompleted(true);
            setInfoMessage('パスワードを再設定しました。新しいパスワードでログインしてください。');
        } catch (error) {
            console.error(error);
            setErrorMessage(
                error instanceof Error ? error.message : 'パスワード再設定に失敗しました。',
            );
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <section className="auth-page">
            <div className="auth-card">
                <h2>パスワード再設定</h2>

                {errorMessage && <p className="page-error">{errorMessage}</p>}
                {infoMessage && <p className="page-loading">{infoMessage}</p>}

                {!isCompleted && (
                    <form className="auth-form" onSubmit={handleSubmit}>
                        <div className="form-row">
                            <label htmlFor="reset-new-password">新しいパスワード</label>
                            <input
                                id="reset-new-password"
                                type="password"
                                value={newPassword}
                                autoComplete="new-password"
                                disabled={isSubmitting}
                                onChange={(event) => setNewPassword(event.target.value)}
                            />
                        </div>

                        <div className="form-row">
                            <label htmlFor="reset-new-password-confirmation">
                                新しいパスワード確認
                            </label>
                            <input
                                id="reset-new-password-confirmation"
                                type="password"
                                value={newPasswordConfirmation}
                                autoComplete="new-password"
                                disabled={isSubmitting}
                                onChange={(event) =>
                                    setNewPasswordConfirmation(event.target.value)
                                }
                            />
                        </div>

                        <button type="submit" disabled={isSubmitting}>
                            {isSubmitting ? '再設定中...' : 'パスワードを再設定する'}
                        </button>
                    </form>
                )}

                <button
                    type="button"
                    className="auth-mode-switch-button"
                    onClick={onCompleted}
                    disabled={isSubmitting}
                >
                    ログイン画面へ
                </button>
            </div>
        </section>
    );
};
