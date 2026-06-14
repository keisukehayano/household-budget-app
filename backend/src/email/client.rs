use std::env;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

type EmailResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct EmailClient {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl EmailClient {
    pub fn from_env() -> Result<Self, String> {
        let smtp_host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST must be set".to_string())?;

        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| "SMTP_PORT must be a valid port number".to_string())?;

        let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap_or_default();

        let smtp_use_tls = env::var("SMTP_USE_TLS")
            .unwrap_or_else(|_| "true".to_string())
            .eq_ignore_ascii_case("true");

        let mail_from = env::var("MAIL_FROM").map_err(|_| "MAIL_FROM must be set".to_string())?;

        let mail_from_name =
            env::var("MAIL_FROM_NAME").unwrap_or_else(|_| "家計簿アプリ".to_string());

        let from_address = mail_from
            .parse()
            .map_err(|_| "MAIL_FROM must be a valid email address".to_string())?;

        let from = Mailbox::new(Some(mail_from_name), from_address);

        let builder = if smtp_use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
                .map_err(|error| format!("failed to create SMTP relay: {error}"))?
                .port(smtp_port)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host).port(smtp_port)
        };

        let builder = if smtp_username.is_empty() && smtp_password.is_empty() {
            builder
        } else {
            builder.credentials(Credentials::new(smtp_username, smtp_password))
        };

        Ok(Self {
            mailer: builder.build(),
            from,
        })
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_url: &str,
    ) -> EmailResult<()> {
        let to = to_email.parse::<Mailbox>()?;

        let body = format!(
            "\
パスワード再設定の申請を受け付けました。

以下のURLからパスワードを再設定してください。

{reset_url}

このURLの有効期限は30分です。
心当たりがない場合は、このメールを破棄してください。

--
家計簿アプリ
"
        );

        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("パスワード再設定のご案内")
            .body(body)?;

        self.mailer.send(message).await?;

        Ok(())
    }

    #[cfg(test)]
    pub fn new_for_tests() -> Self {
        let from = "no-reply@household-budget.local"
            .parse()
            .expect("test from address should be valid");

        Self {
            mailer: AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
                .port(1025)
                .build(),
            from: Mailbox::new(Some("家計簿アプリ".to_string()), from),
        }
    }
}
