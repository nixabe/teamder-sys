//! Hardcoded skill taxonomy and matching algorithm.
//!
//! Skills are organized into categories and cannot be modified by users — they
//! pick from this catalog when editing their profile, ensuring consistency for
//! search, filtering, and matching.

use serde::Serialize;

use crate::models::{project::Project, user::User};

/// A single skill tag with the category it belongs to.
#[derive(Debug, Clone, Serialize)]
pub struct SkillTag {
    pub name: &'static str,
    pub category: &'static str,
}

/// A category of related skills.
#[derive(Debug, Clone, Serialize)]
pub struct SkillCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub skills: Vec<&'static str>,
}

/// Returns the full skill catalog grouped by category.
pub fn catalog() -> Vec<SkillCategory> {
    vec![
        SkillCategory {
            key: "frontend",
            label: "Frontend",
            skills: vec![
                "React", "Next.js", "Vue.js", "Nuxt", "Svelte", "SvelteKit",
                "Angular", "Solid.js", "Astro", "TypeScript", "JavaScript",
                "HTML5", "CSS3", "Tailwind CSS", "Sass", "Styled Components",
                "Redux", "Zustand", "TanStack Query", "Webpack", "Vite",
            ],
        },
        SkillCategory {
            key: "backend",
            label: "Backend",
            skills: vec![
                "Node.js", "Express", "NestJS", "Fastify", "Deno", "Bun",
                "Python", "Django", "Flask", "FastAPI", "Ruby on Rails",
                "Go", "Gin", "Rust", "Rocket", "Actix Web", "Axum",
                "Java", "Spring Boot", "Kotlin", "Ktor", "C#", ".NET",
                "PHP", "Laravel", "GraphQL", "REST API", "gRPC", "WebSockets",
            ],
        },
        SkillCategory {
            key: "mobile",
            label: "Mobile",
            skills: vec![
                "iOS", "Swift", "SwiftUI", "Android", "Kotlin (Android)",
                "Jetpack Compose", "React Native", "Flutter", "Dart",
                "Ionic", "Expo", "Capacitor",
            ],
        },
        SkillCategory {
            key: "ai_ml",
            label: "AI / ML",
            skills: vec![
                "Machine Learning", "Deep Learning", "PyTorch", "TensorFlow",
                "Keras", "scikit-learn", "Hugging Face", "LangChain",
                "OpenAI API", "Computer Vision", "NLP", "LLM Fine-tuning",
                "Prompt Engineering", "RAG", "Reinforcement Learning",
            ],
        },
        SkillCategory {
            key: "data",
            label: "Data",
            skills: vec![
                "Data Analysis", "Data Engineering", "Data Visualization",
                "SQL", "PostgreSQL", "MySQL", "MongoDB", "Redis",
                "Snowflake", "BigQuery", "Apache Spark", "Apache Kafka",
                "Airflow", "dbt", "Tableau", "Power BI", "Pandas", "NumPy",
                "R", "Statistics",
            ],
        },
        SkillCategory {
            key: "design",
            label: "Design",
            skills: vec![
                "UI Design", "UX Design", "Figma", "Sketch", "Adobe XD",
                "Photoshop", "Illustrator", "After Effects", "Premiere Pro",
                "Branding", "Typography", "Design Systems", "Wireframing",
                "Prototyping", "Motion Design", "3D Modeling", "Blender",
                "User Research", "Accessibility (a11y)",
            ],
        },
        SkillCategory {
            key: "product",
            label: "Product",
            skills: vec![
                "Product Management", "Product Strategy", "Roadmapping",
                "User Stories", "Agile", "Scrum", "Kanban", "OKRs",
                "A/B Testing", "Analytics", "Growth", "Customer Research",
                "Stakeholder Management",
            ],
        },
        SkillCategory {
            key: "devops",
            label: "DevOps & Cloud",
            skills: vec![
                "Docker", "Kubernetes", "AWS", "Google Cloud", "Azure",
                "Vercel", "Netlify", "Cloudflare", "Terraform", "Ansible",
                "GitHub Actions", "GitLab CI", "Jenkins", "Linux",
                "Nginx", "CI/CD", "Monitoring", "Site Reliability",
            ],
        },
        SkillCategory {
            key: "security",
            label: "Security",
            skills: vec![
                "Web Security", "OAuth", "JWT", "Penetration Testing",
                "OWASP", "Cryptography", "Network Security", "DevSecOps",
            ],
        },
        SkillCategory {
            key: "gamedev",
            label: "Game Dev",
            skills: vec![
                "Unity", "Unreal Engine", "Godot", "C++", "Game Design",
                "Shader Programming", "Pixel Art", "Level Design",
            ],
        },
        SkillCategory {
            key: "hardware",
            label: "Hardware & Embedded",
            skills: vec![
                "Arduino", "Raspberry Pi", "Embedded C", "FPGA",
                "Circuit Design", "PCB Design", "IoT", "Robotics", "ROS",
            ],
        },
        SkillCategory {
            key: "business",
            label: "Business & Soft Skills",
            skills: vec![
                "Leadership", "Public Speaking", "Pitching", "Marketing",
                "Content Writing", "Copywriting", "SEO", "Social Media",
                "Sales", "Finance", "Project Management", "Translation",
            ],
        },
    ]
}

/// Flat list of all valid skill names (case-sensitive).
pub fn all_skills_flat() -> Vec<&'static str> {
    catalog()
        .into_iter()
        .flat_map(|c| c.skills.into_iter())
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
                .any(|s| c.skills.iter().any(|cs| cs.eq_ignore_ascii_case(s)))
        })
        .map(|c| c.key)
        .collect();

    let target_cats: std::collections::HashSet<&'static str> = cats
        .iter()
        .filter(|c| {
            target
                .skill_tags
                .iter()
                .any(|s| c.skills.iter().any(|cs| cs.eq_ignore_ascii_case(s)))
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
}
