---
name: Claude Hub — Warm Ink
colors:
  bg: '#1a1715'
  surface: '#232020'
  surface-hi: '#2c2826'
  border: '#353130'
  text-1: '#f5f1ec'
  text-2: '#a8a09a'
  text-3: '#6e6864'
  accent: '#d97757'
  accent-hover: '#e88a6c'
  ok: '#7ba05b'
  warn: '#d4a25a'
  danger: '#c1554a'
  info: '#4a7a9e'
  light-bg: '#faf7f2'
  light-surface: '#ffffff'
  light-surface-hi: '#f0ebe3'
  light-border: '#e8e2d8'
  light-text-1: '#2a2521'
  light-text-2: '#6e6864'
  light-text-3: '#a8a09a'
typography:
  display-lg:
    fontFamily: Geist
    fontSize: 56px
    fontWeight: '600'
    lineHeight: 58px
    letterSpacing: -0.025em
  display-md:
    fontFamily: Geist
    fontSize: 40px
    fontWeight: '600'
    lineHeight: 44px
    letterSpacing: -0.02em
  display-sm:
    fontFamily: Geist
    fontSize: 28px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.015em
  h1:
    fontFamily: Geist
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: -0.01em
  h2:
    fontFamily: Geist
    fontSize: 17px
    fontWeight: '600'
    lineHeight: 22px
    letterSpacing: -0.005em
  h3:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '600'
    lineHeight: 19px
  body:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 21px
  body-sm:
    fontFamily: Geist
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 19px
  caption:
    fontFamily: Geist
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 17px
  label:
    fontFamily: Geist
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: 0.005em
  eyebrow:
    fontFamily: Geist
    fontSize: 11px
    fontWeight: '600'
    lineHeight: 13px
    letterSpacing: 0.08em
    textTransform: uppercase
  mono:
    fontFamily: Geist Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 19px
  mono-sm:
    fontFamily: Geist Mono
    fontSize: 11.5px
    fontWeight: '400'
    lineHeight: 16px
  mono-num:
    fontFamily: Geist Mono
    fontSize: 14px
    fontWeight: '600'
    lineHeight: 17px
    fontVariantNumeric: tabular-nums
rounded:
  xs: 0.125rem
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.5rem
  lg: 0.75rem
  xl: 1rem
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 20px
  '2xl': 24px
  '3xl': 32px
  '4xl': 40px
  '5xl': 48px
  '6xl': 64px
  gutter: 16px
  margin: 28px
motion:
  fast: 120ms
  base: 180ms
  slow: 400ms
  ease-out: 'cubic-bezier(0.32, 0.72, 0, 1)'
  ease-in-out: 'cubic-bezier(0.65, 0, 0.35, 1)'
---

# Claude Hub Design System

This document outlines the visual and structural guidelines for Claude Hub, a local-first "Mission Control" desktop app that surfaces every Claude Code session, project, MCP server, and skill on a developer's machine.

## Visual Identity
Claude Hub balances warm, organic neutrals with precise, terminal-adjacent density. The system ships **dark-first** on warm ink (`#1a1715`) — never pure black, never cool slate — with a fully considered cream-paper light mode. A single coral accent (`#d97757`) carries the brand across both modes, used sparingly so it always means "this is the action". The product feels technical, dense, and trustworthy: power users live in it all day, and every pixel should be doing work.

Borders, not shadows, separate planes. Shadows appear only on floating layers (menus, modals, the command palette). There are no gradients anywhere in the chrome — the **one exception** is the context meter's fill, which carries a left-to-right linear gradient so fullness reads at a glance.

## Colors
The palette is centered on warm neutrals, coral accent, and three muted status hues. Status colors answer exactly one question — *how full is the context window?* — and are never used decoratively.

- **Background:** `#1a1715` (Warm Ink) — app background, never pure black.
- **Surface:** `#232020` (Charcoal) — cards, header, title bar.
- **Surface Hi:** `#2c2826` (Hover Plane) — hover states, bar tracks, elevated panels.
- **Border:** `#353130` (Hairline) — 1 px dividers and card outlines do most of the visual work.
- **Text:** `#f5f1ec` primary, `#a8a09a` secondary, `#6e6864` tertiary — three weights of warm gray. Pure white (`#fff`) is never used; `#f5f1ec` is the warm ceiling.
- **Accent (Clay):** `#d97757` — primary CTAs, active outline, focus ring. Hovers to `#e88a6c`. If more than ~3 coral elements appear on one screen, something is wrong.
- **Status — OK:** `#7ba05b` (Moss) — under 60% of context window.
- **Status — Warn:** `#d4a25a` (Amber) — 60–85%.
- **Status — Danger:** `#c1554a` (Brick) — over 85% (context wall imminent).
- **Light mode** mirrors the same structure on cream paper (`#faf7f2` / `#fff` / `#f0ebe3` / `#e8e2d8`) with neutrals flipped and accent + status hues unchanged.

## Typography
The system uses **Geist** and **Geist Mono** — both variable, both self-hosted from `project/fonts/`, no external requests. No serif anywhere. Information-type distinction is carried by family (sans vs. mono), weight, and size, not by face contrast.

- **Geist** — every heading, label, button, caption, body. Headlines use 600 weight with tight negative tracking (`-0.025` to `-0.005em`).
- **Geist Mono** — all numerics, token counts, file paths, session names, MCP IDs — anything code-like or copy-pasteable. Use `font-feature-settings: 'calt' off` to suppress code ligatures.
- **Display (28–56px / 600):** hero numbers and section headlines.
- **Headings (14–22px / 600):** screen titles and section heads inside chrome.
- **Body (13–14px / 400):** default reading size at 1.5x line height.
- **Eyebrow (11px / 600 / +0.08em uppercase):** the only `ALL CAPS` in the system — for section labels at 10–11px, never for emphasis.
- **Mono (11.5–14px):** numbers always `tabular-nums`; the dedicated `mono-num` class is 14px / 600 for headline figures.

## Layout & Spacing
A **4px base scale** governs every dimension: `4, 8, 12, 16, 20, 24, 32, 40, 48, 64`. Never `6, 10, 14, 18` — stay on the grid.

- **Base Unit:** 4px.
- **Dense list rows:** 8px vertical padding.
- **Card padding:** 16–20px.
- **Section breaks:** 32–48px.
- **Window grid:** 44px title bar · fluid main · 24px status bar.
- **Sidebar:** fixed 240px, collapsible to 56px. Min app width 960px — below that, the rail auto-collapses.
- **Detail rail:** 360px, appears whenever a session is selected.
- **Density target:** 30–50 distinct atoms per view; rows scannable in under 200ms.

## Elevation & Depth
Hierarchy comes from **borders, not shadows**. A 1px hairline in `#353130` separates planes; the warm surface progression (`bg` → `surface` → `surface-hi`) gives subtle tonal layering. Shadows are reserved for floating layers only.

- **Menus & popovers:** `0 1px 2px rgba(0,0,0,0.12), 0 8px 24px rgba(0,0,0,0.28)`.
- **Modals & command palette:** `0 2px 4px rgba(0,0,0,0.18), 0 24px 60px rgba(0,0,0,0.40)`.
- **Backdrop blur:** `blur(20–24px) saturate(140%)` for the palette and command menus; `blur(12px)` for sticky toolbars. Everywhere else: opaque. Blur is the consistent cue that *this layer is floating*.

## Shapes
The design language is **restrained-rounded**. **No pills** — `9999px` / pill radii are never used in Hub.

- **Inputs & small chips:** 4px (0.25rem) — `--r-sm`.
- **Buttons & cards:** 8px (0.5rem) — `--r-md`.
- **Modals & the app window:** 12px (0.75rem) — `--r-lg`.
- **Tiny markers:** 2px (0.125rem) — `--r-xs`.

## Motion
Restrained and fast. No bounces, no fades longer than 200ms, no `scale(0.98)` press effects (a mobile trick, wrong for a desktop tool).

- **State changes (hover, focus, press):** 120ms with `ease-out`.
- **Appear / disappear:** 180ms with `ease-out` (e.g. the palette translates up 4px + fades in).
- **Live data (context meter, token counter):** 400ms with `ease-out` — feels alive but not jittery.
- **Focus:** 2px clay outline at 2px offset on every interactive element. Hub is keyboard-first; focus must always be obvious.
- **Hover:** backgrounds warm one step (`surface` → `surface-hi`); text stays put.

## Components
- **Buttons:** Primary is clay fill with white text, hover lifts to `#e88a6c`. Secondary is panel fill with hairline border. Ghost is transparent. Sizes sm/md/lg map to heights 24/32/38 with 6/8/8px radii.
- **Inputs:** 32px tall, 6px radius, hairline border. Focus replaces the border with clay and adds a 3px 18%-opacity clay ring.
- **Chips (status pills):** 22px tall, 4px radius, color-mixed background + dot in OK / Warn / Danger / Idle.
- **Tags (resource chips):** 20px tall, 4px radius, **mono 11/500** on sunken background — used for MCP IDs and "+ N skills" indicators.
- **Kbd:** 18px keycap, mono 11/500, hairline border with a 2px bottom edge for keycap silhouette.
- **Status dots:** 8px circle; running gets a 3px haloed glow in OK green.
- **Context Meter:** the headline atom and the only place a gradient appears. 6–8px track, half-height radius, fill auto-selects OK/Warn/Danger by usage band, width animates 400ms.
- **Cards:** 1px border, 8px radius, opaque ground, 16–20px padding. No tilt, no glow, no gradient borders. Hover lifts the border one step — no shadow change, no translate.
- **Command palette (⌘K):** 580px wide, 12px radius, blurred overlay with modal shadow. The single most-used surface in the product.
