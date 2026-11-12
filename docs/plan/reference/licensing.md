# Licensing

| Source | License | What slopr may do |
|---|---|---|
| fallow | MIT | Run as a separate binary. No code copied, no linking. Credit in README. |
| aislop | MIT (Copyright 2025 heavykenny) | Port regex and phrase lists (for example `narrative-comments-patterns.ts`) with the MIT notice in `THIRD_PARTY_NOTICES.md`. |
| stop-ai-slop | MIT (Copyright 2026 WhiteBite) | Port phrase lists and Unicode ranges with the MIT notice. |
| sloplint, slopsieve, OpenPolicyKit, SynthScan, slopgate, ai-slop-cleaner, rayonx, AI-Slop-Linter-for-code, Slop-Popper, gretchenfitze/anti-slop, scanaislop/skills | MIT | Port with the MIT notice of each repo actually copied from. |
| agent-slop-lint | Apache-2.0 with NOTICE | Port with the Apache-2.0 license text, keep its NOTICE content, mark changed files. Metric formulas are from published research (cited in its `CITATIONS.md`). |
| axonscanner | Apache-2.0 | Same as above if anything is ported. |
| slopo | AGPL-3.0-or-later | Clean-room only. Do not copy code or file structure. The technique (embedding similarity, clustering, distance weighting) may be reimplemented from the README and the author's public benchmark article; pick thresholds from slopr's own measurement. |
| vibecheck | FSL-1.1-Apache-2.0 | Source-available, competing use restricted for two years per release. Ideas only, no code or rule text. |
| aicomplex | unverified (MIT claimed in package.json, no LICENSE file) | Ideas only until a LICENSE file exists. |
| slop-guard | none | All rights reserved. Ideas only. |

Rule names and detection ideas are not copyrightable, but copying regex lists is copying code, so every ported list carries its source and license in a comment.
