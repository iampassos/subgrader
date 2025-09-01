use std::ops::Mul;

use serde::Serialize;

use classroom::models::Student;

#[derive(Serialize, Debug)]
struct Record {
    name: String,
    email: String,
    comments: String,
}

#[derive(Debug)]
pub struct SubmissionResult {
    pub student: Student,
    pub comments: Vec<String>,
    pub errors: Vec<SubmissionError>,
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
            SubmissionError::ZipError => "ERROR WHILE EXTRACTING ZIP".to_string(),
            SubmissionError::Late => "LATE SUBMISSION".to_string(),
        }
    }
}

pub fn generate_report(
    results: Vec<SubmissionResult>,
    path: &str,
) -> Result<Vec<SubmissionResult>, Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;

    for result in &results {
        let mut comments = result.comments.join(", ");

        let errors = result
            .errors
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>()
            .join(", ");

        if !errors.is_empty() {
            if !comments.is_empty() {
                comments.push_str(", ");
            }
            comments.push_str(&errors);
        }

        wtr.serialize(Record {
            name: result.student.profile.name.full_name.clone(),
            email: result.student.profile.email_address.clone(),
            comments,
        })?;
    }

    wtr.flush()?;

    Ok(results)
}
