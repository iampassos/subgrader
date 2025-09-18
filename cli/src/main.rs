use colored::Colorize;
use dialoguer::{
    Input, MultiSelect, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};
use std::{collections::HashMap, path::Path, sync::Arc};

use app::{
    beecrowd_parser::beecrowd_report_parser, classroom_downloader::download_classroom_submissions,
    similarity_checker::similarity_analyzer,
};
use classroom::{api::ClassroomApi, client::ClassroomClient};
use reporter::{SubmissionResult, generate_report};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClassroomClient::new();
    client.auth("./credentials.json").await?;
    let api = Arc::new(ClassroomApi::new(client));

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
        prompt_prefix: style(" :: ?".to_string()).for_stderr().yellow().bold(),
        prompt_style: Style::new().for_stderr().white(),
        error_prefix: style(" ::".to_string()).for_stderr().red(),
        ..ColorfulTheme::default()
    };

    let selection = Select::with_theme(&own_theme)
        .with_prompt("Select the course")
        .default(0)
        .max_length(3)
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

    let options_selection = &[
        ("Check Similarity", true),
        ("Check Beecrowd", true),
        ("Make Report", true),
    ];

    let selections = MultiSelect::with_theme(&own_theme)
        .with_prompt("Select more options")
        .items_checked(options_selection.iter().copied())
        .interact()
        .unwrap();

    let mut input_file = None;

    if selections.contains(&1) {
        let files: Vec<_> = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().into_string().ok()?;
                if name.ends_with(".csv") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        if files.is_empty() {
            println!(
                " :: {} no .csv files found in project root",
                "Error".red().bold()
            );
        } else {
            let selection = Select::with_theme(&own_theme)
                .with_prompt("Beecrowd report .csv file-name")
                .default(0)
                .max_length(5)
                .items(&files)
                .interact()
                .unwrap();

            input_file = Some(files[selection].clone());
        }
    }

    let mut input_thr: Result<u32, dialoguer::Error> = Result::Ok(100);

    if selections.contains(&0) {
        input_thr = Input::<u32>::with_theme(&own_theme)
            .with_prompt("Similarity threshold (%)")
            .validate_with(|input: &u32| -> Result<(), &str> {
                if *input <= 100 {
                    Ok(())
                } else {
                    Err("Invalid percentage threshold")
                }
            })
            .default(100)
            .show_default(true)
            .allow_empty(false)
            .interact_text();
    }

    let mut results: HashMap<String, SubmissionResult> = HashMap::new();

    println!(
        " :: {} all students and submissions [CID {}/AID {}]",
        "Fetching".green().bold(),
        course.id,
        work.id
    );

    let download_res =
        download_classroom_submissions(api.clone(), &course.id, &work.id, &mut results).await;

    if let Err(e) = download_res {
        println!(" :: {} {e}", "Error".red().bold());
    } else if let Ok(s) = download_res {
        println!(
            " :: {} and formatted all submissions in {:.2}s",
            "Finished".green().bold(),
            s
        );

        if selections.contains(&0) {
            if let Ok(thr) = input_thr {
                println!(
                    " :: {} all files and generating pairs",
                    "Loading".green().bold()
                );

                let similarity_res = similarity_analyzer(&course.id, &work.id, &mut results, thr)?;

                println!(
                    " :: {} and analyzed all submissions in {:.2}s",
                    "Finished".green().bold(),
                    similarity_res
                );
            }
        }

        if selections.contains(&1) {
            if let Some(f) = input_file {
                beecrowd_report_parser(&mut results, Path::new(&f)).unwrap();

                println!(
                    " :: {} parsing and checking Beecrowd report",
                    "Finished".green().bold(),
                );
            }
        }

        if selections.contains(&2) {
            let path = format!("./submissions/{}/{}/report.csv", course.id, work.id);

            generate_report(results, &path)?;

            println!(
                " :: {} student report at {}",
                "Generated".green().bold(),
                path
            );
        }
    }

    Ok(())
}
