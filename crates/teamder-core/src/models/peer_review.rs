use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Per-axis scores out of 5, used to rate a collaborator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReviewScores {
    /// Technical skill / quality of work.
    pub skill: u8,
    /// Communication and responsiveness.
    pub communication: u8,
    /// Reliability — showed up, met commitments.
    pub reliability: u8,
    /// Teamwork and collaboration.
    pub teamwork: u8,
}

impl ReviewScores {
    /// Average across all axes (1.0 – 5.0).
    pub fn average(&self) -> f32 {
        let sum =
            self.skill as u32 + self.communication as u32 + self.reliability as u32 + self.teamwork as u32;
        sum as f32 / 4.0
    }

    pub fn clamp(&mut self) {
        self.skill = self.skill.clamp(1, 5);
        self.communication = self.communication.clamp(1, 5);
        self.reliability = self.reliability.clamp(1, 5);
        self.teamwork = self.teamwork.clamp(1, 5);
    }
}

/// A peer review submitted after a project.
///
/// Stored as its own collection (`peer_reviews`) — distinct from the embedded
/// `Review` aggregate cached on the user document. Both sides of a project
/// can leave one review per (reviewer, reviewee, project).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReview {
    #[serde(rename = "_id")]
    pub id: String,
    pub reviewer_id: String,
    pub reviewer_name: String,
    pub reviewee_id: String,
    pub project_id: Option<String>,
    #[serde(default)]
    pub study_group_id: Option<String>,
    pub project_name: String,
    pub scores: ReviewScores,
    pub body: String,
    #[serde(default)]
    pub endorsed_skills: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl PeerReview {
    pub fn new(
        reviewer_id: impl Into<String>,
        reviewer_name: impl Into<String>,
        reviewee_id: impl Into<String>,
        project_id: Option<String>,
        study_group_id: Option<String>,
        project_name: impl Into<String>,
        scores: ReviewScores,
        body: impl Into<String>,
        endorsed_skills: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            reviewer_id: reviewer_id.into(),
            reviewer_name: reviewer_name.into(),
            reviewee_id: reviewee_id.into(),
            project_id,
            study_group_id,
            project_name: project_name.into(),
            scores,
            body: body.into(),
            endorsed_skills,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePeerReviewRequest {
    pub reviewee_id: String,
    pub project_id: Option<String>,
    #[serde(default)]
    pub study_group_id: Option<String>,
    pub project_name: String,
    pub scores: ReviewScores,
    pub body: String,
    #[serde(default)]
    pub endorsed_skills: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PeerReviewResponse {
    pub id: String,
    pub reviewer_id: String,
    pub reviewer_name: String,
    pub reviewee_id: String,
    pub project_id: Option<String>,
    pub study_group_id: Option<String>,
    pub project_name: String,
    pub scores: ReviewScores,
    pub average: f32,
    pub body: String,
    pub endorsed_skills: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl From<PeerReview> for PeerReviewResponse {
    fn from(r: PeerReview) -> Self {
        let avg = r.scores.average();
        Self {
            id: r.id,
            reviewer_id: r.reviewer_id,
            reviewer_name: r.reviewer_name,
            reviewee_id: r.reviewee_id,
            project_id: r.project_id,
            study_group_id: r.study_group_id,
            project_name: r.project_name,
            scores: r.scores,
            average: avg,
            body: r.body,
            endorsed_skills: r.endorsed_skills,
            created_at: r.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_score_is_arithmetic_mean() {
        let s = ReviewScores { skill: 5, communication: 4, reliability: 5, teamwork: 4 };
        assert!((s.average() - 4.5).abs() < 0.01);
    }

    #[test]
    fn clamp_keeps_in_range() {
        let mut s = ReviewScores { skill: 9, communication: 0, reliability: 3, teamwork: 5 };
        s.clamp();
        assert_eq!(s.skill, 5);
        assert_eq!(s.communication, 1);
    }

    #[test]
    fn new_review_has_uuid() {
        let s = ReviewScores { skill: 5, communication: 5, reliability: 5, teamwork: 5 };
        let r = PeerReview::new("u1", "Alice", "u2", None, None, "Proj", s, "great", vec![]);
        assert_eq!(r.id.len(), 36);
        assert_eq!(r.reviewer_id, "u1");
        assert_eq!(r.reviewee_id, "u2");
    }
}
