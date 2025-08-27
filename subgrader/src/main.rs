use classroom::{
    api::{ClassroomApi, SubmissionState},
    client::ClassroomClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClassroomClient::new();
    client.auth("./credentials.json").await?;
    let api = ClassroomApi::new(client);

    let courses = api.list_courses().await?;
    for course in &courses.courses {
        println!("{} : {}", course.id, course.name);
    }

    let course = &courses.courses[0];
    let works = api.list_course_works(&course.id).await?;

    println!();

    for work in &works.course_work {
        println!("{} : {}", work.id, work.title);
    }

    let submissions = api
        .get_student_submissions(&course.id, &works.course_work[0].id)
        .await?;

    println!();

    for submission in submissions.student_submissions {
        if let SubmissionState::TurnedIn = submission.state {
            if let Some(sub) = submission.assignment_submission {
                if let Some(attach) = sub.attachments {
                    for att in attach {
                        println!("{} -> {:#?}", submission.user_id, att.drive_file);
                    }
                }
            }
        }
    }

    Ok(())
}
