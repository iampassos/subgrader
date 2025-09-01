use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    fs,
    sync::{Arc, Mutex},
};

use reporter::{SubmissionError, SubmissionResult};
use similarity::compare_contents;

pub fn similarity_analyzer(
    course_id: &str,
    assignment_id: &str,
    results: Vec<SubmissionResult>,
) -> Result<Vec<SubmissionResult>, Box<dyn std::error::Error>> {
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

    let file_contents: Vec<Arc<(String, String, String)>> = files
        .iter()
        .map(|path| {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let mut email = file_name.split('_').nth(1).unwrap().to_string();
            email.truncate(email.len() - 2);
            let content = std::fs::read_to_string(path).unwrap();
            Arc::new((email, file_name.to_string(), content))
        })
        .collect();

    let mut pairs = vec![];

    for i in 0..file_contents.len() {
        for j in (i + 1)..file_contents.len() {
            pairs.push((file_contents[i].clone(), file_contents[j].clone()));
        }
    }

    let bar = ProgressBar::new(pairs.len() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            " ::{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len} {percent}%",
        )?
        .progress_chars("## "),
    );
    bar.set_prefix("Analyzing");

    let results = Arc::new(Mutex::new(results));

    pairs.into_par_iter().for_each(|(p1, p2)| {
        let res = compare_contents(&p1.2, &p2.2).unwrap();

        if res >= 1.0 {
            let mut lock = results.lock().unwrap();

            if let Some(r) = lock
                .iter_mut()
                .find(|r| *r.student.profile.email_address == p1.0)
            {
                r.errors.push(SubmissionError::PlagiarismDetected(
                    p1.1.clone(),
                    p2.1.clone(),
                    res,
                ));
            }

            if let Some(r) = lock
                .iter_mut()
                .find(|r| *r.student.profile.email_address == p2.0)
            {
                r.errors.push(SubmissionError::PlagiarismDetected(
                    p2.1.clone(),
                    p1.1.clone(),
                    res,
                ));
            }

            bar.println(format!(
                " :: {} {} with {} ({:.2}%) similarity detected",
                "Warning".yellow().bold(),
                p1.1,
                p2.1,
                res * 100.0
            ));
        }

        bar.inc(1);
    });

    bar.finish();

    Ok(Arc::try_unwrap(results).unwrap().into_inner().unwrap())
}
