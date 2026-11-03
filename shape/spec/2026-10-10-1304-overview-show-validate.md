# Record 1: `roostr`, `roostr show <id>`, `roostr validate`

Date: 2026-10-10. First spec record of the project. Supersedes the command proposal
discussed on 2026-10-09 (init, new, list, show, validate, status, review, scan, context,
instructions) and an earlier design document in full.

Status: scenario list confirmed on 2026-10-10 with four changes, noted inline. Tests written
for every scenario except 1 and 14. The screen layout below is provisional until a trial on a
real project comes back; scenarios 1 and 14, which freeze whole screens, wait for that.

## The problem this record serves

Coming back to a project after a week, the owner has lost the picture: what it does, what it
exposes, how to run and deploy it, how to try a feature, what changed. Roostr keeps that
picture in committed files under `.roostr/` and prints it instantly. Agents read it through
one line in `AGENTS.md` that tells them to run the same two commands, and agents do the
upkeep. Roostr contains no model and runs no hooks.

This record covers the three read commands. They read the files, never Git, and never run
repository code. They write nothing.

## Decided

### Names and files

- Command `roostr`, folder `.roostr/`. The folder's marker is `.roostr/project.yml`.
- `.roostr/project.yml`: `name`, `stack` (one line), `operations` (map of operation name to
  command, in file order), `environments` (map of environment name to where it runs, in
  file order). `name` is required; the rest are optional.
- `.roostr/entrypoints.tsv`: written by `scan` (record 2), committed. First line
  `# covers: <kinds separated by spaces>`, then one line per entrypoint, three tab-separated
  columns `kind`, `name`, `file`, sorted by line. The header is not part of the sort.
- `.roostr/processes/<id>.yml`: one file per process. **The file name is the id**; there is
  no `id` key. Keys: `name`, `description`, `group`, `entrypoints` (list), `files` (list),
  `try` (free text, never run), `outcomes`, `why`. Required: `name`, `description`.
  Everything else is optional and is validated when present. (The direction document marked
  only `outcomes` and `why` as optional; this record relaxes that so that a process without
  an entrypoint, without files, or not yet tried can still be recorded. See "Pushed back".)
- Entrypoint kinds are a closed list: `http`, `job`, `mail`, `schedule`, `event`, `cli`.
- A process refers to an entrypoint as the string `<kind> <name>`, exactly the first two
  columns of its line in `entrypoints.tsv` joined by one space, for example
  `http POST /bookings` or `job BookingReminderJob`. Roostr treats the name as opaque.
- `outcomes` is a map from outcome name to `when` (the condition) and `then` (what happens),
  both one line:

  ```yaml
  outcomes:
    confirmed:
      when: payment succeeds
      then: booking saved, confirmation mail sent
    full:
      when: no places left
      then: booking refused, nothing saved
  ```

### Behaviour common to the three commands

- Plain text, 80 columns, no colour, no `--json`, no width detection.
- Exit 0 success, 1 findings or refusal (including an id that does not exist), 2 could not
  run (unknown command, missing argument, no `.roostr/project.yml`).
- Without `.roostr/project.yml` every command exits 2 and says to run `roostr init`.
- A section with nothing in it is left out. Absence is never shown as zero: a kind the scan
  does not cover reads `not scanned`, and a project that has never been scanned reads
  `not scanned yet`.
- None of the three commands writes a file, ever.
- A process record that cannot be read (bad YAML, or missing a required key) never stops
  the overview: `roostr` prints everything that reads, lists the file under NEEDS ATTENTION
  with `run roostr validate`, and exits 0. `roostr show` on that record exits 2 naming the
  file and pointing at `roostr validate`. `roostr validate` reports it as a finding and
  keeps checking the other records.

### The overview, `roostr`

Sections in this order, separated by one blank line. LAST 7 DAYS and the `!` marker on
process lines belong to record 3 and are never shown in this record. NEEDS ATTENTION appears
in this record only when a process record cannot be read; its Git-based lines come in
record 3.

```
<name>                                                      <stack, right-aligned to 80>

FUNCTIONALITY
  <Group>
    <process-id>      <description, one line>

  <n> of <total> entrypoints are not described by a process

ENTRYPOINTS
  http <n>   job <n>   mail <n>   not scanned: <kinds not covered>

WORK WITH THIS PROJECT
  <operation>     <command>
  <environment>   <where it runs>

NEEDS ATTENTION
  <file>   cannot be read, run roostr validate
```

Rules:

- The stack on the heading line is right-aligned so that it ends at column 80.
- Groups are listed in alphabetical order, processes within a group by id. Processes without
  a `group` are listed first, directly under FUNCTIONALITY, with no group heading, at the
  same indent as grouped processes so the id column lines up.
- The id column is as wide as the longest id shown plus three spaces. The description is its
  first line, cut so that the line does not pass column 77 (columns 78 to 80 are reserved
  for the record 3 marker); a cut description ends in `...`.
- The "not described" line counts entrypoints in `entrypoints.tsv` that no process lists.
  It is shown whenever `entrypoints.tsv` exists, even when there are no processes. With no
  processes and no `entrypoints.tsv`, FUNCTIONALITY is left out.
- ENTRYPOINTS shows one `<kind> <count>` per covered kind in the closed list's order, then
  `not scanned: <kinds>` for the rest. A covered kind with no entrypoints shows `0`. Without
  `entrypoints.tsv` the section is one line: `not scanned yet`.
- WORK WITH THIS PROJECT lists operations in file order, then environments in file order.
  The label column is as wide as the longest label plus four spaces. Left out when both are
  empty.
- NEEDS ATTENTION lists each process file that cannot be read, one per line, ending in
  `run roostr validate`. Left out when every record reads.

### One process, `roostr show <id>`

```
<id>   <name>                                                <Group, right-aligned>
<description, wrapped at 80>

  entry    <kind> <name>                     <file from entrypoints.tsv>
  code     <first file>
           <next file>
  try      <free text, continuation lines aligned>

OUTCOMES
  <outcome>   <when>: <then>

WHY
  <why, wrapped at 80>
```

Rules:

- No `group`: the heading line is just id and name. Each of `entry`, `code`, `try`,
  OUTCOMES and WHY is left out when the record has nothing for it.
- One `entry` line per entrypoint, in record order; the entrypoint column is as wide as
  the longest entrypoint shown plus two spaces. The file comes from `entrypoints.tsv`. An
  entrypoint that is not there shows `not in entrypoints.tsv` where the file would be. A line
  is allowed to pass column 80 rather than cut a file path.
- Outcome names are padded to the longest plus three spaces.
- An id that does not exist: exit 1, stderr names the id. No id: exit 2.

### `roostr validate`

Checks, in this order, and reports every finding as one line `<file>: <problem>` on stdout:

1. `project.yml` parses and has `name`.
2. Every `processes/*.yml` parses and has `name` and `description`.
3. Every entry in `files` exists in the repository (relative to the repository root).
4. `entrypoints.tsv`, when present, is well formed: a `# covers:` first line naming only
   kinds from the closed list, then rows of exactly three columns whose kind is in the list.
5. Every entry in `entrypoints` is `<kind> <name>` with a kind from the closed list and
   appears in `entrypoints.tsv`. When a record lists entrypoints and `entrypoints.tsv` does
   not exist, that is one finding for the whole run: the entrypoints cannot be checked, run
   `roostr scan`.

Exit 1 with one or more findings, exit 0 with the line `No problems found.` A broken record
does not stop the other checks.

## The fixture these scenarios use

A small workshop-booking Rails app called Atelier, chosen so nothing in the tests can be
mistaken for the voucher shop used as an example in `TESTING.md`. It lives in
`test/fixtures/workspaces/atelier/`.

- `project.yml`: name Atelier; stack `Rails 8, PostgreSQL, Solid Queue`; operations
  `start: bin/dev`, `test: bin/rails test`, `deploy: bin/kamal deploy`; environments
  `staging: staging.atelier.example (Hetzner)`, `production: atelier.example (Hetzner)`.
- `entrypoints.tsv` covering `http job mail`, seven entrypoints: four http
  (`DELETE /bookings/:id`, `GET /bookings/:id`, `GET /workshops`, `POST /bookings`), one
  job (`BookingReminderJob`), two mail (`BookingMailer#confirmation`,
  `BookingMailer#reminder`).
- Three processes: `book-workshop` and `cancel-booking` in group Bookings,
  `remind-attendees` in group Notifications. Together they describe five of the seven
  entrypoints; the two GET routes are not described.
- `book-workshop` is the full record: two entrypoints, two files, a `try`, two outcomes and
  a `why`. `cancel-booking` has no try, no outcomes and no why. `remind-attendees` has a
  `try` but no outcomes and no why.
- The referenced source files exist in the fixture as small placeholder files.
- Loose fixture records used by single scenarios, in `test/fixtures/processes/`:
  `import-workshops` (no group), `sync-calendar` (a description of about 130 characters),
  `refund-booking` (lists `http POST /refunds`, which the scan did not find),
  `export-bookings` (lists a file that does not exist), `receive-payment` (an entrypoint of
  kind `webhook`). In `test/fixtures/entrypoints/`: a copy of the TSV whose header also
  covers `schedule`, and one with an added row of kind `webhook`.

Variants for refusals change one thing in a copy of this workspace, so each breaks exactly
one rule.

## Scenario list

### The overview

1. In Atelier, `roostr` prints the overview exactly as laid out, and writes nothing.
   **Waits for the trial.**
2. The overview says how many entrypoints no process describes: 2 of 7.
3. A kind the scan covers but found nothing of is shown as 0.
4. A kind the scan does not cover is shown as not scanned, never as 0.
5. A process without a group is listed before the groups without a heading.
6. A description too long for its column is cut so the line fits.
7. With entrypoints but no processes, FUNCTIONALITY has only the "not described" line.
8. With no `entrypoints.tsv`, ENTRYPOINTS reads "not scanned yet" and the "not described"
   line is left out.
9. With no operations and no environments, WORK WITH THIS PROJECT is left out.
10. A project with only `project.yml` prints the heading and ENTRYPOINTS "not scanned yet",
    nothing else. (Changed on confirmation: the draft said "nothing else", contradicting 8.)
11. In a folder that is not a Git repository, with every record readable, LAST 7 DAYS and
    NEEDS ATTENTION are left out.
12. A process record that does not parse does not stop the overview: the rest is printed,
    the file is listed under NEEDS ATTENTION with "run roostr validate", exit 0. (Changed on
    confirmation: the draft made the overview exit 2.)
13. Without `.roostr/project.yml`, `roostr` exits 2, says to run `roostr init`, and writes
    nothing.

### One process

14. `roostr show book-workshop` prints the record exactly as laid out, with two entry lines,
    two code lines, the try line, outcomes and why. **Waits for the trial.**
15. A process with no try, no outcomes and no why leaves all three out.
16. A process without a group has a heading of id and name only.
17. An entrypoint missing from `entrypoints.tsv` is shown as "not in entrypoints.tsv".
18. A long description is wrapped at 80 columns.
19. An id that does not exist exits 1 and names the id.
20. `roostr show` with no id exits 2.
33. `roostr show` on a record that does not parse exits 2, names the file and points at
    `roostr validate`. (Added on confirmation, the other half of 12.)

### Validate

21. Atelier validates with exit 0 and "No problems found".
22. Dropped on confirmation: the file name is the id, there is no `id` key to check.
23. A listed file that does not exist is a finding naming the record and the file.
24. A listed entrypoint missing from `entrypoints.tsv` is a finding naming it.
25. An entrypoint with a kind outside the closed list is a finding.
26. A record that does not parse is a finding, and the other records are still checked.
27. A record without a description is a finding.
28. Records that list entrypoints while `entrypoints.tsv` is missing give one finding that
    says to run `roostr scan`.
29. An `entrypoints.tsv` row with an unknown kind is a finding.
30. Two broken records give two findings in one run.
31. Validate writes nothing, whether or not it finds problems.
32. An unknown command exits 2.

## Pushed back on, and how it was resolved

- **ENTRYPOINTS with no scan.** The direction says a section with nothing in it is left out
  and also that absence is never shown as zero. Leaving ENTRYPOINTS out entirely in an
  unscanned project makes it look like a project without entrypoints. Resolved: the section
  stays and reads `not scanned yet`.
- **Required keys.** The direction implies `group`, `entrypoints`, `files` and `try` are
  required. That would make an internal process, or one not yet tried, fail validation and
  there is no `new` command to produce a draft. Resolved: only `name`, `description` are
  required.
- **80 columns versus real content.** The ENTRYPOINTS line with three uncovered kinds
  written as `<kind>: not scanned` each is 89 characters. Resolved: one trailing
  `not scanned: <kinds>`. In `show`, a Rails controller path after a padded entrypoint
  column passes column 80 on realistic data. Resolved: a path is never cut; the line may
  overflow.
- **Entrypoint references.** The direction does not say how a process names an entrypoint.
  Resolved: the string `<kind> <name>`.
- **Outcome shape.** Resolved: `when` and `then`.
- **Broken records in read commands.** Not covered by the direction. First resolved as exit
  2; changed on confirmation to: the overview carries on and lists the file under NEEDS
  ATTENTION, `show` exits 2.
- **`# covers:` header.** "Sorted output of the scan script" cannot include a header in the
  sort. Resolved: header first, rows sorted after it.
- **`scan covers:` in the ENTRYPOINTS heading** duplicated the counts on the line below.
  Dropped on confirmation.
- **`id` key in process files** duplicated the file name. Dropped on confirmation.

## Left out of this record

Everything Git: the `!` marker, LAST 7 DAYS, the Git-based lines of NEEDS ATTENTION
(record 3). `scan` and `init` (record 2). Id character rules. Files in `processes/` that do
not end in `.yml`. Multi-line `description`: only the first line is used in the overview and
nothing is checked.

## Still open

- Whether the layout survives the trial on a real project; scenarios 1 and 14 wait for that.
- How `try` should look when it spans several lines; proposed as aligned continuation lines.
- Whether a record missing `name` or `description` counts as unreadable in the overview (as
  decided here) or is listed with what it has.
