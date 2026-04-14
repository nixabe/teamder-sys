use anyhow::Result;
use mongodb::bson::doc;
use teamder_core::models::{
    competition::{Competition, CompetitionStatus},
    project::{CollabMode, Project, ProjectRole, ProjectStatus},
    study_group::StudyGroup,
    user::{AvailabilityStatus, PortfolioItem, Review, Skill, User, WorkMode},
};

use crate::DbClient;

/// Insert seed data into every collection if the `users` collection is empty.
/// Safe to call on every startup — does nothing when data already exists.
pub async fn seed_if_empty(db: &DbClient) -> Result<()> {
    let count = db
        .db
        .collection::<User>("users")
        .count_documents(doc! {})
        .await?;

    if count > 0 {
        tracing::info!("Database already has data — skipping seed");
        return Ok(());
    }

    tracing::info!("Empty database detected — inserting seed data");

    seed_users(db).await?;
    seed_projects(db).await?;
    seed_competitions(db).await?;
    seed_study_groups(db).await?;

    tracing::info!("Seed complete");
    Ok(())
}

// ── Users ──────────────────────────────────────────────────────────────────

async fn seed_users(db: &DbClient) -> Result<()> {
    use chrono::Utc;

    let hash = |pw: &str| bcrypt::hash(pw, bcrypt::DEFAULT_COST).unwrap();
    let now = Utc::now();

    let mut admin = User::new(
        "admin@teamder.app",
        hash("admin1234"),
        "Andy Chen",
        "Full-Stack Developer",
        "Computer Science",
    );
    admin.is_admin = true;
    admin.university = "Fu Jen Catholic University".into();
    admin.year = "Year 3".into();
    admin.location = Some("New Taipei City, Taiwan".into());
    admin.bio = vec![
        "Building things that matter, one commit at a time.".into(),
        "Passionate about open-source and developer tooling.".into(),
    ];
    admin.skills = vec![
        Skill { name: "Rust".into(), level: 85 },
        Skill { name: "TypeScript".into(), level: 90 },
        Skill { name: "React".into(), level: 88 },
        Skill { name: "MongoDB".into(), level: 75 },
    ];
    admin.skill_tags = vec!["Rust".into(), "TypeScript".into(), "React".into(), "MongoDB".into()];
    admin.gradient = "linear-gradient(135deg, #DD6E42, #E89070)".into();
    admin.work_mode = WorkMode::Hybrid;
    admin.availability = AvailabilityStatus::OpenForCollab;
    admin.hours_per_week = "15-20 hrs/week".into();
    admin.languages = vec!["Chinese".into(), "English".into()];
    admin.match_score = 95;
    admin.rating = 4.9;
    admin.projects_done = 12;
    admin.collaborations = 8;
    admin.portfolio = vec![
        PortfolioItem {
            title: "Teamder".into(),
            kind: "Web App".into(),
            description: Some("A team-matching platform for students.".into()),
            url: Some("https://github.com/WatchAndyTW/teamder".into()),
        },
    ];
    admin.created_at = now;
    admin.updated_at = now;

    let mut alice = User::new(
        "alice@example.com",
        hash("password123"),
        "Alice Wang",
        "UI/UX Designer",
        "Digital Media Design",
    );
    alice.university = "Fu Jen Catholic University".into();
    alice.year = "Year 2".into();
    alice.location = Some("Taipei, Taiwan".into());
    alice.bio = vec![
        "Design is not just what it looks like — design is how it works.".into(),
    ];
    alice.skills = vec![
        Skill { name: "Figma".into(), level: 92 },
        Skill { name: "CSS".into(), level: 80 },
        Skill { name: "User Research".into(), level: 85 },
    ];
    alice.skill_tags = vec!["Figma".into(), "CSS".into(), "User Research".into()];
    alice.gradient = "linear-gradient(135deg, #4F6D7A, #6B8D9E)".into();
    alice.work_mode = WorkMode::Remote;
    alice.availability = AvailabilityStatus::OpenForCollab;
    alice.hours_per_week = "10-15 hrs/week".into();
    alice.match_score = 88;
    alice.rating = 4.7;
    alice.projects_done = 6;
    alice.collaborations = 4;
    alice.reviews = vec![Review {
        reviewer_id: admin.id.clone(),
        reviewer_name: admin.name.clone(),
        project_name: "Campus App Redesign".into(),
        stars: 5,
        body: "Alice's designs were clean, intuitive, and delivered on time. Highly recommend!".into(),
        created_at: now,
    }];

    let mut bob = User::new(
        "bob@example.com",
        hash("password123"),
        "Bob Lin",
        "Backend Engineer",
        "Information Engineering",
    );
    bob.university = "Fu Jen Catholic University".into();
    bob.year = "Year 4".into();
    bob.location = Some("Taichung, Taiwan".into());
    bob.bio = vec![
        "Go, Python, and strong opinions about database indexes.".into(),
        "Open to backend & infrastructure collaboration.".into(),
    ];
    bob.skills = vec![
        Skill { name: "Go".into(), level: 88 },
        Skill { name: "Python".into(), level: 82 },
        Skill { name: "PostgreSQL".into(), level: 79 },
        Skill { name: "Docker".into(), level: 85 },
    ];
    bob.skill_tags = vec!["Go".into(), "Python".into(), "PostgreSQL".into(), "Docker".into()];
    bob.gradient = "linear-gradient(135deg, #C0D6DF, #7FA8BB)".into();
    bob.work_mode = WorkMode::InPerson;
    bob.availability = AvailabilityStatus::Busy;
    bob.hours_per_week = "5-10 hrs/week".into();
    bob.match_score = 76;
    bob.rating = 4.5;
    bob.projects_done = 9;
    bob.collaborations = 5;

    let col: mongodb::Collection<User> = db.db.collection("users");
    col.insert_many(vec![admin, alice, bob]).await?;
    tracing::info!("  ✓ Inserted 3 seed users");
    Ok(())
}

// ── Projects ───────────────────────────────────────────────────────────────

async fn seed_projects(db: &DbClient) -> Result<()> {
    // Grab the admin user to use as lead
    let user_col: mongodb::Collection<User> = db.db.collection("users");
    let admin = user_col
        .find_one(doc! { "email": "admin@teamder.app" })
        .await?
        .expect("seed_users must run before seed_projects");

    let mut p1 = Project::new(
        "Campus Event Finder",
        &admin.id,
        &admin.name,
        "A mobile-first web app that aggregates campus events across all departments and lets students RSVP, bookmark, and get reminders.",
    );
    p1.icon = "CE".into();
    p1.icon_bg = "linear-gradient(135deg, #DD6E42, #B85530)".into();
    p1.status = ProjectStatus::Recruiting;
    p1.goals = Some("Launch MVP before the semester break; onboard 200 active users.".into());
    p1.roles = vec![
        ProjectRole { name: "Mobile Developer".into(), count_needed: 2, filled: 0 },
        ProjectRole { name: "UI/UX Designer".into(), count_needed: 1, filled: 0 },
    ];
    p1.skills = vec!["React Native".into(), "Node.js".into(), "Figma".into()];
    p1.deadline = Some("2026-06-30".into());
    p1.collab = CollabMode::Hybrid;
    p1.duration = Some("3 months".into());
    p1.category = Some("Mobile App".into());

    let mut p2 = Project::new(
        "Open-Source Course Scheduler",
        &admin.id,
        &admin.name,
        "A constraint-solver-powered tool that auto-generates conflict-free course timetables from student preference forms.",
    );
    p2.icon = "CS".into();
    p2.icon_bg = "linear-gradient(135deg, #4F6D7A, #2C3E45)".into();
    p2.status = ProjectStatus::Active;
    p2.goals = Some("Support at least 500 concurrent users; publish to npm.".into());
    p2.roles = vec![
        ProjectRole { name: "Algorithm Engineer".into(), count_needed: 1, filled: 1 },
        ProjectRole { name: "Frontend Developer".into(), count_needed: 1, filled: 0 },
    ];
    p2.skills = vec!["TypeScript".into(), "React".into(), "Algorithms".into()];
    p2.deadline = Some("2026-08-15".into());
    p2.collab = CollabMode::Remote;
    p2.duration = Some("4 months".into());
    p2.category = Some("Developer Tool".into());

    let mut p3 = Project::new(
        "AI Study Buddy",
        &admin.id,
        &admin.name,
        "A RAG-based chatbot that ingests your lecture notes and answers questions, generates quizzes, and summarises key concepts.",
    );
    p3.icon = "AI".into();
    p3.icon_bg = "linear-gradient(135deg, #E8DAB2, #C9A96E)".into();
    p3.status = ProjectStatus::Recruiting;
    p3.roles = vec![
        ProjectRole { name: "ML Engineer".into(), count_needed: 1, filled: 0 },
        ProjectRole { name: "Backend Developer".into(), count_needed: 1, filled: 0 },
        ProjectRole { name: "UI/UX Designer".into(), count_needed: 1, filled: 0 },
    ];
    p3.skills = vec!["Python".into(), "LangChain".into(), "FastAPI".into(), "React".into()];
    p3.deadline = Some("2026-09-01".into());
    p3.collab = CollabMode::Remote;
    p3.duration = Some("5 months".into());
    p3.category = Some("AI / ML".into());

    let col: mongodb::Collection<Project> = db.db.collection("projects");
    col.insert_many(vec![p1, p2, p3]).await?;
    tracing::info!("  ✓ Inserted 3 seed projects");
    Ok(())
}

// ── Competitions ───────────────────────────────────────────────────────────

async fn seed_competitions(db: &DbClient) -> Result<()> {
    let mut c1 = Competition::new(
        "FJCU Hackathon 2026",
        "Fu Jen Catholic University",
        "48-hour hackathon open to all students. Build anything that solves a real campus problem. Prizes for top 3 teams.",
    );
    c1.icon = "FH".into();
    c1.icon_bg = "linear-gradient(135deg, #DD6E42, #B85530)".into();
    c1.status = CompetitionStatus::Open;
    c1.prize = "NT$60,000 total prize pool".into();
    c1.team_size_min = 2;
    c1.team_size_max = 4;
    c1.deadline = Some("2026-05-10".into());
    c1.duration = "48 hours".into();
    c1.tags = vec!["Hackathon".into(), "Open Theme".into(), "Campus".into()];
    c1.is_featured = true;

    let mut c2 = Competition::new(
        "AI Innovation Challenge",
        "Taiwan AI Academy",
        "Design an AI-powered solution for social good. Submissions judged on impact, novelty, and technical execution.",
    );
    c2.icon = "AI".into();
    c2.icon_bg = "linear-gradient(135deg, #4F6D7A, #2C3E45)".into();
    c2.status = CompetitionStatus::ClosingSoon;
    c2.prize = "NT$120,000 + mentorship".into();
    c2.team_size_min = 1;
    c2.team_size_max = 3;
    c2.deadline = Some("2026-04-25".into());
    c2.duration = "4 weeks".into();
    c2.tags = vec!["AI".into(), "Social Good".into(), "Research".into()];
    c2.is_featured = true;

    let mut c3 = Competition::new(
        "Web Dev Cup 2026",
        "Google Developer Student Clubs",
        "Build a full-stack web application in 72 hours. Any stack, any idea. Judged on UX, performance, and code quality.",
    );
    c3.icon = "WD".into();
    c3.icon_bg = "linear-gradient(135deg, #C0D6DF, #7FA8BB)".into();
    c3.status = CompetitionStatus::Upcoming;
    c3.prize = "Swag, cloud credits & certificates".into();
    c3.team_size_min = 2;
    c3.team_size_max = 5;
    c3.deadline = Some("2026-07-01".into());
    c3.duration = "72 hours".into();
    c3.tags = vec!["Web".into(), "Full-Stack".into(), "GDSC".into()];

    let col: mongodb::Collection<Competition> = db.db.collection("competitions");
    col.insert_many(vec![c1, c2, c3]).await?;
    tracing::info!("  ✓ Inserted 3 seed competitions");
    Ok(())
}

// ── Study Groups ───────────────────────────────────────────────────────────

async fn seed_study_groups(db: &DbClient) -> Result<()> {
    let user_col: mongodb::Collection<User> = db.db.collection("users");
    let admin = user_col
        .find_one(doc! { "email": "admin@teamder.app" })
        .await?
        .expect("seed_users must run before seed_study_groups");

    let mut g1 = StudyGroup::new(
        "Rust Systems Programming",
        "Work through 'The Rust Programming Language' book together, implement weekly exercises, and pair-review each other's code.",
        &admin.id,
    );
    g1.icon = "RS".into();
    g1.icon_bg = "linear-gradient(135deg, #DD6E42, #B85530)".into();
    g1.subject = "Systems Programming".into();
    g1.tags = vec!["Rust".into(), "Systems".into(), "Backend".into()];
    g1.max_members = 6;
    g1.schedule = "Every Saturday 14:00–16:00".into();
    g1.duration_weeks = 10;
    g1.current_week = 3;
    g1.is_open = true;

    let mut g2 = StudyGroup::new(
        "Algorithms & LeetCode Grind",
        "Daily LeetCode problems (easy → hard), weekly contest debrief, and mock interview sessions before graduation season.",
        &admin.id,
    );
    g2.icon = "AL".into();
    g2.icon_bg = "linear-gradient(135deg, #4F6D7A, #6B8593)".into();
    g2.subject = "Algorithms".into();
    g2.tags = vec!["Algorithms".into(), "LeetCode".into(), "Interview Prep".into()];
    g2.max_members = 8;
    g2.schedule = "Mon / Wed / Fri 21:00–22:00".into();
    g2.duration_weeks = 12;
    g2.current_week = 5;
    g2.is_open = true;

    let col: mongodb::Collection<StudyGroup> = db.db.collection("study_groups");
    col.insert_many(vec![g1, g2]).await?;
    tracing::info!("  ✓ Inserted 2 seed study groups");
    Ok(())
}
