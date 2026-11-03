# Record 3: `roostr changes`, the two Git-based sections of the overview, and `checked`

Date: 2026-10-10 22:13. Adds one command, two overview sections and one record key to
records 1 to 2b, which all stand. Supersedes, in record 2b
(`2026-10-10-2159-catch-all-rows.md`), the line `init` prints for `AGENTS.md` and the
sentence above it. Closes what record 1 left for "record 3"; the `!` marker is withdrawn.

Status: scenario list confirmed by the owner on 2026-10-10 with the changes noted inline.
Every point marked **Proposed** was accepted as written; the marks are kept to show what came
from the brief and what from this record. Tests written for every scenario.

Changed on confirmation, all the owner's decisions:

- A process record gains the optional key `checked`, so that a NEEDS ATTENTION line can be
  cleared when the record is still right. Scenarios 136 to 139 are new; 121 and 127 now use
  it.
- The entrypoint rule of NEEDS ATTENTION is dropped in full, with scenarios 129 to 134.
- No `!` marker. NEEDS ATTENTION names the process.
- The line for `AGENTS.md` spells "behavior" in every sentence, and its last sentence
  mentions `checked`.

Amended on 2026-10-10 after an outside review, before implementation started; the owner's
decisions:

- "Newer" is defined by reachability, with scenario 140 for an ordinary merge.
- Scenario 141: three commits that touch one process are one changed process.
- A day is a day in the local time zone, stated as a decision; the tests pin `TZ=UTC`.
- Two limits of NEEDS ATTENTION are written down, here and in FORMATS.md.

## The problem this record serves

Record 1 named "what changed" as part of the picture the owner has lost after a week away.
Until now roostr read only the files. This record lets it read Git as well, for two things:
what changed in a period, and where a process record has probably fallen behind the code.
Roostr still writes nothing and runs no repository code for either.

## Decided

### The clock handle: `ROOSTR_TODAY`

- When the environment variable `ROOSTR_TODAY` holds a date written `YYYY-MM-DD`, roostr
  takes that as today's date. Without it, today is the machine's date.
- It is the only handle. It needs no time of day, because a period is counted in whole days
  (below), and it is an environment variable rather than an option so that it also reaches
  the overview, which takes no options.
- Common practice for pinning a clock from outside is an environment variable:
  `SOURCE_DATE_EPOCH` (reproducible builds), `GIT_TEST_DATE_NOW` (Git's own test suite),
  `FAKETIME` (libfaketime). Those carry a second; roostr needs a day.

### A period is whole days

**Proposed.** A period starts at 00:00 on a date and runs until now.

- `--since 2026-10-03` starts at 00:00 on 3 October.
- `--since 7d` starts at 00:00 on the day seven days before today. On 10 October that is the
  same period as `--since 2026-10-03`. The brief gave exactly this pair as its examples.
- A commit belongs to the period when its commit date is at or after that moment. The last
  second of 2 October is outside; 00:00:00 on 3 October is inside.
- A day is a day in the local time zone: the zone roostr runs under, which the standard
  `TZ` variable sets. "Since 3 October" means since midnight where the owner is, not
  midnight in UTC. The tests pin `TZ=UTC` on every `changes` and overview command, so that
  00:00 is the same moment on every machine; no test runs under another zone.

Why whole days and not "168 hours before this minute", which is what `docker logs --since 7d`
means: the two forms of `<when>` then name the same thing, the report can say which date it
started from, and the answer does not change between a morning and an afternoon run.
`journalctl --since 2026-10-03` and `docker logs --since 2026-10-03` both read a bare date as
00:00 local time, which is the rule taken here for both forms.

### `roostr changes`

- `roostr changes --since <when>`; `--since=<when>` is the same. `<when>` is a number of
  days (`7d`) or a date (`2026-10-03`). Without `--since` it means `7d`.
- Reads Git and the files. Writes nothing. Never runs `.roostr/scan`.
- Only commits reachable from `HEAD` count.

**Entrypoints.** "Then" is `.roostr/entrypoints.tsv` as committed when the period started:
its content in the last commit made before the start. "Now" is the file in the working tree,
committed or not, so a rescan shows up before it is committed. An entrypoint is its
`<kind> <name>`, as in `scan`: one in the file now and not then is added, one then and not
now is removed, and a row whose file or area changed is neither.

**Proposed**: when there is no "then" (no `entrypoints.tsv` was committed before the period
started) or no file now, entrypoints are **not compared**. Nothing is listed as added, and
the count line says `not compared`. This follows `scan`, whose first run prints `first scan`
instead of every entrypoint with a `+`: no earlier list is not the same as an empty one, and
in the week a project is set up the alternative is its whole list (93 lines on the trial project).

**Processes.** A process has changed when a commit in the period touched its record or a
file its record lists (as the record reads now). Under the process come the subjects of
those commits, newest first, each commit once however many of the process's files it
touched. A commit that touched files of two processes is given under both. A commit that
touched nothing any process lists is not in the report. Uncommitted edits are not commits
and are not in the report; they belong to NEEDS ATTENTION.

**The report**, on stdout, exit 0. **Proposed** layout:

```
CHANGES SINCE 2026-10-03

ENTRYPOINTS
  + http POST /waiting_list
  + job WaitingListJob
  - http GET /bookings/:id

PROCESSES
  book-workshop
    Offer a freed place to the next person waiting
    Add the waiting list
  cancel-booking
    Offer a freed place to the next person waiting
    Add the waiting list
  remind-attendees
    Send the reminder two days ahead

entrypoints: 2 added, 1 removed   processes: 3 changed
```

- The first line is `CHANGES SINCE <date>`, the date the period starts on, whichever form
  `--since` had. It is always printed.
- ENTRYPOINTS lists added entrypoints, then removed ones, each group in the file's sort
  order, exactly as `scan` reports them. Left out when nothing was added or removed, and
  when entrypoints are not compared.
- PROCESSES lists the changed processes by id, in alphabetical order, the id indented two
  and each subject indented four. Left out when no process changed.
- The last line is the count line and is always printed. It has two parts separated by three
  spaces: `entrypoints: <a> added, <r> removed`, or `entrypoints: not compared`; then
  `processes: <n> changed`. A number that is zero is printed as `0`: the list was compared
  and nothing was found, which is a count and not an absence. The label-first form needs no
  singular and plural.
- With nothing to report, the output is the heading, a blank line and the count line:
  `entrypoints: 0 added, 0 removed   processes: 0 changed`. The same as `scan`, whose totals
  line is always printed and is the only line when nothing changed.

**Could not run**, exit 2, reason on stderr, nothing on stdout, nothing written:

- Outside a Git repository: says that this is not a Git repository.
- `--since` with a value that is neither form (`yesterday`, `14`, a date that does not exist
  such as `2026-02-30`): names the value.
- `--since` with no value.
- No `.roostr/project.yml`: says to run `roostr init`, as every command does.

A Git repository that has no commits yet is not a refusal: there is nothing to compare and
nothing changed, so the count line reads `entrypoints: not compared   processes: 0 changed`.

### The overview: LAST 7 DAYS

```
LAST 7 DAYS
  entrypoints: 2 added, 1 removed   processes: 3 changed
  run roostr changes to see them
```

- The first line is the count line of `roostr changes --since 7d`, character for character.
  The second line is the pointer.
- Left out when nothing changed: no entrypoint added or removed (or not compared) and no
  process changed. Left out outside a Git repository and in a repository without commits.
- **Proposed** place: after ENVIRONMENTS and before NEEDS ATTENTION. What stays the same is
  at the top; what moves is at the bottom, where the eye is when the command returns.

### The overview: NEEDS ATTENTION

The last section. Two kinds of line, in this order: records that cannot be read (record 1,
unchanged), then processes.

```
NEEDS ATTENTION
  .roostr/processes/refund-booking.yml   cannot be read, run roostr validate
  book-workshop      app/models/booking.rb changed since this record was updated
  book-workshop      app/services/book_workshop.rb changed since this record was
                     updated
  remind-attendees   app/jobs/booking_reminder_job.rb changed since this record
                     was updated
```

**A process** is listed when a file it lists

1. has a commit newer than the last commit of its record, or
2. has uncommitted changes (edited in the working tree, staged or not).

- One line per file: the process id, then `<file> changed since this record was updated`.
  Lines are sorted by id, and within a process follow the order of its `files`.
  (**Proposed**; the brief gave the wording, not the arrangement.)
- The id column is as wide as the longest id among these lines plus three spaces, after the
  indent of two. The sentence wraps at 80 under its own column, the rule process
  descriptions already follow in FUNCTIONALITY. A path is never broken.
- A file that two processes list names both.
- A record committed in the same commit as the file, or in a later one, is up to date.
- A record that has never been committed is skipped.

**"Newer" means later in history, not a later date.** Made precise in the amendment: a
listed file is newer than its record when a commit that touched the file is reachable from
`HEAD` but not from the record's last commit (the last commit that touched the record).
Dates are not compared. After a rebase every commit carries the same commit date to the
second, and a clock set wrong on one machine puts dates out of order; reachability is right
in both cases. It is also right for a merge: a file changed on a branch, even on a date
before the record's last commit, is newer than the record once the branch is merged after
it, because the record's last commit never contained that change. The first wording, "comes
after the record's last commit on the way to `HEAD`", left that case open. NEEDS ATTENTION
does not depend on the clock at all: only LAST 7 DAYS and `changes` do.

**Proposed: a record being edited counts as updated.** A record that itself has uncommitted
changes is not listed. Without this, an agent that follows the new `AGENTS.md` sentence
(changes `booking.rb`, then updates the record) still sees "booking.rb changed since this
record was updated" until it commits, and cannot tell that its update was seen.

NEEDS ATTENTION never changes the exit code. The overview exits 0 with it.

**Two limits**, stated in the amendment so that nobody reads more into the section than it
holds:

- Any change to a record counts as an update. Roostr does not know whether the change dealt
  with the code change.
- Roostr does not report whether `entrypoints.tsv` is current. Running `roostr scan` is the
  check.

**No entrypoint rule.** The brief had a second kind of line, `entrypoints may be out of
date: run roostr scan`, for a file named in `entrypoints.tsv` with a commit newer than
`entrypoints.tsv`. Dropped on confirmation: it would fire on every commit to any file that
holds an entrypoint, and a rescan that finds nothing new leaves `entrypoints.tsv` as it was,
so nothing could clear it. Scenario 125 now guards the absence: a newer commit to a file
that `entrypoints.tsv` names, and no process lists, adds nothing.

### `checked`, a new key of a process record

Added on confirmation. A process line could not be cleared either when the answer was
"nothing needs changing": after a refactor that leaves the behaviour alone there is nothing
in the record to edit, so nothing to commit.

- `checked` is optional and holds a date written `YYYY-MM-DD`. It means: someone compared
  the record with the code on that day and it still holds.
- Roostr does not compare the date with anything. Setting it is an edit to the record, so
  the process is no longer listed while that edit is uncommitted (the rule above) and stays
  unlisted once it is committed (the record's last commit is then the newer one).
- `validate` reports a `checked` that is not a date as a finding naming the record and the
  key. A day that does not exist, `2026-02-30`, is not a date. **Proposed**: the finding
  gives the form `YYYY-MM-DD`. An agent in a real project learns about `checked` from the
  one line in `AGENTS.md` and has no FORMATS.md; the finding is where it learns how the date
  is written.
- `roostr show` prints it on a line of its own when the record has it. **Proposed** place
  and form: the last line of the label block, after `try`, with the label `checked` in the
  same nine-wide label column: `  checked  2026-10-08`. Left out when the record has none.

### `init`

The line for `AGENTS.md` gains a sentence. It stays one unbroken line:

```
When you need to understand this project's functionality, structure, or recent changes, run `roostr` for the project overview. Use `roostr show <id>` to inspect a specific process before modifying its behavior. After changing behavior a process describes, update its record, or set its `checked` date if the record still holds; after adding or removing an entrypoint, run `roostr scan`.
```

**Proposed**, not asked for: the sentence above the line says why the line is there, and the
line now has a second purpose. It reads:

```
The owner adds this line to AGENTS.md, so that agents read the overview before
they work on the project and keep it current afterwards:
```

Everything else `init` prints is as in record 2b. The setup instruction and the starter
`project.yml` do not mention `checked`: a record that is being written for the first time
has nothing to check against.

## Raised with the owner, and how it was resolved

1. **A NEEDS ATTENTION line that cannot be cleared.** Both rules of the brief cleared only
   when the record, or `entrypoints.tsv`, got a new commit, and "nothing needs changing"
   gives nothing to commit. Resolved: `checked` for process records; the entrypoint rule
   dropped.
2. **Uncommitted changes counted for a process and not for the entrypoint list.** No longer
   applies.
3. **The `!` marker** that record 1 kept columns 78 to 80 for. Resolved: there is none.
4. **Spelling.** The line for `AGENTS.md` had "behavior" and the new sentence "behaviour".
   Resolved: "behavior" in both.

## The fixture: Atelier after the week of the waiting list

The two frozen screens use Atelier under Git with this history. Today is 2026-10-10, so the
last 7 days start on 2026-10-03.

| Date | Subject | Touches |
|---|---|---|
| 2026-01-15 | Initial state | the whole fixture |
| 2026-09-24 | Round prices to whole euros | `app/services/book_workshop.rb` |
| 2026-10-05 | Add the waiting list | `app/models/booking.rb`, two new files (`waiting_list_controller.rb`, `waiting_list_job.rb`), `entrypoints.tsv` (+ `http POST /waiting_list`, + `job WaitingListJob`, both area Bookings) |
| 2026-10-06 | Drop the booking page | `app/controllers/bookings_controller.rb`, `entrypoints.tsv` (− `http GET /bookings/:id`) |
| 2026-10-07 | Offer a freed place to the next person waiting | `app/models/booking.rb`, `.roostr/processes/cancel-booking.yml` |
| 2026-10-08 | Send the reminder two days ahead | `app/jobs/booking_reminder_job.rb` |
| 2026-10-09 | Show the trainer on the workshop page | `app/controllers/workshops_controller.rb` |

Nothing is uncommitted. What follows from it:

- `changes` is the report laid out above. The September commit is outside the period, the
  9 October commit touches nothing a process lists.
- `book-workshop` is behind on both its files (September and 5/7 October).
  `cancel-booking` is up to date: its record was committed with the last change to
  `booking.rb`. `remind-attendees` is behind on the job.
- FUNCTIONALITY reads `8 entrypoints, 5 described`, and Bookings reads `http 3   job 1`.

## Scenario list

Numbering continues from record 2. Unless a scenario says otherwise it starts from Atelier
committed once on 2026-01-15, with today pinned to 2026-10-10.

### changes: what it reports

96. In Atelier after the week of the waiting list, `roostr changes` prints the report
    exactly as laid out above, and writes nothing.
97. The `entrypoints.tsv` in the folder, committed or not, is compared with the one
    committed before the period: an entrypoint a rescan added is listed with `+`, one it no
    longer found with `-`.
98. An entrypoint that was already in the `entrypoints.tsv` committed before the period
    started is not listed as added.
99. An entrypoint whose file changed in the period is neither added nor removed.
100. When no `entrypoints.tsv` was committed before the period started, no entrypoint is
     listed as added and the count line reads `entrypoints: not compared`. (**Proposed**)
101. A process whose record was committed in the period is listed with that commit's
     subject.
102. A process is listed with the subject of a commit that touched a file it lists, and a
     process that does not list that file is not.
103. A commit that touched both a record and a file it lists is given once under that
     process.
104. A commit that touched only files no process lists is not reported, and the count line
     reads `processes: 0 changed`.
105. When nothing changed in the period, the report is the heading and
     `entrypoints: 0 added, 0 removed   processes: 0 changed`.
106. In a Git repository with no commits yet, `changes` exits 0 and the count line reads
     `entrypoints: not compared   processes: 0 changed`.

### changes: the period

107. With today pinned to 2026-10-10, the heading names the first day of the period:
     no `--since` gives `CHANGES SINCE 2026-10-03`, `--since 7d` the same, `--since 3d`
     gives `2026-10-07`, `--since 2026-09-21` gives `2026-09-21`; and `--since=3d`, written
     with an equals sign, gives `2026-10-07` (a test of its own).
108. `ROOSTR_TODAY` sets today: with `ROOSTR_TODAY=2026-11-02` the last 7 days start on
     `2026-10-26`.
109. A commit made at 00:00:00 on the first day of the period is reported.
110. A commit made at 23:59:59 on the day before is not.

### changes: could not run (exit 2, nothing written)

111. Outside a Git repository, `changes` says this is not a Git repository.
112. `--since yesterday`, `--since 14` and `--since 2026-02-30` are each refused, naming the
     value. (The list shown for confirmation had `7`; a message that offers `7d` as an
     example would have named it by accident.)
113. `--since` with no value is refused.
114. Without `.roostr/project.yml`, `changes` says to run `roostr init`.
115. `roostr changes` never runs `.roostr/scan`.

### The overview: LAST 7 DAYS

116. In Atelier after the week of the waiting list, the overview prints LAST 7 DAYS and
     NEEDS ATTENTION exactly as laid out, after ENVIRONMENTS, and writes nothing.
117. When the newest change was committed at 23:59:59 on the day before the 7 days began,
     LAST 7 DAYS is left out.
118. In a Git repository with no commits yet, the overview is printed with no LAST 7 DAYS
     and no NEEDS ATTENTION.

Unchanged from record 1a: outside a Git repository LAST 7 DAYS is left out.

### The overview: a process needs attention

119. When a file was committed after the records that list it, each process that lists it
     is named with `<file> changed since this record was updated`, and a process that does
     not list it is not named.
120. A record committed in the same commit as its file is not listed.
121. Once a record is committed after its file, here with a new `checked` date, the process
     is no longer listed. (Reworded on confirmation to use `checked`.)
122. Newer means later in history: a file committed after its record with the very same
     commit date is still listed. (**Proposed**)
123. A listed file with an uncommitted edit names its process.
124. A staged edit still names it.
125. A newer commit to a file that no process lists adds nothing, even when
     `entrypoints.tsv` names the file: NEEDS ATTENTION is left out.
126. A record that has never been committed is skipped, while a committed record that lists
     the same changed file is named.
127. A record with an uncommitted edit of its own, here a new `checked` date, counts as
     updated and is not listed. (**Proposed**; reworded on confirmation to use `checked`.)
128. A record that cannot be read is listed before the processes that need attention.

129 to 134, the entrypoint list needing attention: dropped on confirmation with the rule.

### init

135. The line `init` prints for `AGENTS.md` ends with the sentence about keeping the record
     and the entrypoints current, and the sentence above it says why (alters scenarios 88
     and 91).

### `checked` (added on confirmation)

136. `roostr show` prints a record's `checked` date on a line of its own after `try`,
     labelled `checked`.
137. A record without `checked` has no such line.
138. `validate` accepts `checked: 2026-10-08`.
139. `checked: yesterday` and `checked: 2026-02-30` are each a finding naming the record,
     the key `checked` and the form `YYYY-MM-DD`.

### Added in the amendment

140. A file changed on a branch that was merged after the record's last commit is newer
     than the record, and the process is listed. In the test the branch commit is dated a
     day before the record's last commit, so neither dates nor "the file's last commit
     descends from the record's" give the answer; only reachability does.
141. Three commits in the period that touch one process give `processes: 1 changed`.

## Existing tests this record alters

- `test/init/init_test.rb`, scenarios 88 and 91: both compare the whole output of `init`
  with `test/expected/init_guidance.txt`, which has the longer line and the changed
  sentence above it. Both passed before this record and fail until `init` prints the new
  text.
- `test/scan/only_scan_runs_code_test.rb` gains the test for scenario 115,
  `test/show/show_process_test.rb` those for 136 and 137, `test/validate/validate_test.rb`
  those for 138 and 139.
- No other test changes meaning. The overview helper pins `ROOSTR_TODAY` and `TZ` for every
  overview test, so their transcripts start with `ROOSTR_TODAY=2026-10-10 TZ=UTC`; what they
  assert is the same.

## What the program prints today and these tests change

Seen when the new tests were first run against the implementation of record 2b. No earlier
test pinned any of the three.

- **Two kinds on an area line.** Record 1a's layout separates them by three spaces
  (`http 3   job 1`). No fixture had an area with two kinds until this record; the program
  prints two spaces. The frozen screen of scenario 116 pins three.
- **Where NEEDS ATTENTION sits.** Record 1 puts it last. The program prints it between
  FUNCTIONALITY and WORK WITH THIS PROJECT. Scenario 116 pins it last, after LAST 7 DAYS.
- **A record with a key roostr does not know.** The program lists such a record under NEEDS
  ATTENTION as unreadable, and `show` refuses it. Nothing here settles whether that is
  right for an unknown key in general; `checked` is now a known key.

## Tests that pass before any work is done

Five, because they guard an absence that already holds: a record committed with its file is
not listed (120), a newer commit to a file no process lists adds nothing (125), LAST 7 DAYS
is left out when nothing changed in the 7 days (117), the overview in a repository without
commits (118), and no `checked` line without a `checked` key (137).

## Test kit

- `commit "Add the waiting list", on: "2026-10-05"` gives the commit that date (noon UTC);
  `on:` also takes a full timestamp, `"2026-10-03T00:00:00+00:00"`, for the edge tests.
  Without `on:` a commit keeps the kit's fixed date, 2026-01-15.
- `changes` and `overview` run with `ROOSTR_TODAY=2026-10-10` unless a test says
  `today: "..."`, and with `TZ=UTC` so that "00:00" means the same on every machine. Both
  are shown on the command line of a failure's transcript.
- `edit_file path` changes a file's content, for a commit or an uncommitted edit.
  `start_repository` turns the test's folder into a Git repository with one commit, for
  tests that have to arrange the folder before the first commit.
- A failure in a Git workspace shows the repository under the transcript: every commit with
  its date, subject and files, newest first, and what is uncommitted. Without it the
  implementer could not tell a boundary test from any other.
- `branch "longer-reminders"`, `switch_to "main"` and `merge "longer-reminders", on: "..."`
  make a side branch and merge it back with a merge commit. When the history has a merge, a
  failure also shows it as `git log --graph` draws it, because dates do not show which
  commit follows which.
- `atelier_under_git` is Atelier committed once, and adds to a failure which process lists
  which file. `atelier_after_the_waiting_list_week` builds the history in the table above.
- The kit's private word for "what the last command did to the files" was `changes`; it is
  now `file_changes`, so that `changes` can mean the command.
- Fixtures: `entrypoints/waiting-list-added.tsv`, `entrypoints/booking-page-dropped.tsv`,
  `entrypoints/booking-moved.tsv`, `processes/remind-attendees-checked.yml`. Expected:
  `changes_atelier.txt`, `overview_atelier_after_the_week.txt`.

## FORMATS.md

Updated, to be copied into `app/` again: the `checked` key and what `validate` checks about
it; a section on what roostr reads from Git (a period, `--since`, the entrypoints of then
and now); `ROOSTR_TODAY` and `TZ`; `changes` in the exit codes. With the amendment: a
section "What needs attention" with the reachability definition of "newer" and the two
limits, and the sentence that a day is a day in the local time zone. The layouts stay out
of it.

## Left out of this record

- `roostr show` saying that its record is behind.
- Comparing the `checked` date with anything: with today, with commit dates, with the date
  it had before. A `checked` date in the future. A quoted date.
- Dates, hashes and authors of commits in `changes`; a subject longer than 76 characters;
  renamed files (a rename is a removal and an addition); branches other than
  the one checked out.
- Which of a commit's two dates counts. `git log --since` uses the commit date and the
  record says the same; the tests give both dates the same value.
- A test under a time zone other than UTC. A day is a local day by decision; every test
  runs under `TZ=UTC`, so no test tells local from UTC.
- Merges in `changes`: whether a commit that came in through a merge inside the period, but
  was made before it, belongs to the period. A merge with conflicts, and a merge commit
  that itself changes a listed file.
- A value of `ROOSTR_TODAY` that is not a date. `--since 0d`, a date in the future, `--since`
  given twice, other options and extra arguments.
- A process that was deleted in the period, and a process whose record cannot be read, in
  `changes`. A `project.yml` that does not parse, for `changes`.
- A listed file that Git does not track, that is ignored, or that was deleted. An
  `entrypoints.tsv` deleted in the period.
- A `.roostr/` folder that is not at the top of its Git repository, and running from a
  subfolder (carried over from record 2).
- Git not being installed.

## Still open

- Whether the overview should treat a record with an unknown key as unreadable (see "What
  the program prints today").
- Closed in the amendment: nothing tells the owner that `entrypoints.tsv` is behind.
  Running `roostr scan` is the check.
- The label `other` (carried over from record 1a).
