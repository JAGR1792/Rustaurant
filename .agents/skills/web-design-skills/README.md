# Web Design System Skills for Claude Code

A comprehensive collection of Claude Code skills for building world-class websites. These skills work together as an integrated design system — from visual philosophy and responsive patterns to full website rebuild pipelines.

Built by [Jesse Anglen](https://github.com/lotfb86) at [Ruh AI](https://ruh.ai).

---

## What Are These?

These are **Claude Code skills** — structured instructions that give Claude deep expertise in specific domains. Drop them into your `~/.claude/skills/` directory and Claude gains the ability to build production-grade websites with agency-level quality.

The system is designed to produce sites that pass the **AI Slop Test**: if someone looked at the output and was told "AI made this," they wouldn't believe it.

---

## Skills Overview

### 1. Frontend Design (`01-frontend-design/`)

The core visual design philosophy. Defines how to make bold, distinctive interfaces that avoid generic AI aesthetics.

**Key concepts:**
- Bold aesthetic direction — pick an extreme and commit
- Typography: distinctive font pairings (with a banned font list)
- Color systems with CSS custom properties
- Motion: CSS-only animations, staggered reveals, scroll-triggered effects
- Spatial composition: asymmetry, overlap, grid-breaking layouts
- Visual depth: textures, gradients, layered compositions

**Use when:** Building any UI component, page, or application.

---

### 2. Responsive Design (`02-responsive-design/`)

Mobile-first layout patterns, breakpoint systems, fluid typography, and container queries.

**Key concepts:**
- Standard breakpoint system (640/768/1024/1280/1536px)
- Mobile-first with `min-width` media queries only
- CSS Grid auto-responsive patterns (no media queries needed)
- Fluid typography with `clamp()`
- Container queries for component-level responsiveness
- Responsive images with `srcset` and `<picture>`
- Testing checklist for all viewport sizes

**Use when:** Any UI that must work across mobile, tablet, and desktop.

---

### 3. Web Design Guidelines (`03-web-design-guidelines/`)

Reviews UI code against Vercel's Web Interface Guidelines for compliance.

**How it works:**
1. Fetches latest guidelines from Vercel's repo
2. Reads specified files
3. Checks against all rules
4. Outputs findings in `file:line` format

**Use when:** Reviewing or auditing existing UI code.

---

### 4. Theme Factory (`04-theme-factory/`)

10 pre-built professional themes with color palettes and font pairings, plus custom theme generation.

**Available themes:**
1. Ocean Depths — Professional maritime
2. Sunset Boulevard — Warm, vibrant
3. Forest Canopy — Natural earth tones
4. Modern Minimalist — Clean grayscale
5. Golden Hour — Rich autumnal
6. Arctic Frost — Cool winter
7. Desert Rose — Soft, sophisticated
8. Tech Innovation — Bold, modern
9. Botanical Garden — Fresh, organic
10. Midnight Galaxy — Dramatic, cosmic

**Use when:** Styling slides, docs, reports, HTML pages, or any artifact that needs a cohesive theme.

---

### 5. Website Rebuild (`05-website-rebuild/`)

The full-power website rebuild agent. Takes an existing URL and rebuilds it from scratch into a $20K+ agency-quality site.

**14-phase process:**
1. Visual Inspection & Brand Extraction
2. Data Scraping & Content Classification
3. Comprehensive SEO Audit
4. Design & UX/UI Audit
5. Rebuild Plan
6. Build the Website
7. Copy Audit & Optimization
8. QA Round 1 — Functional & Content
9. QA Round 2 — Copy & Messaging
10. QA Round 3 — Final Visual & Polish
11a. Competitive Tools Research Sprint
11b. Client-Facing Tools Build
12. QA Round 4 — Tools QA
13. Pre-Deployment Checklist
14. Deploy to Vercel

**Tech stack:** Astro 5.x + Tailwind CSS v4, deployed to Vercel via GitHub.

**Reference files included:**
- `design-rules.md` — Non-negotiable visual design rules
- `accessibility-spec.md` — WCAG AA specification with contrast requirements
- `phase-details.md` — Detailed instructions for all 14 phases
- `nano-banana.md` — AI image enhancement via OpenRouter

**Scripts included:**
- `contrast-check.js` — WCAG contrast ratio verification tool

**Use when:** Rebuilding any existing website from scratch.

---

### 6. Local Business Rebuild (`06-local-business-rebuild/`)

Streamlined version of the website rebuild, optimized for speculative local business pitches.

**10-phase process** (removes tools research/build, consolidates QA to 1 round):
1. Brand Extraction
2. SEO Audit
3. Design & UX Audit
4. Rebuild Plan
5. Build the Website
6. Copy Audit & Optimization
7. Combined QA
8. Pre-Deploy Checklist
9. Prepare for Deployment
10. Summary

**Key features:**
- Image Selection Protocol — detailed rules for choosing culturally appropriate, business-specific images
- Text-over-image contrast enforcement (the #1 accessibility failure)
- Local SEO with Google Business schema
- Monorepo output structure for prospect pipeline

**Use when:** Building speculative websites for local businesses you want to pitch.

---

### 7. Azerbaijan Website Build (`07-azerbaijan-website-build/`)

Premium tri-lingual website builder for Azerbaijani businesses. Every site ships in Azerbaijani, Russian, and English.

**8-phase process:**
1. Data Gathering (website scrape, social media, or interview)
2. Brand Development (logo, colors, typography)
3. Content Strategy & Site Architecture
4. Image Curation
5. Build the Website
6. Copy & Translation
7. QA (Combined Round)
8. Deployment

**Key features:**
- Tri-lingual i18n system (directory-based: `/az/`, `/ru/`, `/en/`)
- Complete i18n utility (`i18n.ts`) with language switcher component
- Pre-built JSON string files for all 3 languages
- Font verification for Latin Extended + Cyrillic support
- Azerbaijani cultural design guide (colors, imagery, business types)
- Interview questionnaire for businesses with no online presence
- Logo generation via NanoBanana 2 (Google Gemini API)

**Reference files included:**
- `design-rules.md` — Azerbaijan-specific design rules
- `accessibility-spec.md` — WCAG AA spec
- `i18n-spec.md` — Complete i18n routing, components, and JSON strings
- `azerbaijani-design-guide.md` — Cultural design considerations
- `interview-questionnaire.md` — Structured business interview
- `nano-banana-logo.md` — Logo generation with Gemini API

**Use when:** Building websites for businesses in Azerbaijan.

---

### 8. Design System Generator (`08-design-system-generator/`)

Produces structured DESIGN.md files from scratch for any business. Takes business context (name, industry, brand personality, existing colors) and generates a complete 9-section design system specification.

**Key features:**
- Step-by-step generation for all 9 DESIGN.md sections
- Cross-references design-rules.md to enforce banned fonts/colors/layouts
- WCAG AA contrast verification built into the workflow
- Azerbaijan-specific extensions (Section 10: Multi-Lingual Considerations)
- Validation checklist ensuring completeness

**Use when:** Any build pipeline needs a structured design system. Invoked during the planning phase of website-rebuild, local-business-rebuild, and azerbaijan-website-build.

---

### Design References (`00-design-references/`)

Not a skill — a shared reference library containing the DESIGN.md format specification and 10 curated brand reference files from real websites.

**Organized by aesthetic category:**

| Category | Brands | Best For |
|----------|--------|----------|
| Luxury Minimal | Stripe, Apple | Fintech, premium SaaS, cinematic |
| Warm Editorial | Airbnb, Notion | Hospitality, marketplace, productivity |
| Dark Premium | Linear, Vercel | Developer tools, ultra-precise |
| Bold Energetic | Nike, Spotify | Retail, entertainment, high-energy |
| Clean Professional | Shopify, Wise | E-commerce, professional services |

**Key files:**
- `design-md-format.md` — The DESIGN.md format specification and blank template
- `design-md-index.md` — Index of all references with selection guide

Based on [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md). Files fetched via `npx getdesign@latest add [brand]`.

---

## How to Install

### Option 1: Install All Skills

```bash
# Clone the repo
git clone https://github.com/lotfb86/web-design-skills.git

# Copy all skills to your Claude Code skills directory
cp -r web-design-skills/01-frontend-design ~/.claude/skills/frontend-design-anthropic
cp -r web-design-skills/02-responsive-design ~/.claude/skills/responsive-design
cp -r web-design-skills/03-web-design-guidelines ~/.claude/skills/web-design-guidelines
cp -r web-design-skills/04-theme-factory ~/.claude/skills/theme-factory
cp -r web-design-skills/05-website-rebuild ~/.claude/skills/website-rebuild
cp -r web-design-skills/06-local-business-rebuild ~/.claude/skills/local-business-rebuild
cp -r web-design-skills/07-azerbaijan-website-build ~/.claude/skills/azerbaijan-website-build
cp -r web-design-skills/08-design-system-generator ~/.claude/skills/design-system-generator

# Copy the shared reference library (used by all build skills)
cp -r web-design-skills/00-design-references ~/.claude/skills/00-design-references
```

### Option 2: Install Individual Skills

Copy only the skill folders you need. Each skill is self-contained (reference files are included in each skill's directory).

### Option 3: Use as Reference

Read the SKILL.md files directly for design patterns, code snippets, and best practices — even without Claude Code.

---

## How Skills Relate to Each Other

```
frontend-design-anthropic (visual philosophy)
    |
    +-- 00-design-references/ (DESIGN.md format spec + brand reference library)
    |       |
    |       +-- design-md-format.md (template + 9-section schema)
    |       +-- design-md-index.md (10 brands indexed by aesthetic)
    |       +-- references/by-aesthetic/ (Stripe, Apple, Airbnb, Nike, Linear, etc.)
    |
    +-- design-system-generator (produces DESIGN.md from business context)
    |
    +-- responsive-design (layout patterns, sub-skill)
    |
    +-- web-design-guidelines (validation/review)
    |
    +-- theme-factory (theming system + DESIGN.md extended themes)
    |
    +-- website-rebuild (14-phase pipeline → produces DESIGN.md at Phase 5b)
    |
    +-- local-business-rebuild (10-phase → produces DESIGN.md at Phase 4)
    |
    +-- azerbaijan-website-build (8-phase → produces DESIGN.md at Phase 2)
```

**Flow:** During planning, rebuild skills invoke `frontend-design-anthropic` → read reference DESIGN.md files → produce a project-specific `DESIGN.md` via `design-system-generator`. During build, the DESIGN.md is the single source of truth for all visual implementation. During QA, `web-design-guidelines` validates the output.

### DESIGN.md Integration

All three build pipelines now produce a structured `DESIGN.md` during their planning phase:

- **website-rebuild** → Phase 5b generates DESIGN.md after the Rebuild Plan
- **local-business-rebuild** → Phase 4 generates DESIGN.md as part of the Design Commitment
- **azerbaijan-website-build** → Phase 2 generates DESIGN.md (with Section 10: Multi-Lingual) after the Brand Card

The DESIGN.md format captures the complete design system in 9 sections: Visual Theme, Color Palette, Typography, Component Stylings, Layout Principles, Depth/Elevation, Do's/Don'ts, Responsive Behavior, and Agent Prompt Guide. Based on the format from [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md).

---

## Tech Stack (Used Across All Build Skills)

| Component | Choice |
|-----------|--------|
| Framework | Astro 5.x |
| Styling | Tailwind CSS v4 (Vite plugin) |
| Images | Astro `<Image>` component (WebP/AVIF optimization) |
| Fonts | @fontsource (self-hosted) |
| SEO | @astrojs/sitemap + JSON-LD |
| Transitions | Astro View Transitions (zero JS) |
| Deploy | Vercel via GitHub |
| Package Manager | npm |

---

## Design Philosophy

Every skill in this system shares these principles:

1. **The AI Slop Test** — If it looks like AI made it, redesign.
2. **Bold aesthetic direction** — Pick an extreme and commit. No safe, generic middle ground.
3. **Banned fonts** — Inter, Roboto, Arial, Helvetica, Space Grotesk, Lato, Open Sans, Source Sans Pro.
4. **WCAG AA mandatory** — Every color combination verified computationally, not visually.
5. **Break the grid** — No two consecutive sections share the same layout pattern.
6. **Visual depth** — Textures, gradients, overlaps, shadows. Never flat solid backgrounds.
7. **Copy sells** — Every page is a sales tool. Hook, Problem, Solution, Proof, Action.
8. **Performance** — Lighthouse 95+. Static HTML, zero JS by default, optimized images.

---

## File Inventory

```
web-design-skills/
  00-design-references/                     # Shared reference library (not a skill)
    README.md                               # What this is, how to use it
    design-md-format.md                     # DESIGN.md template + 9-section schema
    design-md-index.md                      # Index of all references by aesthetic
    references/
      by-aesthetic/
        luxury-minimal/
          stripe.md                         # Stripe DESIGN.md
          apple.md                          # Apple DESIGN.md
        warm-editorial/
          airbnb.md                         # Airbnb DESIGN.md
          notion.md                         # Notion DESIGN.md
        dark-premium/
          linear.md                         # Linear DESIGN.md
          vercel.md                         # Vercel DESIGN.md
        bold-energetic/
          nike.md                           # Nike DESIGN.md
          spotify.md                        # Spotify DESIGN.md
        clean-professional/
          shopify.md                        # Shopify DESIGN.md
          wise.md                           # Wise DESIGN.md
  01-frontend-design/
    SKILL.md                                # Core visual design philosophy
  02-responsive-design/
    SKILL.md                                # Mobile-first patterns & breakpoints
  03-web-design-guidelines/
    SKILL.md                                # UI review against Vercel guidelines
  04-theme-factory/
    SKILL.md                                # Theme application system
    themes/                                 # Simple themes (4 colors + 2 fonts)
      arctic-frost.md
      botanical-garden.md
      ...10 themes
    design-md/                              # Extended themes (full DESIGN.md format)
      arctic-frost.md
      botanical-garden.md
      ...10 themes
  05-website-rebuild/
    SKILL.md                                # Full 14-phase rebuild (+ Phase 5b DESIGN.md)
    references/
      accessibility-spec.md
      design-rules.md
      nano-banana.md
      phase-details.md
    scripts/
      contrast-check.js
  06-local-business-rebuild/
    SKILL.md                                # Streamlined 10-phase (+ Phase 4 DESIGN.md)
    references/
      accessibility-spec.md
      design-rules.md
  07-azerbaijan-website-build/
    SKILL.md                                # Tri-lingual pipeline (+ Phase 2 DESIGN.md)
    references/
      accessibility-spec.md
      azerbaijani-design-guide.md
      design-rules.md
      i18n-spec.md
      interview-questionnaire.md
      nano-banana-logo.md
    scripts/
      contrast-check.js
  08-design-system-generator/
    SKILL.md                                # Produces DESIGN.md from business context
```

---

## License

These skills are shared for use with Claude Code. Use them, adapt them, build on them.
