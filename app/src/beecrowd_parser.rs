use serde::Deserialize;
use std::{collections::HashMap, fs::File, path::Path};

use reporter::{SubmissionError, SubmissionResult};

#[derive(Debug, Deserialize)]
struct Record {
    email: String,
    exercises: i32,
    solved: i32,
}

pub fn beecrowd_report_parser(
    results: &mut HashMap<String, SubmissionResult>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let records: HashMap<String, Record> = rdr
        .deserialize()
        .map(|r| {
            let r: Record = r?;
            Ok((r.email.clone(), r))
        })
        .collect::<Result<_, csv::Error>>()?;

    for result in results.values_mut() {
        let student = records.get(&result.student.profile.email_address);

        match student {
            Some(std) => {
                if std.solved == 0 {
                    result.errors.push(SubmissionError::NoBeecrowdSubmission);
                }

                if std.solved < std.exercises {
                    result
                        .errors
                        .push(SubmissionError::IncompleteBeecrowdSubmission);
                }

                if result.solved < std.exercises {
                    result
                        .errors
                        .push(SubmissionError::IncompleteClassroomSubmission);
                }
            }
            None => {
                result.errors.push(SubmissionError::NoBeecrowd);
            }
        }
    }

    Ok(())
}
