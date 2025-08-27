use std::pin::Pin;

use yup_oauth2::{
    InstalledFlowAuthenticator, InstalledFlowReturnMethod,
    authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate},
    read_application_secret,
};

async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    _ = webbrowser::open(url);
    let def_delegate = DefaultInstalledFlowDelegate;
    def_delegate.present_user_url(url, need_code).await
}

#[derive(Copy, Clone)]
struct InstalledFlowBrowserDelegate;

impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(browser_user_url(url, need_code))
    }
}

#[derive(Default)]
pub struct ClassroomClient {
    access_token: Option<String>,
}

impl ClassroomClient {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn auth(&mut self, credentials_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let secret = read_application_secret(credentials_path).await?;

        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("tokencache.json")
                .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
                .build()
                .await?;

        let scopes = &[
            "https://www.googleapis.com/auth/classroom.courses.readonly",
            "https://www.googleapis.com/auth/classroom.coursework.students.readonly",
            "https://www.googleapis.com/auth/drive.readonly",
        ];

        let token = auth.token(scopes).await?;
        self.access_token = Some(token.token().ok_or("No token")?.to_owned());

        Ok(())
    }

    #[must_use]
    pub fn token(&self) -> Option<&str> {
        Some(self.access_token.as_ref()?)
    }
}
