//! Ported phrase lists. `{a|b}` expands to each alternative and `{a|}` makes `a` optional.
//! Only the English alternatives that expand to fixed phrases are ported.

/// Comment openers for `comment/narrative`.
///
/// From scanaislop/aislop `src/engines/ai-slop/narrative-comments-patterns.ts` at
/// 9e83199f503fb8dd82c2688efdeca9ce8b404a32, MIT: `JUSTIFICATION_OPENERS` and
/// `EXPLANATORY_OPENERS`.
/// From WhiteBite/stop-ai-slop `skill/scripts/src/markers.mjs` at
/// d2c586881bf4ead6ab2865dde90f45f247d58972, MIT: `THIS_OPENER`, whose English part is a
/// subset of the `this ...` row.
pub const NARRATIVE: &[&str] = &[
    "{the idea here|the trick is|this was needed|originally}",
    "this {function|method|class|module|component|hook|util|helper|handler|service}",
    "it {does|handles|takes|returns|processes|reads|writes|sends|fetches|loads|creates|deletes|updates|parses|validates}",
    "{first|then|finally|next|lastly|subsequently}{,|} {it|we|the function|the method|the class}",
    "{matches|detects|represents|holds|stores|tracks|handles|manages|controls|contains|captures|encapsulates|wraps|describes}",
];

/// Phrases for `comment/agent-artifact`.
///
/// From scanaislop/aislop `src/engines/ai-slop/meta-comment.ts` at
/// 9e83199f503fb8dd82c2688efdeca9ce8b404a32, MIT: `PLAN_REFERENCE_RES` and
/// `BEFORE_AFTER_RES`. Left out: the step header (`comment/step-numbered` covers it) and
/// the patterns that need a number or a wildcard (`step N of the plan`, `requirement N`,
/// `changed ... from ... to`, `this was ... but now`).
/// From WhiteBite/stop-ai-slop `skill/scripts/src/markers.mjs` at
/// d2c586881bf4ead6ab2865dde90f45f247d58972, MIT: `AI_PLAN_REFERENCE` and `AI_PLAN_ACK`.
pub const AGENT_ARTIFACT: &[&str] = &[
    // PLAN_REFERENCE_RES
    "as {per|requested} {the |}{requirement|requirements|spec|task|ticket|prompt|instruction|instructions}",
    "per the {spec|requirement|requirements|task|ticket|plan|prompt|instruction|instructions}",
    "from the {task|todo|plan|spec|ticket|prompt|requirement|requirements}",
    "implement{ing|s|ed|} use{ |}case",
    "{requirement|requirements} doc",
    "as {instructed|specified|outlined} {above|below|in the}",
    // BEFORE_AFTER_RES
    "previously{,|:|} {this|we|it|the}",
    "used to {be|use|call|return|do|have|rely}",
    "no longer {needed|used|required|necessary|call|calls|return|returns|does}",
    "we {now|used to} {no longer |}{use|call|return|do|have}",
    "replaced the {old|previous|former}",
    "{was|were} {renamed|moved|removed|refactored|extracted} {from|to|out of}",
    // AI_PLAN_REFERENCE
    "per the {spec|task|ticket|prompt|plan}",
    "as per {requirement|requirements}",
    // AI_PLAN_ACK
    "as {instructed|requested}",
    "the user asked",
];
