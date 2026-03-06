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
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences summarising \
     what changes and why), ai_impact (high|medium|low — based on scope of changes, \
     risk, and blast radius), \
     ai_hours (realistic decimal hours for review — calibrate by lines changed: \
     1-5=0.1, 6-30=0.25, 31-100=0.5, 101-300=1, 301-600=2, 600+=3+). \
     No markdown wrapping.";

pub const SYSTEM_PROMPT_MEETING: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences describing \
     the meeting purpose and expected outcome), \
     ai_impact (high|medium|low — assess based on attendees, decision-making significance, \
     and business impact). \
     Omit ai_hours (duration is already known from the calendar event). \
     No markdown wrapping.";

pub const SYSTEM_PROMPT_GENERIC: &str =
    "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
     ai_impact (high|medium|low — assess by urgency and business scope), \
     ai_hours (realistic decimal hours to complete this task; default to 0.5 if unclear). \
     No markdown wrapping.";

pub const SYSTEM_PROMPT_WEEK_SUMMARY: &str =
    "You are summarising a developer's work week from their Kanban cards. \
     Write a concise summary covering exactly these three things in order: \
     (1) Top 3 focus areas — name the specific work done, not generic categories; \
     be concrete about what was built, reviewed, or resolved. \
     (2) Work split — if clocked time data is provided, state the percentage and hours \
     per category; if not, estimate from card counts and hour estimates. \
     (3) What to carry into next week — one sentence. \
     Use first-person past tense. No markdown headers or bullet points — write flowing prose. \
     Do not invent details not present in the card data.";

/// Builds the Linear system prompt at runtime because it embeds two
/// context-dependent values: `ai_impact` (derived from priority) and
/// `hours_hint` (derived from story-point estimate).
pub fn build_linear_system_prompt(ai_impact: &str, hours_hint: &str) -> String {
    format!(
        "JSON only. Keys: ai_title (<=6 words), ai_description (1-2 sentences), \
         ai_impact (high|medium|low — use \"{ai_impact}\" as default based on priority), \
         ai_hours (realistic decimal hours).{hours_hint}"
    )
}
