use clap::Parser;
use dialoguer::{
    Confirm, MultiSelect, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};
use std::sync::Arc;

use classroom::{api::ClassroomApi, client::ClassroomClient};
use cli::{ClassroomCommands, Cli, Commands};
use downloader::download_classroom_submissions;

mod cli;
mod downloader;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Classroom { command } => {
            let mut client = ClassroomClient::new();
            client.auth("./credentials.json").await?;
            let api = Arc::new(ClassroomApi::new(client));

            match command {
                ClassroomCommands::Assistant => {
                    println!(" :: Classroom Subgrader Assistant");

                    let courses = api.list_courses().await?;
                    let course_selection: Vec<&str> =
                        courses.courses.iter().map(|c| c.name.as_str()).collect();

                    let own_theme = ColorfulTheme {
                        active_item_style: Style::new().for_stderr().green().bold(),
                        checked_item_prefix: style("  [x]".to_string()).for_stderr().green().bold(),
                        unchecked_item_prefix: style(" [ ]".to_string()).for_stderr().white(),
                        active_item_prefix: style(" ›".to_string()).for_stderr().white().bold(),
                        prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
                        defaults_style: Style::new().for_stderr().green(),
                        values_style: Style::new().for_stderr().green().bold(),
                        success_prefix: style(" ::".to_string()).for_stderr().white(),
                        success_suffix: style("·".to_string()).for_stderr().black().bright(),
                        prompt_prefix: style(" ::".to_string()).for_stderr().green().bold(),
                        prompt_style: Style::new().for_stderr().white(),
                        ..ColorfulTheme::default()
                    };

                    let selection = Select::with_theme(&own_theme)
                        .with_prompt("Select the course")
                        .default(0)
                        .max_length(5)
                        .items(&course_selection[..])
                        .interact()
                        .unwrap();

                    let course = courses.courses.get(selection).unwrap();

                    let works = api.list_course_works(&course.id).await?;
                    let works_selection: Vec<&str> =
                        works.course_work.iter().map(|w| w.title.as_str()).collect();

                    let selection = Select::with_theme(&own_theme)
                        .with_prompt("Select the assignment")
                        .default(0)
                        .max_length(5)
                        .items(&works_selection[..])
                        .interact()
                        .unwrap();

                    let work = works.course_work.get(selection).unwrap();

                    if Confirm::with_theme(&own_theme)
                        .with_prompt("Do you want to continue?")
                        .default(true)
                        .show_default(false)
                        .wait_for_newline(true)
                        .interact()
                        .unwrap()
                    {
                        println!();
                        download_classroom_submissions(api, course.id.clone(), work.id.clone())
                            .await?;
                    } else {
                        println!(" :: Cancelled");
                    }
                }
                ClassroomCommands::ListCourses => {
                    let courses = api.list_courses().await?;
                    for course in &courses.courses {
                        println!("{} -> {}", course.id, course.name);
                    }
                }
                ClassroomCommands::ListAssignments { course_id } => {
                    let works = api.list_course_works(&course_id).await?;

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
            }
        }
    }

    Ok(())
}
