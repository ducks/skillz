# skillz

Claude Code skill package manager. Install, manage, and share Claude Code skills from GitHub.

## Installation

### From crates.io

```bash
cargo install skillz-rs
```

### From source

```bash
git clone https://github.com/jakegoldsborough/skillz
cd skillz
cargo build --release
cp target/release/skillz ~/.local/bin/
```

### From GitHub releases

Download the latest binary for your platform from [releases](https://github.com/jakegoldsborough/skillz/releases)

## Quick Start

```bash
# Configure skills directory (for dotfiles integration)
skillz config set skills-dir ~/dotfiles/claude/skills

# Install a skill from GitHub
skillz install github:anthropics/example-skill
skillz install https://github.com/user/repo

# List installed skills
skillz list

# Remove a skill
skillz remove example-skill

# View config
skillz config show
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

### List Installed Skills

```bash
skillz list
```

Shows all skills in your configured skills directory that contain a valid `SKILL.md` file.

### Remove Skills

```bash
skillz remove skill-name
```

Removes the skill directory. This operation cannot be undone.

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

## Skill Structure

A valid skill must contain a `SKILL.md` file:

```
my-skill/
└── SKILL.md
```

The `SKILL.md` file contains the skill prompt that Claude Code will load.

## Future Features

- [ ] Registry support for skill discovery
- [ ] `skillz search <term>` to find skills
- [ ] `skillz update <name>` to update installed skills
- [ ] Local path installation (copy instead of clone)
- [ ] Skill templates: `skillz new my-skill`
- [ ] Skill validation and linting
- [ ] Version pinning and updates

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
