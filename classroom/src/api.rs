use reqwest::Client;
use serde::Deserialize;

use crate::client::ClassroomClient;

#[derive(Deserialize, Debug)]
pub struct Courses {
    pub courses: Vec<Course>,
}

#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct DriveFile {
    pub id: String,
    pub title: String,
    #[serde(rename = "alternateLink")]
    pub drive_link: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub drive_file: DriveFile,
}

#[derive(Deserialize, Debug)]
pub struct AssignmentSubmission {
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmissionState {
    SubmissionStateUnspecified,
    New,
    Created,
    TurnedIn,
    Returned,
    ReclaimedByStudent,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StudentSubmission {
    pub user_id: String,
    pub late: Option<bool>,
    pub state: SubmissionState,
    pub assignment_submission: Option<AssignmentSubmission>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentSubmissions {
    pub student_submissions: Vec<StudentSubmission>,
}

#[derive(Deserialize, Debug)]
pub struct CourseWork {
    pub id: String,
    pub title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorks {
    pub course_work: Vec<CourseWork>,
}

pub struct ClassroomApi {
    http_client: Client,
    classroom_client: ClassroomClient,
}

impl ClassroomApi {
    #[must_use]
    pub fn new(classroom_client: ClassroomClient) -> Self {
        Self {
            http_client: Client::new(),
            classroom_client,
        }
    }

    pub async fn list_courses(&self) -> Result<Courses, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get("https://classroom.googleapis.com/v1/courses")
            .bearer_auth(token)
            .send()
            .await?;

        let courses: Courses = resp.json().await?;

        Ok(courses)
    }

    pub async fn list_course(&self, course_id: &str) -> Result<Course, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get(format!(
                "https://classroom.googleapis.com/v1/courses/{course_id}"
            ))
            .bearer_auth(token)
            .send()
            .await?;

        let course: Course = resp.json().await?;

        Ok(course)
    }

    pub async fn list_course_works(
        &self,
        course_id: &str,
    ) -> Result<CourseWorks, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get(format!(
                "https://classroom.googleapis.com/v1/courses/{course_id}/courseWork"
            ))
            .bearer_auth(token)
            .send()
            .await?;

        let works: CourseWorks = resp.json().await?;

        Ok(works)
    }

    pub async fn get_student_submissions(
        &self,
        course_id: &str,
        course_work_id: &str,
    ) -> Result<StudentSubmissions, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get(format!("https://classroom.googleapis.com/v1/courses/{course_id}/courseWork/{course_work_id}/studentSubmissions"))
            .bearer_auth(token)
            .send()
            .await?;

        let submissions: StudentSubmissions = resp.json().await?;

        Ok(submissions)
    }
}
