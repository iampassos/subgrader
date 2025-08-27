use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;

use classroom::{api::ClassroomApi, client::ClassroomClient};
use cli::{ClassroomCommands, Cli, Commands};
use downloader::download_classroom_submissions;

mod cli;
mod downloader;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut client = ClassroomClient::new();
    client.auth("./credentials.json").await?;
    let api = Arc::new(Mutex::new(ClassroomApi::new(client)));

    match cli.command {
        Commands::Classroom { command } => match command {
            ClassroomCommands::ListCourses => {
                let courses = api.lock().await.list_courses().await?;
                for course in &courses.courses {
                    println!("{} -> {}", course.id, course.name);
                }
            }
            ClassroomCommands::ListAssignments { course_id } => {
                let works = api.lock().await.list_course_works(&course_id).await?;

                for work in &works.course_work {
                    println!("{} : {}", work.id, work.title);
                }
            }
            ClassroomCommands::DownloadSubmissions {
                course_id,
                assignment_id,
            } => {
                download_classroom_submissions(api, course_id, assignment_id).await?;
            }
        },
    }

    Ok(())
}
