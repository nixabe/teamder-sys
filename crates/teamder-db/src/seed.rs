use chrono::Utc;
use teamder_core::models::skill_catalog::{StoredSkillCategory, StoredSkillTag};
use uuid::Uuid;

use crate::client::DbClient;

/// Skill catalog data: (category_key, label_en, label_zh, tags)
/// Each tag is (name_en, name_zh).
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

/// Seed the database with initial data if the relevant collections are empty.
///
/// This is called once at startup. In development it can pre-populate skill
/// catalogues, demo users, etc. In production it is a no-op when data already
/// exists.
pub async fn seed_if_empty(db: &DbClient) -> anyhow::Result<()> {
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
