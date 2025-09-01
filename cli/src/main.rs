use colored::Colorize;
use dialoguer::{
    Confirm, MultiSelect, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};
use rayon::prelude::*;
use std::{fs, sync::Arc};

use classroom::{api::ClassroomApi, client::ClassroomClient};
use downloader::download_classroom_submissions;
use reporter::{SubmissionResult, generate_report};
use similarity::compare_files;

mod downloader;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut files = vec![];
    //
    // for dir in fs::read_dir("./submissions/779775636211/799310705791/")? {
    //     let dir = dir?;
    //     let path = dir.path();
    //
    //     if path.is_dir() {
    //         for entry in fs::read_dir(&path)? {
    //             let entry = entry?;
    //             files.push(entry.path());
    //         }
    //     }
    // }
    //
    // let mut pairs = vec![];
    //
    // for i in 0..files.len() {
    //     for j in (i + 1)..files.len() {
    //         pairs.push((files[i].clone(), files[j].clone()));
    //     }
    // }
    //
    // for p in pairs.clone() {
    //     let f1 = p.0.file_name().unwrap().to_string_lossy();
    //     let f2 = p.1.file_name().unwrap().to_string_lossy();
    //
    //     let res = compare_files(
    //         p.0.as_path().to_str().unwrap(),
    //         p.1.as_path().to_str().unwrap(),
    //     );
    //     // println!("{} - {} -> {}", f1, f2, res?);
    // }
    //
    // return Ok(());

    let mut client = ClassroomClient::new();
    client.auth("./credentials.json").await?;
    let api = Arc::new(ClassroomApi::new(client));

    println!(" :: Classroom Subgrader Assistant");

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

    let options_selection = &[("Generate report", true)];

    let selections = MultiSelect::with_theme(&own_theme)
        .with_prompt("Select more options")
        .items_checked(options_selection.iter().copied())
        .interact()
        .unwrap();

    let mut results: Vec<SubmissionResult> = vec![];

    if Confirm::with_theme(&own_theme)
        .with_prompt("Do you want to continue?")
        .default(true)
        .show_default(false)
        .wait_for_newline(true)
        .interact()
        .unwrap()
    {
        println!();
        results = download_classroom_submissions(
            api.clone(),
            course.id.clone(),
            work.id.clone(),
            results,
        )
        .await?;
    } else {
        println!(" :: Cancelled");
    }

    if selections.contains(&0) {
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
