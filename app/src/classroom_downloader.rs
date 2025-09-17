use futures_util::StreamExt;

use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
};

use classroom::{
    api::ClassroomApi,
    models::{Attachment, Student, StudentSubmission, StudentSubmissions, SubmissionState},
};
use reporter::{SubmissionError, SubmissionResult};

pub fn validate_submissions(
    students: &Arc<HashMap<String, Student>>,
    submissions: StudentSubmissions,
    results: &mut HashMap<String, SubmissionResult>,
) -> Vec<StudentSubmission> {
    submissions
        .student_submissions
        .into_iter()
        .filter(|s| {
            if s.state == SubmissionState::TurnedIn {
                if s.assignment_submission
                    .as_ref()
                    .is_some_and(|a| a.attachments.is_some())
                {
                    return true;
                }

                let std = students.get(&s.user_id).unwrap().clone();

                results.insert(
                    std.profile.email_address.to_string(),
                    SubmissionResult {
                        student: std,
                        errors: vec![SubmissionError::InvalidSubmission],
                        comments: vec![],
                        solved: 0,
                    },
                );
            }

            let std = students.get(&s.user_id).unwrap().clone();

            results.insert(
                std.profile.email_address,
                SubmissionResult {
                    student: students.get(&s.user_id).unwrap().clone(),
                    errors: vec![SubmissionError::NoSubmission],
                    comments: vec![],
                    solved: 0,
                },
            );

            false
        })
        .collect::<Vec<StudentSubmission>>()
}

pub async fn download_classroom_submissions(
    api: Arc<ClassroomApi>,
    course_id: &str,
    assignment_id: &str,
    results: &mut HashMap<String, SubmissionResult>,
) -> Result<f32, Box<dyn std::error::Error>> {
    let started = Instant::now();

    let students = api.list_students(course_id).await?;
    let students: Arc<HashMap<String, _>> = Arc::new(
        students
            .students
            .into_iter()
            .map(|s| (s.user_id.clone(), s))
            .collect(),
    );

    let submissions = api
        .get_student_submissions(course_id, assignment_id)
        .await?;

    let valid_submissions = validate_submissions(&students, submissions, results);

    if valid_submissions.is_empty() {
        return Err("no valid submissions were downloaded".into());
    }

    let path = format!("./submissions/{course_id}/{assignment_id}");
    let dir = Path::new(&path);

    if dir.exists() && dir.is_dir() {
        fs::remove_dir_all(&path).await?;
    }

    let mut handles = vec![];

    fs::create_dir_all(&path).await?;

    for submission in valid_submissions {
        let sub = submission.assignment_submission.unwrap();
        for att in sub.attachments.unwrap() {
            let api = Arc::clone(&api);
            let students = Arc::clone(&students);

            let path = path.clone();
            let user_id = submission.user_id.clone();

            handles.push(tokio::spawn(async move {
                worker(
                    att,
                    api,
                    students,
                    path,
                    user_id,
                    submission.late.unwrap_or(false),
                )
                .await
            }));
        }
    }

    let task_results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|s| match s {
            Ok(s2) => s2.ok(),
            Err(_) => None,
        })
        .collect();

    results.extend(
        task_results
            .into_iter()
            .map(|s| (s.student.profile.email_address.to_string(), s)),
    );

    Ok(Instant::now().duration_since(started).as_secs_f32())
}

async fn worker(
    att: Attachment,
    api: Arc<ClassroomApi>,
    students: Arc<HashMap<String, Student>>,
    path: String,
    user_id: String,
    late: bool,
) -> Result<SubmissionResult, Box<dyn std::error::Error + Send + Sync>> {
    let download = api
        .download_student_submission(&att.drive_file.id)
        .await
        .unwrap();

    let student = students.get(&user_id).unwrap();

    let path_assignment = format!("{}/{}.zip", &path, student.profile.email_address.clone());

    let mut file = fs::File::create(&path_assignment).await.unwrap();

    let mut stream = download.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        file.write_all(&chunk).await.unwrap();
    }

    file.flush().await.unwrap();

    let is_zip = match fs::read(&path_assignment).await {
        Ok(bytes) => bytes.starts_with(b"PK\x03\x04"),
        Err(_) => false,
    };

    let mut errors = vec![];
    let mut solved = 0;

    if is_zip {
        let res = crate::utils::unzip_submission(&path_assignment);

        match res {
            Ok(num) => solved = num,
            Err(_) => {
                errors.push(SubmissionError::ZipError);
            }
        }
    } else {
        errors.push(SubmissionError::InvalidZip);
    }

    if late {
        errors.push(SubmissionError::Late);
    }

    Ok(SubmissionResult {
        student: student.clone(),
        errors,
        comments: vec![],
        solved,
    })
}
