use colored::Colorize;
use dialoguer::{
    MultiSelect, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};
use std::sync::Arc;

use classroom::{api::ClassroomApi, client::ClassroomClient};
use comparator::similarity_analyzer;
use downloader::download_classroom_submissions;
use reporter::{SubmissionResult, generate_report};

mod comparator;
mod downloader;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClassroomClient::new();
    client.auth("./credentials.json").await?;
    let api = Arc::new(ClassroomApi::new(client));

    println!(" :: Subgrader Assistant");

    let courses = api.list_courses().await?;
    let course_selection: Vec<&str> = courses.courses.iter().map(|c| c.name.as_str()).collect();

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
    let works_selection: Vec<&str> = works.course_work.iter().map(|w| w.title.as_str()).collect();

    let selection = Select::with_theme(&own_theme)
        .with_prompt("Select the assignment")
        .default(0)
        .max_length(5)
        .items(&works_selection[..])
        .interact()
        .unwrap();

    let work = works.course_work.get(selection).unwrap();

    let options_selection = &[("Similarity check", true), ("Generate report", true)];

    let selections = MultiSelect::with_theme(&own_theme)
        .with_prompt("Select more options")
        .items_checked(options_selection.iter().copied())
        .interact()
        .unwrap();

    let mut results: Vec<SubmissionResult> = vec![];

    println!();
    results = download_classroom_submissions(api.clone(), &course.id, &work.id, results).await?;

    if selections.contains(&0) {
        println!();
        results = similarity_analyzer(&course.id, &work.id, results)?;
    }

    if selections.contains(&1) {
        let path = format!("./submissions/{}/{}/report.csv", course.id, work.id);
        generate_report(results, &path)?;

        println!(
            "\n :: {} student report at {}",
            "Generated".green().bold(),
            path
        );
    }

    Ok(())
}
