/// Onboarding utilities for FeverCode projects.
pub mod onboarder;
pub mod profile;
pub mod questions;
pub mod scaffold;

/// Public exports for onboarding results and builders.
pub use onboarder::{OnboardResult, Onboarder};
pub use profile::ProjectProfile;
pub use questions::{Question, QuestionBlock, QuestionOption, Validation, all_questions};
pub use scaffold::{GeneratedFile, ScaffoldGenerator};
