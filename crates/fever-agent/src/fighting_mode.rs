use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvaluationCriterion {
    Correctness,
    Security,
    Speed,
    Maintainability,
    TestEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionWeights {
    pub weights: HashMap<EvaluationCriterion, f32>,
}

impl Default for CriterionWeights {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(EvaluationCriterion::Correctness, 1.0);
        weights.insert(EvaluationCriterion::Security, 0.8);
        weights.insert(EvaluationCriterion::Speed, 0.6);
        weights.insert(EvaluationCriterion::Maintainability, 0.7);
        weights.insert(EvaluationCriterion::TestEvidence, 0.9);
        Self { weights }
    }
}

impl CriterionWeights {
    pub fn get(&self, criterion: &EvaluationCriterion) -> f32 {
        *self.weights.get(criterion).unwrap_or(&0.5)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionProposal {
    pub id: String,
    pub agent_role: String,
    pub task_description: String,
    pub code: String,
    pub scores: HashMap<EvaluationCriterion, f32>,
    pub created_at: String,
}

impl SolutionProposal {
    pub fn total_weighted_score(&self, weights: &CriterionWeights) -> f32 {
        self.scores.iter().map(|(c, s)| weights.get(c) * s).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredSolution {
    pub proposals: Vec<SolutionProposal>,
    pub winner_index: Option<usize>,
    pub selection_rationale: String,
}

pub struct SolutionArbiter {
    weights: CriterionWeights,
}

impl SolutionArbiter {
    pub fn new(weights: CriterionWeights) -> Self {
        Self { weights }
    }

    pub fn evaluate(&self, proposals: Vec<SolutionProposal>) -> ScoredSolution {
        if proposals.is_empty() {
            return ScoredSolution {
                proposals: vec![],
                winner_index: None,
                selection_rationale: "No proposals to evaluate".to_string(),
            };
        }

        if proposals.len() == 1 {
            return ScoredSolution {
                proposals: proposals.clone(),
                winner_index: Some(0),
                selection_rationale: format!(
                    "Only one proposal (id={}), selected by default.",
                    proposals[0].id
                ),
            };
        }

        let scores: Vec<f32> = proposals
            .iter()
            .map(|p| p.total_weighted_score(&self.weights))
            .collect();

        let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        let winner_indices: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter_map(|(i, &s)| {
                if (s - max_score).abs() < f32::EPSILON {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        let winner_index = if winner_indices.len() == 1 {
            winner_indices.into_iter().next()
        } else {
            None
        };
        let rationale = if let Some(idx) = winner_index {
            let proposal = &proposals[idx];
            let score = scores[idx];
            format!(
                "Solution {} (id={}) selected with weighted score {:.1}. \
                 Scores: correctness={:.0}, security={:.0}, speed={:.0}, \
                 maintainability={:.0}, test_evidence={:.0}.",
                idx + 1,
                proposal.id,
                score,
                proposal
                    .scores
                    .get(&EvaluationCriterion::Correctness)
                    .copied()
                    .unwrap_or(0.0),
                proposal
                    .scores
                    .get(&EvaluationCriterion::Security)
                    .copied()
                    .unwrap_or(0.0),
                proposal
                    .scores
                    .get(&EvaluationCriterion::Speed)
                    .copied()
                    .unwrap_or(0.0),
                proposal
                    .scores
                    .get(&EvaluationCriterion::Maintainability)
                    .copied()
                    .unwrap_or(0.0),
                proposal
                    .scores
                    .get(&EvaluationCriterion::TestEvidence)
                    .copied()
                    .unwrap_or(0.0),
            )
        } else {
            "Tie: multiple proposals have the same score. No clear winner.".to_string()
        };

        ScoredSolution {
            proposals,
            winner_index,
            selection_rationale: rationale,
        }
    }
}

pub struct RuleBasedScorer;

impl RuleBasedScorer {
    pub fn score_solution(code: &str) -> HashMap<EvaluationCriterion, f32> {
        let mut scores = HashMap::new();

        // Correctness: has function signatures, no obvious panics
        let correctness = Self::score_correctness(code);
        scores.insert(EvaluationCriterion::Correctness, correctness);

        // Security: no unwrap() on user input, no unsafe blocks
        let security = Self::score_security(code);
        scores.insert(EvaluationCriterion::Security, security);

        // Speed: no unnecessary allocations, no O(n^2) patterns
        let speed = Self::score_speed(code);
        scores.insert(EvaluationCriterion::Speed, speed);

        // Maintainability: has docs, reasonable function length
        let maintainability = Self::score_maintainability(code);
        scores.insert(EvaluationCriterion::Maintainability, maintainability);

        // Test evidence: has #[test] attributes
        let test_evidence = Self::score_test_evidence(code);
        scores.insert(EvaluationCriterion::TestEvidence, test_evidence);

        scores
    }

    fn score_correctness(code: &str) -> f32 {
        let mut score: f32 = 50.0;
        if code.contains("fn ") {
            score += 15.0;
        }
        if code.contains("pub fn ") {
            score += 10.0;
        }
        if code.contains("-> Result<") {
            score += 15.0;
        }
        if code.contains("-> Option<") {
            score += 10.0;
        }
        if code.contains("unwrap()") {
            score -= 20.0;
        }
        if code.contains("panic!(") {
            score -= 30.0;
        }
        if code.contains("todo!") {
            score -= 10.0;
        }
        if code.contains("unimplemented!") {
            score -= 30.0;
        }
        score.clamp(0.0, 100.0)
    }

    fn score_security(code: &str) -> f32 {
        let mut score: f32 = 70.0;
        if code.contains("unsafe") {
            score -= 40.0;
        }
        if code.contains("unwrap()") {
            score -= 15.0;
        }
        if code.contains("expect(") {
            score += 5.0;
        }
        if code.contains("validate") || code.contains("sanitize") {
            score += 15.0;
        }
        if code.contains("allowlist") || code.contains("denylist") {
            score += 10.0;
        }
        score.clamp(0.0, 100.0)
    }

    fn score_speed(code: &str) -> f32 {
        let mut score: f32 = 70.0;
        if code.contains(".clone()") {
            score -= 5.0;
        }
        if code.contains("O(n^2)") || code.contains("O(n²)") {
            score -= 15.0;
        }
        if code.contains("O(n log n)") {
            score += 10.0;
        }
        if code.contains("HashMap") || code.contains("HashSet") {
            score += 5.0;
        }
        if code.contains("Vec::new()") && code.contains("for") {
            score -= 3.0;
        }
        score.clamp(0.0, 100.0)
    }

    fn score_maintainability(code: &str) -> f32 {
        let mut score: f32 = 60.0;
        let lines: Vec<&str> = code.lines().collect();
        if lines.len() > 200 {
            score -= 15.0;
        }
        if code.contains("/// ") || code.contains("//! ") {
            score += 15.0;
        }
        if code.contains("#[derive(") {
            score += 5.0;
        }
        let has_mod = lines.iter().any(|l| l.trim_start().starts_with("pub mod "));
        if has_mod {
            score += 5.0;
        }
        let long_fns: Vec<_> = lines
            .iter()
            .filter(|l| l.contains("fn ") && !l.contains("//") && !l.contains("#["))
            .collect();
        let avg_len = if long_fns.is_empty() {
            0
        } else {
            long_fns.iter().map(|f| f.len()).sum::<usize>() / long_fns.len()
        };
        if avg_len > 100 {
            score -= 10.0;
        }
        score.clamp(0.0, 100.0)
    }

    fn score_test_evidence(code: &str) -> f32 {
        let mut score: f32 = 20.0;
        if code.contains("#[test]") {
            score += 40.0;
        }
        if code.contains("#[tokio::test]") {
            score += 40.0;
        }
        if code.contains("assert!") || code.contains("assert_eq!") {
            score += 10.0;
        }
        if code.contains("mock") || code.contains("Mock") {
            score += 10.0;
        }
        score.clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(id: &str, code: &str) -> SolutionProposal {
        let scores = RuleBasedScorer::score_solution(code);
        SolutionProposal {
            id: id.to_string(),
            agent_role: "coder".to_string(),
            task_description: "test".to_string(),
            code: code.to_string(),
            scores,
            created_at: "2026-03-31T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_single_proposal_always_wins() {
        let arbiter = SolutionArbiter::new(CriterionWeights::default());
        let sol = make_proposal("s1", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
        let result = arbiter.evaluate(vec![sol]);
        assert_eq!(result.winner_index, Some(0));
        assert!(result.selection_rationale.contains("s1"));
    }

    #[test]
    fn test_better_code_wins() {
        let arbiter = SolutionArbiter::new(CriterionWeights::default());
        let good = make_proposal(
            "s1",
            "pub fn add(a: i32, b: i32) -> Result<i32, String> {\n    if a < 0 || b < 0 {\n        return Err(\"negative\".to_string());\n    }\n    Ok(a + b)\n}",
        );
        let bad = make_proposal("s2", "fn add(a: i32, b: i32) -> i32 { a + b }");
        let result = arbiter.evaluate(vec![good, bad]);
        assert_eq!(result.winner_index, Some(0));
        assert!(result.selection_rationale.contains("s1"));
    }

    #[test]
    fn test_tie_returns_none() {
        let arbiter = SolutionArbiter::new(CriterionWeights::default());
        let s1 = make_proposal("s1", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
        let s2 = make_proposal("s2", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
        let result = arbiter.evaluate(vec![s1, s2]);
        assert!(result.winner_index.is_none());
        assert!(result.selection_rationale.contains("Tie"));
    }

    #[test]
    fn test_empty_proposals() {
        let arbiter = SolutionArbiter::new(CriterionWeights::default());
        let result = arbiter.evaluate(vec![]);
        assert!(result.winner_index.is_none());
        assert_eq!(result.proposals.len(), 0);
    }

    #[test]
    fn test_rule_based_correctness_scoring() {
        let good = RuleBasedScorer::score_solution(
            "pub fn validated_add(a: i32, b: i32) -> Result<i32, String> { Ok(a + b) }",
        );
        let bad =
            RuleBasedScorer::score_solution("fn add(a: i32, b: i32) -> i32 { unimplemented!() }");
        assert!(good[&EvaluationCriterion::Correctness] > bad[&EvaluationCriterion::Correctness]);
    }

    #[test]
    fn test_rule_based_security_scoring() {
        let safe = RuleBasedScorer::score_solution(
            "pub fn validate(input: &str) -> Result<bool, String> {\n    if input.is_empty() { return Err(\"empty\".to_string()); }\n    Ok(true)\n}",
        );
        let unsafe_code =
            RuleBasedScorer::score_solution("unsafe fn raw_ptr() -> *const u8 { 0 as *const _ }");
        assert!(safe[&EvaluationCriterion::Security] > unsafe_code[&EvaluationCriterion::Security]);
    }

    #[test]
    fn test_custom_weights() {
        let mut weights = CriterionWeights::default();
        weights
            .weights
            .insert(EvaluationCriterion::TestEvidence, 2.0);
        weights
            .weights
            .insert(EvaluationCriterion::Correctness, 0.1);
        let arbiter = SolutionArbiter::new(weights);
        let tested = make_proposal("s1", "#[test]\nfn test_add() { assert_eq!(1 + 1, 2); }");
        let untested = make_proposal("s2", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
        let result = arbiter.evaluate(vec![tested, untested]);
        assert_eq!(result.winner_index, Some(0));
    }
}
