use anyhow::Result;
use chrono::{Duration, Utc};
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use teamder_core::models::{
    competition::{Competition, CompetitionStatus, PublishStatus, Registration},
    project::{CollabMode, Project, ProjectRole, ProjectStatus, TeamMember},
    skill_catalog::{StoredSkillCategory, StoredSkillTag},
    study_group::{GroupMember, StudyGroup},
    user::{AvailabilityStatus, Review, Skill, SocialLink, User, WorkMode},
};

use crate::DbClient;

// ── Shared palettes ──────────────────────────────────────────────────────────

/// Avatar / icon background gradients, cycled by index for visual variety.
const GRADIENTS: &[&str] = &[
    "linear-gradient(135deg, #DD6E42, #E89070)",
    "linear-gradient(135deg, #4F6D7A, #6B8D9E)",
    "linear-gradient(135deg, #C0D6DF, #7FA8BB)",
    "linear-gradient(135deg, #E8DAB2, #C9A96E)",
    "linear-gradient(135deg, #9B5DE5, #C77DFF)",
    "linear-gradient(135deg, #00BBF9, #48CAE4)",
    "linear-gradient(135deg, #F15BB5, #FF8FAB)",
    "linear-gradient(135deg, #2EC4B6, #43E8B5)",
    "linear-gradient(135deg, #FF7B00, #FFB703)",
    "linear-gradient(135deg, #5F0F40, #9A031E)",
];

/// Solid colors used for member chips.
const COLORS: &[&str] = &[
    "#DD6E42", "#4F6D7A", "#7FA8BB", "#C9A96E", "#9B5DE5", "#00BBF9", "#F15BB5", "#2EC4B6",
    "#FF7B00", "#9A031E",
];

/// Reusable review bodies: (project_name, stars, body). Cycled to give every
/// user a handful of distinct-looking 評價 (peer reviews) in the seed data.
const REVIEW_TEMPLATES: &[(&str, u8, &str)] = &[
    ("Campus Event Finder", 5, "A fantastic collaborator — proactive, reliable, and always shipped ahead of schedule."),
    ("AI Study Buddy", 5, "Brilliant problem-solver. Communication was crystal clear from kickoff to launch."),
    ("Open-Source Course Scheduler", 4, "Solid technical skills and a real team player. Would happily work together again."),
    ("FJCU Hackathon 2025", 5, "Carried our team during the hackathon — incredible energy and laser focus."),
    ("Portfolio Website Revamp", 4, "Delivered high-quality work. Occasionally needed a deadline nudge but always came through."),
    ("Mobile Banking App", 5, "Exceptional attention to detail. The polish on the final product was outstanding."),
    ("Open Data Dashboard", 4, "Very dependable and easy to communicate with — a genuine pleasure to collaborate with."),
    ("Startup MVP Sprint", 5, "Took ownership of the hardest parts and made them look easy. Highly recommend!"),
    ("Research Paper Tooling", 4, "Strong domain knowledge and a thoughtful, patient approach to teamwork."),
    ("E-commerce Platform", 5, "Great mentor to the junior members and a true anchor for the whole team."),
    ("Smart Campus IoT", 4, "Reliable, skilled, and always willing to jump in and help others debug."),
    ("Design System Revamp", 5, "Creative and meticulous — elevated the quality of the entire project."),
];

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
    seed_skill_catalog(db).await?;

    tracing::info!("Seed complete");
    Ok(())
}

/// Seed the skill catalog from the hardcoded default if the
/// `skill_categories` collection is empty. Safe on every boot.
pub async fn seed_skill_catalog_if_empty(db: &DbClient) -> Result<()> {
    let count = db
        .db
        .collection::<StoredSkillCategory>("skill_categories")
        .count_documents(doc! {})
        .await?;
    if count > 0 {
        return Ok(());
    }
    seed_skill_catalog(db).await
}

async fn seed_skill_catalog(db: &DbClient) -> Result<()> {
    use teamder_core::skills::catalog as default_catalog;
    let cats_col = db.db.collection::<StoredSkillCategory>("skill_categories");
    let tags_col = db.db.collection::<StoredSkillTag>("skill_tags");

    for (cat_idx, cat) in default_catalog().into_iter().enumerate() {
        let stored_cat = StoredSkillCategory::new(
            cat.key.to_string(),
            cat.label.to_string(),
            cat.label_zh.to_string(),
            cat_idx as i32,
        );
        cats_col.insert_one(&stored_cat).await?;

        for (tag_idx, tag) in cat.skills.into_iter().enumerate() {
            let stored_tag = StoredSkillTag::new(
                tag.name.to_string(),
                tag.name_zh.to_string(),
                cat.key.to_string(),
                tag_idx as i32,
            );
            tags_col.insert_one(&stored_tag).await?;
        }
    }
    tracing::info!("Seeded skill catalog from default");
    Ok(())
}

// ── Users ──────────────────────────────────────────────────────────────────

/// Compact spec for a seeded user; expanded into a full `User` below.
struct UserSpec {
    email: &'static str,
    name: &'static str,
    role: &'static str,
    dept: &'static str,
    year: &'static str,
    location: &'static str,
    headline: &'static str,
    bio: &'static [&'static str],
    skills: &'static [(&'static str, u8)],
    interests: &'static [&'static str],
    languages: &'static [&'static str],
    work_mode: WorkMode,
    avail: AvailabilityStatus,
    hours: &'static str,
    rating: f32,
    projects_done: u32,
    collaborations: u32,
    match_score: u8,
    is_admin: bool,
}

async fn seed_users(db: &DbClient) -> Result<()> {
    let now = Utc::now();

    let specs: Vec<UserSpec> = vec![
        UserSpec {
            email: "admin@teamder.app", name: "Andy Chen", role: "Full-Stack Developer",
            dept: "Computer Science", year: "Year 3", location: "New Taipei City, Taiwan",
            headline: "Builder of developer tooling & team-matching platforms",
            bio: &["Building things that matter, one commit at a time.", "Passionate about open-source and developer tooling."],
            skills: &[("Rust", 85), ("TypeScript", 90), ("React", 88), ("MongoDB", 75)],
            interests: &["Open Source", "Hackathons", "Side Projects"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "15-20 hrs/week",
            rating: 4.9, projects_done: 12, collaborations: 8, match_score: 95, is_admin: true,
        },
        UserSpec {
            email: "alice@example.com", name: "Alice Wang", role: "UI/UX Designer",
            dept: "Digital Media Design", year: "Year 2", location: "Taipei, Taiwan",
            headline: "Designing experiences that feel effortless",
            bio: &["Design is not just what it looks like — design is how it works."],
            skills: &[("Figma", 92), ("CSS", 80), ("User Research", 85)],
            interests: &["Product Design", "Typography", "Illustration"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.7, projects_done: 6, collaborations: 4, match_score: 88, is_admin: false,
        },
        UserSpec {
            email: "bob@example.com", name: "Bob Lin", role: "Backend Engineer",
            dept: "Information Engineering", year: "Year 4", location: "Taichung, Taiwan",
            headline: "Go, Python, and strong opinions about database indexes",
            bio: &["Go, Python, and strong opinions about database indexes.", "Open to backend & infrastructure collaboration."],
            skills: &[("Go", 88), ("Python", 82), ("PostgreSQL", 79), ("Docker", 85)],
            interests: &["Distributed Systems", "Databases", "Performance"], languages: &["Chinese", "English"],
            work_mode: WorkMode::InPerson, avail: AvailabilityStatus::Busy, hours: "5-10 hrs/week",
            rating: 4.5, projects_done: 9, collaborations: 5, match_score: 76, is_admin: false,
        },
        UserSpec {
            email: "carol@example.com", name: "Carol Wu", role: "Frontend Developer",
            dept: "Computer Science", year: "Year 3", location: "Taipei, Taiwan",
            headline: "Pixel-perfect interfaces with a love for animation",
            bio: &["Turning Figma files into living, breathing interfaces."],
            skills: &[("React", 87), ("TypeScript", 84), ("Tailwind CSS", 90), ("Next.js", 80)],
            interests: &["Web Animation", "Design Systems", "Accessibility"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.6, projects_done: 7, collaborations: 6, match_score: 84, is_admin: false,
        },
        UserSpec {
            email: "david@example.com", name: "David Chang", role: "Data Scientist",
            dept: "Statistics", year: "Graduate", location: "Hsinchu, Taiwan",
            headline: "Finding the signal in the noise",
            bio: &["Numbers tell stories — I help them speak clearly."],
            skills: &[("Python", 90), ("Pandas", 88), ("SQL", 82), ("scikit-learn", 80)],
            interests: &["Data Visualization", "Statistics", "Research"], languages: &["Chinese", "English", "Japanese"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.8, projects_done: 10, collaborations: 7, match_score: 90, is_admin: false,
        },
        UserSpec {
            email: "emma@example.com", name: "Emma Liu", role: "Product Manager",
            dept: "Business Administration", year: "Year 4", location: "Taipei, Taiwan",
            headline: "Bridging users, design, and engineering",
            bio: &["I keep teams aligned and shipping the right things."],
            skills: &[("Product Strategy", 85), ("Figma", 70), ("Notion", 88), ("User Research", 80)],
            interests: &["Startups", "Growth", "UX"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "15-20 hrs/week",
            rating: 4.7, projects_done: 8, collaborations: 9, match_score: 86, is_admin: false,
        },
        UserSpec {
            email: "frank@example.com", name: "Frank Huang", role: "Mobile Developer",
            dept: "Information Management", year: "Year 3", location: "Kaohsiung, Taiwan",
            headline: "iOS & Android, one codebase at a time",
            bio: &["Cross-platform mobile dev who sweats the small UX details."],
            skills: &[("Flutter", 86), ("Dart", 84), ("React Native", 78), ("Firebase", 80)],
            interests: &["Mobile Apps", "Indie Dev", "UX"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::Busy, hours: "5-10 hrs/week",
            rating: 4.4, projects_done: 5, collaborations: 4, match_score: 79, is_admin: false,
        },
        UserSpec {
            email: "grace@example.com", name: "Grace Tsai", role: "ML Engineer",
            dept: "Computer Science", year: "Graduate", location: "Taipei, Taiwan",
            headline: "Teaching machines, learning from them",
            bio: &["Deep learning researcher with a soft spot for NLP."],
            skills: &[("Python", 91), ("PyTorch", 87), ("LangChain", 75), ("FastAPI", 78)],
            interests: &["LLMs", "Computer Vision", "Research"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "15-20 hrs/week",
            rating: 4.9, projects_done: 11, collaborations: 8, match_score: 93, is_admin: false,
        },
        UserSpec {
            email: "henry@example.com", name: "Henry Kuo", role: "DevOps Engineer",
            dept: "Information Engineering", year: "Year 4", location: "Taoyuan, Taiwan",
            headline: "Automate everything, sleep peacefully",
            bio: &["CI/CD, Kubernetes, and infrastructure as code enthusiast."],
            skills: &[("Kubernetes", 85), ("Docker", 90), ("Terraform", 80), ("AWS", 82)],
            interests: &["Cloud Native", "Automation", "SRE"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.6, projects_done: 9, collaborations: 6, match_score: 83, is_admin: false,
        },
        UserSpec {
            email: "ivy@example.com", name: "Ivy Chen", role: "Graphic Designer",
            dept: "Visual Communication Design", year: "Year 2", location: "Tainan, Taiwan",
            headline: "Visual storyteller & brand builder",
            bio: &["Branding, posters, and everything in between."],
            skills: &[("Illustrator", 90), ("Photoshop", 88), ("Branding", 85), ("Figma", 75)],
            interests: &["Branding", "Print Design", "Illustration"], languages: &["Chinese", "English"],
            work_mode: WorkMode::InPerson, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.5, projects_done: 6, collaborations: 5, match_score: 81, is_admin: false,
        },
        UserSpec {
            email: "jack@example.com", name: "Jack Yang", role: "Game Developer",
            dept: "Computer Science", year: "Year 3", location: "Taipei, Taiwan",
            headline: "Building worlds, one frame at a time",
            bio: &["Unity & Unreal dev who loves gameplay programming."],
            skills: &[("Unity", 88), ("C#", 85), ("Blender", 70), ("Game Design", 80)],
            interests: &["Game Dev", "3D Art", "Indie Games"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::Busy, hours: "5-10 hrs/week",
            rating: 4.3, projects_done: 4, collaborations: 3, match_score: 77, is_admin: false,
        },
        UserSpec {
            email: "karen@example.com", name: "Karen Hsu", role: "Marketing Specialist",
            dept: "Advertising", year: "Year 4", location: "Taipei, Taiwan",
            headline: "Growth-minded storyteller",
            bio: &["Content, campaigns, and community building."],
            skills: &[("SEO", 82), ("Copywriting", 88), ("Social Media", 85), ("Analytics", 75)],
            interests: &["Growth", "Branding", "Content"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.4, projects_done: 5, collaborations: 6, match_score: 78, is_admin: false,
        },
        UserSpec {
            email: "leo@example.com", name: "Leo Cheng", role: "Security Researcher",
            dept: "Information Engineering", year: "Graduate", location: "Hsinchu, Taiwan",
            headline: "Breaking things so others can't",
            bio: &["CTF player and web security enthusiast."],
            skills: &[("Penetration Testing", 86), ("Python", 84), ("Networking", 80), ("Cryptography", 78)],
            interests: &["CTF", "Security", "Privacy"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.7, projects_done: 7, collaborations: 5, match_score: 85, is_admin: false,
        },
        UserSpec {
            email: "mia@example.com", name: "Mia Lin", role: "Content Writer",
            dept: "Journalism", year: "Year 3", location: "Taipei, Taiwan",
            headline: "Words that connect & convert",
            bio: &["Tech writer translating complexity into clarity."],
            skills: &[("Copywriting", 90), ("Editing", 85), ("SEO", 72), ("Research", 80)],
            interests: &["Writing", "Journalism", "Tech"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.5, projects_done: 6, collaborations: 4, match_score: 80, is_admin: false,
        },
        UserSpec {
            email: "nathan@example.com", name: "Nathan Wang", role: "Embedded Engineer",
            dept: "Electrical Engineering", year: "Year 4", location: "Hsinchu, Taiwan",
            headline: "From silicon to software",
            bio: &["Firmware, IoT, and low-level optimization."],
            skills: &[("C", 88), ("Embedded C++", 85), ("Rust", 70), ("RTOS", 80)],
            interests: &["IoT", "Hardware", "Robotics"], languages: &["Chinese", "English"],
            work_mode: WorkMode::InPerson, avail: AvailabilityStatus::Busy, hours: "5-10 hrs/week",
            rating: 4.6, projects_done: 8, collaborations: 5, match_score: 82, is_admin: false,
        },
        UserSpec {
            email: "olivia@example.com", name: "Olivia Hsieh", role: "UX Researcher",
            dept: "Psychology", year: "Graduate", location: "Taipei, Taiwan",
            headline: "Understanding people to build better products",
            bio: &["Mixed-methods researcher who loves a good user interview."],
            skills: &[("User Research", 90), ("Usability Testing", 86), ("Figma", 70), ("Survey Design", 82)],
            interests: &["UX", "Psychology", "Accessibility"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.8, projects_done: 9, collaborations: 7, match_score: 88, is_admin: false,
        },
        UserSpec {
            email: "peter@example.com", name: "Peter Kao", role: "Blockchain Developer",
            dept: "Information Management", year: "Year 4", location: "Taipei, Taiwan",
            headline: "Smart contracts & decentralized apps",
            bio: &["Solidity dev exploring the web3 frontier."],
            skills: &[("Solidity", 84), ("Web3.js", 80), ("TypeScript", 78), ("Node.js", 76)],
            interests: &["Web3", "DeFi", "Crypto"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::Unavailable, hours: "5-10 hrs/week",
            rating: 4.2, projects_done: 4, collaborations: 3, match_score: 74, is_admin: false,
        },
        UserSpec {
            email: "queenie@example.com", name: "Queenie Lai", role: "Illustrator",
            dept: "Fine Arts", year: "Year 2", location: "Taichung, Taiwan",
            headline: "Bringing ideas to life with color",
            bio: &["Digital illustrator and concept artist."],
            skills: &[("Procreate", 90), ("Illustrator", 85), ("Concept Art", 88), ("Animation", 70)],
            interests: &["Illustration", "Concept Art", "Animation"], languages: &["Chinese", "English"],
            work_mode: WorkMode::InPerson, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.6, projects_done: 6, collaborations: 4, match_score: 80, is_admin: false,
        },
        UserSpec {
            email: "ryan@example.com", name: "Ryan Su", role: "Cloud Architect",
            dept: "Computer Science", year: "Graduate", location: "Taipei, Taiwan",
            headline: "Designing systems that scale",
            bio: &["Cloud-native architect with a systems-design obsession."],
            skills: &[("AWS", 88), ("System Design", 90), ("Go", 80), ("Kubernetes", 84)],
            interests: &["Architecture", "Cloud", "Scalability"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "15-20 hrs/week",
            rating: 4.9, projects_done: 13, collaborations: 10, match_score: 94, is_admin: false,
        },
        UserSpec {
            email: "sophia@example.com", name: "Sophia Chou", role: "Business Analyst",
            dept: "Economics", year: "Year 4", location: "Taipei, Taiwan",
            headline: "Data-driven decisions, business outcomes",
            bio: &["Bridging business needs and technical solutions."],
            skills: &[("SQL", 84), ("Excel", 88), ("Tableau", 82), ("Python", 70)],
            interests: &["Analytics", "Finance", "Strategy"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Hybrid, avail: AvailabilityStatus::OpenForCollab, hours: "10-15 hrs/week",
            rating: 4.5, projects_done: 7, collaborations: 6, match_score: 81, is_admin: false,
        },
        UserSpec {
            email: "tom@example.com", name: "Tom Lee", role: "AI Researcher",
            dept: "Computer Science", year: "Graduate", location: "Hsinchu, Taiwan",
            headline: "Pushing the boundaries of machine intelligence",
            bio: &["Researcher focused on reinforcement learning and agents."],
            skills: &[("Python", 92), ("PyTorch", 88), ("Reinforcement Learning", 85), ("JAX", 75)],
            interests: &["AI Research", "RL", "Agents"], languages: &["Chinese", "English"],
            work_mode: WorkMode::Remote, avail: AvailabilityStatus::OpenForCollab, hours: "15-20 hrs/week",
            rating: 4.8, projects_done: 10, collaborations: 8, match_score: 92, is_admin: false,
        },
    ];

    // First pass: build all users so their IDs exist for cross-referencing reviews.
    let mut users: Vec<User> = specs
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let mut u = User::new(s.email, s.name, s.role, s.dept);
            if s.is_admin {
                u.is_admin = true;
                u.is_publisher = true;
            }
            u.university = "Fu Jen Catholic University".into();
            u.year = s.year.into();
            u.location = Some(s.location.into());
            u.headline = Some(s.headline.into());
            u.bio = s.bio.iter().map(|b| b.to_string()).collect();
            u.skills = s
                .skills
                .iter()
                .map(|(n, l)| Skill { name: n.to_string(), level: *l })
                .collect();
            u.skill_tags = s.skills.iter().map(|(n, _)| n.to_string()).collect();
            u.interests = s.interests.iter().map(|x| x.to_string()).collect();
            u.languages = s.languages.iter().map(|x| x.to_string()).collect();
            u.gradient = GRADIENTS[i % GRADIENTS.len()].into();
            u.work_mode = s.work_mode;
            u.availability = s.avail;
            u.hours_per_week = s.hours.into();
            u.rating = s.rating;
            u.projects_done = s.projects_done;
            u.collaborations = s.collaborations;
            u.match_score = s.match_score;
            u.onboarded = true;
            u.timezone = Some("Asia/Taipei".into());
            u.social_links = vec![SocialLink {
                label: "GitHub".into(),
                url: format!("https://github.com/{}", s.email.split('@').next().unwrap()),
            }];
            u.created_at = now;
            u.updated_at = now;
            u
        })
        .collect();

    // Second pass: give every user ~5 peer reviews (評價) from other users.
    let id_name: Vec<(String, String)> = users
        .iter()
        .map(|u| (u.id.clone(), u.name.clone()))
        .collect();
    let n = id_name.len();
    for i in 0..n {
        let mut reviews = Vec::with_capacity(5);
        for k in 0..5 {
            let reviewer = (i + k + 1) % n; // always a different user
            let t = REVIEW_TEMPLATES[(i + k) % REVIEW_TEMPLATES.len()];
            reviews.push(Review {
                reviewer_id: id_name[reviewer].0.clone(),
                reviewer_name: id_name[reviewer].1.clone(),
                project_name: t.0.into(),
                stars: t.1,
                body: t.2.into(),
                created_at: now - Duration::days((k as i64 + 1) * 9),
            });
        }
        users[i].reviews = reviews;
    }

    let col: mongodb::Collection<User> = db.db.collection("users");
    col.insert_many(&users).await?;
    tracing::info!("  ✓ Inserted {} seed users (each with 5 reviews)", n);
    Ok(())
}

/// Load all seeded users so projects / groups can reference real IDs.
async fn all_users(db: &DbClient) -> Result<Vec<User>> {
    let user_col: mongodb::Collection<User> = db.db.collection("users");
    let users: Vec<User> = user_col.find(doc! {}).await?.try_collect().await?;
    Ok(users)
}

// ── Projects ───────────────────────────────────────────────────────────────

struct ProjectSpec {
    name: &'static str,
    icon: &'static str,
    status: ProjectStatus,
    description: &'static str,
    goals: &'static str,
    roles: &'static [(&'static str, u8, u8)],
    skills: &'static [&'static str],
    deadline: &'static str,
    collab: CollabMode,
    duration: &'static str,
    category: &'static str,
}

async fn seed_projects(db: &DbClient) -> Result<()> {
    let users = all_users(db).await?;
    let n = users.len();
    let now = Utc::now();

    let specs: Vec<ProjectSpec> = vec![
        ProjectSpec { name: "Campus Event Finder", icon: "CE", status: ProjectStatus::Recruiting,
            description: "A mobile-first web app that aggregates campus events across all departments and lets students RSVP, bookmark, and get reminders.",
            goals: "Launch MVP before the semester break; onboard 200 active users.",
            roles: &[("Mobile Developer", 2, 0), ("UI/UX Designer", 1, 0)],
            skills: &["React Native", "Node.js", "Figma"], deadline: "2026-06-30", collab: CollabMode::Hybrid, duration: "3 months", category: "Mobile App" },
        ProjectSpec { name: "Open-Source Course Scheduler", icon: "CS", status: ProjectStatus::Active,
            description: "A constraint-solver-powered tool that auto-generates conflict-free course timetables from student preference forms.",
            goals: "Support at least 500 concurrent users; publish to npm.",
            roles: &[("Algorithm Engineer", 1, 1), ("Frontend Developer", 1, 0)],
            skills: &["TypeScript", "React", "Algorithms"], deadline: "2026-08-15", collab: CollabMode::Remote, duration: "4 months", category: "Developer Tool" },
        ProjectSpec { name: "AI Study Buddy", icon: "AI", status: ProjectStatus::Recruiting,
            description: "A RAG-based chatbot that ingests your lecture notes and answers questions, generates quizzes, and summarises key concepts.",
            goals: "Ship a working demo for the AI showcase.",
            roles: &[("ML Engineer", 1, 0), ("Backend Developer", 1, 0), ("UI/UX Designer", 1, 0)],
            skills: &["Python", "LangChain", "FastAPI", "React"], deadline: "2026-09-01", collab: CollabMode::Remote, duration: "5 months", category: "AI / ML" },
        ProjectSpec { name: "Smart Campus IoT", icon: "IO", status: ProjectStatus::Active,
            description: "A network of sensors monitoring classroom occupancy, air quality, and energy usage, surfaced through a real-time dashboard.",
            goals: "Deploy 20 sensor nodes across the engineering building.",
            roles: &[("Embedded Engineer", 2, 1), ("Backend Developer", 1, 0)],
            skills: &["C", "MQTT", "Grafana", "Python"], deadline: "2026-07-20", collab: CollabMode::InPerson, duration: "6 months", category: "IoT" },
        ProjectSpec { name: "Peer Tutoring Marketplace", icon: "PT", status: ProjectStatus::Recruiting,
            description: "A two-sided platform connecting students who need help with peers who can tutor, complete with scheduling and payments.",
            goals: "Validate with 50 tutoring sessions in the first month.",
            roles: &[("Full-Stack Developer", 2, 0), ("Product Manager", 1, 0)],
            skills: &["Next.js", "PostgreSQL", "Stripe"], deadline: "2026-10-01", collab: CollabMode::Hybrid, duration: "4 months", category: "Web App" },
        ProjectSpec { name: "Carbon Footprint Tracker", icon: "CF", status: ProjectStatus::Recruiting,
            description: "A gamified app that helps students track and reduce their daily carbon footprint with challenges and leaderboards.",
            goals: "Reach 1,000 downloads and a 30% weekly retention rate.",
            roles: &[("Mobile Developer", 1, 0), ("Data Scientist", 1, 0), ("UI/UX Designer", 1, 0)],
            skills: &["Flutter", "Firebase", "Python"], deadline: "2026-11-15", collab: CollabMode::Remote, duration: "5 months", category: "Mobile App" },
        ProjectSpec { name: "Open Data Dashboard", icon: "OD", status: ProjectStatus::Active,
            description: "An interactive visualization of Taiwan's open government datasets, making public data accessible and actionable.",
            goals: "Cover 10 datasets with rich, filterable charts.",
            roles: &[("Frontend Developer", 1, 1), ("Data Scientist", 1, 0)],
            skills: &["D3.js", "React", "Pandas"], deadline: "2026-08-30", collab: CollabMode::Remote, duration: "3 months", category: "Data Viz" },
        ProjectSpec { name: "Indie Game Jam Collective", icon: "GJ", status: ProjectStatus::Recruiting,
            description: "A recurring game jam crew building small, polished games every two months and releasing them on itch.io.",
            goals: "Release 3 games this semester.",
            roles: &[("Game Developer", 2, 0), ("Illustrator", 1, 0), ("Sound Designer", 1, 0)],
            skills: &["Unity", "C#", "Blender"], deadline: "2026-12-01", collab: CollabMode::Hybrid, duration: "6 months", category: "Game Dev" },
        ProjectSpec { name: "Mental Health Companion", icon: "MH", status: ProjectStatus::Recruiting,
            description: "A privacy-first journaling app with mood tracking and gentle, evidence-based prompts for student wellbeing.",
            goals: "Partner with the campus counseling center for a pilot.",
            roles: &[("Mobile Developer", 1, 0), ("UX Researcher", 1, 0)],
            skills: &["React Native", "Node.js", "Figma"], deadline: "2026-09-30", collab: CollabMode::Remote, duration: "4 months", category: "Health" },
        ProjectSpec { name: "Resume Builder AI", icon: "RB", status: ProjectStatus::Active,
            description: "An AI-assisted resume builder tailored for new grads, with ATS scoring and one-click formatting.",
            goals: "Generate 500 resumes during recruiting season.",
            roles: &[("Frontend Developer", 1, 1), ("ML Engineer", 1, 0)],
            skills: &["Next.js", "OpenAI API", "TypeScript"], deadline: "2026-07-10", collab: CollabMode::Remote, duration: "3 months", category: "AI / ML" },
        ProjectSpec { name: "Campus Marketplace", icon: "MP", status: ProjectStatus::Recruiting,
            description: "A trusted second-hand marketplace exclusive to verified university students, with in-app chat and meetup spots.",
            goals: "Onboard 300 listings in the first month.",
            roles: &[("Full-Stack Developer", 2, 0), ("UI/UX Designer", 1, 0)],
            skills: &["React", "MongoDB", "WebSockets"], deadline: "2026-10-20", collab: CollabMode::Hybrid, duration: "4 months", category: "Web App" },
        ProjectSpec { name: "Lecture Note Sharing", icon: "LN", status: ProjectStatus::Active,
            description: "A collaborative platform where students share, rate, and improve lecture notes per course, version-controlled.",
            goals: "Cover 50 popular courses with quality notes.",
            roles: &[("Backend Developer", 1, 1), ("Frontend Developer", 1, 0)],
            skills: &["Django", "PostgreSQL", "React"], deadline: "2026-09-15", collab: CollabMode::Remote, duration: "4 months", category: "Web App" },
        ProjectSpec { name: "AR Campus Navigator", icon: "AR", status: ProjectStatus::Recruiting,
            description: "An augmented-reality wayfinding app that guides freshmen around campus with on-screen directions.",
            goals: "Map all main buildings and demo at orientation.",
            roles: &[("AR Developer", 1, 0), ("3D Artist", 1, 0)],
            skills: &["Unity", "ARKit", "Blender"], deadline: "2026-08-01", collab: CollabMode::InPerson, duration: "5 months", category: "AR / VR" },
        ProjectSpec { name: "Volunteer Hours Tracker", icon: "VH", status: ProjectStatus::Completed,
            description: "A simple, verified system for logging and certifying student volunteer hours for scholarship applications.",
            goals: "Adopted by 5 student organizations.",
            roles: &[("Full-Stack Developer", 1, 1)],
            skills: &["Vue", "Firebase"], deadline: "2026-03-01", collab: CollabMode::Remote, duration: "2 months", category: "Web App" },
        ProjectSpec { name: "Crypto Portfolio Dashboard", icon: "CP", status: ProjectStatus::Recruiting,
            description: "A clean dashboard aggregating crypto holdings across wallets with PnL tracking and tax-friendly exports.",
            goals: "Support the top 5 chains and CSV export.",
            roles: &[("Blockchain Developer", 1, 0), ("Frontend Developer", 1, 0)],
            skills: &["Web3.js", "React", "TypeScript"], deadline: "2026-11-01", collab: CollabMode::Remote, duration: "4 months", category: "Web3" },
        ProjectSpec { name: "Recipe Sharing Social App", icon: "RS", status: ProjectStatus::Active,
            description: "A social app for sharing dorm-friendly recipes, with grocery lists and step-by-step cooking mode.",
            goals: "Build a community of 500 active cooks.",
            roles: &[("Mobile Developer", 1, 1), ("Content Writer", 1, 0)],
            skills: &["Flutter", "Firebase", "Copywriting"], deadline: "2026-10-10", collab: CollabMode::Hybrid, duration: "3 months", category: "Mobile App" },
        ProjectSpec { name: "Study Room Booking", icon: "SR", status: ProjectStatus::Recruiting,
            description: "A real-time booking system for library study rooms with no-show penalties and QR check-in.",
            goals: "Replace the paper sign-up sheet entirely.",
            roles: &[("Backend Developer", 1, 0), ("Frontend Developer", 1, 0)],
            skills: &["Go", "PostgreSQL", "React"], deadline: "2026-09-05", collab: CollabMode::InPerson, duration: "3 months", category: "Web App" },
        ProjectSpec { name: "Podcast Transcription Tool", icon: "PD", status: ProjectStatus::Active,
            description: "A web tool that transcribes and summarizes podcasts and lectures using speech-to-text and LLM summaries.",
            goals: "Process 100 hours of audio in beta.",
            roles: &[("ML Engineer", 1, 1), ("Frontend Developer", 1, 0)],
            skills: &["Python", "Whisper", "Next.js"], deadline: "2026-08-20", collab: CollabMode::Remote, duration: "4 months", category: "AI / ML" },
        ProjectSpec { name: "Fitness Buddy Matcher", icon: "FB", status: ProjectStatus::Recruiting,
            description: "An app that pairs students with compatible workout partners based on goals, schedule, and gym location.",
            goals: "Facilitate 200 successful matches.",
            roles: &[("Mobile Developer", 1, 0), ("Data Scientist", 1, 0)],
            skills: &["React Native", "Python", "Firebase"], deadline: "2026-11-20", collab: CollabMode::Hybrid, duration: "4 months", category: "Mobile App" },
        ProjectSpec { name: "Open-Source UI Kit", icon: "UI", status: ProjectStatus::Active,
            description: "An accessible, themeable React component library built for student hackathon teams to move fast.",
            goals: "Publish v1.0 with 30+ components to npm.",
            roles: &[("Frontend Developer", 2, 1), ("UI/UX Designer", 1, 0)],
            skills: &["React", "TypeScript", "Tailwind CSS", "Storybook"], deadline: "2026-10-30", collab: CollabMode::Remote, duration: "5 months", category: "Developer Tool" },
        ProjectSpec { name: "Scholarship Finder", icon: "SF", status: ProjectStatus::Recruiting,
            description: "A searchable database of scholarships with smart matching based on a student's profile and eligibility.",
            goals: "Index 1,000 scholarships and notify matches.",
            roles: &[("Full-Stack Developer", 1, 0), ("Data Scientist", 1, 0)],
            skills: &["Next.js", "PostgreSQL", "Python"], deadline: "2026-09-25", collab: CollabMode::Remote, duration: "4 months", category: "Web App" },
        ProjectSpec { name: "Campus Carpool", icon: "CC", status: ProjectStatus::Recruiting,
            description: "A ride-sharing board for commuter students to coordinate carpools and split costs safely.",
            goals: "Launch in time for the new commuter cohort.",
            roles: &[("Mobile Developer", 1, 0), ("Backend Developer", 1, 0)],
            skills: &["Flutter", "Node.js", "Maps API"], deadline: "2026-12-15", collab: CollabMode::Hybrid, duration: "4 months", category: "Mobile App" },
    ];

    let mut projects: Vec<Project> = Vec::with_capacity(specs.len());
    for (i, s) in specs.into_iter().enumerate() {
        let lead = &users[i % n];
        let mut p = Project::new(s.name, &lead.id, s.description);
        p.icon = s.icon.into();
        p.icon_bg = GRADIENTS[i % GRADIENTS.len()].into();
        p.status = s.status;
        p.goals = Some(s.goals.into());
        p.roles = s
            .roles
            .iter()
            .map(|(name, needed, filled)| ProjectRole {
                name: name.to_string(),
                count_needed: *needed,
                filled: *filled,
            })
            .collect();
        p.skills = s.skills.iter().map(|x| x.to_string()).collect();
        p.deadline = Some(s.deadline.into());
        p.collab = s.collab;
        p.duration = Some(s.duration.into());
        p.category = Some(s.category.into());
        p.is_promoted = i % 7 == 0;

        // Lead is always on the team; add a couple of other members.
        let mut team = vec![TeamMember {
            user_id: lead.id.clone(),
            initials: lead.initials.clone(),
            color: COLORS[i % COLORS.len()].into(),
            joined_at: now,
            role: Some("Lead".into()),
        }];
        for k in 1..=2 {
            let m = &users[(i + k) % n];
            if m.id == lead.id {
                continue;
            }
            team.push(TeamMember {
                user_id: m.id.clone(),
                initials: m.initials.clone(),
                color: COLORS[(i + k) % COLORS.len()].into(),
                joined_at: now,
                role: None,
            });
        }
        p.team = team;
        p.created_at = now;
        p.updated_at = now;
        projects.push(p);
    }

    let count = projects.len();
    let col: mongodb::Collection<Project> = db.db.collection("projects");
    col.insert_many(&projects).await?;
    tracing::info!("  ✓ Inserted {} seed projects", count);
    Ok(())
}

// ── Competitions ───────────────────────────────────────────────────────────

struct CompSpec {
    name: &'static str,
    organizer: &'static str,
    icon: &'static str,
    status: CompetitionStatus,
    prize: &'static str,
    min: u8,
    max: u8,
    deadline: &'static str,
    duration: &'static str,
    tags: &'static [&'static str],
    description: &'static str,
    featured: bool,
}

async fn seed_competitions(db: &DbClient) -> Result<()> {
    let users = all_users(db).await?;
    let n = users.len();
    let now = Utc::now();

    let specs: Vec<CompSpec> = vec![
        CompSpec { name: "FJCU Hackathon 2026", organizer: "Fu Jen Catholic University", icon: "FH", status: CompetitionStatus::Open,
            prize: "NT$60,000 total prize pool", min: 2, max: 4, deadline: "2026-06-10", duration: "48 hours",
            tags: &["Hackathon", "Open Theme", "Campus"], description: "48-hour hackathon open to all students. Build anything that solves a real campus problem. Prizes for top 3 teams.", featured: true },
        CompSpec { name: "AI Innovation Challenge", organizer: "Taiwan AI Academy", icon: "AI", status: CompetitionStatus::ClosingSoon,
            prize: "NT$120,000 + mentorship", min: 1, max: 3, deadline: "2026-06-25", duration: "4 weeks",
            tags: &["AI", "Social Good", "Research"], description: "Design an AI-powered solution for social good. Submissions judged on impact, novelty, and technical execution.", featured: true },
        CompSpec { name: "Web Dev Cup 2026", organizer: "Google Developer Student Clubs", icon: "WD", status: CompetitionStatus::Upcoming,
            prize: "Swag, cloud credits & certificates", min: 2, max: 5, deadline: "2026-07-01", duration: "72 hours",
            tags: &["Web", "Full-Stack", "GDSC"], description: "Build a full-stack web application in 72 hours. Any stack, any idea. Judged on UX, performance, and code quality.", featured: false },
        CompSpec { name: "Mobile App Showdown", organizer: "Taipei Tech Meetup", icon: "MA", status: CompetitionStatus::Open,
            prize: "NT$80,000 + App Store feature", min: 1, max: 4, deadline: "2026-07-15", duration: "3 weeks",
            tags: &["Mobile", "iOS", "Android"], description: "Ship a polished mobile app in three weeks. Judged on design, usefulness, and execution.", featured: true },
        CompSpec { name: "Data Science Marathon", organizer: "Kaggle Taiwan", icon: "DS", status: CompetitionStatus::Open,
            prize: "NT$100,000 prize pool", min: 1, max: 3, deadline: "2026-08-01", duration: "6 weeks",
            tags: &["Data Science", "ML", "Kaggle"], description: "A real-world prediction challenge with a live leaderboard. Bring your best models.", featured: false },
        CompSpec { name: "Cybersecurity CTF", organizer: "HITCON", icon: "CT", status: CompetitionStatus::ClosingSoon,
            prize: "NT$150,000 + internships", min: 1, max: 4, deadline: "2026-06-20", duration: "36 hours",
            tags: &["Security", "CTF", "Hacking"], description: "Capture the flag across web, pwn, crypto, and forensics. For aspiring security researchers.", featured: true },
        CompSpec { name: "Green Tech Pitch", organizer: "Sustainability Hub", icon: "GT", status: CompetitionStatus::Upcoming,
            prize: "NT$200,000 seed funding", min: 2, max: 5, deadline: "2026-09-01", duration: "5 weeks",
            tags: &["Sustainability", "Startup", "Pitch"], description: "Pitch a tech solution to a climate or sustainability challenge to a panel of investors.", featured: true },
        CompSpec { name: "Game Dev Jam", organizer: "IGDA Taipei", icon: "GD", status: CompetitionStatus::Open,
            prize: "NT$50,000 + publishing deal", min: 1, max: 4, deadline: "2026-07-30", duration: "1 week",
            tags: &["Game Dev", "Unity", "Jam"], description: "Build a game around a surprise theme in one week. Solo or small teams welcome.", featured: false },
        CompSpec { name: "Fintech Build-off", organizer: "Taiwan Fintech Association", icon: "FT", status: CompetitionStatus::Upcoming,
            prize: "NT$180,000 + accelerator slot", min: 2, max: 4, deadline: "2026-09-15", duration: "4 weeks",
            tags: &["Fintech", "Web3", "Startup"], description: "Prototype a fintech product solving a real consumer pain point. Demo day with industry judges.", featured: false },
        CompSpec { name: "UX Design Sprint", organizer: "Design FJCU", icon: "UX", status: CompetitionStatus::Open,
            prize: "NT$40,000 + portfolio review", min: 1, max: 3, deadline: "2026-07-05", duration: "2 weeks",
            tags: &["Design", "UX", "Figma"], description: "Solve a real design brief end-to-end: research, ideation, prototype, and testing.", featured: false },
        CompSpec { name: "Robotics Grand Prix", organizer: "EE Department", icon: "RG", status: CompetitionStatus::Upcoming,
            prize: "NT$90,000 + lab access", min: 3, max: 6, deadline: "2026-10-01", duration: "8 weeks",
            tags: &["Robotics", "Hardware", "Embedded"], description: "Design and race an autonomous robot through an obstacle course. Hardware provided.", featured: false },
        CompSpec { name: "Open Source Contribution Month", organizer: "Open Source Taiwan", icon: "OS", status: CompetitionStatus::Open,
            prize: "Swag + maintainer mentorship", min: 1, max: 1, deadline: "2026-08-31", duration: "1 month",
            tags: &["Open Source", "Community", "Git"], description: "Make meaningful contributions to open-source projects throughout the month. Quality over quantity.", featured: false },
        CompSpec { name: "AR/VR Experience Contest", organizer: "XR Taiwan", icon: "VR", status: CompetitionStatus::Upcoming,
            prize: "NT$70,000 + headset bundle", min: 1, max: 4, deadline: "2026-09-20", duration: "5 weeks",
            tags: &["AR", "VR", "Unity"], description: "Create an immersive AR or VR experience. Education, art, or entertainment — your call.", featured: false },
        CompSpec { name: "Algorithms Olympiad", organizer: "ACM-ICPC Taiwan", icon: "AO", status: CompetitionStatus::ClosingSoon,
            prize: "NT$60,000 + ICPC qualifier", min: 3, max: 3, deadline: "2026-06-18", duration: "5 hours",
            tags: &["Algorithms", "Competitive", "ICPC"], description: "Classic competitive programming contest. Three-person teams, one computer, five hours.", featured: true },
        CompSpec { name: "HealthTech Hackathon", organizer: "Med School Innovation Lab", icon: "HT", status: CompetitionStatus::Upcoming,
            prize: "NT$110,000 + clinical pilot", min: 2, max: 5, deadline: "2026-10-10", duration: "48 hours",
            tags: &["Health", "Hackathon", "Impact"], description: "Build technology that improves patient care or healthcare access. Clinicians on the judging panel.", featured: false },
        CompSpec { name: "EdTech Innovation Award", organizer: "Ministry of Education", icon: "ED", status: CompetitionStatus::Open,
            prize: "NT$250,000 grant", min: 2, max: 6, deadline: "2026-08-15", duration: "6 weeks",
            tags: &["EdTech", "Education", "Impact"], description: "Reimagine learning with technology. Grants awarded to the most promising prototypes.", featured: true },
        CompSpec { name: "Cloud Architecture Challenge", organizer: "AWS Educate", icon: "CA", status: CompetitionStatus::Upcoming,
            prize: "Cloud credits + certification vouchers", min: 1, max: 3, deadline: "2026-09-30", duration: "3 weeks",
            tags: &["Cloud", "AWS", "Architecture"], description: "Design a scalable, cost-efficient cloud architecture for a given scenario. Best design wins.", featured: false },
        CompSpec { name: "Indie Music + Code", organizer: "Creative Coding Club", icon: "MC", status: CompetitionStatus::Open,
            prize: "NT$45,000 + studio time", min: 1, max: 3, deadline: "2026-07-25", duration: "2 weeks",
            tags: &["Creative", "Audio", "Generative"], description: "Blend music and code: generative audio, interactive visualizers, or music tools.", featured: false },
        CompSpec { name: "Accessibility-First Design", organizer: "a11y Taiwan", icon: "A1", status: CompetitionStatus::Upcoming,
            prize: "NT$55,000 + conference passes", min: 1, max: 4, deadline: "2026-10-05", duration: "3 weeks",
            tags: &["Accessibility", "Design", "Inclusive"], description: "Build something that's genuinely accessible to everyone. Judged with assistive tech in mind.", featured: false },
        CompSpec { name: "Smart City Datathon", organizer: "Taipei City Government", icon: "SC", status: CompetitionStatus::Open,
            prize: "NT$130,000 + city pilot", min: 2, max: 5, deadline: "2026-08-20", duration: "4 weeks",
            tags: &["Smart City", "Data", "Civic"], description: "Use open city data to propose solutions for urban challenges. Winning ideas get piloted.", featured: true },
        CompSpec { name: "Startup Weekend FJCU", organizer: "Techstars", icon: "SW", status: CompetitionStatus::Upcoming,
            prize: "NT$100,000 + incubation", min: 3, max: 6, deadline: "2026-11-01", duration: "54 hours",
            tags: &["Startup", "Pitch", "Entrepreneurship"], description: "Go from idea to pitch in 54 hours. Form a team, build an MVP, and present to investors.", featured: false },
    ];

    let mut comps: Vec<Competition> = Vec::with_capacity(specs.len());
    for (i, s) in specs.into_iter().enumerate() {
        let mut c = Competition::new(s.name, s.organizer, s.description);
        c.icon = s.icon.into();
        c.icon_bg = GRADIENTS[i % GRADIENTS.len()].into();
        c.status = s.status;
        c.prize = s.prize.into();
        c.team_size_min = s.min;
        c.team_size_max = s.max;
        c.deadline = Some(s.deadline.into());
        c.duration = s.duration.into();
        c.tags = s.tags.iter().map(|x| x.to_string()).collect();
        c.is_featured = s.featured;
        c.publish_status = PublishStatus::Published;

        // A few registrations + interested users, drawn from the seeded users.
        for k in 0..(2 + i % 4) {
            let u = &users[(i + k) % n];
            c.registrations.push(Registration {
                user_id: u.id.clone(),
                team_name: Some(format!("Team {}", (b'A' + (k as u8 % 26)) as char)),
                registered_at: now - Duration::days(k as i64 + 1),
            });
        }
        for k in 0..(3 + i % 5) {
            let u = &users[(i + k + 2) % n];
            if !c.interested_user_ids.contains(&u.id) {
                c.interested_user_ids.push(u.id.clone());
            }
        }
        c.created_at = now;
        c.updated_at = now;
        comps.push(c);
    }

    let count = comps.len();
    let col: mongodb::Collection<Competition> = db.db.collection("competitions");
    col.insert_many(&comps).await?;
    tracing::info!("  ✓ Inserted {} seed competitions", count);
    Ok(())
}

// ── Study Groups ───────────────────────────────────────────────────────────

struct GroupSpec {
    name: &'static str,
    goal: &'static str,
    icon: &'static str,
    subject: &'static str,
    tags: &'static [&'static str],
    max: u8,
    schedule: &'static str,
    weeks: u8,
    current_week: u8,
    is_open: bool,
    description: &'static str,
}

async fn seed_study_groups(db: &DbClient) -> Result<()> {
    let users = all_users(db).await?;
    let n = users.len();
    let now = Utc::now();

    let specs: Vec<GroupSpec> = vec![
        GroupSpec { name: "Rust Systems Programming", goal: "Work through 'The Rust Programming Language' book together, implement weekly exercises, and pair-review each other's code.",
            icon: "RS", subject: "Systems Programming", tags: &["Rust", "Systems", "Backend"], max: 6, schedule: "Every Saturday 14:00–16:00", weeks: 10, current_week: 3, is_open: true,
            description: "A hands-on group for diving deep into Rust and systems programming." },
        GroupSpec { name: "Algorithms & LeetCode Grind", goal: "Daily LeetCode problems (easy → hard), weekly contest debrief, and mock interview sessions before graduation season.",
            icon: "AL", subject: "Algorithms", tags: &["Algorithms", "LeetCode", "Interview Prep"], max: 8, schedule: "Mon / Wed / Fri 21:00–22:00", weeks: 12, current_week: 5, is_open: true,
            description: "Sharpen your problem-solving and ace technical interviews together." },
        GroupSpec { name: "Machine Learning Reading Circle", goal: "Read and discuss one seminal ML paper each week, with someone presenting and a group discussion afterward.",
            icon: "ML", subject: "Machine Learning", tags: &["ML", "Research", "Papers"], max: 10, schedule: "Every Thursday 19:00–21:00", weeks: 14, current_week: 6, is_open: true,
            description: "From classic papers to the latest LLM research." },
        GroupSpec { name: "Frontend Masters Cohort", goal: "Build progressively complex React projects, review patterns, and learn modern frontend tooling hands-on.",
            icon: "FE", subject: "Frontend Development", tags: &["React", "TypeScript", "Frontend"], max: 8, schedule: "Every Sunday 15:00–17:00", weeks: 10, current_week: 4, is_open: true,
            description: "Level up from React basics to production patterns." },
        GroupSpec { name: "System Design Interview Prep", goal: "Work through 'Designing Data-Intensive Applications' and practice whiteboard system design weekly.",
            icon: "SD", subject: "System Design", tags: &["System Design", "Backend", "Interview Prep"], max: 6, schedule: "Every Tuesday 20:00–22:00", weeks: 12, current_week: 2, is_open: true,
            description: "Prepare for senior-level system design interviews." },
        GroupSpec { name: "Japanese for Tech", goal: "Learn conversational and technical Japanese for internships and study-abroad in Japan's tech industry.",
            icon: "JP", subject: "Language", tags: &["Japanese", "Language", "Career"], max: 8, schedule: "Mon / Thu 18:30–19:30", weeks: 16, current_week: 8, is_open: true,
            description: "From hiragana to reading Japanese documentation." },
        GroupSpec { name: "Competitive Programming Squad", goal: "Train for ICPC and Codeforces rounds with weekly virtual contests and editorial study.",
            icon: "CP", subject: "Competitive Programming", tags: &["Competitive", "Algorithms", "ICPC"], max: 6, schedule: "Every Saturday 09:00–12:00", weeks: 12, current_week: 7, is_open: false,
            description: "Serious training for competitive programming contests." },
        GroupSpec { name: "Cloud & DevOps Bootcamp", goal: "Hands-on with Docker, Kubernetes, and CI/CD, building and deploying a real project together.",
            icon: "CD", subject: "DevOps", tags: &["Docker", "Kubernetes", "DevOps"], max: 8, schedule: "Every Wednesday 19:00–21:00", weeks: 10, current_week: 3, is_open: true,
            description: "Master the modern cloud-native toolchain." },
        GroupSpec { name: "UI/UX Portfolio Workshop", goal: "Build a standout design portfolio with weekly critiques, case-study writing, and Figma deep-dives.",
            icon: "UX", subject: "Design", tags: &["UX", "Figma", "Portfolio"], max: 6, schedule: "Every Friday 16:00–18:00", weeks: 8, current_week: 5, is_open: true,
            description: "Craft a portfolio that lands design interviews." },
        GroupSpec { name: "Web Security & CTF", goal: "Learn web exploitation, practice CTF challenges, and play weekend CTFs as a team.",
            icon: "WS", subject: "Security", tags: &["Security", "CTF", "Web"], max: 6, schedule: "Every Saturday 13:00–16:00", weeks: 12, current_week: 4, is_open: true,
            description: "Hands-on offensive security and CTF practice." },
        GroupSpec { name: "Data Structures from Scratch", goal: "Implement every core data structure in C++ from scratch and analyze their complexity together.",
            icon: "DS", subject: "Data Structures", tags: &["C++", "Data Structures", "CS Fundamentals"], max: 8, schedule: "Mon / Wed 20:00–21:30", weeks: 10, current_week: 1, is_open: true,
            description: "Solidify your CS fundamentals by building from the ground up." },
        GroupSpec { name: "Mobile Dev with Flutter", goal: "Build a complete cross-platform app over the semester, learning Flutter and Dart by doing.",
            icon: "FL", subject: "Mobile Development", tags: &["Flutter", "Dart", "Mobile"], max: 8, schedule: "Every Sunday 10:00–12:00", weeks: 12, current_week: 6, is_open: true,
            description: "Ship a real Flutter app, from idea to store." },
        GroupSpec { name: "English Tech Talk Club", goal: "Practice giving technical presentations and answering interview questions in English.",
            icon: "EN", subject: "Language", tags: &["English", "Presentation", "Career"], max: 10, schedule: "Every Tuesday 18:00–19:00", weeks: 14, current_week: 9, is_open: true,
            description: "Build confidence speaking tech in English." },
        GroupSpec { name: "Database Internals Study", goal: "Understand how databases work under the hood: storage engines, indexing, and query planning.",
            icon: "DB", subject: "Databases", tags: &["Databases", "SQL", "Internals"], max: 6, schedule: "Every Thursday 20:00–22:00", weeks: 10, current_week: 2, is_open: true,
            description: "Go beyond SQL and learn what makes databases tick." },
        GroupSpec { name: "Generative AI Builders", goal: "Build LLM-powered apps with LangChain and the OpenAI API, sharing prompts and patterns.",
            icon: "GA", subject: "AI / ML", tags: &["LLM", "LangChain", "AI"], max: 10, schedule: "Every Saturday 15:00–17:00", weeks: 8, current_week: 3, is_open: true,
            description: "Hands-on building with the latest generative AI tools." },
        GroupSpec { name: "Game Development Collective", goal: "Make a small game each month in Unity, learning gameplay programming and game design.",
            icon: "GC", subject: "Game Development", tags: &["Unity", "C#", "Game Design"], max: 6, schedule: "Every Friday 19:00–21:00", weeks: 12, current_week: 5, is_open: true,
            description: "Learn game dev by shipping playable games." },
        GroupSpec { name: "Linux & Shell Mastery", goal: "Become fluent in the Linux command line, shell scripting, and developer workflow tooling.",
            icon: "LX", subject: "Systems", tags: &["Linux", "Bash", "CLI"], max: 8, schedule: "Mon / Thu 21:00–22:00", weeks: 8, current_week: 4, is_open: true,
            description: "Master the terminal and supercharge your workflow." },
        GroupSpec { name: "Statistics for Data Science", goal: "Build statistical intuition with hands-on Python notebooks and real datasets.",
            icon: "ST", subject: "Statistics", tags: &["Statistics", "Python", "Data Science"], max: 8, schedule: "Every Wednesday 18:30–20:00", weeks: 10, current_week: 6, is_open: true,
            description: "The statistical foundation every data scientist needs." },
        GroupSpec { name: "Open Source Contributors", goal: "Find good first issues, make our first PRs, and support each other through the contribution process.",
            icon: "OS", subject: "Open Source", tags: &["Open Source", "Git", "Community"], max: 10, schedule: "Every Sunday 14:00–16:00", weeks: 8, current_week: 2, is_open: true,
            description: "Make your first open-source contributions together." },
        GroupSpec { name: "Functional Programming with Haskell", goal: "Explore pure functional programming concepts through Haskell and apply them to other languages.",
            icon: "HS", subject: "Programming Languages", tags: &["Haskell", "Functional", "Theory"], max: 6, schedule: "Every Tuesday 19:30–21:00", weeks: 12, current_week: 3, is_open: false,
            description: "Rewire your brain with functional programming." },
        GroupSpec { name: "Product Management Crash Course", goal: "Learn PM fundamentals: discovery, roadmapping, prioritization, and stakeholder communication.",
            icon: "PM", subject: "Product", tags: &["Product", "Strategy", "Career"], max: 8, schedule: "Every Thursday 18:00–19:30", weeks: 8, current_week: 5, is_open: true,
            description: "Break into product management with practical skills." },
    ];

    let mut groups: Vec<StudyGroup> = Vec::with_capacity(specs.len());
    for (i, s) in specs.into_iter().enumerate() {
        let creator = &users[i % n];
        let mut g = StudyGroup::new(s.name, s.goal, &creator.id);
        g.icon = s.icon.into();
        g.icon_bg = GRADIENTS[i % GRADIENTS.len()].into();
        g.subject = s.subject.into();
        g.tags = s.tags.iter().map(|x| x.to_string()).collect();
        g.max_members = s.max;
        g.schedule = s.schedule.into();
        g.duration_weeks = s.weeks;
        g.current_week = s.current_week;
        g.is_open = s.is_open;
        g.description = Some(s.description.into());

        // Creator + a few members with check-in streaks.
        let mut members = vec![GroupMember {
            user_id: creator.id.clone(),
            initials: creator.initials.clone(),
            color: COLORS[i % COLORS.len()].into(),
            joined_at: now - Duration::days(s.current_week as i64 * 7),
            last_checkin: Some(now - Duration::days(1)),
            streak: s.current_week as u32 * 3,
        }];
        for k in 1..=3 {
            let m = &users[(i + k) % n];
            if m.id == creator.id || members.iter().any(|x| x.user_id == m.id) {
                continue;
            }
            members.push(GroupMember {
                user_id: m.id.clone(),
                initials: m.initials.clone(),
                color: COLORS[(i + k) % COLORS.len()].into(),
                joined_at: now - Duration::days((s.current_week as i64 - 1).max(0) * 7),
                last_checkin: Some(now - Duration::days(k as i64)),
                streak: (s.current_week as u32).saturating_sub(k as u32) * 2,
            });
        }
        g.members = members;
        g.created_at = now;
        g.updated_at = now;
        groups.push(g);
    }

    let count = groups.len();
    let col: mongodb::Collection<StudyGroup> = db.db.collection("study_groups");
    col.insert_many(&groups).await?;
    tracing::info!("  ✓ Inserted {} seed study groups", count);
    Ok(())
}
