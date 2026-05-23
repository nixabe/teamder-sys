//! Bilingual skill catalog and match-score computation.

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::models::user::User;

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: &'static str,
    pub name_zh: &'static str,
}

#[derive(Debug, Clone)]
pub struct SkillCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub label_zh: &'static str,
    pub skills: &'static [SkillEntry],
}

// ── Catalog data ─────────────────────────────────────────────────────────────

macro_rules! skills {
    ($( ($name:expr, $zh:expr) ),* $(,)?) => {
        &[ $( SkillEntry { name: $name, name_zh: $zh } ),* ]
    };
}

pub static SKILL_CATALOG: &[SkillCategory] = &[
    SkillCategory {
        key: "frontend",
        label: "Frontend",
        label_zh: "\u{524d}\u{7aef}\u{958b}\u{767c}",
        skills: skills![
            ("React", "React"),
            ("Vue", "Vue"),
            ("TypeScript", "TypeScript"),
            ("Tailwind", "Tailwind"),
            ("Next.js", "Next.js"),
            ("Angular", "Angular"),
            ("Svelte", "Svelte"),
            ("HTML/CSS", "HTML/CSS"),
            ("JavaScript", "JavaScript"),
            ("Sass/SCSS", "Sass/SCSS"),
            ("Webpack", "Webpack"),
            ("Vite", "Vite"),
        ],
    },
    SkillCategory {
        key: "backend",
        label: "Backend",
        label_zh: "\u{5f8c}\u{7aef}\u{958b}\u{767c}",
        skills: skills![
            ("Node.js", "Node.js"),
            ("Django", "Django"),
            ("Rust", "Rust"),
            (".NET", ".NET"),
            ("Spring Boot", "Spring Boot"),
            ("Go", "Go"),
            ("Express", "Express"),
            ("FastAPI", "FastAPI"),
            ("Ruby on Rails", "Ruby on Rails"),
            ("PHP", "PHP"),
            ("GraphQL", "GraphQL"),
            ("REST API", "REST API"),
        ],
    },
    SkillCategory {
        key: "mobile",
        label: "Mobile",
        label_zh: "\u{884c}\u{52d5}\u{958b}\u{767c}",
        skills: skills![
            ("iOS (Swift)", "iOS (Swift)"),
            ("Android (Kotlin)", "Android (Kotlin)"),
            ("Flutter", "Flutter"),
            ("React Native", "React Native"),
            ("Xamarin", "Xamarin"),
            ("SwiftUI", "SwiftUI"),
            ("Jetpack Compose", "Jetpack Compose"),
        ],
    },
    SkillCategory {
        key: "ai_ml",
        label: "AI / ML",
        label_zh: "\u{4eba}\u{5de5}\u{667a}\u{6167} / \u{6a5f}\u{5668}\u{5b78}\u{7fd2}",
        skills: skills![
            ("PyTorch", "PyTorch"),
            ("TensorFlow", "TensorFlow"),
            ("LangChain", "LangChain"),
            ("RAG", "RAG"),
            ("Computer Vision", "Computer Vision"),
            ("NLP", "NLP"),
            ("Scikit-learn", "Scikit-learn"),
            ("OpenAI API", "OpenAI API"),
            ("Hugging Face", "Hugging Face"),
            ("MLOps", "MLOps"),
        ],
    },
    SkillCategory {
        key: "data",
        label: "Data",
        label_zh: "\u{6578}\u{64da}\u{5de5}\u{7a0b}",
        skills: skills![
            ("SQL", "SQL"),
            ("BigQuery", "BigQuery"),
            ("dbt", "dbt"),
            ("Tableau", "Tableau"),
            ("Pandas", "Pandas"),
            ("Spark", "Spark"),
            ("PostgreSQL", "PostgreSQL"),
            ("MongoDB", "MongoDB"),
            ("Redis", "Redis"),
            ("Elasticsearch", "Elasticsearch"),
            ("Data Modeling", "Data Modeling"),
        ],
    },
    SkillCategory {
        key: "design",
        label: "Design",
        label_zh: "\u{8a2d}\u{8a08}",
        skills: skills![
            ("Figma", "Figma"),
            ("UI/UX", "UI/UX"),
            ("Branding", "Branding"),
            ("Prototyping", "Prototyping"),
            ("Illustration", "Illustration"),
            ("Adobe XD", "Adobe XD"),
            ("Sketch", "Sketch"),
            ("Design Systems", "Design Systems"),
            ("User Research", "User Research"),
        ],
    },
    SkillCategory {
        key: "product",
        label: "Product",
        label_zh: "\u{7522}\u{54c1}\u{7ba1}\u{7406}",
        skills: skills![
            ("Product Management", "Product Management"),
            ("OKRs", "OKRs"),
            ("A/B Testing", "A/B Testing"),
            ("User Research", "User Research"),
            ("Roadmapping", "Roadmapping"),
            ("Agile/Scrum", "Agile/Scrum"),
            ("JIRA", "JIRA"),
            ("Analytics", "Analytics"),
        ],
    },
    SkillCategory {
        key: "devops",
        label: "DevOps",
        label_zh: "DevOps \u{8207}\u{96f2}\u{7aef}",
        skills: skills![
            ("Docker", "Docker"),
            ("AWS", "AWS"),
            ("GitHub Actions", "GitHub Actions"),
            ("CI/CD", "CI/CD"),
            ("Kubernetes", "Kubernetes"),
            ("Terraform", "Terraform"),
            ("Linux", "Linux"),
            ("Nginx", "Nginx"),
            ("GCP", "GCP"),
            ("Azure", "Azure"),
        ],
    },
    SkillCategory {
        key: "security",
        label: "Security",
        label_zh: "\u{8cc7}\u{8a0a}\u{5b89}\u{5168}",
        skills: skills![
            ("OAuth", "OAuth"),
            ("JWT", "JWT"),
            ("Web Security", "Web Security"),
            ("Penetration Testing", "Penetration Testing"),
            ("OWASP", "OWASP"),
            ("Encryption", "Encryption"),
            ("Network Security", "Network Security"),
        ],
    },
    SkillCategory {
        key: "game",
        label: "Game Dev",
        label_zh: "\u{904a}\u{6232}\u{958b}\u{767c}",
        skills: skills![
            ("Unity", "Unity"),
            ("Unreal", "Unreal"),
            ("Godot", "Godot"),
            ("Game Design", "Game Design"),
            ("3D Modeling", "3D Modeling"),
            ("Blender", "Blender"),
            ("Shader Programming", "Shader Programming"),
        ],
    },
    SkillCategory {
        key: "hardware",
        label: "Hardware",
        label_zh: "\u{786c}\u{9ad4}\u{8207}\u{5d4c}\u{5165}\u{5f0f}",
        skills: skills![
            ("Arduino", "Arduino"),
            ("IoT", "IoT"),
            ("Robotics", "Robotics"),
            ("FPGA", "FPGA"),
            ("Embedded C", "Embedded C"),
            ("Raspberry Pi", "Raspberry Pi"),
            ("PCB Design", "PCB Design"),
        ],
    },
    SkillCategory {
        key: "business",
        label: "Business",
        label_zh: "\u{5546}\u{696d}\u{8207}\u{8edf}\u{5be6}\u{529b}",
        skills: skills![
            ("Leadership", "Leadership"),
            ("Marketing", "Marketing"),
            ("Sales", "Sales"),
            ("Public Speaking", "Public Speaking"),
            ("Project Management", "Project Management"),
            ("Technical Writing", "Technical Writing"),
            ("Communication", "Communication"),
        ],
    },
];

// ── Pre-computed lookup sets ─────────────────────────────────────────────────

/// Case-insensitive lookup: lowercase name -> canonical name.
static SKILL_NAME_LOWER: LazyLock<std::collections::HashMap<String, &'static str>> =
    LazyLock::new(|| {
        SKILL_CATALOG
            .iter()
            .flat_map(|cat| cat.skills.iter().map(|s| (s.name.to_lowercase(), s.name)))
            .collect()
    });

/// Lowercase skill name -> category key.
static SKILL_TO_CATEGORY: LazyLock<std::collections::HashMap<String, &'static str>> =
    LazyLock::new(|| {
        SKILL_CATALOG
            .iter()
            .flat_map(|cat| {
                cat.skills
                    .iter()
                    .map(move |s| (s.name.to_lowercase(), cat.key))
            })
            .collect()
    });

// ── Public API ───────────────────────────────────────────────────────────────

/// Returns `true` if `name` is a recognized skill (case-insensitive).
pub fn is_valid_skill(name: &str) -> bool {
    SKILL_NAME_LOWER.contains_key(&name.to_lowercase())
}

/// Returns the category key for a skill name (case-insensitive).
pub fn category_of(skill: &str) -> Option<&'static str> {
    SKILL_TO_CATEGORY.get(&skill.to_lowercase()).copied()
}

/// Filters a list of skill strings, keeping only recognized ones.
pub fn filter_valid_skills(skills: &[String]) -> Vec<String> {
    skills
        .iter()
        .filter(|s| is_valid_skill(s))
        .cloned()
        .collect()
}

// ── Match scoring ────────────────────────────────────────────────────────────

/// Compute a 0-100 match score between a viewer and a target user.
///
/// Weights:
/// - 40 %  skill overlap (Jaccard similarity + level proximity)
/// - 15 %  skill complementarity (target has skills viewer lacks)
/// - 20 %  project domain overlap (categories of skills)
/// - 15 %  track record (log-scaled projects_done * rating/5)
/// - 10 %  availability alignment
pub fn compute_match_score(viewer: &User, target: &User) -> u8 {
    let skill_overlap = skill_overlap_score(viewer, target);
    let complementarity = complementarity_score(viewer, target);
    let domain_overlap = domain_overlap_score(viewer, target);
    let track_record = track_record_score(target);
    let availability = availability_score(viewer, target);

    let raw = skill_overlap * 0.40
        + complementarity * 0.15
        + domain_overlap * 0.20
        + track_record * 0.15
        + availability * 0.10;

    // Clamp to 0-100
    (raw.round() as u8).min(100)
}

/// Jaccard similarity of skill tag sets, boosted by level proximity.
fn skill_overlap_score(viewer: &User, target: &User) -> f64 {
    if viewer.skill_tags.is_empty() && target.skill_tags.is_empty() {
        return 50.0; // neutral when no data
    }

    let v_set: HashSet<&str> = viewer.skill_tags.iter().map(|s| s.as_str()).collect();
    let t_set: HashSet<&str> = target.skill_tags.iter().map(|s| s.as_str()).collect();

    let intersection = v_set.intersection(&t_set).count() as f64;
    let union = v_set.union(&t_set).count() as f64;

    if union == 0.0 {
        return 50.0;
    }

    let jaccard = intersection / union;

    // Level proximity bonus: for shared skills, how close are the levels?
    let level_bonus = if intersection > 0.0 {
        let mut total_proximity = 0.0;
        let mut count = 0.0;
        for skill_name in v_set.intersection(&t_set) {
            let v_level = viewer
                .skills
                .iter()
                .find(|s| s.name == *skill_name)
                .map(|s| s.level as f64)
                .unwrap_or(50.0);
            let t_level = target
                .skills
                .iter()
                .find(|s| s.name == *skill_name)
                .map(|s| s.level as f64)
                .unwrap_or(50.0);
            // proximity = 1.0 when levels are equal, 0.0 when 100 apart
            total_proximity += 1.0 - (v_level - t_level).abs() / 100.0;
            count += 1.0;
        }
        (total_proximity / count) * 0.2 // max 20% bonus
    } else {
        0.0
    };

    ((jaccard + level_bonus) * 100.0).min(100.0)
}

/// How many skills does the target have that the viewer lacks?
fn complementarity_score(viewer: &User, target: &User) -> f64 {
    if target.skill_tags.is_empty() {
        return 0.0;
    }

    let v_set: HashSet<&str> = viewer.skill_tags.iter().map(|s| s.as_str()).collect();
    let novel: usize = target
        .skill_tags
        .iter()
        .filter(|s| !v_set.contains(s.as_str()))
        .count();

    let ratio = novel as f64 / target.skill_tags.len() as f64;
    (ratio * 100.0).min(100.0)
}

/// Overlap of skill category domains.
fn domain_overlap_score(viewer: &User, target: &User) -> f64 {
    let v_cats: HashSet<&str> = viewer
        .skill_tags
        .iter()
        .filter_map(|s| category_of(s))
        .collect();
    let t_cats: HashSet<&str> = target
        .skill_tags
        .iter()
        .filter_map(|s| category_of(s))
        .collect();

    if v_cats.is_empty() && t_cats.is_empty() {
        return 50.0;
    }

    let union = v_cats.union(&t_cats).count() as f64;
    if union == 0.0 {
        return 50.0;
    }

    let intersection = v_cats.intersection(&t_cats).count() as f64;
    (intersection / union * 100.0).min(100.0)
}

/// Log-scaled track record: projects_done * (rating / 5).
fn track_record_score(target: &User) -> f64 {
    if target.projects_done == 0 {
        return 0.0;
    }

    let rating_factor = if target.rating > 0.0 {
        target.rating as f64 / 5.0
    } else {
        0.5 // neutral when no ratings
    };

    let raw = (target.projects_done as f64).ln_1p() * rating_factor;
    // Normalize: ln(1+20)*1.0 ≈ 3.04, so scale to 0-100
    (raw / 3.05 * 100.0).min(100.0)
}

/// Availability alignment between viewer and target.
fn availability_score(viewer: &User, target: &User) -> f64 {
    match (&viewer.availability, &target.availability) {
        (Some(v), Some(t)) => {
            if v == t {
                100.0
            } else if t == "open_for_collab" {
                80.0
            } else if t == "busy" {
                30.0
            } else {
                10.0 // unavailable
            }
        }
        (_, Some(t)) => {
            if t == "open_for_collab" {
                70.0
            } else {
                30.0
            }
        }
        _ => 50.0, // no data, neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_skill() {
        assert!(is_valid_skill("React"));
        assert!(is_valid_skill("react")); // case insensitive
        assert!(is_valid_skill("Next.js"));
        assert!(!is_valid_skill("FakeSkill123"));
    }

    #[test]
    fn test_category_of() {
        assert_eq!(category_of("React"), Some("frontend"));
        assert_eq!(category_of("Docker"), Some("devops"));
        assert_eq!(category_of("Unity"), Some("game"));
        assert_eq!(category_of("nonexistent"), None);
    }

    #[test]
    fn test_filter_valid_skills() {
        let input = vec![
            "React".to_string(),
            "Fake".to_string(),
            "Docker".to_string(),
        ];
        let valid = filter_valid_skills(&input);
        assert_eq!(valid, vec!["React", "Docker"]);
    }

    #[test]
    fn test_catalog_has_12_categories() {
        assert_eq!(SKILL_CATALOG.len(), 12);
    }
}
