# Roostr file formats

Everything roostr reads lives under `.roostr/` in the repository root. The marker of an
initialised project is `.roostr/project.yml`. All YAML files are plain YAML 1.2 mappings.
Paths are relative to the repository root. The examples below describe an imaginary
invoicing app called Ledgerline.

How roostr reads every YAML file, before the rules of each file below:

- **A byte order mark is ignored.** A file that starts with the UTF-8 byte order mark (the
  bytes `EF BB BF`, which some Windows editors write) is read as if it did not. The same
  holds for `entrypoints.tsv`, though `roostr scan` never writes one, so `check` still finds
  such a file different from what the scan writes.
- **An empty value is an absent value.** A key with nothing after it (`group:`, which YAML
  reads as null) or with the empty string (`group: ""`) is treated exactly as if it were
  not written, by every command, in text and in JSON: an optional key is not there (`null`
  in JSON, or `[]` for a list), and a required one is missing, which is a finding. This is
  what uncommenting only the first line of an example in the file `roostr init` writes
  gives (`watch_out:` with its items still commented out). The exceptions are an
  operation's command and an environment's value: see `project.yml` below.
- **A key written twice is a finding.** YAML allows a key once in each mapping. A key
  written twice, at any level, is a `validate` finding on the key's path that names the
  lines of both. The file is still read, and the value written last counts: an agent that
  appends `checked: 2026-10-08` to a record that already has a `checked` meant the new one.
- **Names are taken as written.** The names of outcomes, operations and environments, the
  keys of those maps, are strings, whatever YAML would make of them alone: `200:`,
  `8080:`, `1:`, `1.0:` and `true:` are the names `"200"`, `"8080"`, `"1"`, `"1.0"` and
  `"true"`, in text and in JSON.

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

A command is one line. A command of several lines (a `|` block with two lines or more) is a
`validate` finding: put a procedure of several steps in a script, and name the script here.
A `|` block of one line is one line; the newline at its end does not count, and is not
shown. An operation with no command (`deploy:` with nothing after it, or a `command:` with
nothing after it) is a finding, and so is an environment with nothing after its name. None
of these makes `project.yml` unreadable: the overview shows a command of several lines as
its first line followed by ` ...`, and leaves out an operation with no command and an
environment with no value. `roostr --json` gives the whole command, every line and the
final newline, and leaves out the same operations and environments.

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
key. Ids are referred to on the command line as written in the file name. A file whose name
starts with `.` is not a record, for any command, whatever it holds: editors keep such files
beside the one they have open (Emacs's lock `.#cancel-booking.yml`, a symbolic link to
nothing, among them). Nor is a file in a folder whose name starts with `.`.

Only a file named `<id>.yml` directly in `.roostr/processes/` is a record. Any other file
there, or in a folder below it, whose name ends in `.yml` or `.yaml` in any letter case
(`refund-booking.yaml`, `Refund.YML`, `bookings/refund-booking.yml`) is not read, by any
command; `validate` reports it, so that a record saved under the wrong name does not go
unnoticed. Other files (a README, notes) are ignored.

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
| `checked` | no | a date written `YYYY-MM-DD`, or a date and time written `YYYY-MM-DD HH:MM` (one space, local time, 24-hour, two digits each): when someone last compared the record with the code and found that it still holds |

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
later committed, after the code was. The time is there for a record confirmed twice on one
day: the second confirmation must change the file too. `2026-10-10 14:30` is a plain string
in YAML 1.2, and in YAML 1.1 as well, whose timestamps need seconds; that is why the form
has none.

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

A group that matches no area but would match one if case, spaces, `-` and `_` were
ignored (`Gift cards` for `GiftCards`, `bookings` for `Bookings`) is almost always a slip:
`roostr validate` reports it, naming the area as the scan writes it. A group that resembles
no area is allowed. Without `entrypoints.tsv` there are no areas, and no group is reported.

### What belongs in `entrypoints`

Every entrypoint the behaviour starts from or runs through: the request, command or schedule
that starts it, the jobs it enqueues, the mails it sends, the events it emits. An
entrypoint reference is the first two columns of an `entrypoints.tsv` row joined by one
space. The kind is one of the closed list below; the name is opaque to roostr. The overview
counts an entrypoint as described when at least one process lists it, so a job or a mail
that no process lists shows up as not described even when the process that sends it is.

## `.roostr/scan`

The project's own script that lists its entrypoints: an executable file, written in any
language. `roostr scan` and `roostr check` are the only commands that run it, and no other
command runs anything from the repository. Roostr starts the file directly, with the
repository root (the folder `.roostr/` is in) as its working directory, wherever the command
was started, and reads its stdout. `roostr scan` passes whatever the script writes to stderr
on to its own stderr; `roostr check` shows it only when the script fails (see "What
`roostr check` reports"). Output on stderr is never a problem by itself.

A script that is executable can still fail to start: its first line, the shebang line
(`#!/bin/sh`), names an interpreter that does not exist or cannot be run. Then `roostr scan`
exits 2, and `roostr check` reports it as a problem. The reason says that the shebang line
names an interpreter that cannot be started, and names it. When that line ends in a carriage
return, the reason also says that the file has Windows line endings, the usual cause: the
system then looks for a program called `/bin/sh` followed by that character. A file that is
not executable at all has another reason, which gives `chmod +x .roostr/scan`.

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

`roostr check` runs the script the same way and judges its output by the same rules, but
never writes `entrypoints.tsv`: it compares.

## `.roostr/entrypoints.tsv`

Written by `roostr scan`, committed. It may be absent in a project that has never been
scanned. `roostr check` reports when it is not, byte for byte, what `roostr scan` would
write now.

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
square brackets, and never counted from 0. A finding about a row of `entrypoints.tsv` names
its line instead, counted from 1 with the `# covers:` line as line 1.

Each finding is one line: `<file>: <key>: <message>`, `<file>: line <n>: <message>`, or
`<file>: <message>` when it is about neither a key nor a line (a file that does not parse,
a missing `entrypoints.tsv`). `validate --json` gives the same parts as separate values
(see "JSON output").

- `project.yml` and every process record parse as YAML. A file that does not is a finding,
  and the other files are still checked.
- Required keys are present: `name` in `project.yml`; `name` and `description` in a record.
  A key with nothing after it, or an empty string, is not present.
- Every key has the shape given in the tables above: a one-line string, a list or a map. A
  `why` given as one string, or an outcome given as a string instead of a map, is a finding.
- No unknown keys, at any level.
- No key is written twice in one mapping, at any level. The finding is on the key's path
  (`files`, `outcomes.refunded.when`) and names the line of each.
- Every operation has a command of one line, and every environment a value. A command of
  several lines is a finding that says to put the steps in a script and name it here.
- `watch_out` has at most 7 items, each at most 160 characters. An operation `note` and an
  environment value are each at most 160 characters.
- `checked` is a date that exists, written `YYYY-MM-DD`, or such a date and a time written
  `YYYY-MM-DD HH:MM`, hours `00` to `23`, minutes `00` to `59`. `yesterday`, `2026-02-30`,
  `2026-10-10T14:30`, `2026-10-10 24:00`, `2026-10-10 14:30:15` and `2026-10-10 9:30` are
  findings, and the finding gives both forms.
- Every entrypoint reference in a record uses a kind from the closed list and matches a row
  of `entrypoints.tsv`. When records list entrypoints and `entrypoints.tsv` is missing, there
  is one finding, which says to run `roostr scan`.
- No record lists the same entrypoint twice. The finding names the repeat by its position
  (`entrypoints.3`) and gives the entrypoint. Two different records may list the same
  entrypoint; that is not a finding.
- Every `files` path exists in the repository.
- Every row of `entrypoints.tsv` has a kind from the closed list.
- The rows of `entrypoints.tsv` are sorted by their whole line, in byte order. The finding
  names the line of the first row out of order: `.roostr/entrypoints.tsv: line 9: rows must
  be sorted in byte order`.
- Every file in `.roostr/processes/` or a folder below it whose name ends in `.yml` or
  `.yaml`, in any letter case, is a record roostr reads: an `<id>.yml` directly in
  `.roostr/processes/`. Hidden files and files in hidden folders do not count. The finding
  is about the whole file, `<file>: <message>`, and says that a record is a file named
  `<id>.yml directly in .roostr/processes/`, in those words. It names the file to rename or
  move it to, the same name with `.yml` directly in `.roostr/processes/`
  (`bookings/refund-booking.yml` → `.roostr/processes/refund-booking.yml`, `Refund.YML` →
  `.roostr/processes/Refund.yml`), unless a file of that name exists already; then it names
  none.
- A record's file name holds no character that the map removes or changes (see
  "Characters" under "The project map"): no control character, a tab and a line break
  among them, no U+2028 or U+2029, no format character, no default-ignorable code point.
  The map would print such an id with those characters dropped, an id `roostr show` does
  not know. The finding is about the whole file and names each character once, in the
  order they first appear, as for a string, with the file written as every text output
  writes it (see "Characters in the text output"):
  `.roostr/processes/refund<U+202E>redro.yml: the file's name holds U+202E, a character
  that is invisible or changes how text shows: rename the file`; with two, `the file's
  name holds U+001B and U+200B, characters that are invisible or change how text shows:
  rename the file`.
- `project.yml`, `entrypoints.tsv` and every record are regular files, or symbolic links
  to them. A named pipe, a socket, a device, or a link to one (to `/dev/zero`, say) is never
  opened, since reading it may wait or run on forever; the finding is
  `<file>: not a regular file; roostr reads only regular files`. Every other command treats
  such a file as one it cannot read: a record is left out of the overview's FUNCTIONALITY
  and of the map, and listed under NEEDS ATTENTION; `project.yml` stops the overview and
  `show`, and gives no map. A folder whose name ends in `.yml` is a folder, and the files
  in it are looked at as above.
- A record's `group` is not a near miss of an area: when it matches no area in
  `entrypoints.tsv` but equals one with case, spaces, `-` and `_` ignored, the finding is on
  the key `group` and names the area as the scan writes it (see "Areas and groups").
- No string holds a character that is invisible or changes how text shows. Every string
  `project.yml` and the records hold once parsed is checked, keys and values, `try` and
  `why` included, and a character written as an escape (`"\u202E"`) as well as one written
  as itself; comments are not. These characters are reported:
  - control characters but tab and newline: U+0000 to U+001F, U+007F, and U+0080 to
    U+009F. A carriage return reaches a value only as the escape `"\r"`: YAML reads one
    written as itself as a line break;
  - the bidi embeddings, overrides and isolates, U+202A to U+202E and U+2066 to U+2069;
  - U+200B (zero-width space), U+2060 to U+2065 and U+206A to U+206F, U+FEFF inside the
    text (a byte order mark at the start of the file is ignored, as above), U+180E, U+FFF0
    to U+FFFB, U+2028 and U+2029;
  - the Hangul fillers U+115F, U+1160, U+3164 and U+FFA0, and the Khmer vowels U+17B4 and
    U+17B5: characters that show as nothing;
  - the tag characters, U+E0000 to U+E007F, but those of the flags of England, Scotland
    and Wales (below), and the code points after them that Unicode keeps unassigned and
    ignorable, U+E0080 to U+E00FF and U+E01F0 to U+E0FFF (U+2065 and U+FFF0 to U+FFF8,
    above, are such too);
  - a variation selector (U+FE00 to U+FE0F, U+E0100 to U+E01EF) directly after another
    one. One after a character is how an emoji or a CJK variant is written.

  Text people write on purpose is not reported: a tab, U+200C and U+200D (zero-width
  non-joiner and joiner: Persian and Indic words, emoji such as 👩‍💻), U+200E, U+200F and
  U+061C (the direction marks of Hebrew and Arabic), U+00AD (soft hyphen), U+034F
  (combining grapheme joiner), U+180B to U+180D and U+180F (Mongolian variation
  selectors), the rest of Unicode's format characters, one variation selector after a
  character (⚠️), and U+2800 (braille pattern blank, which shows as a blank, as a space
  does). Nor are the flags of England, Scotland and Wales, the three emoji tag sequences
  Unicode recommends for general interchange: U+1F3F4, then the tag characters that spell
  `gbeng`, `gbsct` or `gbwls` (U+E0067 U+E0062 U+E0065 U+E006E U+E0067, and so on), then
  U+E007F. Any other run of tag characters is reported, after U+1F3F4 too: tag letters
  and digits spell text that does not show. The
  finding is on the path of the key whose value, or name, holds them, one per string. It
  names each such character once, as `U+` and four to six hexadecimal digits in capitals,
  in the order they first appear:
  `.roostr/processes/refund-booking.yml: description: holds U+202E, a character that is
  invisible or changes how text shows: remove it`; with two,
  `holds U+202E and U+200B, characters that are invisible or change how text shows:
  remove them`; with three, `U+202E, U+200B and U+E0041`. `entrypoints.tsv` and
  `.roostr/scan` are not checked.

With findings, `validate` exits 1 and lists them on stdout; with none, it prints
`No problems found` and exits 0. It never writes a file.

### What the overview and `show` do with a file `validate` complains about

A process record is **readable** when `validate` reports nothing about its own content: it
parses, is a mapping, has `name` and `description`, has no unknown key, every key has its
shape, every entrypoint uses a kind from the closed list, no entrypoint is listed twice, and
`checked` is in one of its two forms. A finding about how the record relates to other files (a listed file
that does not exist, an entrypoint that is not in `entrypoints.tsv`, a `group` that is a
near miss of an area) leaves it readable. So do two findings about its own content: a key
written twice, whose value written last is the one read, and a string that holds a
character that is invisible or changes how text shows, which the overview and `show` print
with those characters written out (see "Characters in the text output"). A finding about
the file's name leaves it readable too.

- The overview shows the readable records and lists every other one under NEEDS ATTENTION
  as needing checking. `show` refuses a record that is not readable, and exits 2.
- `entrypoints.tsv` with a row of a kind outside the closed list is listed the same way;
  the overview reads its other rows and leaves that one out.
- `project.yml` is readable when it parses, is a mapping, has `name`, has no unknown key,
  and every key has its shape. When it is not, the overview and `show` exit 2. The limits
  on length and count (`watch_out`, an operation's `note`, an environment's value) do not
  make it unreadable: the overview prints the value whole, and only `validate` and `check`
  report it. Nor do a key written twice, an operation with no command or with a command of
  several lines, an environment with no value (see `project.yml` above for what the
  overview shows of them), and a string that holds a character that is invisible or
  changes how text shows.

## What `roostr check` reports

`roostr check [--base <commit> | --staged | --session]` answers whether `.roostr/` is
current with the code, with an exit code a hook, a gate or CI can act on. It never writes a
file of the project; `--session` keeps a count of its holds in Git's folder (see
"`--session`"). It can be started in any folder of the project; paths it prints are
relative to the project folder, the folder `.roostr/` is in.

It reports five kinds of problem, as blocks in this order, each left out when it has
nothing:

1. **NOT STAGED**, only with `--staged`: a file under `.roostr/` that roostr reads, whose
   working-tree state is not what is staged (see "`--staged`"). In path order. It comes
   first: the other blocks judge the files as they stand in the working tree, and this one
   says which of them the commit would not hold. Its paragraph says to stage each with
   `git add <path>`, or to undo the change.
2. **VALIDATE**: every finding `roostr validate` reports, in validate's order.
3. **ENTRYPOINTS**: `entrypoints.tsv` is not current. check runs `.roostr/scan` as
   `roostr scan` does and compares what `roostr scan` would write with `entrypoints.tsv`,
   byte for byte. When they differ, the entrypoints added are listed with `+`, then those
   removed with `-`, each group in the file's sort order, as `roostr scan` reports them; a
   row whose file or area changed, or a changed `# covers:` line, makes them differ without
   adding a line of its own. Also a problem: a scan script and no `entrypoints.tsv` yet; a
   scan script that is not executable, or that is and cannot be started (see
   "`.roostr/scan`"); a script that exits with anything but 0 or prints
   malformed output, with the reasons `roostr scan` gives. When the script fails, what it
   wrote to stderr is shown inside the problem, on stdout; when it succeeds, its stderr is
   dropped. check takes the script's answer once the script has exited and its stdout is
   closed, as `roostr scan` does: a helper the script started that keeps stderr open (a
   preloader such as Spring) does not hold check, and the stderr shown is what was written
   until then. A child that keeps stdout open holds both commands, since the output is not
   complete. Without `.roostr/scan` this part is skipped, and nothing is said about it.
4. **PROCESSES**: a process needs attention, by the rule in "What needs attention". The
   processes by id in alphabetical order, and under each the files that made it need
   attention, in the order of its `files`. Outside a Git repository this part is skipped.
   The paragraph suggests a value for `checked`, one for every process it lists, that
   differs from what each listed record holds: today's date alone (`checked: 2026-10-10`)
   when no listed record holds today's date; otherwise today's date and the time now
   (`checked: 2026-10-10 14:30`); and when a listed record holds that minute already, the
   next minute (`14:31`; at `23:59`, the next day's `00:00`). Writing the value is then
   always a change.
5. **NOT DESCRIBED**, with `--base`, `--staged` or `--session`: an entrypoint in
   `entrypoints.tsv` (the file in the working tree) that no process lists, unless at the
   base it was already in `entrypoints.tsv` and no process listed it there: that is old
   debt, which stays quiet. So a new entrypoint is reported, and so is one that lost its description, its record
   deleted or the entrypoint dropped from the record's list. An entrypoint moved to another
   record is still listed and is not reported, nor is a feature removed with its record and
   its rows. In the file's sort order. Not compared when there was no `entrypoints.tsv` at
   the base. A record that is not readable now (VALIDATE reports it) counts as listing what
   it listed at the base, and a record at the base that roostr cannot read lists nothing
   there. The paragraph:

   ```
   NOT DESCRIBED
     No process lists these entrypoints, though at HEAD each was either new or
     listed by a process. Add each to the entrypoints of the process it belongs to
     in .roostr/processes/<id>.yml, or describe a new process for it. Where a
     process listed it before, restore that record or its entry instead.
   ```

   With `--session`, the paragraph says `though when the session started each was
   either new or listed by a process` instead (see "`--session`").

With problems, check exits 1 and prints them on stdout. With none, it prints
`No problems found` and exits 0.

### `--base`

Without `--base`, check judges the whole project. With `--base <commit>` (anything Git
accepts as a commit: `HEAD`, `main`, `origin/main`, a hash; `--base=<commit>` is the same),
it judges only the work done after that commit.

- A file has **changed after the base** when a commit reachable from `HEAD` and not from
  the base touched it, or when it is changed in the working tree: edited, staged, deleted,
  or not tracked by Git yet. A merge commit touches only the files in which it differs from
  every one of its parents. A file Git ignores, and a file outside the project folder,
  never counts as changed.
- When nothing changed after the base, check prints `No problems found` and exits 0. It
  runs no scan.
- Otherwise VALIDATE is reported in full.
- ENTRYPOINTS: the scan runs unless every file that changed after the base is a record,
  `.roostr/project.yml` or a file under `.roostr/processes/`. When it does not run, nothing
  is said about it.
- PROCESSES lists only the files that also changed after the base. A process whose newer
  files all changed before the base is left out.
- NOT DESCRIBED reads `entrypoints.tsv` and the records at the merge base of the base and
  `HEAD`: the commit where the work left the base. When the base has moved on since, what
  it changed is not the work's.
- In a repository with no commits yet, `--base HEAD` means that nothing is committed:
  everything in the working tree is work after the base, and NOT DESCRIBED is not compared.
  Any other base there does not resolve.

check cannot run, and exits 2, when the base is not a commit in the repository (fetching
more history may help), when the base and `HEAD` share no commit (in a shallow clone,
fetch more history), when `--base`, `--staged` or `--session` is used outside a Git
repository, when `--base` has no value, when two of `--base`, `--staged` and `--session`
are given together (the message says that they cannot be used together), and when it is
given an option it does not take.

In a shallow clone (see "A shallow clone"), check runs with `--base` only when every commit
after the base is in the clone with its parents: when each commit at the clone's edge that
`HEAD` reaches is also reached by the base. Commits at the edge of a branch `HEAD` does not
reach do not matter. `--base HEAD` in a clone of depth 1 always runs: its one edge is
`HEAD`. Otherwise check exits 2: the history after the base is not all in the clone, and
`git fetch --unshallow` fetches it. Without this rule a branch merged after the base and cut
short by the clone would hide its changes: its oldest commit in the clone looks as if it
added every file, records included, at once.

**A gate judges the working tree too.** With `--base`, uncommitted changes count as work,
as the Stop hook of an agent needs. A gate that decides whether a commit or a branch may
go on (a merge gate, a harness that checks an agent's branch) must therefore run on a clean
checkout of that commit, as CI does, or after everything has been committed; run where the
agent worked, it also credits files the commit does not hold. Before a commit, use
`--staged`.

#### A merge in progress

While a merge is in progress (Git has written `MERGE_HEAD`: `git merge` stopped on a
conflict, or was run with `--no-commit`), check answers as it will once the merge is
committed with the tree as it stands. That merge commit has `HEAD` and the merged commits
as its parents, and touches only the files in which it differs from every one of them.

- So during a merge a file is changed in the working tree (or, with `--staged`, in what is
  staged) only where it differs from `HEAD` and from every merged commit. A file taken
  whole from one side is not a change of the merge; a conflict resolved to a version
  neither side had is.
- This holds for records as for code: a record the merge takes as the merged side left it
  is not updated by the merge. Its last commit is the merged side's, as it will be after the
  merge commit.
- With `--base HEAD` and with `--staged`, the merged commits count as part of the base: the
  other side's work was judged where it was made, and only what the merge itself changes
  is the work. With any other base, and without `--base`, the merged side's commits count
  as they will once the merge commit reaches them.
- A rebase or a cherry-pick in progress is not a merge: the change being replayed is the
  work, as any uncommitted change is.

### `--staged`

`--staged` judges the commit being made, for a Git pre-commit hook. It works as
`--base HEAD`, with two differences:

- A code file has changed after `HEAD` when what is staged, or what is in the working
  tree, differs from `HEAD`. A change that was staged and then undone in the working tree
  goes into the commit, so it counts. Records, `project.yml`, the scan and
  `entrypoints.tsv` are judged as they stand in the working tree, as with `--base HEAD`.
- NOT STAGED reports every file roostr reads under `.roostr/` (`project.yml`, each record
  `processes/<id>.yml`, `scan`, `entrypoints.tsv`) whose working-tree state is not what is
  staged: an edit not staged, a new file Git does not track, a deletion not staged. A file
  Git ignores, a hidden file, and a file roostr does not read (`.roostr/NOTES.md`, a
  `.yaml` file in `.roostr/processes/`) never count. The scan runs on the working tree.

During `git commit -a` and `git commit <path>`, Git runs the hook with `GIT_INDEX_FILE`
naming a temporary index that holds exactly what the commit will; roostr judges that index.
For a plain `git commit` Git gives `.git/index`, a path relative to the top of the
repository, also when the hook first changes to the project's folder (`cd services/shop &&
roostr check --staged`); roostr reads a relative `GIT_INDEX_FILE` from the top of the
repository, as Git does. In a repository with no commits yet, `--staged` judges the first
commit: everything staged is work. The Stop hook of an agent uses `--session`: in the
middle of a turn the agent has staged nothing.

### `--session`

`--session` judges what an agent did in its session, committed or not, for the Stop hook
of the agent (Claude Code's, or any that passes the same input). It reads its standard
input as `roostr session start` does (see there), and takes `session_id` and
`permission_mode` from the JSON object there, as every hook of Claude Code gets them. With
the record that `roostr session start` made for that id, it judges as `--base <the commit
the session started at>`, with "changed after the base" narrowed to the files that are
**the session's work**:

- A **commit is the session's** when all of these hold: it is reachable from `HEAD` and not
  from the start commit; it is not a replay, a commit patch-identical to one the start
  commit reaches (those `git rev-list --cherry-pick --right-only <start>...HEAD` leaves
  out); its committer email is the one the record holds (this is left out when the record
  holds none); and its committer date is not more than 5 minutes (300 seconds) before the
  session started. A file a session commit touched is the session's work; a merge commit
  touches only the files in which it differs from every one of its parents.
- An **uncommitted change is the session's** unless the file is as the record's snapshot
  noted it (see "`roostr session start`"): the same content, deleted then and now, the same
  size and modification time for a file noted so, the same commit for a submodule or a
  nested repository. In a work tree other than the one the snapshot was taken in, and when
  the record has no snapshot, every uncommitted change is the session's.
- A file **left by the user** is one the snapshot noted, that is still as noted, and that no
  session commit touched. VALIDATE leaves out the findings in such a file. ENTRYPOINTS
  compares the scan's output with `entrypoints.tsv` without the rows, on either side, whose
  file is one; the `# covers:` line is compared as ever.
- NOT DESCRIBED is compared only when `entrypoints.tsv` or a record is the session's work,
  with `entrypoints.tsv` and the records at the merge base of the start commit and `HEAD`.
  Its paragraph says `though when the session started each was either new or listed by a
  process` in place of `though at <base> each was either new or listed by a process`.

So what was brought in is not the session's: `git pull`, a teammate's branch checked out,
`git reset --hard origin/main` (commits made before the session, or by another committer),
a pull request squash- or rebase-merged on GitHub during the session (committed by GitHub,
not by the session's committer) and then pulled, `git pull --rebase` replaying the user's unpushed
commits, a reword of the start commit. Nor is the user's unfinished work from before the
session, also after `git stash` and `git stash pop`. What the session commits counts,
whatever it holds: `git commit -a` of the user's earlier edit, an amend of its own commit,
a replay in which it resolved a conflict. Merging `main` after the session's own commits
counts only what the merge itself changed. During a merge in progress, the merged commits
are judged as commits are, as they will be once the merge commit reaches them.

- When nothing is the session's work, check prints `No problems found` and exits 0, and
  runs no scan. The scan runs unless every file of the session's work is a record or
  `.roostr/project.yml`. PROCESSES lists only files of the session's work, and a record's
  uncommitted changes count as its update only when they are the session's: a record the
  user edited before the session is judged by its last commit.
- When the session started before the repository's first commit, every commit is judged by
  its committer and its date as above, and NOT DESCRIBED is not compared.
- When the input's `permission_mode` is `plan`, check does not judge: it prints `Not
  judged: the session is in plan mode, where the agent cannot change files.` and exits 0,
  with no scan. With `--json`, `problems` is `null`. Every other `permission_mode`
  (`default`, `acceptEdits`, `auto`, `dontAsk`, `bypassPermissions`, any other value, or
  none) is judged.
- When `HEAD` shares no commit with the start commit (an orphan branch), check cannot run:
  it exits 2, with `Cannot judge the session: HEAD shares no commit with the commit it
  started at, <short hash>.` on stderr.
- It judges as `--base HEAD`, and says nothing about it, when the input names no session it
  can use: stdin is a terminal, empty, or not a JSON object, or there is no `session_id` it
  would use as a file name (see "`roostr session start`"). It also judges as `--base HEAD`,
  and says so in one line on stderr, when the input is over 16 MiB:
  `roostr check --session: the input on stdin is over 16 MiB; judged as --base HEAD`; and
  when it has no usable record of the session, none (the hooks were added during the
  session, or the record was removed after 30 days) or one whose commit no longer exists:
  `roostr check --session: no usable record of session <id>; judged as --base HEAD`.

**No endless holds.** When check would hold (exit 1) for the fourth time in a row with the
same problems, as `--json` would list them, and no change in the session's work (the same
session commits, and the same content in every file whose uncommitted change is the
session's), it exits 0 instead and prints this one line, not wrapped:

```
Not held again: 3 holds in a row found the same problems and no change to the session's work. The problems remain; the pre-commit hook and CI will report them.
```

The fifth such run is let through too; a run that does not hold (it passes, or does not
judge) starts the count again. The count is kept beside the session's record, in Git's
folder, and goes with it; when it cannot be written, check holds as ever. Only the text
output counts and is let through: with `--json`, check neither counts nor lets through.
Claude Code's own limit, 8 holds in a row, starts again at every tool call, so an agent
that runs a command between two Stops never reaches it.

A runner that writes the input and keeps stdin open so holds the Stop hook for 2 seconds
at most, and the session is still judged. Without `--session`, check does not read stdin.
A Stop hook runs at the end of every turn: with nothing done since the session started,
`--session` costs what `--base HEAD` costs; once the session has work other than records,
the scan runs every turn.

What the Stop hook does not see, so that the pre-commit hook and CI are still needed:

- Commits on a branch the session switched away from (back to `main` before finishing):
  they are no longer reachable from `HEAD`.
- A commit backdated more than 5 minutes before the session started (`GIT_COMMITTER_DATE`,
  `git rebase --committer-date-is-author-date`), or made under another committer identity.
- Work in another worktree, when Claude Code runs the hook in the checkout where the
  session started: after `git worktree add ../x && cd ../x` in Bash, the hook may still run
  in the first one. A session that enters a worktree as Claude Code's own folder has its
  hook run there, and its record is found (see "`roostr session start`").

And what it counts that the agent may not have done: roostr cannot tell who changed a
file during the session, so the user saving files, or a second session in the same work
tree, counts as this session's work. A file noted by size and modification time that is
saved again unchanged counts. Amending the start commit with a change of content counts
everything it holds. A finding in a record the user did not change, caused by the user's
change elsewhere (a rescan that dropped an entrypoint the record lists), is reported.

So the gates of a project are three, and none replaces another: the pre-commit hook judges
each commit (`--staged`), the Stop hook each turn of an agent with all it did since its
session started (`--session`), and CI the branch (`--base origin/main`).

## `roostr session start`

`roostr session start` is the command of the SessionStart hook. It does two things, and
neither depends on the other: it notes where an agent's session started and what was
already changed then, so that `roostr check --session` can judge the session's own work at
its end, and it prints the project map, which the agent is given before its first step
(see "The project map" below).

- It reads its standard input until the input ends or 2 seconds have passed since it
  started, whichever comes first, and keeps at most 16 MiB of it: what comes after is read
  and dropped, so that the runner never writes into a closed pipe, and input over 16 MiB is
  no input. A terminal it does not read. What it kept is used when it is one whole JSON
  object, also when stdin was still open: a runner that writes the input and never closes
  stdin still gets the session recorded and the map, after 2 seconds. Anything else (part
  of an object, text, a list) is no input. It takes `session_id` and `source` from the
  object; every other key (`cwd`, ...) is ignored.
- A `session_id` is used only when it is a string of 1 to 128 ASCII letters, digits, `-`,
  `_` and `.` that does not start with `.`. Anything else counts as no id, so that the id
  can name a file and never a path (`../../hooks/pre-commit` writes nothing).
- It records the session in `<git common dir>/roostr/sessions/<session_id>`, where the
  folder is what `git rev-parse --git-common-dir` names: `.git` in a checkout and in every
  linked worktree of it, so that a session that moves into a worktree finds its record
  there. Records an earlier version wrote in `.git/worktrees/<name>/roostr/sessions/` are
  not read.
- The record holds, written first: the commit `HEAD` points at, or that there is no commit
  yet; when the session started, by roostr's clock (`ROOSTR_TODAY`, see "Environment"), in
  seconds; and the committer's email as `git var GIT_COMMITTER_IDENT` gives it, or that Git
  has none. Then it takes the **snapshot** and adds it: every file that differs from that
  commit in the work tree, as `git --no-optional-locks status --untracked-files=all` lists
  it (edited, staged, deleted, not tracked and not ignored), each with its content's blob
  id as `git hash-object --no-filters` computes it (no object is written, no clean filter
  runs), or that it is deleted; and the work tree it was taken in.
  - A file larger than 8 MiB (8,388,608 bytes) is noted by its size and modification time,
    and not read. One of 8 MiB is hashed.
  - A submodule, or a repository nested in the work tree, is noted by the commit its `HEAD`
    is at, and not hashed.
  - The snapshot has 3 seconds from when it begins. Files not hashed by then are noted by
    size and modification time. When `git status` has not answered by then, it is stopped
    and no snapshot is kept. The record and the map never wait for the snapshot longer.
  What else is in the record, and how, is roostr's own and may change between versions;
  nothing else reads it.
- When a record for the id is already there, it is kept as it is, snapshot and all: a
  session that is resumed, compacted or cleared under the same id keeps its start.
- Before that, it removes every record of a session that started more than 30 days ago,
  and with it what `check --session` keeps beside it; one that started 30 days ago exactly
  stays. A session resumed after more than 30 days so starts again where `HEAD` is then.
- Recording needs no `.roostr/`: it records wherever it runs inside a Git work tree, from
  any folder of it, also at the top of a repository whose project is in a subfolder. It
  reads from Git and writes nothing but its records: never a file of the working tree,
  never the index, no Git object.
- It exits 0 whenever it ran, also when it cannot record or print a map: the Stop hook
  then judges as `--base HEAD`. A stdout it cannot write (closed, a broken pipe, a full
  disk) loses the map and nothing else: the session is recorded all the same, and the
  exit is 0. When it cannot record, it writes nothing and says why on stderr, in one line
  that starts `roostr session start: ` and ends `; nothing recorded`: that the input on
  stdin has no `session_id` it can use (the line names `session_id`); that it is `not in a
  Git repository`; that `the current folder no longer exists` (it was deleted while the
  hook's shell was in it; no map is printed then); or why Git cannot be read (see "When
  Git cannot be read"). The map is printed all the same where there is a folder to find it
  from.
- `roostr session` without `start`, another word in its place, or any option (`--json`
  among them) is a mistake in the command: exit 2, with the usage on stderr.

### The project map

What a SessionStart hook prints on stdout goes into the agent's context. `roostr session
start` prints there one line of JSON, then a newline, and nothing else:

```
{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"<the map>"}}
```

Exactly these keys, in this order; not roostr's `format`. JSON escapes only what it must:
`"` and `\` get a `\` in front, and a newline is written `\n`; every other character of the
map, accented letters and emoji among them, is written as UTF-8. Claude Code, Gemini CLI
and Codex CLI each give `additionalContext` to the model. `--help` is the one time
`session start` prints anything else on stdout.

- **When.** The map is printed when the input's `source` is `startup`, `clear` or
  `compact`, and also when there is no `source`, when it is not a string or is a word
  roostr does not know, and when there is no input. When `source` is `resume` or `fork`,
  nothing is printed: the history the agent goes on with holds the map it was given. The
  session is recorded as above whatever the source.
- **Which project.** `.roostr/project.yml` is looked for as every command looks for it,
  from the current folder up. Where there is none (the top of a repository whose project
  is in a subfolder, a hook set for every repository of a user), nothing is printed and
  nothing is said. When `project.yml` cannot be read, as the overview cannot read it (see
  "What the overview and `show` do with a file `validate` complains about"), nothing is
  printed, and stderr has one line that names `.roostr/project.yml` and `roostr validate`.
  A `project.yml` that reads with findings (8 `watch_out` items, a note too long) gives a
  map.
- **From what.** The map is built from `.roostr/project.yml` and the readable records in
  `.roostr/processes/`, and nothing else: not `entrypoints.tsv`, not the scan (never run),
  not Git, not the clock, not `COLUMNS` or a terminal. The same files give the same bytes.
  A record that cannot be read is left out without a word; `validate`, `check` and the
  hooks report it.
- **Without it.** The hook command `roostr session start >/dev/null` records the session
  and gives the agent no map.

Atelier's map, as `additionalContext` holds it:

```
roostr project map of Atelier, from .roostr/, kept by people and agents working here. Records can lag the code; where they differ, the code is current. `roostr show` with a process id prints a process in full: files, entrypoints, how to try it, outcomes. `roostr` prints the overview and recent changes.

Stack: Rails 8, PostgreSQL, Solid Queue
Description: Sells places on a pottery studio's workshops and reminds the people who booked.

Watch out:
- Stripe runs in test mode on staging; never use a real card there.
- The reminder job runs at 18:00 in the studio's time zone, not UTC, so a booking made later that evening for the next morning gets no reminder.

Operations:
- start: `bin/dev`
- test: `bin/rails test`

Processes by group, each as id: description [first files]:
Bookings:
- book-workshop: Reserve a place, take payment and confirm by email. [app/models/booking.rb, app/services/book_workshop.rb]
- cancel-booking: Cancel a booking and free the place for the next person on the waiting list. [app/models/booking.rb]
Notifications:
- remind-attendees: Remind attendees the day before a workshop [app/jobs/booking_reminder_job.rb]
```

Its parts, in this order, each separated from the next by a blank line; a part with
nothing in it is left out with its heading, and there is no newline at the end:

1. The header, one line, always: `roostr project map of <name>, from .roostr/, kept by
   people and agents working here. Records can lag the code; where they differ, the code
   is current. `roostr show` with a process id prints a process in full: files,
   entrypoints, how to try it, outcomes. `roostr` prints the overview and recent changes.`
   A `project.yml` with only a name and no readable record gives the header alone. A name
   of more than 200 characters (Unicode code points, counted as the map prints the name)
   is cut to its first 200, followed by `...`: `roostr project map of <200 characters>...,
   from .roostr/, ...`. The header is cut nowhere else, so it is at most about 1,100 bytes.
2. `Stack: <stack>` and `Description: <description>`, each on a line of its own, each left
   out when the key is absent.
3. `Watch out:`, then every item, in the file's order, as `- <item>`.
4. `Operations:`, then the operations in the file's order, each as
   ``- <name>: `<command>` ``, followed by ` (<note>)` when it has a note. An operation
   that ships is left out: one whose name, as the map prints it, has `deploy`, `release`
   or `publish` as one of its words, in any letter case. The words of a name are its runs
   of letters; every other character (`-`, `_`, `:`, `.`, `/`, a space, a digit) separates
   them. So `deploy`, `Deploy`, `deploy-production`, `deploy:staging`, `staging.deploy`,
   `release` and `publish_gem` are left out, and `deployment-status` and `predeploy-check`
   are not. An operation with no command is left out too. A command of several
   lines shows its first line, then ` ...` after the closing backtick
   (``- setup: `bin/setup` ...``). Environments are left out.
5. The processes: the legend `Processes by group, each as id: description [first files]:`,
   then each group a readable record names, by its name as the map prints it (see
   "Characters"), in byte order of the names, as `<group>:` on a line of its own, followed
   by its processes in byte order of their ids; last, after every named group whatever
   its name, one `other:` with the processes without a group and those whose group is
   `other`. A process is `- <id>: <description> [<path>, <path>]`: the paths of its first
   two `files` items, without their notes; one path when it has one file, no brackets
   when it has none; `- <id> [<path>, <path>]` when its description prints as nothing.

Nothing else is in it: no entrypoint or area (so an area no record names as its group is
not there), no count, nothing of LAST 7 DAYS or NEEDS ATTENTION, no out-of-date mark, and
no record's `name`, `try`, `outcomes`, `why` or `checked`. No line is wrapped or padded.

#### At most 8,000 bytes

The map is at most 8,000 bytes as UTF-8, counted as printed (after the characters below)
and before JSON escapes it. A map of exactly 8,000 bytes is printed as it is. A longer one
is shrunk, and the first of these forms that fits is printed:

- (a) Each process as `- <id> [<first path>]` (`- <id>` when it has no files), under the
  legend `Processes by group, each as id [first file]:`.
- (b) One line per group, `<group>: ` and every id of the group, separated by `, `, in
  the same order, `other` last, under the legend `Process ids by group:`.
- (c) Form (b), cut at its end, one item at a time, until it fits. The process ids go
  first, from the last one back: a group line that has lost some of its ids ends in
  `, ...` after the last id it keeps (`Group D: nightly-import-001, nightly-import-007,
  ...`); a group line with none left goes, and the legend goes with the last one. Then
  the operations go, from the last; then the watch-out items, from the last; then the
  Description line, then the Stack line. A heading with nothing left under it goes, and
  so does the blank line before it. The header is never cut (but see its name above).

A shrunk map ends with a blank line and a closing line, both counted within the 8,000
bytes:

- (a) ``Left out to keep this map within 8,000 bytes: each process's description and all its files but the first. `roostr` prints the descriptions; `roostr show` with a process id prints the files.``
- (b) ``Left out to keep this map within 8,000 bytes: each process's description and files. `roostr` prints the descriptions; `roostr show` with a process id prints the files.``
- (c) while some process id is left: ``Left out to keep this map within 8,000 bytes: each process's description and files, and <n> of the <total> process ids. `roostr` prints the full overview.``, where `<n>` is the number of ids cut and `<total>` the number form (b) lists.
- (c) once every process id is cut: ``Left out to keep this map within 8,000 bytes: every process. `roostr` prints the full overview.``, with what else was cut after `every process`, in this order: `every operation` or `<n> of the <total> operations`, `every watch-out item` or `<n> of the <total> watch-out items`, `the Description line`, `the Stack line`. The items are separated by `, `, and the last by `, and `: ``every process, every operation, and 11 of the 80 watch-out items.``

The JSON line then has 78 bytes more than the map, and one more for each newline, `"` and
`\` in it.

#### Characters

The map holds the text as written, with these changes, made before it is measured:

- A tab becomes a space. A line break inside a value (a carriage return, a carriage return
  and a newline, a newline, U+2028, U+2029) becomes one space: every item is one line.
- Removed: every other control character (U+0000 to U+001F, U+007F, U+0080 to U+009F),
  every character of Unicode's category Cf (format characters: zero-width spaces and
  joiners, direction marks, bidi embeddings, overrides and isolates, the byte order mark,
  the soft hyphen, ...), and every default-ignorable code point (Unicode's property
  Default_Ignorable_Code_Point: the variation selectors, the tag characters, U+034F, the
  Hangul fillers, the Khmer vowels U+17B4 and U+17B5, and code points Unicode keeps for
  more of them). As of Unicode 18.0 that property is U+00AD, U+034F, U+061C, U+115F to
  U+1160, U+17B4 to U+17B5, U+180B to U+180F, U+200B to U+200F, U+202A to U+202E, U+2060
  to U+206F, U+3164, U+FE00 to U+FE0F, U+FEFF, U+FFA0, U+FFF0 to U+FFF8, U+1BCA0 to
  U+1BCA3, U+1D173 to U+1D17A and U+E0000 to U+E0FFF (DerivedCoreProperties.txt).
- Kept: every other character. U+2800, the braille pattern blank, shows as a blank, as a
  space does, and stays.

A record whose id holds a character these rules remove or change, a tab or a line break
among them, is left out of the map: the id it would print is not its id. `validate`
reports its file name (see "What `roostr validate` checks").

A value that is empty once these characters are removed counts as absent: a stack or a
description has no line, a watch-out item, an operation's note and a `files` item are left
out (the first two files are then the first two left), an operation whose name or command
is empty is left out, and a group is no group: its process is under `other`. Groups are
told apart by their name as the map prints it: `Book<U+200B>ings` and `Bookings` are one
group, `Bookings`. `<U+200B>` stands for that character here. The overview keeps its own
rule, the names as written.

This is a backstop: `validate` reports such characters (see "What `roostr validate`
checks"), so that `check` and the hooks refuse them. The map drops more than `validate`
reports: `⚠️` is `⚠` in it, `👩‍💻` is `👩💻`, and the flag of Scotland is U+1F3F4 alone, a
black flag.

### The hooks `roostr init` prints

For Claude Code, one line of JSON for `.claude/settings.json` with both hooks:

```
{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"roostr session start","timeout":10}]}],"Stop":[{"hooks":[{"type":"command","command":"roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }"}]}]}}
```

The Stop command turns exit 1 into 2, which holds the agent and shows it the report; any
other exit (roostr missing, exit 2) is a hook error that holds nothing. The SessionStart
command has nothing after it: a SessionStart hook cannot hold anything, and a missing
roostr is then shown once as a hook error. Its `"timeout":10` is the number of seconds
Claude Code waits for it, so that the start of a session is never held up for long.

Claude Code runs a hook in its own current folder, which may be any folder of the
repository. When the project is below the repository's top (`.roostr/` in
`services/shop`), init prints both commands with `cd "$(git rev-parse
--show-toplevel)/services/shop" && ` in front, and the Stop command in braces, so that a
failing `cd` exits 1 and holds nothing:

```
cd "$(git rev-parse --show-toplevel)/services/shop" && roostr session start
cd "$(git rev-parse --show-toplevel)/services/shop" && { roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }; }
```

The SessionStart entry keeps its `"timeout":10`. The folder is written inside the double
quotes as it is, a space included; a `"`, `$`, `` ` `` or `\` in it gets a `\` in front. In
the JSON each `"` and `\` of the command gets a `\` in front. The pre-commit hook and the
CI command are `cd <folder> && ...`, the folder in single quotes when sh needs it quoted.

## JSON output

With `--json`, a command prints what it found as one JSON object instead of text, for a
program to read: an agent that decides which services a task touches, a dashboard that
shows each service's check. The overview (`roostr --json`), `show`, `changes`, `validate`
and `check` take it. `scan`, `init` and `session start` do not: given `--json`, they exit 2
and name it, as for any option a command does not take.

- `--json` may stand anywhere among the arguments. `roostr --json show book-workshop`,
  `roostr show --json book-workshop` and `roostr show book-workshop --json` are the same.
  It combines with the command's options: `roostr changes --since 3d --json`,
  `roostr check --json --base HEAD`.
- `--json` is never the value of another option, nor a command's argument:
  `roostr changes --since --json` is `--since` without a value, and `roostr show --json`
  is `show` without an id. Both exit 2.

### The contract

- stdout carries exactly one JSON object, in UTF-8, and nothing else. Whitespace, the order
  of keys in an object and the escaping of characters are free.
- The object has `"format": 1` at its top level. The format changes only when a change
  could break a consumer: a key removed or renamed, or a value whose type or meaning
  changes. Adding a key, or a new `kind` of check problem, keeps the format. A consumer
  ignores keys it does not know, and treats a check problem of a kind it does not know as
  a problem.
- Exit codes are those of the text output. Exit 2, and `show`'s exit 1 for an id that does
  not exist, print nothing on stdout; the reason goes to stderr as text, as without
  `--json`. `validate` and `check` with findings exit 1 and print their JSON on stdout.
- Every key in the tables below is always present. A single value that is absent is
  `null`; a list with no items is `[]`. Where a list or an object can be `null` instead,
  `null` means "not known": roostr did not or could not compare. It is never the same as
  `[]`, which means it looked and found none.
- A value from a YAML file is the string YAML reads, unchanged: a `try` written as a `|`
  block keeps its line breaks, its spaces and tabs, and the newline at its end; a `checked`
  date is the string `"2026-10-08"`. Lines, exit codes and the format are JSON numbers.
- An entrypoint is referred to by the string `<kind> <name>`, as records list it. A process
  by its id. Paths are relative to the project folder, as in the text output.
- Keys are written in snake_case.
- Lists follow the order of the text output; each table says which order that is.

### A process, `<process>`

The same object in the overview and in `show`.

| Key | Value |
|---|---|
| `id` | the process id: the record's file name without `.yml` |
| `name` | string |
| `description` | string |
| `group` | string, or `null` when the record has no `group` |
| `entrypoints` | list of `<kind> <name>` strings, in the record's order |
| `files` | list of objects with `path` and `note`, in the record's order. `note` is `null` for an item written as a path alone, or as a map without `note` |
| `try` | string, or `null` |
| `outcomes` | list of objects with `name`, `when` and `then`, in the record's order |
| `why` | list of strings, in the record's order |
| `checked` | string, as written: `YYYY-MM-DD` or `YYYY-MM-DD HH:MM`; or `null` |

### The overview: `roostr --json`

| Key | Value |
|---|---|
| `format` | `1` |
| `project` | object, below |
| `processes` | list of `<process>`: every readable record (see "What the overview and `show` do with a file `validate` complains about"), in id order |
| `entrypoints` | object, below; `null` when there is no `entrypoints.tsv` (the text's `not scanned yet`) |
| `needs_attention` | list of objects, below: the processes that need attention, by the rule in "What needs attention", in id order. `null` when roostr cannot tell: the project is not in a Git repository, Git could not be read, or the repository is a shallow clone. `[]` when Git was read and no process needs attention, also in a repository with no commits yet |
| `needs_checking` | list of objects with one key, `file`: the files the overview lists as needing checking, a process record that is not readable or `entrypoints.tsv` with a row of an unknown kind. In the order of their paths. `[]` when there are none. Why a file needs checking is what `validate --json` says |

LAST 7 DAYS is not in the overview's JSON: `roostr changes --json` gives it.

`project`:

| Key | Value |
|---|---|
| `name` | string |
| `stack` | string, or `null` |
| `description` | string, or `null` |
| `watch_out` | list of strings, in the file's order |
| `operations` | list of objects with `name`, `command` and `note`, in the file's order. `note` is `null` for an operation written as a command alone, or without `note` |
| `environments` | list of objects with `name` and `where` (the value written for it), in the file's order |

`entrypoints`:

| Key | Value |
|---|---|
| `covers` | list of the kinds on the `# covers:` line, in the order of the closed list |
| `rows` | list of objects, below, one per row of `entrypoints.tsv`, in the file's order. A row of a kind outside the closed list is left out |

A row:

| Key | Value |
|---|---|
| `entrypoint` | `<kind> <name>` |
| `kind` | string |
| `name` | string |
| `file` | string |
| `area` | string, or `null` for a row without an area |
| `processes` | list of the ids of the readable records that list this entrypoint, in id order; `[]` when none does |

What the text counts follows from these: an entrypoint is described when its `processes` is
not empty; a kind on the `# covers:` line with no row is `none found`; a kind not on it is
`not scanned`.

An item of `needs_attention`:

| Key | Value |
|---|---|
| `process` | the process id |
| `files` | list of the files it lists that made it need attention, in the order of its `files` |

### `roostr show <id> --json`

| Key | Value |
|---|---|
| `format` | `1` |
| `process` | `<process>` |
| `entrypoints` | list of objects with `entrypoint` and `file`, one per entrypoint the record lists, in its order. `file` is that row's file in `entrypoints.tsv`, or `null` when the entrypoint is not there or there is no `entrypoints.tsv` |

### `roostr changes --json`

| Key | Value |
|---|---|
| `format` | `1` |
| `since` | the first day of the period, `YYYY-MM-DD` |
| `entrypoints` | object with `added` and `removed`, each a list of `<kind> <name>` strings in the file's sort order. `null` when entrypoints are not compared (the text's `not compared`, see "What roostr reads from Git") |
| `processes` | list of objects with `id` and `subjects`, the processes the text lists under PROCESSES, in id order. `subjects` are the subjects of their commits, newest first, as the text lists them |

### `roostr validate --json`

| Key | Value |
|---|---|
| `format` | `1` |
| `problems` | list of `<invalid>`, below, one per finding, in the order `validate` prints them; `[]` when there are none |

`<invalid>`, a finding of `validate`:

| Key | Value |
|---|---|
| `kind` | `"invalid"` |
| `file` | the file the finding is about, such as `.roostr/processes/refund-booking.yml` |
| `key` | the dotted path of the key the finding is about, as in "What `roostr validate` checks" (`entrypoints.1`, `files.2.note`), or `null` |
| `line` | the line of `entrypoints.tsv` the finding is about, counted from 1 with the `# covers:` line as line 1, or `null` |
| `message` | the rest of the finding, in `validate`'s words. The wording is not part of the format |

### `roostr check --json`

| Key | Value |
|---|---|
| `format` | `1` |
| `base` | the value given to `--base`, as written; `"HEAD"` with `--staged`; with `--session`, the full hash of the commit the session started at, `"HEAD"` when it judged as `--base HEAD` (see "`--session`"), and `null` when the session started before the first commit; `null` with none of them |
| `staged` | `true` with `--staged`, otherwise `false` |
| `session` | with `--session`, the `session_id` it read, when it is one roostr uses; otherwise `null`, and `null` without `--session` |
| `problems` | list of problems, below: one per item the text lists, in the text's order. NOT STAGED's first, then VALIDATE's, ENTRYPOINTS', PROCESSES' and NOT DESCRIBED's, each in its block's order. `[]` when check finds nothing and exits 0, also when nothing changed after the base; `null` when check did not judge: `--session` in plan mode |

Every problem has `kind`, and the keys its kind gives:

| `kind` | Other keys | Text block |
|---|---|---|
| `invalid` | `file`, `key`, `line`, `message`: the same object as in `validate --json` | VALIDATE |
| `entrypoints_behind` | `added`, `removed`: lists of `<kind> <name>` strings, each in the file's sort order. Both are `[]` when only a file, an area or the `# covers:` line differs | ENTRYPOINTS |
| `entrypoints_never_written` | none: there is a scan script and no `entrypoints.tsv` | ENTRYPOINTS |
| `scan_not_executable` | none | ENTRYPOINTS |
| `scan_cannot_start` | `interpreter`: the program the script's shebang line names, as written after `#!` up to the end of the line, a carriage return included (`"/bin/sh\r"` for a script with Windows line endings); `null` when the first line is not a shebang line. The script is executable and could not be started | ENTRYPOINTS |
| `scan_failed` | `exit_code`: number; `stderr`: string, all the script wrote to stderr, every line and the final newline included | ENTRYPOINTS |
| `scan_malformed` | `reasons`: list of objects with `line` (number, the line of the script's output) and `message` (the reason, in the words `roostr scan` uses) | ENTRYPOINTS |
| `needs_attention` | `process`, `files`: the same as an item of the overview's `needs_attention` | PROCESSES |
| `not_described` | `entrypoint`: `<kind> <name>` | NOT DESCRIBED |
| `not_staged` | `file`: the path as the text prints it, relative to the project folder (`.roostr/processes/join-waiting-list.yml`) | NOT STAGED |

The paragraphs of the text that say what to do are not in the JSON; run `roostr check`
without `--json` to read them. Whether the scan ran is in neither.

## Names in the text output

Ids, paths, entrypoints (`<kind> <name>`), area and group names, and the names of outcomes,
operations and environments are printed whole: never cut, never shortened with `...`, never
broken across lines, and an entrypoint is never split between its kind and its name. A line
may run past column 80 for them. An agent copies an id from the screen; a cut one names no
process.

Prose (a description, a note, `why`, the condition and result of an outcome, an
environment's value, the paragraphs of `check`, the words after a path in NEEDS ATTENTION)
wraps at spaces within 80 columns. A word longer than the room left, such as a path, starts
a line of its own and is printed whole, past column 80 when it must. An operation's command
is never wrapped: it is copied, not read, and a wrapped one pastes as two lines.

A column of names (ids in FUNCTIONALITY and NEEDS ATTENTION, areas and groups, and the
names of operations, environments and outcomes) is as wide as the longest name it shows,
but never wider than 36 characters, and is followed by its gap. A name longer than 36
characters stands alone on its line, and what would have followed it starts on the next
line, in the column. So one long id does not push every other line on the screen past
column 80.

## Characters in the text output

Text output writes each character that `validate` reports in a string (see "What `roostr
validate` checks") as `<U+`, four to six hexadecimal digits in capitals, and `>`:
`<U+202E>`, `<U+001B>`, `<U+E0041>`. In a key and in a path (a `files` path, a path from
Git), a tab, a carriage return and a newline are written so too (`<U+0009>`, `<U+000D>`,
`<U+000A>`), so that a finding stays on its line. The file a finding is about, and an id,
go further: they write out every character the check on file names can name (see "What
`roostr validate` checks"), that is everything the map removes or changes, U+00AD and
U+200D among them, so that the finding shows where each is: `re<U+00AD>fund-booking.yml`.
This holds for every command and for whatever it writes on stdout or stderr:
the findings of `validate` and `check` and the rest of their report (which the Stop hook
gives the agent), the overview, `show`, `changes`, and the lines on stderr. A written-out
character counts as the characters it is written with (`<U+202E>` as 8) for wrapping and
for the width of a column.

So a raw ESC never reaches the terminal (where it can change colours, hide text, or set
the clipboard), a bidi override never reorders what a person reads, and tag characters
never reach an agent unseen; each shows as the name its finding gives. What people write
on purpose is printed as it is: a tab inside a description, the zero-width joiner of
`👩‍💻`, the marks of Hebrew and Arabic, the flags of England, Scotland and Wales. JSON
output gives every string as it is (JSON escapes control characters itself), and the map
removes these characters (see "Characters" under "The project map").

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | findings, or a refusal: `validate` found problems, `check` found problems (a scan script that fails, prints malformed output, is not executable or cannot be started among them), the id given to `show` does not exist, or `scan` refused what its script did (a non-zero exit, malformed output) |
| 2 | could not run: unknown command, unknown option (`--json` given to `scan`, `init` or `session start` among them), `session` without `start`, missing argument, no `.roostr/project.yml`, `show` asked for a record that cannot be read, `show` was given an option, `scan` found no `.roostr/scan` or one that is not executable or cannot be started, `changes` was run outside a Git repository or with a `--since` value that is neither a number of days (`7d`) nor a date (`2026-10-03`), or `check --base` was run outside a Git repository, without a value, together with `--staged`, or with a base that is not a commit or shares no commit with `HEAD`, or with a base after which the history is not all in a shallow clone, or `check --staged` or `check --session` was run outside a Git repository, or two of `--base`, `--staged` and `--session` were given together, or `changes` or `check` could not read Git or were run in a shallow clone without `--base` (see "When Git cannot be read" and "A shallow clone"), or `check --session` found that `HEAD` shares no commit with the commit the session started at, or a command other than `session start` was run in a folder that no longer exists |

Normal output goes to stdout, as text or, with `--json`, as one JSON object. Reasons for
exit 1 and 2 go to stderr, as text also with `--json`, except `validate` and `check`
findings, which are the command's output and go to stdout.

When stdout is closed before roostr has written all it has to write (`roostr check | head -1`,
once `head` has exited), roostr stops writing, says nothing about it on stderr, and exits
with the code the command would have had: a pipeline run with `pipefail` still sees `check`
fail.

`roostr init` is the one command that runs without `.roostr/project.yml`: it creates it. Run
where the file already exists, it leaves the file alone and still exits 0.
`roostr --version` (or `-V`), which prints `roostr <version>` on one line, and
`roostr --help` also run anywhere, and exit 0. `roostr session start` exits 0 whenever it
ran, also when it could not record, print a map or write its stdout, and when the folder it
was run in no longer exists (see "`roostr session start`").

**`--help`.** `roostr --help`, `roostr -h`, and `--help` or `-h` anywhere among the
arguments of one of roostr's commands (`roostr check --base HEAD --help`,
`roostr show -h`, `roostr --json check --help`) print the usage on stdout and exit 0, and
do nothing else: no project is needed, nothing is read or written. The usage is the same
text for every command, the one `roostr --help` prints; it names every command and option,
`roostr session start` and `check --session` among them. `--help` is never the value of
another option, and after `--` it is an argument like any other: `roostr show -- --help`
looks for the id `--help`. An unknown command stays one: `roostr frobnicate --help` exits
2. Any other option a command does not take exits 2, with the usage on stderr.

`roostr changes` in a Git repository exits 0 also when nothing changed and when the
repository has no commits yet.

`roostr show <id>` with an id that does not exist exits 1 and says so on stderr, in the
line `No process with id "<id>" exists.` When ids are close to it, a second line suggests
them: `Did you mean: book-workshop, cancel-booking?`. An id is close when, with case, `-`
and `_` ignored, it contains the text given; equal counts as containing. At most five, in
alphabetical order, separated by a comma and a space. When none is close, there is no
second line. With `--json` both lines still go to stderr, and stdout stays empty.

The text given to `show` is first taken as a path: any folders in front of it and a final
`.yml` are removed before ids are compared with it, so `show cancel-booking.yml` and
`show .roostr/processes/cancel-booking.yml` (the path `validate` and `check` print) suggest
`cancel-booking`. `show` takes no option but `--json` and `--help`: any other exits 2 and
names it, as for every command. `--` ends the options, so `roostr show -- --verbose` looks
for the id `--verbose`.

## What roostr reads from Git

The overview, `roostr changes` and `roostr check` read the Git repository the project is in:
the commits reachable from `HEAD`, the files each touched, and what is uncommitted in the
working tree. They only read, but for the count of holds `check --session` keeps beside
the session's record. `roostr session start` reads where `HEAD` points, where the Git
folder is, the committer's identity and what is uncommitted, hashes the uncommitted files
without writing an object, and writes its records there (see "`roostr session start`");
its map reads nothing from Git. No other command reads Git. `.roostr/` need not be at the top of the repository; the project
folder is the folder `.roostr/` is in.

- A commit belongs to a **period** by its commit date. A period starts at 00:00:00 on its
  first day and runs until now. A day is a day in the local time zone, the one roostr runs
  under (`TZ`).
- `--since <when>` names the first day of a period: `2026-10-03` is that day, and `7d` is
  the day seven days before today. `--since=<when>` is the same. Without `--since`,
  `roostr changes` means `7d`.
- The entrypoints of "then" are `.roostr/entrypoints.tsv` as it was in the last commit made
  before the period started. The entrypoints of "now" are the file in the working tree,
  committed or not. When either is missing, entrypoints are not compared.
- `roostr check --base <commit>` reads what changed after that commit, as "What
  `roostr check` reports" says.

`roostr changes` lists under PROCESSES each process whose record is in the working tree now,
when a commit in the period touched that record or a file it lists (as the record reads
now). A process whose record was deleted or renamed in the period is not listed under the
id it had.

### When Git cannot be read

roostr runs the `git` it finds on the `PATH`, and needs Git 2.25 or later. Git cannot be
read when `git` is not on the `PATH` (a slim container image), when Git refuses the
repository (for example "detected dubious ownership", for a checkout that belongs to
another user), when it is older than 2.25, or when it fails in any other way inside a Git
repository. Each case has a reason:

- `git` not on the `PATH`: `git was not found on the PATH`;
- Git older than 2.25: that roostr needs Git 2.25 or later, and the version found;
- otherwise: what Git said.

The overview still prints everything it reads from the files and exits 0. It leaves out
LAST 7 DAYS and the processes of NEEDS ATTENTION, and NEEDS ATTENTION says that Git could
not be read and gives the reason, with only the first line of what Git said. Records that
need checking are still listed there, before it. `roostr --json` gives
`needs_attention: null` and everything else.

`roostr changes` and `roostr check` cannot do their work without Git: they exit 2 and give
the reason on stderr, with all that Git said (so that, for a refused repository, the
`git config --global --add safe.directory` command Git suggests is there to copy). `check`
outside a Git repository, without `--base`, still runs as "What `roostr check` reports"
says. `show`, `validate`, `scan` and `init` never read Git.

### A shallow clone

A shallow clone (`git clone --depth 1`, what most CI checkouts make) holds only the newest
commits. Its oldest commits are kept without their parents, and each looks as if it added
every file at once, records included, so that every record would look as new as the code it
describes. In a shallow clone roostr therefore cannot tell which file changed after which
record, and treats Git as not readable, with the reason that the history is shallow, which
also says how to fetch all of it: `git fetch --unshallow`.

- The overview prints, exit 0, as above: no LAST 7 DAYS, no processes in NEEDS ATTENTION,
  and the note with that reason. `roostr --json` gives `needs_attention: null`.
- `roostr changes` exits 2 with that reason.
- `roostr check` without `--base` exits 2 with that reason, and says that `--base` can
  work in a shallow clone.
- `roostr check --base` runs when the history after the base is all in the clone, and
  exits 2 when it is not (see "`--base`").

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
- While a merge is in progress, uncommitted changes are those of the merge commit it will
  make: a file, record or code, has them only where it differs from `HEAD` and from every
  merged commit, and a file taken whole from a merged commit has that commit's history
  (see "A merge in progress" under `roostr check`).

The overview shows these processes under NEEDS ATTENTION and still exits 0. `roostr check`
reports them as problems, under PROCESSES, and exits 1.

One limit: any change to a record counts as an update. roostr does not know whether the
change dealt with the code change.

Whether `entrypoints.tsv` is current is not part of this rule. `roostr check` reports it,
by running `.roostr/scan`.

## Environment

| Variable | Value | Meaning |
|---|---|---|
| `TZ` | a time zone, as the system reads it | the standard variable. It decides where a day starts. The tests set it to `UTC`. |
| `ROOSTR_TODAY` | a date, `YYYY-MM-DD`, or a date and time, `YYYY-MM-DD HH:MM` | roostr takes this as the current date, and time, instead of the machine's clock; a date alone means `00:00` that day. The date decides where the last 7 days and `--since 7d` start, counted in days as ever; date and time decide the value `roostr check` suggests writing as `checked`, and when a session started for `roostr session start`, which removes records more than 30 days old, and which commits `check --session` counts as the session's. |
