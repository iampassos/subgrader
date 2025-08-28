use reqwest::{Client, RequestBuilder, Response};

use crate::client::ClassroomClient;
use crate::models::{Course, CourseWorks, Courses, Student, StudentSubmissions};

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

    pub fn build_student_request(
        &self,
        course_id: &str,
        user_id: &str,
    ) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get(format!(
                "https://classroom.googleapis.com/v1/courses/{course_id}/students/{user_id}"
            ))
            .bearer_auth(token);

        Ok(resp)
    }

    pub async fn handle_student_response(
        &self,
        response: Response,
    ) -> Result<Student, Box<dyn std::error::Error>> {
        let student: Student = response.json().await?;

        Ok(student)
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

    pub fn build_student_submission_download(
        &self,
        file_id: &str,
    ) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
        let token = self.classroom_client.token().ok_or("Token not found")?;

        let resp = self
            .http_client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files/{file_id}?alt=media"
            ))
            .bearer_auth(token);

        Ok(resp)
    }
}
