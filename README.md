# skillz

Claude Code skill package manager. Install, manage, and share Claude Code skills from GitHub.

## Installation

### From crates.io

```bash
cargo install skillz-rs
```

### From source

```bash
git clone https://github.com/ducks/skillz
cd skillz
cargo build --release
cp target/release/skillz ~/.local/bin/
```

### From GitHub releases

Download the latest binary for your platform from [releases](https://github.com/ducks/skillz/releases)

## Quick Start

```bash
# Search for skills on GitHub
skillz search "daily notes"

# Install a skill from GitHub
skillz install github:user/skill
skillz install https://github.com/user/repo

# List installed skills with timestamps
skillz list

# Update all skills
skillz update

# Update a specific skill
skillz update skill-name

# Auto-sync on startup (quiet mode)
skillz update --auto

# Remove a skill
skillz remove skill-name

# Configure skills directory (for dotfiles)
skillz config set skills-dir ~/dotfiles/claude/skills
```

## Usage

### Install Skills

Install from GitHub using shorthand notation:
```bash
skillz install github:user/repo
```

Or full URL:
```bash
skillz install https://github.com/user/my-skill
```

The skill name will be extracted from the repository name.

### Search for Skills

Search GitHub for Claude Code skills:

```bash
skillz search "status report"
skillz search automation
```

Searches repository names, descriptions, and READMEs for skills. Sorts results by stars.

Optional: Set `GITHUB_TOKEN` environment variable for higher API rate limits.

### Update Skills

Update a specific skill:
```bash
skillz update skill-name
```

Update all installed skills:
```bash
skillz update
```

Auto-sync mode (quiet, for startup scripts):
```bash
skillz update --auto
```

### List Installed Skills

```bash
skillz list
```

Shows all installed skills with:
- Source repository URL
- Installation date
- Last sync date

### Remove Skills

```bash
skillz remove skill-name
```

Removes the skill directory and registry entry. This operation cannot be undone.

### Configuration

Configure where skills are installed:

```bash
# Set custom skills directory (e.g., for dotfiles)
skillz config set skills-dir ~/dotfiles/claude/skills

# View current config
skillz config show

# Get specific value
skillz config get skills-dir
```

**Default locations:**
- Config: `~/.config/skillz/config.toml`
- Registry: `~/.config/skillz/registry.toml`
- Skills: `~/.claude/skills/` (or custom via config)

## Dotfiles Integration

If you manage your dotfiles with stow/chezmoi/yadm:

1. Set skills directory to your dotfiles:
   ```bash
   skillz config set skills-dir ~/dotfiles/claude/skills
   ```

2. Install skills:
   ```bash
   skillz install github:user/skill
   ```

3. Skills are now in your dotfiles repo. Commit them:
   ```bash
   cd ~/dotfiles
   git add claude/skills
   git commit -m "Add Claude skill"
   ```

4. On other machines, your dotfile manager will symlink `~/.claude/skills` to your dotfiles location.

## Skill Validation

skillz validates skills on install and update:

**Basic checks:**
- SKILL.md exists
- Valid UTF-8 encoding
- Non-empty content
- Contains markdown headings
- File size under 1MB

**Security checks:**

Scans for potentially dangerous commands and prompts before installation:
- Destructive rm commands (`rm -rf /`, `~`, `*`)
- Pipe to shell from internet (`curl | bash`, `wget | sh`)
- Fork bombs
- Disk fill operations (`dd if=/dev/zero`)
- Dangerous permissions (`chmod 777`)
- System file modifications (`/etc/`, `/bin/`)
- Crypto mining indicators
- Network listeners
- `eval` with variable expansion
- `sudo` without explanation

## Skill Structure

A valid skill must contain a `SKILL.md` file:

```
my-skill/
└── SKILL.md
```

The `SKILL.md` file contains the skill prompt that Claude Code will load.

## Future Features

- [ ] Semantic versioning (currently using timestamps)
- [ ] Dependency management (skill A requires skill B)
- [ ] Rollback to previous versions
- [ ] Local path installation (copy instead of clone)
- [ ] Skill templates: `skillz new my-skill`

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- list
```

## License

MIT OR Apache-2.0
