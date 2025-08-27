use yup_oauth2::InstalledFlowAuthenticator;

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
        let secret = yup_oauth2::read_application_secret(credentials_path).await?;

        let auth = InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await?;

        let scopes = &[
            "https://www.googleapis.com/auth/classroom.courses.readonly",
            "https://www.googleapis.com/auth/classroom.coursework.students.readonly",
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
