use chrono::{Duration, Utc};
use teamder_core::models::competition::Competition;
use teamder_core::models::project::{Project, ProjectRole};
use teamder_core::models::skill_catalog::{StoredSkillCategory, StoredSkillTag};
use teamder_core::models::study_group::StudyGroup;
use teamder_core::models::user::{Skill, User};
use uuid::Uuid;

use crate::client::DbClient;

#[allow(clippy::type_complexity)]
const SKILL_CATALOG: &[(&str, &str, &str, &[(&str, &str)])] = &[
    (
        "frontend",
        "Frontend",
        "前端開發",
        &[
            ("React", "React"),
            ("Vue.js", "Vue.js"),
            ("Angular", "Angular"),
            ("Next.js", "Next.js"),
            ("TypeScript", "TypeScript"),
            ("HTML/CSS", "HTML/CSS"),
            ("Tailwind CSS", "Tailwind CSS"),
            ("Svelte", "Svelte"),
        ],
    ),
    (
        "backend",
        "Backend",
        "後端開發",
        &[
            ("Node.js", "Node.js"),
            ("Python", "Python"),
            ("Rust", "Rust"),
            ("Go", "Go"),
            ("Java", "Java"),
            ("C#/.NET", "C#/.NET"),
            ("PHP", "PHP"),
            ("Ruby", "Ruby"),
        ],
    ),
    (
        "mobile",
        "Mobile",
        "行動開發",
        &[
            ("React Native", "React Native"),
            ("Flutter", "Flutter"),
            ("Swift/iOS", "Swift/iOS"),
            ("Kotlin/Android", "Kotlin/Android"),
        ],
    ),
    (
        "database",
        "Database",
        "資料庫",
        &[
            ("MongoDB", "MongoDB"),
            ("PostgreSQL", "PostgreSQL"),
            ("MySQL", "MySQL"),
            ("Redis", "Redis"),
            ("Firebase", "Firebase"),
        ],
    ),
    (
        "devops",
        "DevOps",
        "DevOps",
        &[
            ("Docker", "Docker"),
            ("Kubernetes", "Kubernetes"),
            ("AWS", "AWS"),
            ("GCP", "GCP"),
            ("CI/CD", "CI/CD"),
            ("Linux", "Linux"),
        ],
    ),
    (
        "design",
        "Design",
        "設計",
        &[
            ("UI/UX Design", "UI/UX 設計"),
            ("Figma", "Figma"),
            ("Adobe XD", "Adobe XD"),
            ("Graphic Design", "平面設計"),
            ("Illustration", "插畫"),
        ],
    ),
    (
        "data",
        "Data Science",
        "資料科學",
        &[
            ("Machine Learning", "機器學習"),
            ("Data Analysis", "資料分析"),
            ("Deep Learning", "深度學習"),
            ("NLP", "自然語言處理"),
            ("Computer Vision", "電腦視覺"),
            ("Statistics", "統計學"),
        ],
    ),
    (
        "blockchain",
        "Blockchain",
        "區塊鏈",
        &[
            ("Solidity", "Solidity"),
            ("Web3.js", "Web3.js"),
            ("Smart Contracts", "智能合約"),
        ],
    ),
    (
        "pm",
        "Project Management",
        "專案管理",
        &[
            ("Agile/Scrum", "敏捷/Scrum"),
            ("Product Management", "產品管理"),
            ("JIRA", "JIRA"),
        ],
    ),
    (
        "marketing",
        "Marketing",
        "行銷",
        &[
            ("Digital Marketing", "數位行銷"),
            ("SEO", "SEO"),
            ("Content Marketing", "內容行銷"),
            ("Social Media", "社群媒體"),
        ],
    ),
    (
        "business",
        "Business",
        "商業",
        &[
            ("Business Analysis", "商業分析"),
            ("Financial Analysis", "財務分析"),
            ("Entrepreneurship", "創業"),
        ],
    ),
    (
        "other",
        "Other",
        "其他",
        &[
            ("Technical Writing", "技術寫作"),
            ("Video Editing", "影片剪輯"),
            ("3D Modeling", "3D 建模"),
            ("Game Development", "遊戲開發"),
        ],
    ),
];

// ── Dummy data arrays ───────────────────────────────────────────────────────

const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "Diana", "Edward",
    "Fiona", "George", "Hannah", "Ivan", "Julia",
    "Kevin", "Lily", "Michael", "Nancy", "Oscar",
    "Penny", "Quincy", "Rachel", "Steven", "Tina",
];

const DEPARTMENTS: &[&str] = &[
    "Computer Science", "Information Management", "Electrical Engineering",
    "Applied Mathematics", "Business Administration", "Digital Media",
    "Software Engineering", "Data Science", "Finance", "Design",
];

const GRADIENTS: &[&str] = &[
    "from-rose-400 to-orange-300",
    "from-blue-400 to-cyan-300",
    "from-violet-400 to-purple-300",
    "from-emerald-400 to-teal-300",
    "from-amber-400 to-yellow-300",
    "from-pink-400 to-fuchsia-300",
    "from-indigo-400 to-blue-300",
    "from-green-400 to-lime-300",
    "from-red-400 to-rose-300",
    "from-sky-400 to-cyan-300",
];

const WORK_MODES: &[&str] = &["remote", "hybrid", "in-person"];
const AVAILABILITIES: &[&str] = &["available", "busy", "away"];
const YEARS: &[&str] = &["Freshman", "Sophomore", "Junior", "Senior", "Graduate"];

const PROJECT_NAMES: &[(&str, &str)] = &[
    ("Campus Navigator", "An AR-based campus wayfinding app"),
    ("StudyBuddy AI", "AI-powered study assistant with flashcards"),
    ("EcoTracker", "Carbon footprint tracking for students"),
    ("FoodShare", "Share surplus meal plans with classmates"),
    ("CodeReview Hub", "Peer code review platform for CS courses"),
    ("EventPulse", "University event discovery and RSVP system"),
    ("SkillSwap", "Barter skills with other students"),
    ("ResearchConnect", "Match undergrads with research labs"),
    ("BudgetBuddy", "Student expense tracking and splitting"),
    ("ClassNotes Live", "Real-time collaborative note-taking"),
    ("DormFinder", "Roommate matching based on lifestyle"),
    ("MentorLink", "Alumni-student mentorship platform"),
    ("HackTrack", "Hackathon project management tool"),
    ("LibraryQ", "Library seat reservation system"),
    ("FitnessPal Campus", "Campus gym scheduling and partners"),
    ("InternHub", "Internship sharing and review board"),
    ("DebateArena", "Online debate practice platform"),
    ("VolunteerMatch", "Connect students with NGO opportunities"),
    ("PodcastStudio", "Student podcast recording and hosting"),
    ("PortfolioForge", "Generate portfolios from GitHub activity"),
];

const PROJECT_CATEGORIES: &[&str] = &[
    "Web App", "Mobile App", "AI/ML", "IoT", "Social Impact",
    "Education", "Health", "Finance", "Entertainment", "Productivity",
];

const COMPETITION_DATA: &[(&str, &str, &str, &str)] = &[
    ("SITCON Hackathon 2025", "SITCON", "NT$100,000", "48 hours"),
    ("Google DSC Solution Challenge", "Google DSC", "US$3,000", "3 months"),
    ("MOPCON App Contest", "MOPCON", "NT$50,000", "2 weeks"),
    ("AWS GameDay", "Amazon Web Services", "US$5,000", "8 hours"),
    ("LINE Bot Challenge", "LINE Taiwan", "NT$80,000", "1 month"),
    ("Microsoft Imagine Cup", "Microsoft", "US$100,000", "6 months"),
    ("HackNTU", "NTU CSIE", "NT$60,000", "36 hours"),
    ("AI CUP 2025", "Ministry of Education", "NT$200,000", "3 months"),
    ("TSMC Coding Contest", "TSMC", "NT$150,000", "5 hours"),
    ("Appworks Demo Day", "Appworks", "Mentorship + Funding", "3 months"),
    ("IEEE Xtreme 19.0", "IEEE", "US$2,500", "24 hours"),
    ("ICPC Asia Regional", "ACM", "Medals + Scholarships", "5 hours"),
    ("Meta Hacker Cup", "Meta", "US$10,000", "3 rounds"),
    ("Cathay Hackathon", "Cathay Financial", "NT$120,000", "48 hours"),
    ("Open Data Hackathon", "g0v", "NT$30,000", "2 days"),
    ("COSCUP Workshop Challenge", "COSCUP", "Community Recognition", "1 week"),
    ("Blockchain Innovation Cup", "BSOS", "NT$200,000", "2 months"),
    ("Design Thinking Marathon", "IDEO x FJU", "NT$40,000", "24 hours"),
    ("Startup Weekend Taipei", "Techstars", "Incubation Spot", "54 hours"),
    ("GDSC WOW Hackathon", "Google DSC Taiwan", "NT$80,000", "36 hours"),
];

const COMPETITION_TAGS: &[&[&str]] = &[
    &["hackathon", "open-source"],
    &["mobile", "social-impact"],
    &["mobile", "app"],
    &["cloud", "aws", "devops"],
    &["chatbot", "api"],
    &["innovation", "global"],
    &["hackathon", "ai"],
    &["ai", "machine-learning", "data"],
    &["algorithms", "competitive-programming"],
    &["startup", "entrepreneurship"],
    &["algorithms", "competitive-programming"],
    &["algorithms", "competitive-programming"],
    &["algorithms", "competitive-programming"],
    &["fintech", "hackathon"],
    &["open-data", "civic-tech"],
    &["open-source", "community"],
    &["blockchain", "web3"],
    &["design", "ux"],
    &["startup", "entrepreneurship"],
    &["hackathon", "community"],
];

const STUDY_GROUP_DATA: &[(&str, &str, &str)] = &[
    ("React Mastery", "Master React 19 and Next.js patterns", "frontend"),
    ("Rust Systems Programming", "Learn Rust from zero to production", "backend"),
    ("ML Paper Reading Club", "Read and discuss latest ML papers weekly", "ai-ml"),
    ("iOS Development", "Build iOS apps with SwiftUI", "mobile"),
    ("System Design Interview", "Practice system design questions", "cs-fundamentals"),
    ("UI/UX Principles", "Learn design thinking and Figma", "design"),
    ("Kubernetes Deep Dive", "Container orchestration mastery", "backend"),
    ("Data Structures & Algorithms", "Leetcode grinding group", "cs-fundamentals"),
    ("Flutter Cross-Platform", "Build one codebase, deploy everywhere", "mobile"),
    ("Python for Data Science", "Pandas, NumPy, and Matplotlib", "ai-ml"),
    ("TypeScript Advanced Types", "Generic wizardry and type gymnastics", "frontend"),
    ("PostgreSQL Optimization", "Query tuning and indexing strategies", "backend"),
    ("Computer Vision Projects", "Hands-on CV with PyTorch", "ai-ml"),
    ("Vue.js 3 Composition API", "Modern Vue patterns and Pinia", "frontend"),
    ("Cloud Architecture", "AWS Solutions Architect prep", "backend"),
    ("Game Dev with Unity", "Build 2D and 3D games together", "design"),
    ("NLP Transformers", "Understanding attention and BERT family", "ai-ml"),
    ("Web Security Basics", "OWASP Top 10 and CTF practice", "cs-fundamentals"),
    ("Android Jetpack Compose", "Modern Android UI development", "mobile"),
    ("GraphQL API Design", "Schema design and performance patterns", "backend"),
];

// ── Seed logic ──────────────────────────────────────────────────────────────

/// Seed the database with initial data if the relevant collections are empty.
///
/// This is called once at startup. In development it can pre-populate skill
/// catalogues, demo users, etc. In production it is a no-op when data already
/// exists.
pub async fn seed_if_empty(db: &DbClient) -> anyhow::Result<()> {
    seed_skill_catalog(db).await?;
    seed_users(db).await?;
    seed_projects(db).await?;
    seed_competitions(db).await?;
    seed_study_groups(db).await?;
    Ok(())
}

async fn seed_skill_catalog(db: &DbClient) -> anyhow::Result<()> {
    let repo = db.skill_catalog_repo();

    let cat_count = repo.count_categories().await.unwrap_or(0);
    if cat_count > 0 {
        tracing::debug!("Skill catalog already seeded ({cat_count} categories), skipping");
        return Ok(());
    }

    tracing::info!("Seeding skill catalog...");

    let now = Utc::now();

    for (order, &(key, label, label_zh, tags)) in SKILL_CATALOG.iter().enumerate() {
        let category = StoredSkillCategory {
            id: key.to_string(),
            label: label.to_string(),
            label_zh: label_zh.to_string(),
            order: order as i32,
            created_at: now,
            updated_at: now,
        };
        repo.create_category(&category).await?;

        for (tag_order, &(name, name_zh)) in tags.iter().enumerate() {
            let tag = StoredSkillTag {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                name_zh: name_zh.to_string(),
                category_key: key.to_string(),
                order: tag_order as i32,
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            repo.create_tag(&tag).await?;
        }
    }

    tracing::info!("Skill catalog seeded ({} categories)", SKILL_CATALOG.len());
    Ok(())
}

async fn seed_users(db: &DbClient) -> anyhow::Result<()> {
    let repo = db.user_repo();
    let count = repo.count().await.unwrap_or(0);
    if count > 0 {
        tracing::debug!("Users already seeded ({count}), skipping");
        return Ok(());
    }

    tracing::info!("Seeding 20 dummy users...");
    let now = Utc::now();

    let skill_pool: &[(&str, u8)] = &[
        ("React", 85), ("TypeScript", 90), ("Python", 75), ("Rust", 70),
        ("Node.js", 80), ("Figma", 65), ("Docker", 60), ("MongoDB", 72),
        ("Flutter", 68), ("Machine Learning", 55), ("PostgreSQL", 78),
        ("Vue.js", 82), ("Swift/iOS", 60), ("AWS", 65), ("Go", 70),
    ];

    // bcrypt hash for "password123"
    let password_hash = "$2b$12$LJ3a5sK5e6wG5V5K5e6wGOXHQZ9Y8e6wG5V5K5e6wG5V5K5e6wG".to_string();

    for i in 0..20 {
        let name = FIRST_NAMES[i];
        let initials = format!("{}{}", &name[..1], &name[1..2]).to_uppercase();
        let dept = DEPARTMENTS[i % DEPARTMENTS.len()];
        let gradient = GRADIENTS[i % GRADIENTS.len()].to_string();
        let year = YEARS[i % YEARS.len()].to_string();
        let work_mode = WORK_MODES[i % WORK_MODES.len()].to_string();
        let availability = AVAILABILITIES[i % AVAILABILITIES.len()].to_string();

        let skills: Vec<Skill> = (0..3)
            .map(|j| {
                let (sname, level) = skill_pool[(i * 3 + j) % skill_pool.len()];
                Skill {
                    name: sname.to_string(),
                    level: level.min(100),
                }
            })
            .collect();

        let skill_tags: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

        let user = User {
            id: Uuid::new_v4().to_string(),
            email: format!("{}@fju.edu.tw", name.to_lowercase()),
            password_hash: password_hash.clone(),
            name: format!("{} Chen", name),
            initials,
            role: "student".to_string(),
            department: dept.to_string(),
            university: "Fu Jen Catholic University".to_string(),
            year,
            location: Some("New Taipei City, Taiwan".to_string()),
            bio: vec![
                format!("Passionate about {} and building cool projects.", skills[0].name),
                format!("Currently exploring {} in my studies.", dept.to_lowercase()),
            ],
            skills,
            skill_tags,
            gradient,
            work_mode: Some(work_mode),
            availability: Some(availability),
            hours_per_week: Some(format!("{}", 10 + (i % 4) * 5)),
            languages: vec!["Chinese".to_string(), "English".to_string()],
            portfolio: vec![],
            reviews: vec![],
            match_score: None,
            rating: 3.5 + (i as f32 % 3.0) * 0.5,
            projects_done: (i as u32 % 5) + 1,
            collaborations: (i as u32 % 8) + 1,
            avatar_url: None,
            resume_url: None,
            reset_token: None,
            reset_token_expires_at: None,
            is_admin: i == 0,
            is_publisher: i < 3,
            is_public: true,
            onboarded: true,
            headline: Some(format!("{} student passionate about technology", dept)),
            notify_email: true,
            notify_in_app: true,
            social_links: vec![],
            interests: vec!["coding".to_string(), "open-source".to_string(), "hackathons".to_string()],
            timezone: Some("Asia/Taipei".to_string()),
            goals: Some("Build impactful projects and grow my skills".to_string()),
            free_days: vec!["Saturday".to_string(), "Sunday".to_string()],
            created_at: now - Duration::days((20 - i as i64) * 3),
            updated_at: now - Duration::days(i as i64 % 7),
        };

        repo.create(&user).await?;
    }

    tracing::info!("Seeded 20 users");
    Ok(())
}

async fn seed_projects(db: &DbClient) -> anyhow::Result<()> {
    let repo = db.project_repo();
    let count = repo.count().await.unwrap_or(0);
    if count > 0 {
        tracing::debug!("Projects already seeded ({count}), skipping");
        return Ok(());
    }

    tracing::info!("Seeding 20 dummy projects...");
    let now = Utc::now();

    let icons = ["Pr", "Cd", "Ai", "Db", "Ux", "Mb", "Sv", "Hw", "Gm", "Sc"];
    let icon_bgs = [
        "#FFE4D6", "#D6F5FF", "#E8D6FF", "#D6FFE4", "#FFF5D6",
        "#FFD6E8", "#D6E8FF", "#E4FFD6", "#FFD6D6", "#D6FFFF",
    ];
    let collabs = ["remote", "hybrid", "in-person"];
    let durations = ["2 weeks", "1 month", "2 months", "3 months", "6 months"];
    let skill_sets: &[&[&str]] = &[
        &["React", "TypeScript", "Node.js"],
        &["Python", "TensorFlow", "FastAPI"],
        &["Flutter", "Firebase", "Dart"],
        &["Rust", "PostgreSQL", "Docker"],
        &["Vue.js", "MongoDB", "Express"],
        &["Swift", "SwiftUI", "CoreML"],
        &["Next.js", "Tailwind CSS", "Prisma"],
        &["Go", "gRPC", "Kubernetes"],
        &["React Native", "Redux", "GraphQL"],
        &["Python", "OpenCV", "PyTorch"],
    ];

    for i in 0..20 {
        let (name, desc) = PROJECT_NAMES[i];
        let category = PROJECT_CATEGORIES[i % PROJECT_CATEGORIES.len()];
        let skills_for_project: Vec<String> = skill_sets[i % skill_sets.len()]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let roles = vec![
            ProjectRole {
                name: "Frontend Developer".to_string(),
                count_needed: 2,
                filled: (i as u8 % 2),
            },
            ProjectRole {
                name: "Backend Developer".to_string(),
                count_needed: 2,
                filled: (i as u8 % 3).min(2),
            },
            ProjectRole {
                name: "Designer".to_string(),
                count_needed: 1,
                filled: i as u8 % 2,
            },
        ];

        let statuses = ["recruiting", "recruiting", "recruiting", "active", "completed"];

        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            lead_user_id: Uuid::new_v4().to_string(),
            icon: icons[i % icons.len()].to_string(),
            icon_bg: icon_bgs[i % icon_bgs.len()].to_string(),
            status: statuses[i % statuses.len()].to_string(),
            description: desc.to_string(),
            goals: Some("Build a working prototype and launch by end of semester".to_string()),
            roles,
            skills: skills_for_project,
            team: vec![],
            deadline: Some(format!("2025-{:02}-{:02}", (i % 6) + 7, (i % 28) + 1)),
            collab: Some(collabs[i % collabs.len()].to_string()),
            duration: Some(durations[i % durations.len()].to_string()),
            category: Some(category.to_string()),
            is_public: true,
            join_mode: if i % 3 == 0 { "approval".to_string() } else { "direct".to_string() },
            is_promoted: i < 5,
            banner_image: None,
            created_at: now - Duration::days((20 - i as i64) * 2),
            updated_at: now - Duration::days(i as i64 % 5),
        };

        repo.create(&project).await?;
    }

    tracing::info!("Seeded 20 projects");
    Ok(())
}

async fn seed_competitions(db: &DbClient) -> anyhow::Result<()> {
    let repo = db.competition_repo();
    let count = repo.count().await.unwrap_or(0);
    if count > 0 {
        tracing::debug!("Competitions already seeded ({count}), skipping");
        return Ok(());
    }

    tracing::info!("Seeding 20 dummy competitions...");
    let now = Utc::now();

    let statuses = ["open", "open", "open", "upcoming", "closing-soon"];

    for i in 0..20 {
        let (name, organizer, prize, duration) = COMPETITION_DATA[i];
        let tags: Vec<String> = COMPETITION_TAGS[i].iter().map(|t| t.to_string()).collect();

        let comp = Competition {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            organizer: organizer.to_string(),
            icon: "Cp".to_string(),
            icon_bg: "#FFE4D6".to_string(),
            status: statuses[i % statuses.len()].to_string(),
            prize: prize.to_string(),
            team_size_min: 2,
            team_size_max: if i % 3 == 0 { 6 } else { 4 },
            deadline: Some(format!("2025-{:02}-{:02}", (i % 5) + 8, (i % 28) + 1)),
            duration: duration.to_string(),
            tags,
            description: format!(
                "Join {} organized by {} and compete for {}! Duration: {}.",
                name, organizer, prize, duration
            ),
            is_featured: i < 6,
            banner_image: None,
            publish_status: "published".to_string(),
            publisher_id: None,
            rejected_note: None,
            registrations: vec![],
            interested_user_ids: vec![],
            winners: vec![],
            created_at: now - Duration::days((20 - i as i64) * 4),
            updated_at: now - Duration::days(i as i64 % 10),
        };

        repo.create(&comp).await?;
    }

    tracing::info!("Seeded 20 competitions");
    Ok(())
}

async fn seed_study_groups(db: &DbClient) -> anyhow::Result<()> {
    let repo = db.study_group_repo();
    let count = repo.count().await.unwrap_or(0);
    if count > 0 {
        tracing::debug!("Study groups already seeded ({count}), skipping");
        return Ok(());
    }

    tracing::info!("Seeding 20 dummy study groups...");
    let now = Utc::now();

    let schedules = [
        "Every Monday 7-9 PM",
        "Every Wednesday 6-8 PM",
        "Every Friday 3-5 PM",
        "Every Saturday 10 AM-12 PM",
        "Every Sunday 2-4 PM",
    ];

    let icons = ["Sg", "Rc", "Ml", "Sw", "Ds", "Fg", "K8", "Lc", "Fl", "Py"];
    let icon_bgs = [
        "#D6F5FF", "#FFE4D6", "#E8D6FF", "#D6FFE4", "#FFF5D6",
        "#FFD6E8", "#D6E8FF", "#E4FFD6", "#FFD6D6", "#D6FFFF",
    ];

    for i in 0..20 {
        let (name, goal, subject) = STUDY_GROUP_DATA[i];

        let group = StudyGroup {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            goal: goal.to_string(),
            icon: icons[i % icons.len()].to_string(),
            icon_bg: icon_bgs[i % icon_bgs.len()].to_string(),
            subject: subject.to_string(),
            tags: vec![subject.to_string()],
            members: vec![],
            max_members: (4 + (i % 4) * 2) as u8,
            schedule: schedules[i % schedules.len()].to_string(),
            duration_weeks: (4 + (i % 5) * 2) as u8,
            current_week: ((i % 4) + 1) as u8,
            is_open: i % 5 != 4,
            status: "active".to_string(),
            join_mode: if i % 4 == 0 { "approval".to_string() } else { "direct".to_string() },
            banner_image: None,
            notes: vec![],
            description: Some(goal.to_string()),
            created_by: Uuid::new_v4().to_string(),
            created_at: now - Duration::days((20 - i as i64) * 3),
            updated_at: now - Duration::days(i as i64 % 6),
        };

        repo.create(&group).await?;
    }

    tracing::info!("Seeded 20 study groups");
    Ok(())
}
