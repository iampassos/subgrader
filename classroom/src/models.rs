use serde::Deserialize;

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

#[derive(Deserialize, Debug, PartialEq)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub full_name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub email_address: String,
    pub name: Name,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Student {
    pub user_id: String,
    pub profile: UserProfile,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Students {
    pub students: Vec<Student>,
    pub next_page_token: Option<String>,
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
