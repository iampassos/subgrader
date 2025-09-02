use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
};

use classroom::{
    api::ClassroomApi,
    models::{Attachment, Student, StudentSubmission, SubmissionState},
};
use reporter::{SubmissionError, SubmissionResult};

pub async fn download_classroom_submissions(
    api: Arc<ClassroomApi>,
    course_id: &str,
    assignment_id: &str,
    mut results: Vec<SubmissionResult>,
) -> Result<Vec<SubmissionResult>, Box<dyn std::error::Error>> {
    let started = Instant::now();

    println!(
        " :: {} all students and submissions [CID {course_id}/AID {assignment_id}]",
        "Fetching".green().bold()
    );

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

    let valid_submissions: Vec<StudentSubmission> = submissions
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

                results.push(SubmissionResult {
                    student: students.get(&s.user_id).unwrap().clone(),
                    errors: vec![SubmissionError::InvalidSubmission],
                    comments: vec![],
                });
            }

            results.push(SubmissionResult {
                student: students.get(&s.user_id).unwrap().clone(),
                errors: vec![SubmissionError::NoSubmission],
                comments: vec![],
            });

            false
        })
        .collect();

    let bar = Arc::new(ProgressBar::new(valid_submissions.len() as u64));
    bar.set_draw_target(ProgressDrawTarget::stdout_with_hz(1));
    bar.set_style(
        ProgressStyle::with_template(
            " ::{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len} {percent}%",
        )?
        .progress_chars("## "),
    );
    bar.set_prefix("Downloading");

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
            let bar = Arc::clone(&bar);

            let path = path.clone();
            let user_id = submission.user_id.clone();

            handles.push(tokio::spawn(async move {
                worker(
                    att,
                    api,
                    students,
                    bar,
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

    results.extend(task_results);

    bar.finish();

    println!(
        " :: {} and formatted all submissions in {:.2}s",
        "Finished".green().bold(),
        Instant::now().duration_since(started).as_secs_f32()
    );

    Ok(results)
}

async fn worker(
    att: Attachment,
    api: Arc<ClassroomApi>,
    students: Arc<HashMap<String, Student>>,
    bar: Arc<ProgressBar>,
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

    if !is_zip {
        bar.println(format!(
            " :: {} {} ({}) invalid zip",
            "Error".red().bold(),
            student.profile.name.full_name.clone().bold(),
            student.profile.email_address.clone().bold()
        ));
        errors.push(SubmissionError::InvalidZip);
    } else if let Err(e) = crate::utils::unzip_submission(&path_assignment) {
        bar.println(format!(
            " :: {} {} ({}) {}",
            "Error".red().bold(),
            student.profile.name.full_name.clone().bold(),
            student.profile.email_address.clone().bold(),
            e
        ));
        errors.push(SubmissionError::ZipError);
    }

    if late {
        bar.println(format!(
            " :: {} {} ({}) late submission",
            "Warning".yellow().bold(),
            student.profile.name.full_name.clone().bold(),
            student.profile.email_address.clone().bold(),
        ));
        errors.push(SubmissionError::Late);
    }

    bar.inc(1);

    Ok(SubmissionResult {
        student: student.clone(),
        errors,
        comments: vec![],
    })
}
