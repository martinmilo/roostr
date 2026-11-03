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
prints. roostr never makes network requests, and only `roostr scan` runs project code.

## Commands

| Command | What it does |
|---|---|
| `roostr` | the project overview |
| `roostr show <id>` | one process in full |
| `roostr changes [--since 7d\|YYYY-MM-DD]` | entrypoints added and removed, and processes changed, since a day (7 days by default) |
| `roostr validate` | checks every record and every reference between them; exits 1 with findings |
| `roostr scan` | runs `.roostr/scan` and updates `entrypoints.tsv` |
| `roostr init` | creates `.roostr/project.yml` and prints the setup steps |

`roostr init` prints two things: a line to add to your `AGENTS.md`, so agents read the
overview before they work and keep it current afterwards, and step-by-step instructions you
can hand to an agent to write the scan script and the first records. The file formats, exit
codes and what roostr reads from Git are in [`shape/FORMATS.md`](shape/FORMATS.md).

## Install

There are no prebuilt binaries yet. Build from source with Rust 1.85 or later:

```sh
git clone https://github.com/martinmilo/roostr.git
cargo install --locked --path roostr/work
```

That puts `roostr` in `~/.cargo/bin`. roostr needs `git` on the PATH for the overview's
recent changes and for `roostr changes`. It is developed and tested on Linux.

## Status

0.2.0, which renames roosty to roostr; [`releases/0.2.0.md`](releases/0.2.0.md) says how to
upgrade from 0.1.0. All six commands work and the acceptance suite passes.
The file formats may still change before 1.0; such a change will be in the release notes,
in [`releases/`](releases/).

## Repository layout

The repository is split in two, and each half is worked on by its own coding agent:

- `shape/` decides what roostr does: the spec records in `spec/`, the acceptance tests in
  `test/`, and `FORMATS.md`.
- `work/` is the program, in Rust. It is written against the output of the tests, without
  reading them, so a passing suite means the behaviour is there.

To work on it you need Linux, Rust with the musl target for your machine (`work/bin/build`
makes a static binary), Ruby 3.0 or later with Minitest, and Git 2.28 or later:

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
