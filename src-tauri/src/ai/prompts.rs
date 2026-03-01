/// System prompts for AI card evaluation and summarisation.
///
/// Static prompts are plain `pub const &str`. The Linear prompt requires
/// runtime values (ai_impact, hours_hint) and is exposed as a builder fn.

pub const SYSTEM_PROMPT_NOTION: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_hours (realistic time to read and act on this document: \
     reading at ~250 words/min + action buffer; express as decimal hours, \
     minimum 0.1). Omit ai_impact.";

pub const SYSTEM_PROMPT_SLACK: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_hours (time to read the thread and compose a reply; \
     express as decimal hours). Omit ai_impact.";

pub const SYSTEM_PROMPT_MR: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_impact (high|medium|low), \
     ai_hours (realistic decimal hours — calibrate by lines changed: \
     1-5 lines=0.1, 6-30 lines=0.25, 31-100 lines=0.5, \
     101-300 lines=1, 301-600 lines=2, 600+ lines=3+; \
     omit for meetings).";

pub const SYSTEM_PROMPT_MEETING: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_impact (high|medium|low), \
     ai_hours (realistic decimal hours — calibrate by lines changed: \
     1-5 lines=0.1, 6-30 lines=0.25, 31-100 lines=0.5, \
     101-300 lines=1, 301-600 lines=2, 600+ lines=3+; \
     omit for meetings).";

pub const SYSTEM_PROMPT_GENERIC: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_impact (high|medium|low), \
     ai_hours (realistic decimal hours — calibrate by lines changed: \
     1-5 lines=0.1, 6-30 lines=0.25, 31-100 lines=0.5, \
     101-300 lines=1, 301-600 lines=2, 600+ lines=3+; \
     omit for meetings).";

pub const SYSTEM_PROMPT_WEEK_SUMMARY: &str =
    "Summarize this work week in exactly 5 sentences.";

/// Builds the Linear system prompt at runtime because it embeds two
/// context-dependent values: `ai_impact` (derived from priority) and
/// `hours_hint` (derived from story-point estimate).
pub fn build_linear_system_prompt(ai_impact: &str, hours_hint: &str) -> String {
    format!(
        "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
         ai_impact (high|mid|low — use \"{ai_impact}\" as default based on priority), \
         ai_hours (realistic decimal hours).{hours_hint}"
    )
}
