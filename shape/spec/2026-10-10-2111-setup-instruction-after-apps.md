# Record 2a: the setup instruction after the Apps trial

Date: 2026-10-10 21:11, rewritten 21:13. Supersedes, in record 2
(`2026-10-10-1938-scan-init.md`), the text under "What `init` prints". Everything else in
record 2 stands: what `init` creates, the starter `project.yml`, `create` and `exists`, the
exit codes, `scan`, and the validate fixes. No behaviour of any command changes; only the
words `init` prints.

Record 2 was implemented and a second project, Apps, was set up by an agent that had nothing
but the output of `roostr init`. The setup worked, but the instruction left an unsafe
boundary open (what the agent may run and write) and some rules about the inventory.

This record first held another list of changes, written at 21:11 and never implemented. The
owner replaced that list in full two minutes later; what follows is the replacement. What it
dropped from the first list is named under "Withdrawn".

## Decided

### An execution boundary, a new paragraph before step 1

- The agent may write only under `.roostr/`.
- Before it boots the framework or runs any application code, it reads what booting does. If
  booting may change data (migrations, queue workers, seeds), it stops and asks the owner
  first. If the owner allows it, `watch_out` says what booting changes.
- It never starts the application, runs jobs or changes data to check a `try` recipe. `try`
  is written from the code.

### Step 1, the scan script

- The `# covers:` line names every kind the script looked for, including kinds it found none
  of. Without this an agent drops a kind with no rows, and the overview then reads
  `not scanned` where `none found` is the truth.
- Every row gets an area where one exists: for http the controller or resource name, for a
  job its queue or namespace, for cli its task namespace or folder. This replaces "give
  every http row an area" from the trial, and the earlier description of an area as a
  controller namespace, an app or a folder.
- A row says the entrypoint is declared, not that it works. The file is the one that
  implements it. When that code is in a dependency or missing, the row has the file that
  declares it, and when it is missing `watch_out` says so.
- A route that dispatches from data (a catch-all) is one entrypoint, named as the route is
  written.

The sentences about name, file and area are now a short list of the three columns, so that
someone looking up one column finds its rule.

### Step 3, behaviours

- "Most important" means what the project exists to do: behaviours that change data or serve
  its main content, not shared plumbing.
- `group` is the area of the first entrypoint the record lists. If that entrypoint has no
  area, `group` is left out. This replaces "the area of the process's main entrypoint":
  "first listed" can be checked, "main" could not. A process without a group is listed
  under `other` in the overview, as before.
- Two processes may list the same entrypoint. (`validate` already accepts this, record 2,
  scenario 93.)

The sentences about `group`, `entrypoints` and `try` are now a list of the three keys.

### Closing paragraphs

- "unknown" may be written wherever a value is text.
- The report to the owner has three separate parts: what the agent saw fail, what it
  suspects but did not verify, and what is unknown.
- The last line is: "Do not commit and do not edit AGENTS.md. The owner reviews .roostr/
  first and adds the line."

### One sentence changed that the owner did not ask for

The section above the instruction began "Add this line to AGENTS.md". With the new last line
the output would tell an agent to add the line and, a hundred lines later, not to. It reads
"The owner adds this line to AGENTS.md", which says the same to the owner and agrees with
the last line. The line itself is untouched. Confirmed by the owner at 21:26, together with
the placing of the "unknown" sentence in the closing paragraphs and the dropping of "give
every http row an area" in favour of the new area rule.

### The example follows the area rule

Added at 21:26, before implementation. The job in the step 1 example had the area `Orders`
with nothing in the row to derive it from, which contradicted "for a job its queue or
namespace". The job is now namespaced, so its area follows from the rule:
`job<TAB>Orders::ReceiptJob<TAB>app/jobs/orders/receipt_job.rb<TAB>Orders`. The example
record in step 3 lists `job Orders::ReceiptJob` to match.

## Withdrawn with the first list

- "Booting the framework is allowed when that is the only reliable way to list entrypoints."
  Booting is now something to read about first and, when it may change data, to ask about.
- "A process whose main entrypoint has no area takes the group that fits best." It now
  leaves `group` out.
- "The script must only read" had moved down in step 1 to sit beside the booting sentence.
  It is back where record 2 had it, at the top of the step.

## What `init` prints

On a second run the first line reads `exists  .roostr/project.yml` and the rest is the same.
The line for `AGENTS.md` is one unbroken line; every other line fits 80 columns.

```
create  .roostr/project.yml

AGENTS.md
The owner adds this line to AGENTS.md, so that agents read the overview before
they work on the project:

When you need to understand this project's functionality, structure, or recent changes, run `roostr` for the project overview. Use `roostr show <id>` to inspect a specific process before modifying its behavior.

SETUP
Give this instruction to an agent working in the repository, or follow it
yourself.

Set up roostr for this project. Do the steps in order. Where something cannot
be established from the code or from the owner, write "unknown". Never guess.

You may write only under .roostr/. Before you boot the framework or run any
application code, read what booting does. If it may change data (migrations,
queue workers, seeds), stop and ask the owner first. If the owner allows it,
say in watch_out what booting changes. Never start the application, run jobs
or change data to check a try recipe. Write try from the code.

1. Write the scan script first.

   Create .roostr/scan and make it executable. It may be written in any
   language. The script must only read. It must not write files, change a
   database or call the network. roostr runs it from the repository root and
   reads what it prints:

     # covers: http job mail
     http<TAB>POST /orders<TAB>app/controllers/orders_controller.rb<TAB>Orders
     job<TAB>Orders::ReceiptJob<TAB>app/jobs/orders/receipt_job.rb<TAB>Orders
     mail<TAB>OrderMailer#receipt<TAB>app/mailers/order_mailer.rb

   The first line names every kind the script looked for, out of http, job,
   mail, schedule, event and cli, including kinds it found none of. Every
   other line is one entrypoint: kind, name, file and area, separated by one
   tab character each, shown above as <TAB>. A row says the entrypoint is
   declared, not that it works.

   - name: a route that dispatches from data (a catch-all) is one entrypoint,
     named as the route is written.
   - file: the file that implements the entrypoint. When that code is in a
     dependency or missing, use the file that declares it, and if it is
     missing say so in watch_out.
   - area: give every row an area where one exists: for http the controller
     or resource name, for a job its queue or namespace, for cli its task
     namespace or folder. A row without an area has three columns.

   Take the list from the framework (its routes command, its job and mailer
   classes), not from reading files by hand.

   Run `roostr scan` and correct the script until it succeeds. It writes
   .roostr/entrypoints.tsv, which is never edited by hand.

2. Fill in .roostr/project.yml.

   The comments in the file describe each key. Keep watch_out to what someone
   must know before touching the project at all. What concerns one behaviour
   belongs in that behaviour's record.

3. Describe the three most important behaviours first.

   Most important means what the project exists to do: behaviours that
   change data or serve its main content, not shared plumbing. Each behaviour
   is one file, .roostr/processes/<id>.yml, and the file name is its id. Only
   name and description are required:

     name: Place an order
     group: Orders
     description: Take a paid order and mail the receipt.
     entrypoints:
       - http POST /orders
       - job Orders::ReceiptJob
       - mail OrderMailer#receipt
     files:
       - path: app/models/order.rb
         note: holds the order states
       - app/services/place_order.rb
     try: |
       Put a mug in the basket and pay with card 4242 4242 4242 4242.
       Check the receipt in the mail preview.
     outcomes:
       paid:
         when: the payment succeeds
         then: the order is saved and the receipt is mailed
       declined:
         when: the card is declined
         then: nothing is saved
     why:
       - Stock is reserved only after payment, so an abandoned basket holds
         nothing back.

   - group: the area of the first entrypoint the record lists, spelled as in
     entrypoints.tsv. If that entrypoint has no area, leave group out.
   - entrypoints: list every entrypoint the behaviour starts from or runs
     through (the request that starts it, the jobs it enqueues, the mails it
     sends), each written as its kind and name. Two processes may list the
     same entrypoint.
   - try: keep each line to 69 characters; it is printed as written.

   Read each record back with `roostr show <id>`.

4. Finish with `roostr validate`.

   Correct every finding until it prints "No problems found". Then run
   `roostr` and read the overview as someone new to the project would.

"unknown" may be written wherever a value is text.

Stop here. Tell the owner what you could not establish and which areas have no
process yet, and let them choose what to describe next. The report has three
separate parts: what you saw fail, what you suspect but did not verify, and
what is unknown.

Do not commit and do not edit AGENTS.md. The owner reviews .roostr/ first and
adds the line.
```

## Tests altered by this record

No test file changed and no scenario was added. Two tests compare the whole output of `init`
with `test/expected/init_guidance.txt`, which was rewritten, so both now expect the text
above:

- `test/init/init_test.rb`: "init prints create and the file it created, then the line for
  AGENTS.md and the setup instruction" (scenario 88).
- `test/init/init_test.rb`: "run in a project that has .roostr/project.yml, init succeeds,
  prints exists instead of create, the same line and instruction, and overwrites nothing"
  (scenario 91).

FORMATS.md is unchanged.

## Left out of this record

Checking any of the new guidance by program. Nothing in roostr verifies that an agent wrote
only under `.roostr/`, asked before booting, chose the area or the file as described, or set
`group` to the area of the first entrypoint. `scan` and `validate` check what they checked
before.

## Still open

The owner looked at the first two on 2026-10-10 and left them as they are.

- Whether `validate` should check `group` against the area of the record's first entrypoint,
  now that the rule can be checked. Not asked for; it would turn every hand-written group on
  the trial project into a finding.
- "unknown" wherever a value is text: `validate` still reports a `files` path or an
  `entrypoints` reference written as `unknown`, because the first is not a file and the
  second is not in `entrypoints.tsv`. Step 4 then has the agent correct the finding, which in
  practice means removing the item.
- The label `other` (carried over from record 1a).
