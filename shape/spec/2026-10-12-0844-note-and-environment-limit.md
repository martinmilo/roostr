# Record 3a: an operation note and an environment value are at most 160 characters

Date: 2026-10-12 08:44. Extends record 1b (`2026-10-10-1547-trial-defects.md`), which limited
a `watch_out` item to 160 characters, and record 2a (`2026-10-10-2111-setup-instruction-after-apps.md`),
which fixed the words `init` prints. Everything else in records 1 to 3 stands. No command
gains or loses a behaviour; `validate` gains one check and `init` gains a few words.

## The problem this record serves

Real use on a trial project found a gap. An agent updated the overview and three values grew to four or
five lines each: the `setup` and `test` operation notes and the `test` environment value.
WATCH OUT stayed readable because its items are limited to 160 characters. Notes and
environment values had no limit, so all the detail went there. The limit on `watch_out`
moved the problem instead of solving it.

## Decided

### The limit

An operation note (`operations.<name>.note`) and an environment value
(`environments.<name>`) are each at most 160 characters, the same limit as a `watch_out`
item, counted the same way. 160 is allowed; 161 is not.

An operation note says what someone must know before running the command: a password it
asks for, how long it takes, what it must not be run against. An environment value says
where the project runs. Neither is the place for a procedure. A procedure about one
behaviour belongs in that behaviour's record; a procedure that concerns the whole project
belongs in `watch_out`, within its limit, or outside roostr altogether.

### `validate`

`validate` reports a longer one the same way it reports a `watch_out` item that is too
long: one finding naming `project.yml`, the key as a dotted path from the top of the file,
and the limit of 160. The key of a note is `operations.<name>.note`
(`operations.test.note`); the key of an environment value is `environments.<name>`
(`environments.staging`). Two values that are too long are two findings in one run.

The overview prints whatever is there, wrapped as records 1a and 1b lay out; only
`validate` complains. This is the rule record 1b set for `watch_out` and it does not
change.

### `init`

The starter `project.yml` states the limit in the comments above `operations` and
`environments`, as it already does above `watch_out`. Step 2 of the setup instruction
("Fill in .roostr/project.yml") defers to those comments, so they are where an agent reads
the rule. The step also states it in its own words, so the limit is in the instruction an
owner pastes to an agent and not only in a file the agent may skim.

The two comments in the starter file:

```yaml
# How to work with the project. A command, or a command with a note of at
# most 160 characters.
# operations:
#   start: bin/dev
#   test: bin/rails test
#   deploy:
#     command: bin/deploy
#     note: asks for the production password

# Where the project runs, one line of at most 160 characters each.
# environments:
#   staging: staging.example.com
#   production: example.com
```

Step 2 of the setup instruction:

```
2. Fill in .roostr/project.yml.

   The comments in the file describe each key. Keep watch_out to what someone
   must know before touching the project at all. What concerns one behaviour
   belongs in that behaviour's record. An operation's note and an environment
   are each at most 160 characters; a procedure does not belong in either.
```

Every line of both fits 80 columns. Nothing else `init` prints or creates changes.

## Scenario list

### validate

60. An operation note of 160 characters is accepted.
61. An operation note of 161 characters is a finding naming `project.yml`,
    `operations.test.note` and the limit of 160.
62. An environment value of 160 characters is accepted.
63. An environment value of 161 characters is a finding naming `project.yml`,
    `environments.staging` and the limit of 160.
64. A note and an environment value that are both too long are two findings in one run,
    one naming `operations.test.note` and one naming `environments.staging`.

### init

65. The starter `project.yml` states the limit above `operations` and above `environments`.
    Covered by the existing test "in an empty folder called pottery-studio, init creates
    .roostr/project.yml with name: pottery-studio and every other key as a commented
    example, and nothing else"; its expected file `test/expected/starter_project.yml`
    changes.
66. Step 2 of the setup instruction states the limit. Covered by the existing tests "init
    prints create and the file it created, then the line for AGENTS.md and the setup
    instruction" and "run in a project that has .roostr/project.yml, init succeeds, prints
    exists instead of create, the same line and instruction, and overwrites nothing"; their
    expected file `test/expected/init_guidance.txt` changes.

No new test for `init`: the two expected files are compared whole, and a test of its own
would repeat them.

## Existing tests this record alters

- `test/init/init_test.rb`: the three tests named under scenarios 65 and 66 keep their
  names and their meaning; the expected text they compare with gains the words above.
- `test/overview/work_with_section_test.rb`: unchanged. Its note of 123 characters is within
  the limit, and the overview wraps a long note regardless of `validate`.
- `test/validate/validate_test.rb`: gains scenarios 60 to 64. Nothing there changes.

## Fixture additions

- `test/fixtures/project/`: `note-160.yml`, `note-161.yml`, `environment-160.yml`,
  `environment-161.yml` and `note-and-environment-161.yml`. Each is Atelier's `project.yml`
  with the one value at the length its name gives; the pair for scenario 64 breaks both
  rules and nothing else.

## FORMATS.md

Three lines change: the `operations` row and the `environments` row of the `project.yml`
table say "at most 160 characters", and the list of what `validate` checks gains the two
keys. It needs copying into `app/` again.

## Left out of this record

- A limit on an operation's command, on `stack` or on `description`. The trial project showed no problem
  with them.
- A note or an environment value written over several lines. FORMATS.md says "one line";
  nothing checks it, before or after this record.
- How the overview prints an environment value longer than its column. Record 1b fixed the
  ENVIRONMENTS layout for one-line values that fit; a value near 160 characters does not
  fit, and no record says whether it wraps like a note. Raise it when it happens.

## Still open

- `roostr changes` lists changed process records, but a change to `project.yml` (a new
  `watch_out` item, a new operation, a changed note) is not shown anywhere. Recorded here
  without tests. Carried over from record 3: whether the overview should treat a record
  with an unknown key as unreadable; the label `other`.

## Added on confirmation

The owner confirmed the list above and added one scenario. The check it asked for found that
the existing `watch_out` limit tests use only ASCII, so the same scenario was added for
`watch_out` too.

67. An operation note of 160 characters that contains non-ASCII characters (é, €) is
    accepted: the limit counts characters, not bytes. The fixture value is 160 characters
    and 163 bytes.
68. A `watch_out` item of 160 characters that contains non-ASCII characters is accepted,
    for the same reason. The fixture value is 160 characters and 164 bytes.

Test names give the byte count rather than the characters, because Minitest drops é and €
from a method name and the implementer reads the name.

Fixtures: `test/fixtures/project/note-160-non-ascii.yml` and
`test/fixtures/project/watch-out-160-non-ascii.yml`.

## Tests that pass before any work is done

Scenarios 60, 62, 67 and 68 pass today: the program accepts what it does not check. They
are there so that the limit, once added, is not drawn too tight.
