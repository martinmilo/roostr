# Roostr file formats

Everything roostr reads lives under `.roostr/` in the repository root. The marker of an
initialised project is `.roostr/project.yml`. All YAML files are plain YAML 1.2 mappings.
Paths are relative to the repository root. The examples below describe an imaginary
invoicing app called Ledgerline.

## `.roostr/project.yml`

| Key | Required | Value |
|---|---|---|
| `name` | yes | the project's name, one line |
| `stack` | no | the stack, one line |
| `description` | no | one sentence |
| `watch_out` | no | a list of at most 7 sentences, each at most 160 characters: the things that bite anyone working on the project. Detail about one behaviour belongs in that process's record. |
| `operations` | no | a map of operation name to either a command string, or a map with `command` and an optional `note` of at most 160 characters |
| `environments` | no | a map of environment name to where it runs, one line of at most 160 characters each |

No other keys are allowed, here or inside an operation. Operations and environments keep the
order they are written in.

```yaml
name: Ledgerline
stack: Django 5, PostgreSQL, Celery
description: Issues invoices for a small agency and chases late payers.
watch_out:
  - Reminder mails go out for real from staging; use the test client list.
operations:
  start: make dev
  test: pytest
  deploy:
    command: make deploy
    note: needs the production .env in your shell
environments:
  staging: staging.ledgerline.example (Fly.io)
  production: ledgerline.example (Fly.io)
```

## `.roostr/processes/<id>.yml`

One file per process. **The file name without `.yml` is the process id.** There is no `id`
key. Ids are referred to on the command line as written in the file name.

| Key | Required | Value |
|---|---|---|
| `name` | yes | one line |
| `description` | yes | one line |
| `group` | no | the area the process belongs to, one line; see "Areas and groups" below |
| `entrypoints` | no | a list of entrypoint references, each the string `<kind> <name>`; see "What belongs in `entrypoints`" below |
| `files` | no | a list of items, each either a path string or a map with `path` and an optional `note` |
| `try` | no | free text, possibly several lines, describing how to exercise the process by hand. It is printed exactly as written, so indentation and spacing in a snippet are kept. roostr never runs it |
| `outcomes` | no | a map of outcome name to a map with `when` (the condition) and `then` (what happens), each one line |
| `why` | no | a list of sentences or short paragraphs |
| `checked` | no | a date written `YYYY-MM-DD`: the day someone last compared the record with the code and found that it still holds |

No other keys are allowed, at any level: an outcome has `when` and `then`, a `files` item
has `path` and `note`.

```yaml
name: Send a payment reminder
group: Collections
description: Mail a client whose invoice is overdue.
entrypoints:
  - schedule remind_overdue
  - mail ReminderMail
files:
  - path: billing/reminders.py
    note: picks which invoices qualify
  - billing/mail.py
try: |
  Create an invoice dated last month in the admin.
  Run `python manage.py remind_overdue` and check the outbox.
outcomes:
  sent:
    when: the invoice is overdue and no reminder went out this week
    then: one reminder mail is sent and logged on the invoice
  skipped:
    when: a reminder went out this week
    then: nothing happens
why:
  - One reminder a week keeps clients from marking us as spam.
checked: 2026-09-30
```

`checked` is for the record that is still right after the code under it changed. Nothing
else in the record needs editing then, and setting the date is the edit that says so. Roostr
does not compare the date with anything: what counts is that the record was changed, and
later committed, after the code was.

### Areas and groups

An **area** is a part of the project that the scan can tell apart from the rest, written by
the scan script into the fourth column of `entrypoints.tsv`: a Django app, a Rails controller
namespace, a folder. Areas come from the code. A **group** is written by hand into a process
record and says where the process belongs.

The overview lists every area with its entrypoint counts and puts each process under the
area equal to its `group`, compared exactly, same spelling and case. When they differ, both
still appear: an area with no process under it is listed with its counts and nothing below,
and a group that matches no area is listed as a heading of its own, with no counts. A process
without a group is listed under `other`, with the entrypoints that have no area. To make a
process sit under its area, write the group as the scan writes the area.

### What belongs in `entrypoints`

Every entrypoint the behaviour starts from or runs through: the request, command or schedule
that starts it, the jobs it enqueues, the mails it sends, the events it emits. An
entrypoint reference is the first two columns of an `entrypoints.tsv` row joined by one
space. The kind is one of the closed list below; the name is opaque to roostr. The overview
counts an entrypoint as described when at least one process lists it, so a job or a mail
that no process lists shows up as not described even when the process that sends it is.

## `.roostr/scan`

The project's own script that lists its entrypoints: an executable file, written in any
language. `roostr scan` is the only command that runs it, and no other command runs anything
from the repository. Roostr starts the file directly, with the repository root (the folder
`.roostr/` is in) as its working directory, reads its stdout, and passes whatever it writes
to stderr on to its own stderr. Output on stderr is never a problem by itself.

The script prints on stdout:

- first, a line `# covers: ` followed by the kinds it looks for, separated by spaces, in any
  order;
- then one line per entrypoint, in any order: three or four columns separated by a tab
  character: `kind`, `name`, `file`, and optionally `area`. A fourth column that is empty
  counts as left off.

```
# covers: mail http
http	POST /invoices	billing/views.py	Invoices
http	GET /invoices	billing/views.py	Invoices
mail	ReminderMail	billing/mail.py
```

`roostr scan` refuses the result, and leaves `entrypoints.tsv` exactly as it was (or absent),
when the script exits with anything but 0, or when its output is malformed:

- the first line is not a `# covers:` line;
- the `# covers:` line names a kind outside the closed list;
- a row has fewer than three or more than four columns;
- a row's kind is outside the closed list, or is not named on the `# covers:` line;
- a row's kind, name or file is empty;
- two rows have the same kind and name.

Every problem in the output is reported, each naming the line of the output it is on,
counted from 1: `line 3: ...`. For two rows with the same kind and name, that is the line of
the second, and the entrypoint is named as `<kind> <name>`.

Otherwise roostr writes `.roostr/entrypoints.tsv` in the format below, and nothing else.

## `.roostr/entrypoints.tsv`

Written by `roostr scan`, committed. It may be absent in a project that has never been
scanned.

- The first line is `# covers: ` followed by the kinds the scan handles, separated by
  single spaces, in the order of the closed list below whatever order the script named them
  in. A kind not listed here has not been scanned; a listed kind with no rows has been
  scanned and has none.
- Every other line is one entrypoint: three or four columns separated by a tab character:
  `kind`, `name`, `file`, and optionally `area`. Rows with three and four columns may be
  mixed in one file.
- Rows are sorted by their full line, byte order. The header is not part of the sort.
- No trailing tabs, one trailing newline at the end of the file. A row without an area has
  three columns, never an empty fourth.
- An entrypoint is identified by its kind and name together; no two rows share both.

```
# covers: http schedule mail
http	GET /invoices	billing/views.py	Invoices
http	POST /invoices	billing/views.py	Invoices
mail	ReminderMail	billing/mail.py
schedule	remind_overdue	billing/tasks.py	Collections
```

(The separators above are tab characters.)

## Entrypoint kinds

A closed list, in this order: `http`, `job`, `mail`, `schedule`, `event`, `cli`.

## What `roostr validate` checks

Every finding names the file. When it is about a key, it names the key as a dotted path from
the top of the file, with no trailing dot: `watch_out`, `outcomes.refunded`,
`operations.deploy.note`. An item of a list is one more step of the path: its position in
the list, counted from 1. The second `watch_out` item is `watch_out.2`, the third entrypoint
of a record is `entrypoints.3`, the note of a record's second file is `files.2.note`. No
square brackets, and never counted from 0.

- `project.yml` and every process record parse as YAML. A file that does not is a finding,
  and the other files are still checked.
- Required keys are present: `name` in `project.yml`; `name` and `description` in a record.
- Every key has the shape given in the tables above: a one-line string, a list or a map. A
  `why` given as one string, or an outcome given as a string instead of a map, is a finding.
- No unknown keys, at any level.
- `watch_out` has at most 7 items, each at most 160 characters. An operation `note` and an
  environment value are each at most 160 characters.
- `checked` is a date that exists, written `YYYY-MM-DD`. `yesterday` and `2026-02-30` are
  findings, and the finding gives the form `YYYY-MM-DD`.
- Every entrypoint reference in a record uses a kind from the closed list and matches a row
  of `entrypoints.tsv`. When records list entrypoints and `entrypoints.tsv` is missing, there
  is one finding, which says to run `roostr scan`.
- No record lists the same entrypoint twice. The finding names the repeat by its position
  (`entrypoints.3`) and gives the entrypoint. Two different records may list the same
  entrypoint; that is not a finding.
- Every `files` path exists in the repository.
- Every row of `entrypoints.tsv` has a kind from the closed list.

With findings, `validate` exits 1 and lists them on stdout; with none, it prints
`No problems found` and exits 0. It never writes a file.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | findings, or a refusal: `validate` found problems, the id given to `show` does not exist, or `scan` refused what its script did (a non-zero exit, malformed output) |
| 2 | could not run: unknown command, missing argument, no `.roostr/project.yml`, `show` asked for a record that cannot be read, `scan` found no `.roostr/scan` or one that is not executable, or `changes` was run outside a Git repository or with a `--since` value that is neither a number of days (`7d`) nor a date (`2026-10-03`) |

Normal output goes to stdout. Reasons for exit 1 and 2 go to stderr, except `validate`
findings, which are the command's output and go to stdout.

`roostr init` is the one command that runs without `.roostr/project.yml`: it creates it. Run
where the file already exists, it leaves the file alone and still exits 0.

`roostr changes` in a Git repository exits 0 also when nothing changed and when the
repository has no commits yet.

## What roostr reads from Git

The overview and `roostr changes` read the Git repository the project is in: the commits
reachable from `HEAD`, the files each touched, and what is uncommitted in the working tree.
They only read. No other command reads Git.

- A commit belongs to a **period** by its commit date. A period starts at 00:00:00 on its
  first day and runs until now. A day is a day in the local time zone, the one roostr runs
  under (`TZ`).
- `--since <when>` names the first day of a period: `2026-10-03` is that day, and `7d` is
  the day seven days before today. `--since=<when>` is the same. Without `--since`,
  `roostr changes` means `7d`.
- The entrypoints of "then" are `.roostr/entrypoints.tsv` as it was in the last commit made
  before the period started. The entrypoints of "now" are the file in the working tree,
  committed or not. When either is missing, entrypoints are not compared.

## What needs attention

A process needs attention when a file its record lists is **newer** than the record, or has
uncommitted changes.

- A listed file is newer than its record when a commit that touched the file is reachable
  from `HEAD` but not from the record's last commit, the last commit that touched the
  record. Commit dates play no part: two commits with the same date still have an order,
  and a commit made on a branch before the record's last commit is newer than the record
  once that branch is merged after it.
- A record that has never been committed is skipped. A record that itself has uncommitted
  changes counts as updated.

Two limits:

- Any change to a record counts as an update. roostr does not know whether the change dealt
  with the code change.
- roostr does not report whether `entrypoints.tsv` is current. Running `roostr scan` is the
  check.

## Environment

| Variable | Value | Meaning |
|---|---|---|
| `TZ` | a time zone, as the system reads it | the standard variable. It decides where a day starts. The tests set it to `UTC`. |
| `ROOSTR_TODAY` | a date, `YYYY-MM-DD` | roostr takes this as today's date instead of the machine's. It decides where the last 7 days and `--since 7d` start. |
