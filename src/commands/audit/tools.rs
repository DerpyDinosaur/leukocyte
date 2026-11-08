use crate::commands::audit::types::{FindingScoreRule, Verdict};

pub fn is_prerelease(version: &str) -> bool {
    version.split('+').next().unwrap_or("").contains('-')
}

fn normalize(value: u8, min: u8, max: u8, inverse: bool) -> f32 {
    let (min, max) = (min as f32, max as f32);
    let range = max - min;
    let normalized = if range > 0.0 {
        ((value as f32).clamp(min, max) - min) / range
    } else {
        0.0
    };
    if inverse { 1.0 - normalized } else { normalized }
}

pub fn calculate_verdict(rules: &[FindingScoreRule]) -> u8 {
    /*
        TODO: Could be better to return a map of scores that correlate to declared rules.
        Then have the rules as constants somewhere.
        This would remove the unique test from the audit report to check if the weight adds up to 1
    */
    const TOTAL_WEIGHT: f32 = 1.0;
    let score: f32 = rules.iter().map(|r| normalize(r.value, r.min, r.max, r.inverse) * r.weight).sum();
    ((score / TOTAL_WEIGHT) * 100.0).round() as u8
}
