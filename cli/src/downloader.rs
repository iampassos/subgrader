use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{path::Path, sync::Arc};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
};

use classroom::{api::ClassroomApi, models::SubmissionState};

pub async fn download_classroom_submissions(
    api: Arc<ClassroomApi>,
    course_id: String,
    assignment_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let submissions = api
        .get_student_submissions(&course_id, &assignment_id)
        .await?;

    let multi = MultiProgress::new();

    let mut handles = vec![];

    let path = format!("./submissions/{course_id}/{assignment_id}");
    let dir = Path::new(&path);

    if dir.exists() && dir.is_dir() {
        fs::remove_dir_all(&path).await.unwrap();
    }

    for submission in submissions.student_submissions {
        if let SubmissionState::TurnedIn = submission.state {
            if let Some(sub) = submission.assignment_submission {
                if let Some(attach) = sub.attachments {
                    for att in attach {
                        let pb = multi.add(ProgressBar::new(0));
                        pb.set_style(ProgressStyle::with_template(
                                        "{spinner:.green} [{elapsed_precise}] [{bar:40.white/white}] {bytes}/{total_bytes} {prefix:.bold} {msg}",
                                    )?.progress_chars("#>-"));

                        let path = path.clone();

                        fs::create_dir_all(&path).await?;

                        let api = Arc::clone(&api);
                        let user_id = submission.user_id.clone();
                        let course_id = course_id.clone();

                        handles.push(tokio::spawn(async move {
                            let download = api
                                .download_student_submission(&att.drive_file.id)
                                .await
                                .unwrap();
                            let student = api.get_student(&course_id, &user_id).await.unwrap();

                            pb.set_prefix(student.profile.email_address.clone());
                            pb.set_length(download.content_length().unwrap_or(0));

                            let path_assignment =
                                format!("{}/{}.zip", &path, student.profile.email_address);

                            let mut file = fs::File::create(&path_assignment).await.unwrap();

                            let mut stream = download.bytes_stream();

                            while let Some(item) = stream.next().await {
                                let chunk = item.unwrap();
                                file.write_all(&chunk).await.unwrap();
                                pb.inc(chunk.len() as u64);
                            }

                            file.flush().await.unwrap();

                            let is_zip = match fs::read(&path_assignment).await {
                                Ok(bytes) => bytes.starts_with(b"PK\x03\x04"),
                                Err(_) => false,
                            };

                            if !is_zip {
                                pb.abandon_with_message(format!(
                                    "{} invalid or blocked file",
                                    "error".red()
                                ));
                                return;
                            }

                            if let Err(e) = crate::utils::unzip_submission(&path_assignment) {
                                pb.abandon_with_message(format!("{} -> {e}", "error".red()));
                            } else {
                                pb.finish_with_message(format!("{}", "success".green()));
                            }
                        }));
                    }
                }
            }
        }
    }

    futures::future::join_all(handles).await;

    Ok(())
}
