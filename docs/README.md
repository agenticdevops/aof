# AOF Documentation Structure

## 📁 Directory Overview

```
docs/
├── user-docs/              ➜ Symlink to ../docusaurus-site/docs
│                              (SINGLE SOURCE OF TRUTH for user documentation)
├── architecture/           Internal: System design and architecture decisions
├── research/              Internal: Analysis and research findings
├── schemas/               Internal: Data structure definitions
├── agentflow/             Internal: Workflow specifications
├── examples/              Internal: Example configurations
├── reference/             Internal: Technical references
├── tutorials/             Internal: Tutorial materials
└── README.md             This file
```

## 🎯 Single Source of Truth

**All user-facing documentation lives in one place:**

```
/docusaurus-site/docs/     ← Edit here for user-facing docs
     ↑ (symlinked from)
/docs/user-docs/           ← Convenient reference location
```

### User Documentation (`/docusaurus-site/docs/`)

This is the **production documentation** published at https://aof.sh

Contains:
- **getting-started.md** - Installation and setup
- **concepts.md** - AOF fundamentals
- **tutorials/** - Step-by-step guides
  - first-agent.md
  - incident-response.md
  - slack-bot.md
- **examples/** - Copy-paste ready configurations
- **reference/** - Complete API reference
  - aofctl-complete.md
  - api-resources.md
  - agent-spec.md
- **guides/** - Specialized guides
  - migration-guide.md
  - kubernetes-compatibility.md

### Internal Documentation (`/docs/`)

This folder contains:
- **architecture/** - ADRs and system design
- **research/** - Analysis and implementation notes
- **schemas/** - Data structure definitions
- **agentflow/** - Workflow specifications
- **Other folders** - Project-specific documentation

## 📝 How to Update Documentation

### For User-Facing Documentation
**Edit:** `/docusaurus-site/docs/` (the real source)

**Or use the symlink:** `docs/user-docs/` (points to same location)

```bash
# Both of these edit the same files:
vim docusaurus-site/docs/getting-started.md
vim docs/user-docs/getting-started.md  # Same file!
```

### For Internal Documentation
**Edit:** `/docs/architecture/`, `/docs/research/`, etc.

These are separate from user documentation and never published to the website.

## 🔄 Automatic Publication

GitHub Actions workflow (`/.github/workflows/deploy-docs.yml`):

1. **Triggers on:** Push to main/dev branches in docusaurus-site/ or docs/
2. **Builds:** Docusaurus site from `/docusaurus-site/docs/`
3. **Deploys:** To GitHub Pages (https://aof.sh when configured)

## ✅ Best Practices

### When Editing User Docs
1. Always edit in `/docusaurus-site/docs/`
2. Use the **verb-noun** command pattern (kubectl-compatible)
3. Test locally: `cd docusaurus-site && npm run start`
4. Commit and push to trigger auto-deployment

### When Adding Internal Docs
1. Create in appropriate folder under `/docs/`
2. Link to from relevant user documentation when applicable
3. These are NOT published to the website

### One Source of Truth Rules
- ✅ Edit `/docusaurus-site/docs/` for user documentation
- ✅ Use `docs/user-docs/` as a convenient reference
- ❌ Don't maintain separate copies in different locations
- ❌ Don't expect manual publication - GitHub Actions handles it

## 🔗 Symlink Explanation

The `docs/user-docs` symlink means:

```bash
ls docs/user-docs/           # Shows docusaurus-site/docs/ contents
cat docs/user-docs/concepts.md  # Same as cat docusaurus-site/docs/concepts.md
```

This provides:
- Single source of truth (only one location to edit)
- Convenient navigation from the docs/ folder
- Clear indication that user docs are symlinked

## 📚 For Content Contributors

### Adding a New Tutorial

1. Create file: `docusaurus-site/docs/tutorials/my-tutorial.md`
2. Follow template structure (see getting-started.md)
3. Use current **verb-noun** patterns
4. Update `docusaurus-site/sidebars.ts` if needed
5. Test locally
6. Commit and push

### Updating Existing Docs

1. Edit directly in `/docusaurus-site/docs/`
2. Test locally to verify formatting
3. Commit and push
4. GitHub Actions auto-deploys

### Internal Documentation

1. Add to appropriate folder in `/docs/`
2. Reference from user docs when relevant
3. These won't be published publicly

## 🚀 Quick Commands

```bash
# Test user documentation locally
cd docusaurus-site
npm run start  # Opens http://localhost:3000

# Build production
npm run build

# Check what user docs look like
ls docs/user-docs/  # Same as ls docusaurus-site/docs/

# View symlink
readlink docs/user-docs  # Shows: ../docusaurus-site/docs
```

## 📖 More Information

- **For user doc guidelines:** See `docusaurus-site/DOCUMENTATION_GUIDE.md`
- **For architecture decisions:** See `docs/architecture/ADR-001-kubectl-cli.md`
- **For CLI reference:** See `docs/user-docs/reference/aofctl-complete.md`

---

**Remember:** Edit in `/docusaurus-site/docs/` - it's the single source of truth! 🎯
