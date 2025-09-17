use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
    time::Instant,
};

use reporter::{SubmissionError, SubmissionResult};
use similarity::{AnalyzedFile, analyze_code, compare_two_codes_cached};

pub fn similarity_analyzer(
    course_id: &str,
    assignment_id: &str,
    results: &mut HashMap<String, SubmissionResult>,
    threshold: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();

    println!(
        " :: {} all files and generating pairs",
        "Loading".green().bold()
    );

    let path_or = format!("./submissions/{course_id}/{assignment_id}");

    let mut files = vec![];

    for dir in fs::read_dir(path_or)? {
        let dir = dir?;
        let path = dir.path();

        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                files.push(entry.path());
            }
        }
    }

    let file_contents: Vec<Arc<(String, String, AnalyzedFile)>> = files
        .iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(path).unwrap();
            let file_name = path.file_name().unwrap().to_string_lossy();
            let mut email = file_name.split('_').nth(1).unwrap().to_string();
            email.truncate(email.len() - 2);

            if content.trim().is_empty() {
                if let Some(r) = results.get_mut(&email) {
                    r.errors
                        .push(SubmissionError::EmptyFile(file_name.to_string()));
                }

                None
            } else {
                let analyzed = analyze_code(&content).unwrap();
                Some(Arc::new((email, file_name.to_string(), analyzed)))
            }
        })
        .collect();

    let bar = ProgressBar::new(((file_contents.len() * (file_contents.len() - 1)) / 2) as u64);
    bar.set_style(
        ProgressStyle::with_template(
            " ::{prefix:>10.cyan.bold} [{bar:57}] {pos}/{len} {percent}%",
        )?
        .progress_chars("## "),
    );
    bar.set_prefix("Analyzing");

    let results = Arc::new(Mutex::new(results));

    (0..file_contents.len())
        .into_par_iter()
        .flat_map(|i| {
            ((i + 1)..file_contents.len())
                .map(|j| (Arc::clone(&file_contents[i]), Arc::clone(&file_contents[j])))
                .collect::<Vec<_>>()
        })
        .for_each(|(p1, p2)| worker(&results, &bar, &p1, &p2, threshold));

    bar.finish();

    println!(
        " :: {} and analyzed all submissions in {:.2}s",
        "Finished".green().bold(),
        Instant::now().duration_since(started).as_secs_f32()
    );

    Ok(())
}

fn worker(
    results: &Arc<Mutex<&mut HashMap<String, SubmissionResult>>>,
    bar: &ProgressBar,
    p1: &Arc<(String, String, AnalyzedFile)>,
    p2: &Arc<(String, String, AnalyzedFile)>,
    threshold: u32,
) {
    let res = compare_two_codes_cached(&p1.2, &p2.2);

    if res >= threshold as f32 / 100.0 {
        let mut lock = results.lock().unwrap();

        if let Some(r) = lock.get_mut(&p1.0) {
            r.errors.push(SubmissionError::PlagiarismDetected(
                p1.1.clone(),
                p2.1.clone(),
                res,
            ));
        }

        if let Some(r) = lock.get_mut(&p2.0) {
            r.errors.push(SubmissionError::PlagiarismDetected(
                p2.1.clone(),
                p1.1.clone(),
                res,
            ));
        }
    }

    bar.inc(1);
}
