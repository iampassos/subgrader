use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::{
    io::{self, Write},
    path::Path,
    sync::Arc,
    time::Instant,
};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
};

use classroom::{
    api::ClassroomApi,
    models::{StudentSubmission, SubmissionState},
};

pub async fn download_classroom_submissions(
    api: Arc<ClassroomApi>,
    course_id: String,
    assignment_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();

    println!("{} all student submissions", "Fetching".green().bold());
    io::stdout().flush().unwrap();

    let submissions = api
        .get_student_submissions(&course_id, &assignment_id)
        .await?;

    let valid_submissions: Vec<StudentSubmission> = submissions
        .student_submissions
        .into_iter()
        .filter(|s| {
            s.state == SubmissionState::TurnedIn
                && s.assignment_submission
                    .as_ref()
                    .is_some_and(|a| a.attachments.is_some())
        })
        .collect();

    let bar = Arc::new(ProgressBar::new(valid_submissions.len() as u64));
    bar.set_draw_target(ProgressDrawTarget::stdout_with_hz(1));
    bar.set_style(
        ProgressStyle::with_template("{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len} {percent}%")?
            .progress_chars("## "),
    );
    bar.set_prefix("Downloading");

    let path = format!("./submissions/{course_id}/{assignment_id}");
    let dir = Path::new(&path);

    if dir.exists() && dir.is_dir() {
        fs::remove_dir_all(&path).await?;
    }

    let mut handles = vec![];

    for submission in valid_submissions {
        let sub = submission.assignment_submission.unwrap();
        for att in sub.attachments.unwrap() {
            let path = path.clone();
            fs::create_dir_all(&path).await?;

            let api = Arc::clone(&api);
            let user_id = submission.user_id.clone();
            let course_id = course_id.clone();
            let bar = Arc::clone(&bar);

            handles.push(tokio::spawn(async move {
                let download = api
                    .download_student_submission(&att.drive_file.id)
                    .await
                    .unwrap();
                let student = api.get_student(&course_id, &user_id).await.unwrap();

                let path_assignment =
                    format!("{}/{}.zip", &path, student.profile.email_address.clone());

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

                if !is_zip {
                    bar.println(format!(
                        "{} {} invalid zip",
                        "Error".red().bold(),
                        student.profile.email_address.clone().bold()
                    ));
                    return;
                }

                if let Err(e) = crate::utils::unzip_submission(&path_assignment) {
                    bar.println(format!(
                        "{} {} {}",
                        "Error".red().bold(),
                        student.profile.email_address.clone().bold(),
                        e
                    ));
                }

                bar.inc(1);
            }));
        }
    }

    futures::future::join_all(handles).await;

    bar.finish();

    println!(
        "{} and formatted all submissions in {:.2}s",
        "Finished".green().bold(),
        Instant::now().duration_since(started).as_secs_f32()
    );

    Ok(())
}
