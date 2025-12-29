# VHP GitHub Pages Theme

The VHP project features a **modern, minimalist, and elegant GitHub Pages theme**. Designed with simplicity and polish in mind, it combines sophisticated colors, smooth interactions, and clean typography.

## 🎨 Design Philosophy

The theme embraces:
- **Simplicity** — Clean layouts without unnecessary clutter
- **Elegance** — Refined colors and subtle interactions
- **Readability** — Excellent typography and spacing
- **Performance** — Pure CSS, minimal JavaScript, no external dependencies
- **Accessibility** — WCAG AA+ contrast, semantic HTML, keyboard navigation

## 🌈 Color Palette

| Element | Color | Hex | Purpose |
|---------|-------|-----|---------|
| Dark | #1a202c | `--color-dark` | Headings, main text |
| Slate | #2d3748 | `--color-slate` | Secondary headings |
| **Teal** | #06b6d4 | `--color-teal` | Primary accent (tech/modern feel) |
| Teal Dark | #0891b2 | `--color-teal-dark` | Hover states |
| **Amber** | #f59e0b | `--color-accent` | Highlights, h2 underlines |
| Text | #1f2937 | `--color-text` | Body text |
| Light Gray | #f3f4f6 | `--color-gray-light` | Backgrounds, cards |
| Border | #e5e7eb | `--color-gray` | Dividers, borders |

## ✨ Key Features

### Modern Interactions
- **Links**: Animated underline on hover (amber bar slides in)
- **Buttons**: Lift effect with shadow on hover
- **Cards**: Move up with glow effect on hover
- **Navigation**: Smooth color transitions

### Visual Hierarchy
- **H1**: Large gradient text (cyan → dark)
- **H2**: Underlined with amber border (distinctive)
- **H3-H6**: Proper size and color relationships
- **Spacing**: Generous, 7-level spacing system

### Components
- **Navigation**: Sticky header with backdrop blur
- **Hero Section**: Large gradient heading with prominent CTAs
- **Feature Cards**: Grid with animated top border gradient
- **Code Blocks**: Dark gradient background with accent left border
- **Tables**: Hover effects, gradient headers
- **Buttons**: Gradient backgrounds with shadows

## 📱 Responsive Design

Mobile-first approach with three breakpoints:

- **Desktop** (769px+): Full layout with navigation
- **Tablet** (481-768px): Optimized spacing
- **Mobile** (≤480px): Single column, adjusted font sizes

## 🔤 Typography

- **Body Font**: System font stack (-apple-system, Segoe UI, Roboto, etc.)
- **Monospace**: Fira Code → Courier New
- **Responsive Sizing**: 3rem → 1.75rem on mobile
- **Letter Spacing**: Negative tracking on headings for elegance

## 📐 Spacing Scale

```css
--space-xs: 0.25rem   /* Minimal */
--space-sm: 0.5rem    /* Small */
--space-md: 1rem      /* Base */
--space-lg: 1.5rem    /* Default paragraph/spacing */
--space-xl: 2rem      /* Generous */
--space-2xl: 3rem     /* Large sections */
--space-3xl: 4rem     /* Hero/major spacing */
```

## 📁 Theme Structure

```
docs/
├── _layouts/
│   ├── default.html    # Standard page layout
│   ├── home.html       # Hero-focused homepage
│   └── page.html       # Simple article layout
├── assets/css/
│   └── style.css       # ~700 lines, well-organized
└── THEME.md           # This file
```

The theme files are integrated directly into `docs/` for GitHub Pages compatibility.

## 🎯 Layout Templates

### default.html
Used for standard content pages. Includes:
- Sticky navigation header
- Main content area
- Footer
- Mobile menu toggle

### home.html
Specialized for landing pages. Features:
- `.hero` container for centered content
- Large gradient headings
- `.hero-links` for prominent buttons
- Perfect for showcasing projects

### page.html
Simple article layout extending default.

## 🔧 Customization

### Change the Primary Accent

```css
:root {
  --color-teal: #0ea5e9;     /* Sky blue instead */
  --color-teal-dark: #0284c7;
}
```

### Change the Warm Accent

```css
:root {
  --color-accent: #ef4444;   /* Red highlights */
}
```

### Adjust Spacing Globally

```css
:root {
  --space-md: 1.25rem;  /* More spacious */
  --space-lg: 2rem;
  --space-xl: 2.5rem;
}
```

### Modify Typography

```css
--font-base: "Charter", Georgia, serif;  /* Serif instead of sans */
--font-mono: "IBM Plex Mono", monospace;
```

## 🚀 Performance

- **No external fonts** — Uses system fonts
- **Pure CSS** — No heavy frameworks
- **Minimal JS** — Only for mobile navigation
- **~700 lines** — Complete theme in one CSS file
- **Optimized** — Proper cascade, no redundancy

## ♿ Accessibility

- ✅ WCAG AA+ contrast ratios
- ✅ Semantic HTML5
- ✅ ARIA labels on interactive elements
- ✅ Keyboard navigation support
- ✅ Focus indicators on interactive elements

## 🌐 Browser Support

- ✅ Chrome/Edge (latest)
- ✅ Firefox (latest)
- ✅ Safari (latest)
- ✅ Mobile browsers (iOS Safari, Chrome Mobile)
- ⚠️ IE 11 (graceful degradation, features may not work)

## 💡 Design Highlights

### Gradients with Purpose
```css
/* Hero heading: cyan fades to dark */
background: linear-gradient(135deg, var(--color-teal) 0%, var(--color-dark) 100%);

/* Footer: dark slate gradient for depth */
background: linear-gradient(135deg, var(--color-dark) 0%, var(--color-slate) 100%);

/* Card accent: teal to amber gradient */
background: linear-gradient(90deg, var(--color-teal) 0%, var(--color-accent) 100%);
```

### Smooth Transitions
```css
--transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
--transition-fast: all 0.15s ease;
```

### Depth with Shadows
```css
box-shadow: 0 10px 25px rgba(6, 182, 212, 0.3);  /* Teal glow on buttons */
box-shadow: 0 10px 30px rgba(6, 182, 212, 0.1);  /* Card hover glow */
```

## 📝 Configuration

The theme respects these settings in `docs/_config.yml`:

```yaml
title: VHP
description: Your project description
nav:
  - title: Home
    url: /
  - title: Docs
    url: /docs
  - title: GitHub
    url: https://github.com/...
```

## 🎁 What Makes It Shine

1. **Gradient Effects** — Modern, subtle use of gradients for depth
2. **Color Harmony** — Cyan (tech) + Amber (warmth) create balance
3. **Generous Spacing** — Breathing room for better readability
4. **Smooth Interactions** — Animated underlines, lift effects, glows
5. **Cohesive Typography** — Proper hierarchy with modern sizing
6. **Responsive** — Adapts beautifully to any screen size

## 🔄 Future Enhancements

Possible additions:
- Dark mode variant
- Search functionality
- Syntax highlighting for code blocks
- Sidebar navigation
- Print stylesheet refinement

## 📄 License

This theme is part of the VHP project and follows the same license.

## 🙌 Credits

**Design**: Modern minimalist aesthetic with elegant touches
**Colors**: Carefully curated for tech/development projects
**Typography**: Optimized system fonts for all platforms
**Interactions**: Smooth transitions for polished feel

---

**Live Site**: [VHP Documentation](https://leocavalcante.github.io/vhp/)
**Theme Directory**: `_vhp_theme/` (reference) | `docs/` (active)
