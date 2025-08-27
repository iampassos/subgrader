use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{path::Path, sync::Arc};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
    sync::Mutex,
};

use classroom::api::{ClassroomApi, SubmissionState};

pub async fn download_classroom_submissions(
    api: Arc<Mutex<ClassroomApi>>,
    course_id: String,
    assignment_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let submissions = api
        .lock()
        .await
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
                        pb.set_prefix(att.drive_file.title.clone());

                        let api_clone = Arc::clone(&api);

                        let path = path.clone();

                        fs::create_dir_all(&path).await?;

                        handles.push(tokio::spawn(async move {
                            let resp = {
                                let lock = api_clone.lock().await;
                                lock.build_student_submission_download(&att.drive_file.id)
                                    .unwrap()
                            };

                            let resp = resp.send().await.unwrap();

                            let size = resp.content_length().unwrap_or(0);
                            pb.set_length(size);

                            let path_assignment = format!(
                                "{}/{}",
                                &path,
                                att.drive_file.title.clone().to_lowercase()
                            );

                            let mut file = fs::File::create(&path_assignment).await.unwrap();

                            let mut stream = resp.bytes_stream();

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
                                pb.abandon_with_message(format!("{} {e}", "error".red()));
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
