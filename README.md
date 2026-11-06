# roostr

Know your project again in ten seconds: what it does, what it exposes, how to run it, and
what changed this week. roostr prints it from a few plain files committed next to the code,
for you and for the coding agents working in the repository.

```
$ roostr
Atelier                                         Rails 8, PostgreSQL, Solid Queue
Sells places on a pottery studio's workshops and reminds the people who booked.

WATCH OUT
  - Stripe runs in test mode on staging; never use a real card there.
  - The reminder job runs at 18:00 in the studio's time zone, not UTC, so a
    booking made later that evening for the next morning gets no reminder.

FUNCTIONALITY                                         7 entrypoints, 5 described
  Bookings        http 3
    book-workshop      Reserve a place, take payment and confirm by email.
    cancel-booking     Cancel a booking and free the place for the next person
                       on the waiting list.
  Notifications   job 1
    remind-attendees   Remind attendees the day before a workshop
  Workshops       http 1
  other           mail 2
  not scanned: schedule event cli

WORK WITH THIS PROJECT
  start     bin/dev
  test      bin/rails test
  deploy    bin/kamal deploy
            asks for the production vault password

ENVIRONMENTS
  staging       staging.atelier.example (Hetzner)
  production    atelier.example (Hetzner)
```

In a Git repository the overview adds LAST 7 DAYS, a count of what changed, and NEEDS
ATTENTION, the processes whose files changed after their record was last updated.

## How it works

Everything roostr reads lives in `.roostr/` at the repository root:

| File | What it holds | Who writes it |
|---|---|---|
| `project.yml` | name, stack, one-line description, things to watch out for, how to start, test and deploy, where it runs | you or an agent |
| `processes/<id>.yml` | one thing the project does: its entrypoints, the files that hold it, how to try it by hand, its outcomes and why it works that way | you or an agent |
| `scan` | a read-only script, in any language, that lists the project's entrypoints | you or an agent, once |
| `entrypoints.tsv` | what `scan` found: HTTP routes, jobs, mail, schedules, events, CLI tasks | `roostr scan` |

roostr knows no framework. The scan script knows yours, and roostr only reads what it
prints. roostr never makes network requests, and only `roostr scan` and `roostr check` run
project code: the scan script, nothing else.

## Agents start with the map

With the SessionStart hook `roostr init` prints, every new session of Claude Code, Gemini CLI
or Codex begins with a short project map in the agent's context: what to watch out for, how
to start and test, and each process with its first files, in at most 8,000 bytes. Commands
that deploy, release or publish are left out of it, so an agent is not handed them. The agent
gets it again after `/clear` and after its context is compacted. The map is built from
`.roostr/` alone, so it takes milliseconds and runs no project code.

Because that text goes straight to the agent, `roostr validate` (and so every check below)
reports invisible characters in `.roostr/` files and their names, such as text-direction
controls that make a line read differently from what it says. The map leaves them out, and
roostr's text output shows them as `<U+202E>`.

## Commands

| Command | What it does |
|---|---|
| `roostr` | the project overview |
| `roostr show <id>` | one process in full |
| `roostr changes [--since 7d\|YYYY-MM-DD]` | entrypoints added and removed, and processes changed, since a day (7 days by default) |
| `roostr check [--base <commit> \| --staged \| --session]` | whether `.roostr/` is still true to the code; exits 1 with what to fix |
| `roostr session start` | for an agent's session hook: notes the commit the session started at and gives the agent the project map |
| `roostr validate` | checks every record and every reference between them; exits 1 with findings |
| `roostr scan` | runs `.roostr/scan` and updates `entrypoints.tsv` |
| `roostr init` | creates `.roostr/project.yml` and prints the setup steps |

`roostr init` prints three things: a line to add to your `AGENTS.md`, so agents read the
overview before they work and keep it current afterwards; the checks that make sure they
do (below); and step-by-step instructions you can hand to an agent to write the scan script
and the first records. Claude Code reads AGENTS.md only where the project has no CLAUDE.md,
and Gemini CLI reads GEMINI.md: give them the line with a line `@AGENTS.md` in that file,
which brings AGENTS.md in, or with the line itself. The file formats, exit codes and what
roostr reads from Git are in [`shape/FORMATS.md`](shape/FORMATS.md).

## Keep it current

A map that is out of date is worse than none, and asking agents to update it is not
enough: some will, some won't. `roostr check` makes it a rule a machine enforces. It runs
the scan and compares what it finds with `entrypoints.tsv`, checks every record, and lists
the processes whose files changed after their record. It exits 1 with what to fix, worded
for the agent that has to fix it, and 0 when there is nothing.

With `--base <commit>` it judges only the work done after that commit, so no one is held up
by somebody else's old debt, and it also reports entrypoints that no process describes,
unless they were already undescribed at that commit.
When code under a record changed and the record still holds, setting its `checked` date is
the edit that says so.

Run it where work ends. `roostr init` prints all three:

- **Claude Code**: a SessionStart and a Stop hook in `.claude/settings.json`, committed with
  the project. `roostr session start` notes where the session started, and
  `roostr check --session` judges the session's own work, committed or not: an agent cannot
  finish its turn while its work leaves `.roostr/` behind. It is told what to fix and does
  it. What it found or brought in is not its work: your unfinished edits from before the
  session, and commits that arrive by `git pull` or a checkout. In plan mode, where the
  agent cannot change files, it is not held, nor after three identical holds in a row with
  nothing changed; the pre-commit hook and CI still report the problems. If roostr cannot
  run at all, the hooks let the agent finish. Headless runs
  (`claude -p`) use the hooks too. Gemini CLI and Codex send the same session id to their
  hooks, and their stop hooks work the same way where they block on exit code 2.
- **Git**: `roostr check --staged` as the pre-commit hook. It judges the commit being made,
  so a record written but not added to the commit is caught.
- **CI**: `roostr check --base origin/main` on pull requests, with the history fetched
  (`fetch-depth: 0` in GitHub Actions; in a shallow clone roostr says so rather than guess).

A harness that runs agents in a loop, such as [farmr](https://github.com/martinmilo/farmr)
(isolated dev containers for you and your coding agents), can use
`roostr check --base <where the run started>` as its gate, on a clean checkout of what the
agent committed, as CI does: in the agent's own working tree, files it never committed would
count.

The layers overlap on purpose: the Stop hook covers an agent's session, the pre-commit hook
every commit whoever makes it, CI every pull request, and a gate every run.

Things to know:

- `entrypoints.tsv` is written by `roostr scan`. When two branches both changed it, resolve
  the conflict by running `roostr scan` after the merge rather than by hand.
- If your repository uses a hook manager (husky, lefthook, pre-commit) or Git's
  `core.hooksPath`, Git does not read `.git/hooks/pre-commit`: add the line to the manager
  instead.
- When `.roostr/` is in a subfolder (a repository holding several projects), `roostr init`
  prints every hook and the CI line with a `cd` into it first, since hooks start at the top
  of the repository or wherever the agent is: `cd services/shop && roostr check --staged`.

## For tools

The commands that read take `--json` and print one JSON object marked `"format": 1`, with
the same exit codes. `roostr --json` is the whole map: the project, every process in full,
every entrypoint with the processes that list it, and what needs attention. `roostr check
--json` gives each problem with its kind. The shapes, and the rule for how they may change,
are in [`shape/FORMATS.md`](shape/FORMATS.md) under "JSON output". `roostr --version` says
which roostr is installed.

## Install

Each [release](https://github.com/martinmilo/roostr/releases) has binaries for Linux (static,
x86_64 and arm64, no runtime dependency) and macOS (Apple Silicon and Intel), and a
`SHA256SUMS` file to check them against. This line picks the right one; `~/.local/bin` has to
be on your PATH:

```sh
mkdir -p ~/.local/bin
curl -fsSLo ~/.local/bin/roostr "https://github.com/martinmilo/roostr/releases/latest/download/roostr-$(uname -s | tr A-Z a-z)-$(uname -m)"
chmod +x ~/.local/bin/roostr
roostr --version
```

Elsewhere, build from source with Rust 1.85 or later:

```sh
git clone https://github.com/martinmilo/roostr.git
cargo install --locked --path roostr/work
```

That puts `roostr` in `~/.cargo/bin`. roostr needs Git 2.25 or later on the PATH for the
overview's recent changes, `roostr changes` and `roostr check`. Every release is tested
on Linux and macOS.

## Status

0.3.0, which adds `roostr check` and JSON output; [`releases/0.3.0.md`](releases/0.3.0.md)
says how to upgrade from 0.2.0. All eight commands work and the acceptance suite passes.
The file formats may still change before 1.0; such a change will be in the release notes,
in [`releases/`](releases/).

## Repository layout

The repository is split in two, and each half is worked on by its own coding agent:

- `shape/` decides what roostr does: the spec records in `spec/`, the acceptance tests in
  `test/`, and `FORMATS.md`.
- `work/` is the program, in Rust. It is written against the output of the tests, without
  reading them, so a passing suite means the behaviour is there.

To work on it you need Linux or macOS, Rust (on Linux with the musl target for your machine:
`work/bin/build` makes a static binary there), Ruby 3.0 or later with Minitest, and Git 2.28
or later:

```sh
work/bin/verify --all   # builds work/bin/roostr, then runs every acceptance test
work/bin/check          # formatting, Clippy and cargo-deny
```

CI runs both on every push and pull request.

## Contributing

GitHub is a mirror of this repository. Issues are welcome there. Pull requests there are
proposals: they are not merged on GitHub. Accepted changes are applied in the source
repository, arrive in the next release, and each contributor is credited in its release
notes.

## License

[MIT](LICENSE)
