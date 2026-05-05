//! Hardcoded skill taxonomy and matching algorithm.
//!
//! Skills are organized into bilingual (English / 繁體中文) categories and
//! cannot be modified by users — they pick from this catalog when editing
//! their profile, ensuring consistency for search, filtering, and matching.

use serde::Serialize;

use crate::models::{project::Project, user::User};

/// A single skill tag with its bilingual labels.
#[derive(Debug, Clone, Serialize)]
pub struct SkillEntry {
    /// Canonical English name. This is what is stored on user documents.
    pub name: &'static str,
    /// Traditional Chinese display name (繁體中文).
    pub name_zh: &'static str,
}

/// A category of related skills.
#[derive(Debug, Clone, Serialize)]
pub struct SkillCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub label_zh: &'static str,
    pub skills: Vec<SkillEntry>,
}

fn s(name: &'static str, name_zh: &'static str) -> SkillEntry {
    SkillEntry { name, name_zh }
}

/// Returns the full skill catalog grouped by category.
pub fn catalog() -> Vec<SkillCategory> {
    vec![
        SkillCategory {
            key: "frontend",
            label: "Frontend",
            label_zh: "前端開發",
            skills: vec![
                s("React", "React"),
                s("Next.js", "Next.js"),
                s("Vue.js", "Vue.js"),
                s("Nuxt", "Nuxt"),
                s("Svelte", "Svelte"),
                s("SvelteKit", "SvelteKit"),
                s("Angular", "Angular"),
                s("Solid.js", "Solid.js"),
                s("Astro", "Astro"),
                s("TypeScript", "TypeScript"),
                s("JavaScript", "JavaScript"),
                s("HTML5", "HTML5"),
                s("CSS3", "CSS3"),
                s("Tailwind CSS", "Tailwind CSS"),
                s("Sass", "Sass"),
                s("Styled Components", "Styled Components"),
                s("Redux", "Redux"),
                s("Zustand", "Zustand"),
                s("TanStack Query", "TanStack Query"),
                s("Webpack", "Webpack"),
                s("Vite", "Vite"),
            ],
        },
        SkillCategory {
            key: "backend",
            label: "Backend",
            label_zh: "後端開發",
            skills: vec![
                s("Node.js", "Node.js"),
                s("Express", "Express"),
                s("NestJS", "NestJS"),
                s("Fastify", "Fastify"),
                s("Deno", "Deno"),
                s("Bun", "Bun"),
                s("Python", "Python"),
                s("Django", "Django"),
                s("Flask", "Flask"),
                s("FastAPI", "FastAPI"),
                s("Ruby on Rails", "Ruby on Rails"),
                s("Go", "Go"),
                s("Gin", "Gin"),
                s("Rust", "Rust"),
                s("Rocket", "Rocket"),
                s("Actix Web", "Actix Web"),
                s("Axum", "Axum"),
                s("Java", "Java"),
                s("Spring Boot", "Spring Boot"),
                s("Kotlin", "Kotlin"),
                s("Ktor", "Ktor"),
                s("C#", "C#"),
                s(".NET", ".NET"),
                s("PHP", "PHP"),
                s("Laravel", "Laravel"),
                s("GraphQL", "GraphQL"),
                s("REST API", "REST API"),
                s("gRPC", "gRPC"),
                s("WebSockets", "WebSockets"),
            ],
        },
        SkillCategory {
            key: "mobile",
            label: "Mobile",
            label_zh: "行動開發",
            skills: vec![
                s("iOS", "iOS"),
                s("Swift", "Swift"),
                s("SwiftUI", "SwiftUI"),
                s("Android", "Android"),
                s("Kotlin (Android)", "Kotlin (Android)"),
                s("Jetpack Compose", "Jetpack Compose"),
                s("React Native", "React Native"),
                s("Flutter", "Flutter"),
                s("Dart", "Dart"),
                s("Ionic", "Ionic"),
                s("Expo", "Expo"),
                s("Capacitor", "Capacitor"),
            ],
        },
        SkillCategory {
            key: "ai_ml",
            label: "AI / ML",
            label_zh: "AI / 機器學習",
            skills: vec![
                s("Machine Learning", "機器學習"),
                s("Deep Learning", "深度學習"),
                s("PyTorch", "PyTorch"),
                s("TensorFlow", "TensorFlow"),
                s("Keras", "Keras"),
                s("scikit-learn", "scikit-learn"),
                s("Hugging Face", "Hugging Face"),
                s("LangChain", "LangChain"),
                s("OpenAI API", "OpenAI API"),
                s("Computer Vision", "電腦視覺"),
                s("NLP", "自然語言處理"),
                s("LLM Fine-tuning", "LLM 微調"),
                s("Prompt Engineering", "提示工程"),
                s("RAG", "檢索增強生成 (RAG)"),
                s("Reinforcement Learning", "強化學習"),
            ],
        },
        SkillCategory {
            key: "data",
            label: "Data",
            label_zh: "資料 / 數據",
            skills: vec![
                s("Data Analysis", "資料分析"),
                s("Data Engineering", "資料工程"),
                s("Data Visualization", "資料視覺化"),
                s("SQL", "SQL"),
                s("PostgreSQL", "PostgreSQL"),
                s("MySQL", "MySQL"),
                s("MongoDB", "MongoDB"),
                s("Redis", "Redis"),
                s("Snowflake", "Snowflake"),
                s("BigQuery", "BigQuery"),
                s("Apache Spark", "Apache Spark"),
                s("Apache Kafka", "Apache Kafka"),
                s("Airflow", "Airflow"),
                s("dbt", "dbt"),
                s("Tableau", "Tableau"),
                s("Power BI", "Power BI"),
                s("Pandas", "Pandas"),
                s("NumPy", "NumPy"),
                s("R", "R"),
                s("Statistics", "統計學"),
            ],
        },
        SkillCategory {
            key: "design",
            label: "Design",
            label_zh: "設計",
            skills: vec![
                s("UI Design", "UI 設計"),
                s("UX Design", "UX 設計"),
                s("Figma", "Figma"),
                s("Sketch", "Sketch"),
                s("Adobe XD", "Adobe XD"),
                s("Photoshop", "Photoshop"),
                s("Illustrator", "Illustrator"),
                s("After Effects", "After Effects"),
                s("Premiere Pro", "Premiere Pro"),
                s("Branding", "品牌設計"),
                s("Typography", "字體排印"),
                s("Design Systems", "設計系統"),
                s("Wireframing", "線框稿"),
                s("Prototyping", "原型設計"),
                s("Motion Design", "動態設計"),
                s("3D Modeling", "3D 建模"),
                s("Blender", "Blender"),
                s("User Research", "使用者研究"),
                s("Accessibility (a11y)", "無障礙設計"),
            ],
        },
        SkillCategory {
            key: "product",
            label: "Product",
            label_zh: "產品",
            skills: vec![
                s("Product Management", "產品管理"),
                s("Product Strategy", "產品策略"),
                s("Roadmapping", "路線規劃"),
                s("User Stories", "使用者故事"),
                s("Agile", "敏捷開發"),
                s("Scrum", "Scrum"),
                s("Kanban", "看板"),
                s("OKRs", "OKR 目標管理"),
                s("A/B Testing", "A/B 測試"),
                s("Analytics", "數據分析"),
                s("Growth", "成長駭客"),
                s("Customer Research", "用戶研究"),
                s("Stakeholder Management", "利害關係人管理"),
            ],
        },
        SkillCategory {
            key: "devops",
            label: "DevOps & Cloud",
            label_zh: "DevOps 與雲端",
            skills: vec![
                s("Docker", "Docker"),
                s("Kubernetes", "Kubernetes"),
                s("AWS", "AWS"),
                s("Google Cloud", "Google Cloud"),
                s("Azure", "Azure"),
                s("Vercel", "Vercel"),
                s("Netlify", "Netlify"),
                s("Cloudflare", "Cloudflare"),
                s("Terraform", "Terraform"),
                s("Ansible", "Ansible"),
                s("GitHub Actions", "GitHub Actions"),
                s("GitLab CI", "GitLab CI"),
                s("Jenkins", "Jenkins"),
                s("Linux", "Linux"),
                s("Nginx", "Nginx"),
                s("CI/CD", "CI/CD"),
                s("Monitoring", "系統監控"),
                s("Site Reliability", "SRE 維運"),
            ],
        },
        SkillCategory {
            key: "security",
            label: "Security",
            label_zh: "資訊安全",
            skills: vec![
                s("Web Security", "網頁安全"),
                s("OAuth", "OAuth"),
                s("JWT", "JWT"),
                s("Penetration Testing", "滲透測試"),
                s("OWASP", "OWASP"),
                s("Cryptography", "密碼學"),
                s("Network Security", "網路安全"),
                s("DevSecOps", "DevSecOps"),
            ],
        },
        SkillCategory {
            key: "gamedev",
            label: "Game Dev",
            label_zh: "遊戲開發",
            skills: vec![
                s("Unity", "Unity"),
                s("Unreal Engine", "Unreal Engine"),
                s("Godot", "Godot"),
                s("C++", "C++"),
                s("Game Design", "遊戲設計"),
                s("Shader Programming", "Shader 程式設計"),
                s("Pixel Art", "像素藝術"),
                s("Level Design", "關卡設計"),
            ],
        },
        SkillCategory {
            key: "hardware",
            label: "Hardware & Embedded",
            label_zh: "硬體與嵌入式",
            skills: vec![
                s("Arduino", "Arduino"),
                s("Raspberry Pi", "Raspberry Pi"),
                s("Embedded C", "嵌入式 C"),
                s("FPGA", "FPGA"),
                s("Circuit Design", "電路設計"),
                s("PCB Design", "PCB 設計"),
                s("IoT", "物聯網"),
                s("Robotics", "機器人"),
                s("ROS", "ROS 機器人作業系統"),
            ],
        },
        SkillCategory {
            key: "business",
            label: "Business & Soft Skills",
            label_zh: "商務與軟實力",
            skills: vec![
                s("Leadership", "領導力"),
                s("Public Speaking", "公開演講"),
                s("Pitching", "提案簡報"),
                s("Marketing", "行銷"),
                s("Content Writing", "內容寫作"),
                s("Copywriting", "文案"),
                s("SEO", "SEO"),
                s("Social Media", "社群經營"),
                s("Sales", "業務銷售"),
                s("Finance", "財務"),
                s("Project Management", "專案管理"),
                s("Translation", "翻譯"),
            ],
        },
    ]
}

/// Flat list of all valid skill names (English canonical form, case-sensitive).
pub fn all_skills_flat() -> Vec<&'static str> {
    catalog()
        .into_iter()
        .flat_map(|c| c.skills.into_iter().map(|s| s.name))
        .collect()
}

/// Returns true if a skill name is valid (exists in the catalog).
pub fn is_valid_skill(name: &str) -> bool {
    let lower = name.to_lowercase();
    all_skills_flat()
        .iter()
        .any(|s| s.to_lowercase() == lower)
}

/// Filter a list of submitted skill names to only those in the catalog.
/// Used to sanitize user input — anything not in the catalog is dropped.
pub fn filter_valid_skills<S: AsRef<str>>(input: &[S]) -> Vec<String> {
    input
        .iter()
        .map(|s| s.as_ref().trim().to_string())
        .filter(|s| !s.is_empty() && is_valid_skill(s))
        .collect()
}

/// Look up the canonical English skill name for a given Chinese name.
/// Returns `None` if no Chinese name in the catalog matches.
pub fn zh_to_en(zh_query: &str) -> Option<&'static str> {
    let q = zh_query.trim();
    if q.is_empty() { return None; }
    for cat in catalog() {
        for skill in cat.skills {
            if skill.name_zh == q {
                return Some(skill.name);
            }
        }
    }
    None
}

/// Returns every English skill name whose **Chinese** label contains `needle`
/// as a substring. Empty needle returns an empty vector.
pub fn search_en_by_zh(needle: &str) -> Vec<&'static str> {
    let n = needle.trim();
    if n.is_empty() { return vec![]; }
    let mut out = Vec::new();
    for cat in catalog() {
        // Allow matching the category label too — useful for "前端" → all
        // frontend skills.
        let cat_match = cat.label_zh.contains(n);
        for skill in cat.skills {
            if cat_match || skill.name_zh.contains(n) {
                out.push(skill.name);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

// ─── Match score algorithm ────────────────────────────────────────────────────

/// Compute a 0–100 match score between a viewer and a target user.
///
/// Inputs:
/// - `viewer`: the user requesting the score (their skills weight the match)
/// - `target`: the user being scored
/// - `viewer_projects`: projects the viewer has been on
/// - `target_projects`: projects the target has been on
///
/// Components (weighted):
/// - **Skill overlap (40%)** — Jaccard similarity over skill tags + level proximity bonus
/// - **Skill complementarity (15%)** — points for filling categories the viewer lacks
/// - **Project domain overlap (20%)** — Jaccard over skills used in past projects
/// - **Track record (15%)** — log-scaled projects_done * rating/5
/// - **Availability (10%)** — open_for_collab adds full credit, else degraded
pub fn compute_match_score(
    viewer: &User,
    target: &User,
    viewer_projects: &[Project],
    target_projects: &[Project],
) -> u8 {
    let skill_overlap = skill_overlap_score(viewer, target);
    let complementarity = skill_complementarity_score(viewer, target);
    let project_overlap = project_overlap_score(viewer_projects, target_projects);
    let track_record = track_record_score(target);
    let availability = availability_score(target);

    let total = skill_overlap * 0.40
        + complementarity * 0.15
        + project_overlap * 0.20
        + track_record * 0.15
        + availability * 0.10;

    total.round().clamp(0.0, 100.0) as u8
}

/// 0–100 — Jaccard over skill_tags + bonus when matched skills have similar levels.
fn skill_overlap_score(viewer: &User, target: &User) -> f64 {
    let v_tags: std::collections::HashSet<String> =
        viewer.skill_tags.iter().map(|s| s.to_lowercase()).collect();
    let t_tags: std::collections::HashSet<String> =
        target.skill_tags.iter().map(|s| s.to_lowercase()).collect();

    if v_tags.is_empty() || t_tags.is_empty() {
        return 30.0; // neutral baseline when no info
    }

    let intersection: Vec<&String> = v_tags.intersection(&t_tags).collect();
    let union_size = v_tags.union(&t_tags).count() as f64;
    let jaccard = intersection.len() as f64 / union_size;

    // Level proximity: for each shared skill, average inverse distance.
    let mut level_bonus = 0.0;
    let mut level_n = 0;
    for shared in &intersection {
        let v = viewer.skills.iter().find(|s| &s.name.to_lowercase() == *shared).map(|s| s.level as f64);
        let t = target.skills.iter().find(|s| &s.name.to_lowercase() == *shared).map(|s| s.level as f64);
        if let (Some(vl), Some(tl)) = (v, t) {
            let dist = (vl - tl).abs();
            level_bonus += 1.0 - (dist / 100.0);
            level_n += 1;
        }
    }
    let level_factor = if level_n > 0 { level_bonus / level_n as f64 } else { 0.5 };

    (jaccard * 80.0 + level_factor * 20.0).clamp(0.0, 100.0)
}

/// 0–100 — credit for filling categories the viewer doesn't cover.
fn skill_complementarity_score(viewer: &User, target: &User) -> f64 {
    let cats = catalog();
    let viewer_cats: std::collections::HashSet<&'static str> = cats
        .iter()
        .filter(|c| {
            viewer
                .skill_tags
                .iter()
                .any(|s| c.skills.iter().any(|cs| cs.name.eq_ignore_ascii_case(s)))
        })
        .map(|c| c.key)
        .collect();

    let target_cats: std::collections::HashSet<&'static str> = cats
        .iter()
        .filter(|c| {
            target
                .skill_tags
                .iter()
                .any(|s| c.skills.iter().any(|cs| cs.name.eq_ignore_ascii_case(s)))
        })
        .map(|c| c.key)
        .collect();

    let new_cats: usize = target_cats.difference(&viewer_cats).count();
    // Cap so adding 4+ new categories saturates at 100.
    ((new_cats as f64 / 4.0) * 100.0).clamp(0.0, 100.0)
}

/// 0–100 — Jaccard between skills used in viewer's vs target's past projects.
fn project_overlap_score(viewer_projects: &[Project], target_projects: &[Project]) -> f64 {
    if viewer_projects.is_empty() || target_projects.is_empty() {
        return 25.0;
    }
    let v: std::collections::HashSet<String> = viewer_projects
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.to_lowercase()))
        .collect();
    let t: std::collections::HashSet<String> = target_projects
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.to_lowercase()))
        .collect();
    if v.is_empty() || t.is_empty() {
        return 25.0;
    }
    let inter = v.intersection(&t).count() as f64;
    let union = v.union(&t).count() as f64;
    (inter / union * 100.0).clamp(0.0, 100.0)
}

/// 0–100 — log-scaled projects_done × rating fraction.
fn track_record_score(target: &User) -> f64 {
    let projects = (target.projects_done as f64).max(0.0);
    let log_factor = ((projects + 1.0).ln() / 4.0).clamp(0.0, 1.0); // saturates ~e^4 ≈ 54 projects
    let rating_factor = if target.rating > 0.0 {
        (target.rating as f64 / 5.0).clamp(0.0, 1.0)
    } else {
        0.6 // neutral baseline
    };
    log_factor * rating_factor * 100.0
}

/// 0–100 — open ≥ hybrid > busy > unavailable.
fn availability_score(target: &User) -> f64 {
    use crate::models::user::AvailabilityStatus;
    match target.availability {
        AvailabilityStatus::OpenForCollab => 100.0,
        AvailabilityStatus::Busy => 40.0,
        AvailabilityStatus::Unavailable => 10.0,
    }
}

/// Compute a 0–100 match score between a **project** and a user. Used by the
/// "suggested teammates" feature so the project lead can see who fits the
/// declared skill stack.
///
/// Components:
/// - **Skill overlap (60%)** — Jaccard between project.skills and target.skill_tags
/// - **Track record (20%)** — log-scaled projects_done × rating
/// - **Availability (20%)** — open_for_collab → full credit
pub fn compute_project_match_score(project: &Project, target: &User) -> u8 {
    let p_skills: std::collections::HashSet<String> =
        project.skills.iter().map(|s| s.to_lowercase()).collect();
    let t_skills: std::collections::HashSet<String> =
        target.skill_tags.iter().map(|s| s.to_lowercase()).collect();

    let skill_overlap = if p_skills.is_empty() || t_skills.is_empty() {
        30.0 // neutral baseline when one side has no info
    } else {
        let inter = p_skills.intersection(&t_skills).count() as f64;
        let union = p_skills.union(&t_skills).count() as f64;
        (inter / union * 100.0).clamp(0.0, 100.0)
    };

    let track = track_record_score(target);
    let avail = availability_score(target);

    let total = skill_overlap * 0.60 + track * 0.20 + avail * 0.20;
    total.round().clamp(0.0, 100.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::{Skill, User};

    fn user_with_skills(name: &str, tags: Vec<&str>) -> User {
        let mut u = User::new("e@x.com", "h", name, "Dev", "CS");
        u.skill_tags = tags.iter().map(|s| s.to_string()).collect();
        u.skills = tags
            .iter()
            .map(|s| Skill { name: s.to_string(), level: 70 })
            .collect();
        u
    }

    #[test]
    fn catalog_is_non_empty() {
        let cats = catalog();
        assert!(cats.len() >= 8);
        for c in &cats {
            assert!(!c.skills.is_empty());
            assert!(!c.label_zh.is_empty());
        }
    }

    #[test]
    fn is_valid_skill_works() {
        assert!(is_valid_skill("React"));
        assert!(is_valid_skill("python")); // case-insensitive
        assert!(!is_valid_skill("MadeUpFramework9000"));
    }

    #[test]
    fn filter_valid_drops_garbage() {
        let input = vec!["React", "totally-fake", "Python"];
        let out = filter_valid_skills(&input);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn match_identical_skills_high() {
        let a = user_with_skills("A", vec!["React", "TypeScript", "Node.js"]);
        let b = user_with_skills("B", vec!["React", "TypeScript", "Node.js"]);
        let score = compute_match_score(&a, &b, &[], &[]);
        assert!(score >= 50, "got {}", score);
    }

    #[test]
    fn match_disjoint_skills_low() {
        let a = user_with_skills("A", vec!["React", "TypeScript"]);
        let b = user_with_skills("B", vec!["Unity", "C++"]);
        let score = compute_match_score(&a, &b, &[], &[]);
        assert!(score < 60, "got {}", score);
    }

    #[test]
    fn empty_inputs_return_baseline() {
        let a = User::new("a@x.com", "h", "A", "Dev", "CS");
        let b = User::new("b@x.com", "h", "B", "Dev", "CS");
        let s = compute_match_score(&a, &b, &[], &[]);
        assert!(s > 0 && s < 100);
    }

    #[test]
    fn zh_to_en_finds_match() {
        assert_eq!(zh_to_en("資料分析"), Some("Data Analysis"));
        assert_eq!(zh_to_en("機器學習"), Some("Machine Learning"));
        assert_eq!(zh_to_en("nope"), None);
    }

    #[test]
    fn search_en_by_zh_substring() {
        // "前端" matches the Frontend category label → returns all frontend skills.
        let hits = search_en_by_zh("前端");
        assert!(hits.contains(&"React"));
        assert!(hits.contains(&"TypeScript"));

        // "資料" matches Data Analysis, Data Engineering, Data Visualization, plus
        // the category label "資料 / 數據".
        let hits = search_en_by_zh("資料");
        assert!(hits.contains(&"Data Analysis"));
        assert!(hits.contains(&"SQL"));
    }
}
