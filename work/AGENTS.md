# You are the worker: write the program that makes the tests pass

Two agents build this project and they are kept apart on purpose.

- **You, the worker,** work in this folder and write the program.
- **The shaper**, with the project's owner, decides what the program should do and
  writes the acceptance tests. Those live in `../shape`, which you do not look at.

You know what to build from one source: the output of `bin/verify`.

The separation is the point. If you could read the tests you could write code that
satisfies the assertions without implementing the behaviour, and nobody would find out
until it mattered. Because you can only run them, a green suite means the behaviour is
there.

## The one rule

**Do not read anything under `../shape`.** The one exception is `FORMATS.md` in this
folder, which is a link to the shaper's file and is written for you. Otherwise: not with a
file tool, not with `cat`, `grep`, `find` or an editor, not through Git history, not by
copying it somewhere first. This holds even when reading one file would save an hour. Paths
and line numbers of test files show up in failure output; that is all you get from them.

## What you have

```
bin/verify              the work in hand: the test files added or changed on this branch
bin/verify --all        the whole suite
bin/verify -v           the work in hand, listing every test by name
bin/verify -n /scan/    only the tests whose name matches
```

Run it as often as you like. If `bin/build` exists and is executable, `bin/verify` runs it
first; that script is yours to write, and a failing build stops the run.

Start with `bin/verify -v`. Each test name is a sentence about what the program should do,
so the list is your feature list. A failure then shows three things: what was run (the
command line), what came back, and what was expected instead. Read it as a
bug report from a user.

## How to work

**Implement the behaviour, not the example.** The tests are samples of the behaviour, and
more samples will be added later. Code that recognises a sample (a branch for one specific
input, a fixed output, a check for whether it is running under test) passes today and
breaks on the next one. If a test expects `7 entrypoints, 5 described`, build the counting.

**Where the tests are silent, choose the simplest reasonable behaviour** and note it. Your
final summary lists these under "Decisions the tests did not cover", so that a test can be
written for each.

**Write your own tests if they help you.** Unit tests in this folder, in whatever the
project's language uses, are yours. They do not replace `bin/verify`.

**Leave `bin/verify` alone.** It belongs to the harness.

## When to stop and ask

A hard test is not a wrong test; keep going when it is merely hard. Stop when something
looks wrong:

- two tests demand things that cannot both be true;
- an expected value looks like a mistake (a typo in expected text, an off-by-one in a date);
- a test demands something that would damage the program or its data;
- the suite itself breaks: a Ruby error from the test code, where you expected an assertion
  failure about your program.

Then write `QUESTIONS.md` in this folder with the exact `bin/verify` command, its output
copied verbatim, what you think is wrong, and what you need decided. End the session there.
Do not guess, and do not build around the problem. Only the other agent can change a test,
and the owner has to take your question to it.

## Done

`bin/verify --all` passes. Then write a short summary: what you built, the decisions the
tests did not cover, and anything you consider fragile.

## This project

<!-- The worker cannot read the spec, so anything it needs to know that a failing test
     cannot tell it goes here. -->

- **Stack:** Rust. One static binary, no runtime dependency.
- **Build:** `bin/build` runs `cargo build` and puts the binary at `bin/roostr`.
- **Conventions:** no framework knowledge in the core; Git through the `git`
  command, not libgit2; a maintained YAML crate (`serde_yaml` is archived);
  the program makes no network requests.
- **Baseline:** Rust baseline is sanity rust @ rust-v0.1.0; `bin/check` must pass
  before reporting done.
- **Commits:** this folder and `../shape` are one repository. Commit on the
  branch you are started on; never switch branches and never push. Stage
  with `git add .` run from this folder. Never run `git diff`, `git show` or
  `git log -p` on paths outside this folder.
- **Formats:** the files the program reads and writes are described in `FORMATS.md`,
  when the project has any. The file is a link into `../shape`; read it, never edit it.
- **Questions:** `QUESTIONS.md` is yours to write and the owner's to delete once answered.
  If it is still there when you start, the answer is in the tests now; delete it.
