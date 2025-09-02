use std::collections::HashMap;

use serde::Serialize;

use classroom::models::Student;

#[derive(Serialize, Debug)]
struct Record {
    name: String,
    email: String,
    #[serde(rename = "score percent")]
    score_percent: f32,
    comments: String,
}

#[derive(Debug)]
pub struct SubmissionResult {
    pub student: Student,
    pub comments: Vec<String>,
    pub errors: Vec<SubmissionError>,
    pub solved: i32,
}

#[derive(Debug)]
pub enum SubmissionError {
    NoSubmission,
    InvalidSubmission,
    InvalidZip,
    InvalidFormat,
    PlagiarismDetected(String, String, f32),
    ZipError,
    Late,
    EmptyFile(String),
    NoBeecrowd,
    NoBeecrowdSubmission,
    IncompleteBeecrowdSubmission,
    IncompleteClassroomSubmission,
}

impl SubmissionError {
    pub fn message(&self) -> String {
        match self {
            SubmissionError::NoSubmission => "NO SUBMISSION".to_string(),
            SubmissionError::InvalidSubmission => "INVALID SUBMISSION".to_string(),
            SubmissionError::InvalidZip => "INVALID ZIP".to_string(),
            SubmissionError::InvalidFormat => "INVALID FORMAT".to_string(),
            SubmissionError::PlagiarismDetected(f1, f2, percentage) => {
                format!(
                    "PLAGIARISM DETECTED {f1} WITH {f2} ({:.2}%)",
                    *percentage * 100.0
                )
            }
            SubmissionError::EmptyFile(f) => {
                format!("EMPTY FILE {f}")
            }
            SubmissionError::ZipError => "ERROR WHILE EXTRACTING ZIP".to_string(),
            SubmissionError::Late => "LATE SUBMISSION".to_string(),
            SubmissionError::NoBeecrowd => "NOT LISTED IN BEECROWD CLASS".to_string(),
            SubmissionError::NoBeecrowdSubmission => "NO BEECROWD SUBMISSION".to_string(),
            SubmissionError::IncompleteBeecrowdSubmission => {
                "INCOMPLETE BEECROWD SUBMISSION".to_string()
            }
            SubmissionError::IncompleteClassroomSubmission => {
                "INCOMPLETE CLASSROOM SUBMISSION".to_string()
            }
        }
    }
}

pub fn generate_report(
    results: HashMap<String, SubmissionResult>,
    path: &str,
) -> Result<HashMap<String, SubmissionResult>, Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;

    for result in results.values() {
        let mut comments = result.comments.join(", ");

        let errors = result
            .errors
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>()
            .join("\n");

        if !errors.is_empty() {
            if !comments.is_empty() {
                comments.push_str(", ");
            }
            comments.push_str(&errors);
        }

        wtr.serialize(Record {
            name: result.student.profile.name.full_name.clone(),
            email: result.student.profile.email_address.clone(),
            score_percent: 1.0,
            comments,
        })?;
    }

    wtr.flush()?;

    Ok(results)
}
