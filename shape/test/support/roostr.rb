# The commands of this project, each run in the test's workspace.
#
#   overview               roostr with no arguments
#   show "book-workshop"   roostr show <id>; `show` alone runs it without an id
#   validate               roostr validate
#   scan                   roostr scan
#   init                   roostr init
#   changes                roostr changes
#   check                  roostr check; check from: "app/models" runs it in a subfolder
#
# Each passes further arguments on: changes "--since", "3d" · check "--base", "HEAD" ·
# overview "--json" · show "book-workshop", "--json". And the options of cli: check
# stdout: :closed runs it with a stdout whose reader has already gone.
#
# The overview, changes and check read the date. Their clock is always pinned, so a test never
# changes its answer as real time passes: today is 2026-10-10 unless the test says
# today: "2026-11-02", and a day starts at 00:00 UTC on every machine.
#
#   atelier_under_git                           the Atelier fixture as a Git repository with one commit
#   atelier_after_the_waiting_list_week         ...and a week of commits on top
#   atelier_halfway_through_the_waiting_list    ...and an agent's uncommitted work on top of a scan script
#                                               scanned: false: entrypoints.tsv as committed, not scanned since
#   atelier_inside_a_larger_repository          Atelier in apps/atelier, Git at the top, one commit;
#                                               folder: "apps/my atelier" puts it there instead
#   the_waiting_list_added_with_its_record      the waiting-list code, entrypoints and join-waiting-list.yml
#                                               written, nothing staged
#   record_checked "remind-attendees", "2026-10-10 14:30"   the record gets that checked value, committed
#   merging_main on_main: [...], on_the_branch: [...]   each side commits a change to those files, then
#                                               the branch trainer-page merges main and Git stops before
#                                               committing: on a conflict where both changed a file
#   take_mains_side "app/models/booking.rb" · resolve_by_hand "app/models/booking.rb"   ...and staged
#   cherry_picking_a_change_to "app/models/booking.rb"   a cherry-pick stopped on a conflict, resolved
#                                               to the picked change
#   main_described_the_workshop_list_after_the_branch_left   after atelier_under_git
#   on_a_branch_after_main_moved_on             after atelier_under_git: a branch checked out, main moved on
#   a_side_branch_merged_into_waiting_list      after atelier_under_git: main moved on, then a branch of four
#                                               commits, the first changing booking.rb, merged into waiting-list
#   emacs_is_editing "cancel-booking"           the lock link Emacs keeps beside a file it has open
#   reminder_job_changed_and_committed          booking_reminder_job.rb changed and committed, its record
#                                               remind-attendees.yml not: what an agent that commits leaves
#                                               behind; on 2026-10-10 at noon, in a session of that day
#
# roostr scan and roostr check run the project's own script, .roostr/scan. In a test that
# script is a fake:
#
#   scan_prints "# covers: http\n"                      a script that prints this and exits 0
#   scan_prints text, stderr: "no database", exit: 3    ...and writes to stderr, and exits 3
#   scan_script "leaves-a-trace.sh"                     test/fixtures/scans/leaves-a-trace.sh
#   commit_scan_prints text · commit_scan_script name   ...and commit it, so that it is part of HEAD
#
# roostr finds git on the PATH. For the ways that can fail, a test changes the PATH roostr runs
# with; the tests' own git (commit, branch, ...) stays the real one:
#
#   git_not_on_the_path           the PATH holds sh, cat and the like, and no git
#   git_refuses_the_repository    git is test/fixtures/git/refuses-the-repository
#   git_older_than_supported      git is test/fixtures/git/version-2.20.0
#
# Git can run roostr itself, from a hook. Then the command under test is git commit:
#
#   pre_commit_hook "roostr check --staged"    .git/hooks/pre-commit runs this; roostr is on its PATH
#   git_commit "-a", "-m", "Add the waiting list"   git commit, as a person types it; from: "apps/atelier"
#   assert_commit_made · refute_commit_made    HEAD moved on, or stayed where it was
#
# An agent's session: Claude Code runs a hook's command with JSON about the session on stdin,
# session_id among it. The two commands of the hooks, each given that input:
#
#   session_start                  roostr session start, as the SessionStart hook runs it, for
#                                  the session SESSION; session_start "session-two" for another
#   session_start source: "resume"     ...as when the session is resumed (the same id)
#   session_start today: "2026-09-09"  ...on another day: the clock is pinned as for check
#   session_start stdin: "text"    ...with this on stdin instead of the hook's input
#   session_start source: nil      ...with no source in the input; source: 42 gives a number
#   session_start env: { "COLUMNS" => "40" }   ...with these environment variables as well
#   session_start stdin_held_open: true   ...and stdin kept open after the input, until it exits
#   check_session                  roostr check --session, as the Stop hook runs it; id: "session-two",
#                                  stdin: "text", stdin_held_open: true, and further arguments:
#                                  check_session "--json"; permission_mode: "plan", the mode Claude
#                                  Code is in; input_bytes: 16_777_216, an input of exactly that size
#   session_held 3                 check --session run 3 times, each holding (exit 1), nothing changed
#   session_start env: a_git_whose_status_never_answers    ...with a git that hangs on git status
#
# What happens around the agent during a session. The kit commits as tests@example.com, also for
# the agent; a session starts on 2026-10-10 at 00:00 UTC:
#
#   teammate_pushed_a_change_to "app/models/booking.rb"   Dana Whitfield's commit on origin/main,
#                                  fetched, not merged; on: (2026-10-09 unless said), branch:
#                                  "waiting-list" for origin/waiting-list, fetched_from_a_url: true
#                                  for FETCH_HEAD only, as git pull <url> <branch> leaves it
#   pull_request_squash_merged_on_github "app/models/booking.rb"   a commit by GitHub on origin/main,
#                                  at 10:00 during the session
#   agent_runs_git "rebase", "origin/main"   git as the agent runs it: commits dated 2026-10-10 12:00
#   rebase_onto_origin_main_resolving_a_conflict_in "app/x.rb"   ...stopped on a conflict, resolved
#   the_user_left_edited "app/x.rb"   the user's own change, uncommitted, before the session starts
#   saved_again_unchanged "app/x.rb"   the same content saved again: a later modification time only
#   formatted "app/x.rb"           a formatter re-saved it with # frozen_string_literal: true on top
#   edited_to_a_size_of "app/x.rb", 8_388_608   edited to exactly that many bytes
#   nested_repository "vendor/kiln-firmware"   a Git repository of its own inside the work tree
#   untracked_files_under_a_clean_filter   .gitattributes and a .psd under a Git LFS-style clean
#                                  filter that leaves the file filter-ran when it runs
#   claude_code_runs "Stop", "roostr check --session ..."   a hook's whole command, as Claude Code
#                                  runs it: bash -c, with the input on stdin and roostr first on
#                                  the PATH; from: "apps/storefront" runs it in that folder
#
# What session start prints for the agent's context, the project map: one line of JSON,
# {"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"<the map>"}}.
# Each assertion first checks that stdout is that line, keys and order included, and says
# which part is not; then it reads the map, the text of additionalContext once decoded:
#
#   assert_map "Stack: Rails 8"      the map includes this text; a pattern also works
#   refute_map "deploy"              ...does not include it
#   assert_map_is expected("session_map_atelier.txt")   the whole map, character for character.
#                                    The file holds the map exactly: no newline at its end
#   assert_map_at_most 8000          the map is at most this many bytes, as UTF-8
#
# Records for the map to list, beside Atelier's three:
#
#   records_added "sync-calendar", "import-workshops"   test/fixtures/processes/<id>.yml, each
#                                  written as .roostr/processes/<id>.yml
#   processes_written 200, groups: 6   process-001 to process-200, the first in Group A, the
#                                  next in Group B, and so on through Group F, then Group A
#                                  again; each described "Step <n> of the nightly import."
#                                  and listing app/imports/step_<n>.rb and
#                                  app/imports/step_<n>_check.rb, files that do not exist.
#                                  Nothing is committed
#   processes_written 300, groups: 6, id: "nightly-import-of-partner-workshops"   ...with ids
#                                  nightly-import-of-partner-workshops-001 and so on
#   processes_written 350, groups: nil, id: "nightly-import"   ...with no group: all under other
module Roostr
  TODAY = "2026-10-10"
  SESSION = "5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f" # a session id as Claude Code gives it: a UUID
  # A session in the tests starts on TODAY at 00:00 UTC; what the agent commits is dated at noon.
  SESSION_COMMIT_DATE = { "GIT_COMMITTER_DATE" => "2026-10-10T12:00:00+00:00" }.freeze
  TEAMMATE = { "GIT_AUTHOR_NAME" => "Dana Whitfield", "GIT_AUTHOR_EMAIL" => "dana.whitfield@example.com",
               "GIT_COMMITTER_NAME" => "Dana Whitfield", "GIT_COMMITTER_EMAIL" => "dana.whitfield@example.com" }.freeze

  def overview(*args, today: TODAY, **options) = cli(*args, env: clock(today), **options)
  def changes(*args, today: TODAY, **options)  = cli("changes", *args, env: clock(today), **options)
  def check(*args, today: TODAY, **options)    = cli("check", *args, env: clock(today), **options)
  def show(*args, **options)     = cli("show", *args, **options)
  def validate(*args, **options) = cli("validate", *args, **options)
  def scan(*args, **options)     = cli("scan", *args, **options)
  def init(*args, **options)     = cli("init", *args, **options)

  def scan_prints(output, stderr: nil, exit: 0)
    script = ["#!/bin/sh"]
    script += ["cat <<'END_OF_OUTPUT'", output.chomp, "END_OF_OUTPUT"] unless output.empty?
    script += ["cat >&2 <<'END_OF_ERRORS'", stderr, "END_OF_ERRORS"] if stderr
    write_executable ".roostr/scan", [*script, "exit #{exit}"].join("\n") + "\n"

    printed = output.empty? ? "  (nothing)" : output.chomp.gsub("\t", "→").gsub(/^/, "  ")
    note_situation ".roostr/scan is an executable that #{"writes #{stderr.inspect} to stderr, " if stderr}" \
                   "exits #{exit} and prints this to stdout (→ is a tab character):\n#{printed}"
  end

  # The comment under the script's first line says what it does, and is shown when a test fails.
  def scan_script(name)
    source = fixture("scans/#{name}")
    write_executable ".roostr/scan", source
    note_situation ".roostr/scan is an executable that #{source.lines.drop(1).take_while { |line| line.start_with?("#") }.map { |line| line.delete_prefix("#").strip }.join(" ")}"
  end

  # The same fakes, committed, for a check that judges only the work after HEAD.
  def commit_scan_prints(output, **options)
    scan_prints(output, **options)
    commit "Add the scan script"
  end

  def commit_scan_script(name)
    scan_script name
    commit "Add the scan script"
  end

  # Atelier as a Git repository with one commit, dated 2026-01-15. A failure then says which
  # process lists which file, which the transcript of a command cannot show.
  def atelier_under_git
    workspace "atelier", git: true
    note_situation "In this project book-workshop lists app/models/booking.rb and app/services/book_workshop.rb, " \
                   "cancel-booking lists app/models/booking.rb, and remind-attendees lists app/jobs/booking_reminder_job.rb."
  end

  # Atelier, committed on 2026-01-15, then six commits. With today on 2026-10-10 the last 7 days
  # start on 2026-10-03, so the first of the six is outside them. Nothing is left uncommitted.
  def atelier_after_the_waiting_list_week
    atelier_under_git

    edit_file "app/services/book_workshop.rb"
    commit "Round prices to whole euros", on: "2026-09-24"

    edit_file "app/models/booking.rb"
    write_file "app/controllers/waiting_list_controller.rb", "# Placeholder standing in for app/controllers/waiting_list_controller.rb in the Atelier fixture.\n"
    write_file "app/jobs/waiting_list_job.rb", "# Placeholder standing in for app/jobs/waiting_list_job.rb in the Atelier fixture.\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")
    commit "Add the waiting list", on: "2026-10-05"

    edit_file "app/controllers/bookings_controller.rb"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/booking-page-dropped.tsv")
    commit "Drop the booking page", on: "2026-10-06"

    edit_file "app/models/booking.rb"
    edit_file ".roostr/processes/cancel-booking.yml"
    commit "Offer a freed place to the next person waiting", on: "2026-10-07"

    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-08"

    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-09"
  end

  # An agent halfway through adding a waiting list, nothing of it committed yet. HEAD is Atelier
  # with a scan script that finds the waiting list and no longer finds the reminder mail.
  # Uncommitted: booking.rb edited, the waiting-list controller and job added, entrypoints.tsv
  # rewritten by a scan run while the reminder mail still existed, then booking_mailer.rb edited.
  # scanned: false leaves entrypoints.tsv as committed: the scan would now add http POST
  # /waiting_list (waiting_list_controller.rb) and job WaitingListJob (waiting_list_job.rb), and
  # drop mail BookingMailer#reminder (booking_mailer.rb).
  def atelier_halfway_through_the_waiting_list(scanned: true)
    atelier_under_git
    commit_scan_prints fixture("scans/atelier-with-waiting-list.txt")

    edit_file "app/models/booking.rb"
    write_file "app/controllers/waiting_list_controller.rb", "# Placeholder standing in for app/controllers/waiting_list_controller.rb in the Atelier fixture.\n"
    write_file "app/jobs/waiting_list_job.rb", "# Placeholder standing in for app/jobs/waiting_list_job.rb in the Atelier fixture.\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") if scanned
    edit_file "app/mailers/booking_mailer.rb"
    if scanned
      note_situation ".roostr/entrypoints.tsv now holds the 7 entrypoints of Atelier and http POST /waiting_list and job WaitingListJob; " \
                     "at HEAD it held only the 7. No record lists the two new ones."
    else
      note_situation "Not committed: booking.rb and booking_mailer.rb edited, waiting_list_controller.rb and waiting_list_job.rb new. " \
                     ".roostr/entrypoints.tsv is as committed: not scanned since. The committed scan script now prints " \
                     "+ http POST /waiting_list (waiting_list_controller.rb), + job WaitingListJob (waiting_list_job.rb) and " \
                     "drops mail BookingMailer#reminder (booking_mailer.rb), compared with entrypoints.tsv."
    end
  end

  # After atelier_under_git: work on a branch while main moves on. The branch trainer-page left
  # main at Atelier's first commit and has one commit of its own, to workshops_controller.rb,
  # which no record lists. Then main dropped the booking page: GET /bookings/:id is gone from
  # main's entrypoints.tsv. trainer-page is checked out, and its entrypoints.tsv still has it.
  def on_a_branch_after_main_moved_on
    branch "trainer-page"
    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-06"
    switch_to "main"
    edit_file "app/controllers/bookings_controller.rb"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/only-booking-page-dropped.tsv")
    commit "Drop the booking page", on: "2026-10-07"
    switch_to "trainer-page"
    note_situation "trainer-page is checked out. It left main at the initial commit. After that, main dropped http GET /bookings/:id " \
                   "from its entrypoints.tsv; trainer-page still has it, and no record lists it."
  end

  # After atelier_under_git: main gets one more commit, to workshops_controller.rb. From there
  # the branch waiting-list starts, and from it waiting-list-job, with four commits: the first
  # changes app/models/booking.rb, the other three bookings_controller.rb, which no record
  # lists. waiting-list-job is merged into waiting-list with a merge commit, and waiting-list
  # is checked out. No record was touched after Atelier's first commit.
  def a_side_branch_merged_into_waiting_list
    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-01"
    branch "waiting-list"
    branch "waiting-list-job"
    edit_file "app/models/booking.rb"
    commit "Keep a waiting list for each workshop", on: "2026-10-02"
    edit_file "app/controllers/bookings_controller.rb"
    commit "Show the waiting list on the booking page", on: "2026-10-03"
    edit_file "app/controllers/bookings_controller.rb"
    commit "Offer a freed place to the next person waiting", on: "2026-10-04"
    edit_file "app/controllers/bookings_controller.rb"
    commit "Let a person leave the waiting list", on: "2026-10-05"
    switch_to "waiting-list"
    merge "waiting-list-job", on: "2026-10-06"
    note_situation "waiting-list is checked out. It left main at main's newest commit and then merged waiting-list-job, " \
                   "a branch of four commits whose first changed app/models/booking.rb; no record was changed after Atelier's first commit."
  end

  # Emacs, while it has a file open with unsaved changes, keeps a lock beside it: a symbolic
  # link named .#<file> that points to no file, only to who holds the lock.
  def emacs_is_editing(id)
    File.symlink("dev@workstation.example.com.48213:1730851200", File.join(dir, ".roostr/processes/.##{id}.yml"))
    note_situation ".roostr/processes/.##{id}.yml is the lock Emacs keeps while #{id}.yml is open in it: a symbolic link to nothing."
  end

  # A change to app/jobs/booking_reminder_job.rb, which remind-attendees lists, committed
  # without touching remind-attendees.yml, on 2026-10-10 at noon UTC: after a session that
  # starts that day (at 00:00) has started, by the committer the kit always uses.
  def reminder_job_changed_and_committed
    edit_file "app/jobs/booking_reminder_job.rb"
    git "add", "--", "app/jobs/booking_reminder_job.rb"
    git "commit", "--quiet", "--message", "Send the reminder two days ahead", "--", "app/jobs/booking_reminder_job.rb", env: commit_date("2026-10-10")
    note_situation "A change to app/jobs/booking_reminder_job.rb was committed (\"Send the reminder two days ahead\", 2026-10-10 12:00 UTC, " \
                   "by tests@example.com); remind-attendees.yml, which lists it, was not changed."
  end

  # A repository that holds more than one project: Atelier in apps/atelier, with its .roostr/
  # there, and apps/storefront beside it. Committed once. Paths in Atelier's records are
  # relative to apps/atelier.
  def atelier_inside_a_larger_repository(folder: "apps/atelier")
    workspace "atelier", into: folder
    write_file "apps/storefront/README.md", "# Storefront\n\nThe shop's public pages, a project of its own.\n"
    start_repository
    note_situation "The top of the Git repository holds two folders: #{folder}, where .roostr/ is, and apps/storefront. " \
                   "In Atelier remind-attendees lists app/jobs/booking_reminder_job.rb, a path relative to #{folder}."
  end

  # An agent added the waiting list and described it, and has staged nothing: the controller
  # and the job are new, entrypoints.tsv lists their two entrypoints, and the new record
  # join-waiting-list.yml lists both.
  def the_waiting_list_added_with_its_record
    write_file "app/controllers/waiting_list_controller.rb", "# Placeholder standing in for app/controllers/waiting_list_controller.rb in the Atelier fixture.\n"
    write_file "app/jobs/waiting_list_job.rb", "# Placeholder standing in for app/jobs/waiting_list_job.rb in the Atelier fixture.\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")
    write_file ".roostr/processes/join-waiting-list.yml", fixture("processes/join-waiting-list.yml")
    note_situation "Not committed: app/controllers/waiting_list_controller.rb and app/jobs/waiting_list_job.rb are new, " \
                   ".roostr/entrypoints.tsv now also lists http POST /waiting_list and job WaitingListJob, and the new record " \
                   ".roostr/processes/join-waiting-list.yml lists both."
  end

  # Someone compared the record with the code and wrote down when: checked: <value>, replacing
  # a checked line the record has, and committed.
  def record_checked(id, value)
    path = ".roostr/processes/#{id}.yml"
    write_file path, read_file(path).sub(/^checked:.*\n/, "") + "checked: #{value}\n"
    commit "Confirm #{id} against the code", on: "2026-10-09"
    note_situation "#{path} holds checked: #{value}, committed."
  end

  # After atelier_under_git: the branch trainer-page commits a change to each file of
  # on_the_branch, then main a change to each file of on_main, each side a line of its own at
  # the end. trainer-page is checked out and merges main; Git stops before committing, on a
  # conflict in a file both sides changed.
  def merging_main(on_main:, on_the_branch:)
    branch "trainer-page"
    on_the_branch.each { |path| write_file path, "#{read_file(path)}# Changed on trainer-page.\n" }
    commit "Work on the trainer page", on: "2026-10-06"
    switch_to "main"
    on_main.each { |path| write_file path, "#{read_file(path)}# Changed on main.\n" }
    commit "Work on main", on: "2026-10-07"
    switch_to "trainer-page"
    Open3.capture2e(Workspace::GIT_ENV, "git", "merge", "--no-ff", "--no-commit", "main", chdir: dir)
    raise "merging_main: Git did not leave a merge in progress" unless File.exist?(File.join(dir, ".git/MERGE_HEAD"))
    conflicts = git("diff", "--name-only", "--diff-filter=U").split("\n")
    note_situation "trainer-page changed #{on_the_branch.join(", ")} in a commit; main then changed #{on_main.join(", ")} in one. " \
                   "trainer-page is checked out, merging main, and Git stopped before committing" \
                   "#{conflicts.empty? ? "" : ", on a conflict in #{conflicts.join(", ")}"}: a merge is in progress."
  end

  # A conflict settled the simple way: the file as main has it, staged.
  def take_mains_side(path)
    git "checkout", "--theirs", "--", path
    git "add", "--", path
    note_situation "The conflict in #{path} is resolved by taking main's side, and staged."
  end

  # A conflict settled by writing a version of the file that neither side had: both changes kept.
  def resolve_by_hand(path)
    write_file path, git("show", ":1:#{path}") + "# Changed on trainer-page.\n# Changed on main.\n"
    git "add", "--", path
    note_situation "The conflict in #{path} is resolved by hand, keeping both sides' lines: a version that differs from both sides. Staged."
  end

  # After atelier_under_git: main and the branch longer-bookings each change the file in a
  # commit. On main, git cherry-pick longer-bookings stops on a conflict; it is resolved to the
  # picked change and staged, and the cherry-pick is still in progress.
  def cherry_picking_a_change_to(path)
    branch "longer-bookings"
    write_file path, "#{read_file(path)}# Changed on longer-bookings.\n"
    commit "Let a booking last a whole day", on: "2026-10-06"
    switch_to "main"
    write_file path, "#{read_file(path)}# Changed on main.\n"
    commit "Work on main", on: "2026-10-07"
    Open3.capture2e(Workspace::GIT_ENV, "git", "cherry-pick", "longer-bookings", chdir: dir)
    raise "cherry_picking_a_change_to: Git did not stop the cherry-pick" unless File.exist?(File.join(dir, ".git/CHERRY_PICK_HEAD"))
    git "checkout", "--theirs", "--", path
    git "add", "--", path
    note_situation "main is checked out, cherry-picking longer-bookings' change to #{path}; the conflict is resolved to the picked change " \
                   "and staged, and the cherry-pick is in progress."
  end

  # After atelier_under_git: the branch trainer-page left main and changed
  # workshops_controller.rb. Then main described http GET /workshops, which no record listed,
  # in the new record list-workshops.yml. trainer-page is checked out and does not have it.
  def main_described_the_workshop_list_after_the_branch_left
    branch "trainer-page"
    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-06"
    switch_to "main"
    write_file ".roostr/processes/list-workshops.yml", fixture("processes/list-workshops.yml")
    commit "Describe the workshop list", on: "2026-10-07"
    switch_to "trainer-page"
    note_situation "trainer-page is checked out. After it left main, main added the record list-workshops.yml, which lists " \
                   "http GET /workshops; at the commit where trainer-page left, no record listed it, and trainer-page has no list-workshops.yml."
  end

  # Git runs .git/hooks/pre-commit before it makes a commit, at the top of the work tree. The
  # roostr under test comes first on the PATH the hook runs with.
  def pre_commit_hook(script)
    write_executable ".git/hooks/pre-commit", "#!/bin/sh\n#{script}\n"
    roostr_folder = File.dirname(Target.path(PROGRAM))
    environment_for_commands "PATH" => [roostr_folder, ENV.fetch("PATH")].join(File::PATH_SEPARATOR)
    note_situation "Git runs .git/hooks/pre-commit before each commit, and roostr is the program under test, first on the PATH:\n" \
                   "  #!/bin/sh\n  #{script}"
  end

  # The real git commit, as a person types it.
  def git_commit(*args, from: nil)
    @head_before_commit = head_commit
    execute installed("git"), ["commit", *args], shown_as: "git", from: from, env: clock(TODAY)
  end

  def assert_commit_made
    assert head_commit != @head_before_commit, "Expected git commit to make a commit, but HEAD is where it was.\n\n#{finished}"
  end

  def refute_commit_made
    assert head_commit == @head_before_commit, "Expected git commit to make no commit, but HEAD moved on.\n\n#{finished}"
  end

  # The SessionStart hook: roostr session start, with the input Claude Code gives the hook.
  # source: nil leaves the key source out of the input.
  def session_start(id = SESSION, source: "startup", today: TODAY, stdin: nil, from: nil, env: {}, **options)
    input = stdin || hook_input(id, "SessionStart", from, source.nil? ? {} : { "source" => source })
    said = stdin ? "with #{stdin.inspect} on stdin" : "with the SessionStart hook's input on stdin, session_id #{id.inspect} and #{source.nil? ? "no source" : "source #{source.inspect}"}"
    note_situation "roostr session start ran #{said}, on #{today}, with HEAD at #{head_described}."
    cli "session", "start", stdin: input, env: clock(today).merge(env), from: from, **options
  end

  # The project map: additionalContext in the one line of JSON session start printed.
  def map
    problem = map_line_problem
    flunk "Expected stdout to be one line, {\"hookSpecificOutput\":{\"hookEventName\":\"SessionStart\",\"additionalContext\":\"<the map>\"}}, " \
          "and nothing else, but #{problem}.\n\n#{finished}" if problem
    JSON.parse(stdout).dig("hookSpecificOutput", "additionalContext")
  end

  def assert_map(text)
    assert text === map || map.include?(text.to_s), "Expected the map to include #{text.inspect}.#{map_shown}"
  end

  def refute_map(text)
    refute text === map || map.include?(text.to_s), "Expected the map not to include #{text.inspect}.#{map_shown}"
  end

  def assert_map_is(text)
    assert_same_text text, map, "The map in additionalContext", after: "\n\n#{finished}"
  end

  def assert_map_at_most(bytes)
    assert map.bytesize <= bytes, "Expected the map to be at most #{bytes} bytes; it is #{map.bytesize}.#{map_shown}"
  end

  # Fixture records, each written under its own name as a record of the project.
  def records_added(*ids)
    ids.each { |id| write_file ".roostr/processes/#{id}.yml", fixture("processes/#{id}.yml") }
  end

  # Many generated records; see the list at the top of this file.
  def processes_written(count, groups:, id: "process")
    (1..count).each do |n|
      number = format("%03d", n)
      group = groups ? "group: Group #{("A".ord + (n - 1) % groups).chr}\n" : ""
      write_file ".roostr/processes/#{id}-#{number}.yml",
                 "name: Step #{number} of the nightly import\n#{group}" \
                 "description: Step #{number} of the nightly import.\n" \
                 "files:\n  - app/imports/step_#{number}.rb\n  - app/imports/step_#{number}_check.rb\n"
    end
    note_situation "#{count} records written beside Atelier's, not committed: #{id}-001 to #{id}-#{format("%03d", count)}. Record <n> " \
                   "has #{groups ? "group Group A, Group B, ... in turn (#{groups} groups)" : "no group"}, description \"Step <n> of the nightly import.\" " \
                   "and files app/imports/step_<n>.rb and app/imports/step_<n>_check.rb, which do not exist."
  end

  # The Stop hook: roostr check --session, with the input Claude Code gives the hook.
  # permission_mode: "plan" is the mode Claude Code is in; input_bytes: 16_777_216 makes the
  # input exactly that long, with a long last_assistant_message, as Claude Code passes it.
  def check_session(*args, id: SESSION, stdin: nil, permission_mode: "default", input_bytes: nil, **options)
    input = stdin || stop_input(id, options[:from], permission_mode, input_bytes)
    mode = permission_mode == "default" ? "" : ", permission_mode #{permission_mode.inspect}"
    note_situation(stdin ? "roostr check --session gets #{stdin.inspect} on stdin." : "roostr check --session gets the Stop hook's input on stdin, session_id #{id.inspect}#{mode}.")
    check "--session", *args, stdin: input, **options
  end

  # check --session run this many times, as the Stop hook runs it at the end of each turn; each
  # must hold (exit 1). Nothing changes between the runs.
  def session_held(times)
    times.times do |run|
      check_session
      raise "session_held: run #{run + 1} of check --session did not hold (exit #{finished.exit}, not 1)\n\n#{finished}" unless finished.exit == 1
    end
    note_situation "check --session had run #{times} times in a row before, each holding (exit 1) with the same report, and nothing changed between the runs."
  end

  # A hook's whole command, run as Claude Code runs it: with bash, in its current folder, the
  # hook's input on stdin. The roostr under test comes first on the PATH.
  def claude_code_runs(event, command, id: SESSION, from: nil, permission_mode: "default")
    roostr_folder = File.dirname(Target.path(PROGRAM))
    environment_for_commands "PATH" => [roostr_folder, ENV.fetch("PATH")].join(File::PATH_SEPARATOR)
    note_situation "Claude Code ran the #{event} hook in #{from || "the top folder"}, session_id #{id.inspect}, permission_mode #{permission_mode.inspect}, roostr first on the PATH."
    input = hook_input(id, event, from, "permission_mode" => permission_mode)
    execute installed("bash"), ["-c", command], shown_as: "bash", stdin: input, from: from, env: clock(TODAY)
  end

  # Someone else's work on the shared repository, as the agent's clone has fetched it: a commit
  # by Dana Whitfield <dana.whitfield@example.com>, dated on (2026-10-09, the day before the
  # session, unless said), that adds the line "# Changed by Dana Whitfield." to the file. It is on
  # origin/<branch>, after what is there already, or, with fetched_from_a_url: true, only in
  # FETCH_HEAD, as git pull <url> <branch> leaves it. HEAD, the index and the files are as they were.
  def teammate_pushed_a_change_to(path, on: "2026-10-09", branch: "main", fetched_from_a_url: false)
    ref = "refs/remotes/origin/#{branch}"
    base = fetched_from_a_url ? "HEAD" : (resolved(ref) || "HEAD")
    sha = commit_beside_the_working_tree(path, "# Changed by Dana Whitfield.\n", "Change #{File.basename(path)}", base: base, on: on, identity: TEAMMATE)
    if fetched_from_a_url
      File.write(File.join(dir, ".git/FETCH_HEAD"), "#{sha}\t\tbranch '#{branch}' of https://git.example.com/atelier\n")
      note_situation "Dana Whitfield <dana.whitfield@example.com> committed a change to #{path} (dated #{on}), fetched from a URL: " \
                     "it is in .git/FETCH_HEAD only, on no branch and no origin/ branch, as git pull <url> #{branch} leaves it before merging."
    else
      git "update-ref", ref, sha
      note_situation "Dana Whitfield <dana.whitfield@example.com>, a teammate, committed a change to #{path} (dated #{on}); it is on origin/#{branch}, " \
                     "fetched, not merged: HEAD and the files are as they were."
    end
  end

  # A pull request by Dana Whitfield squash-merged on GitHub during the session: GitHub writes a
  # new commit, committed by GitHub and dated when it merged. It is on
  # origin/main, fetched, not merged.
  def pull_request_squash_merged_on_github(path, on: "2026-10-10T10:00:00+00:00")
    identity = TEAMMATE.merge("GIT_COMMITTER_NAME" => "GitHub", "GIT_COMMITTER_EMAIL" => "noreply@example.com") # stands in for GitHub's own address
    sha = commit_beside_the_working_tree(path, "# Changed in pull request 42.\n", "Change #{File.basename(path)} (#42)", base: resolved("refs/remotes/origin/main") || "HEAD", on: on, identity: identity)
    git "update-ref", "refs/remotes/origin/main", sha
    note_situation "Dana Whitfield's pull request 42, a change to #{path}, was squash-merged on GitHub at #{on}, during the session: a new commit " \
                   "authored by Dana Whitfield and committed by GitHub (here <noreply@example.com>). It is on origin/main, fetched, not merged."
  end

  # Git as the agent runs it during the session: any commit it writes (a rebase, an amend, a
  # merge) is committed by the kit's committer, tests@example.com, at 2026-10-10 12:00 UTC.
  def agent_runs_git(*args)
    git(*args, env: SESSION_COMMIT_DATE.merge("GIT_EDITOR" => "true"))
    note_situation "During the session the agent ran git #{args.join(" ")}; a commit it wrote has the committer date 2026-10-10 12:00 UTC."
  end

  # During the session, git rebase origin/main stops on a conflict in the file, between Dana's
  # change on origin/main and the commit being replayed. The agent resolves it to a version
  # neither side had (Dana's file and a line of its own), stages it, and the rebase goes on.
  def rebase_onto_origin_main_resolving_a_conflict_in(path)
    output, status = Open3.capture2e(Workspace::GIT_ENV.merge(SESSION_COMMIT_DATE), "git", "rebase", "origin/main", chdir: dir)
    raise "rebase_onto_origin_main_resolving_a_conflict_in: git rebase origin/main did not stop on a conflict:\n#{output}" if status.success?
    write_file path, git("show", "origin/main:#{path}") + "# The reminder goes two days ahead, kept with Dana's change.\n"
    git "add", "--", path
    git "rebase", "--continue", env: SESSION_COMMIT_DATE.merge("GIT_EDITOR" => "true")
    note_situation "During the session the agent ran git rebase origin/main, which stopped on a conflict in #{path}; it resolved it to a version " \
                   "neither side had, and the rebase went on. The commits it wrote have the committer date 2026-10-10 12:00 UTC."
  end

  # A git first on the PATH that never answers git status (it sleeps for 30 s) and passes every
  # other command to the real git. For session_start env:.
  def a_git_whose_status_never_answers
    path = folder_outside_the_workspace("path")
    File.write(File.join(path, "git"), "#!/bin/sh\n# sleeps 30 s when asked for git status; passes everything else to the real git\n" \
                                       "case \" $* \" in *\" status \"*) exec sleep 30 ;; esac\nexec '#{installed("git")}' \"$@\"\n")
    File.chmod(0o755, File.join(path, "git"))
    note_situation "The git roostr finds first on its PATH, #{path}/git, sleeps 30 s when asked for git status, and passes every other command to the real git."
    { "PATH" => [path, ENV.fetch("PATH")].join(File::PATH_SEPARATOR) }
  end

  # As git lfs install and git lfs track "*.psd" set a repository up, with a clean filter that
  # leaves the file filter-ran in the top folder whenever Git runs it: .gitattributes and
  # designs/poster.psd are new, not tracked.
  def untracked_files_under_a_clean_filter
    git "config", "filter.lfs.clean", "touch filter-ran && cat"
    git "config", "filter.lfs.smudge", "cat"
    write_file ".gitattributes", "*.psd filter=lfs diff=lfs merge=lfs -text\n"
    write_file "designs/poster.psd", "Placeholder standing in for the studio's spring poster.\n"
    note_situation ".gitattributes (*.psd filter=lfs) and designs/poster.psd are new and not tracked; .git/config sets the lfs clean filter " \
                   "to a command that leaves the file filter-ran in the top folder whenever Git runs it."
  end

  # A Git repository of its own inside the work tree, with one commit, not a submodule: the
  # outer repository lists it as one untracked entry, vendor/kiln-firmware/.
  def nested_repository(path)
    write_file "#{path}/README.md", "# Kiln firmware\n\nThe controller's firmware, kept here while it is tried out.\n"
    git "-C", path, "init", "--quiet", "--initial-branch=main"
    git "-C", path, "add", "--all"
    git "-C", path, "commit", "--quiet", "--message", "Start the firmware"
    note_situation "#{path} is a Git repository of its own, with one commit, inside the work tree: the outer repository lists it as one untracked entry, #{path}/."
  end

  # The person working with the agent changed the file before the session started, and left
  # the change uncommitted: one more empty line at its end, as edit_file makes it.
  def the_user_left_edited(path)
    edit_file path
    note_situation "Before the session started, the user edited #{path} and left the change uncommitted."
  end

  # The same content saved again, as an editor or git stash pop does: only the modification
  # time changes, a minute later.
  def saved_again_unchanged(path)
    file = File.join(dir, path)
    later = File.mtime(file) + 60
    File.utime(later, later, file)
    note_situation "#{path} was saved again with the same content: only its modification time changed, a minute later."
  end

  # What RuboCop's autocorrect does to a Ruby file: a magic comment on top.
  def formatted(path)
    write_file path, "# frozen_string_literal: true\n\n#{read_file(path)}"
    note_situation "A formatter re-saved #{path} with a change of its own: # frozen_string_literal: true on top."
  end

  # The file edited so that it is exactly this many bytes: a long comment line added at its end.
  def edited_to_a_size_of(path, bytes)
    content = read_file(path)
    padding = bytes - content.bytesize - 2
    raise ArgumentError, "edited_to_a_size_of: #{path} is already #{content.bytesize} bytes" if padding.negative?
    write_file path, "#{content}##{"-" * padding}\n"
    note_situation "#{path} was edited to exactly #{bytes} bytes (a long comment line added at its end)."
  end

  # A slim container image: no git at all.
  def git_not_on_the_path
    path = folder_outside_the_workspace("path")
    tools = %w[sh cat touch basename env].select { |tool| File.symlink(installed(tool), File.join(path, tool)) if installed(tool) }
    environment_for_commands "PATH" => path
    note_situation "roostr runs with PATH=#{path}, a folder that holds #{tools.join(", ")} and no git."
  end

  def git_refuses_the_repository = fake_git("refuses-the-repository")
  def git_older_than_supported   = fake_git("version-2.20.0")

  private

  # The comment under the script's first line says what it does, and is shown when a test fails.
  def fake_git(name)
    path = folder_outside_the_workspace("path")
    source = fixture("git/#{name}")
    File.write(File.join(path, "git"), source)
    File.chmod(0o755, File.join(path, "git"))
    environment_for_commands "PATH" => [path, ENV.fetch("PATH")].join(File::PATH_SEPARATOR)
    said = source.lines.drop(1).take_while { |line| line.start_with?("#") }.map { |line| line.delete_prefix("#").strip }.join(" ")
    note_situation "The git roostr finds first on its PATH is #{path}/git, a script that #{said}"
  end

  def installed(tool)
    ENV.fetch("PATH").split(File::PATH_SEPARATOR).map { |folder| File.join(folder, tool) }.find { |file| File.executable?(file) }
  end

  def clock(today) = { "ROOSTR_TODAY" => today, "TZ" => "UTC" }

  # What is not as FORMATS has it about the line session start printed, or nil.
  def map_line_problem
    return "stdout is empty" if stdout.empty?
    return "stdout is #{stdout.lines.size} lines" if stdout.lines.size > 1
    return "the line does not end in a newline" unless stdout.end_with?("\n")
    object = Commands.json_object(stdout) or return "the line is not one JSON object"
    return "the object's keys are #{object.keys.inspect}" unless object.keys == ["hookSpecificOutput"]
    inner = object["hookSpecificOutput"]
    return "hookSpecificOutput is #{JSON.generate(inner)[0, 60]}, not an object" unless inner.is_a?(Hash)
    return "hookSpecificOutput's keys are #{inner.keys.inspect}" unless inner.keys == %w[hookEventName additionalContext]
    return "hookEventName is #{inner["hookEventName"].inspect}" unless inner["hookEventName"] == "SessionStart"
    "additionalContext is #{JSON.generate(inner["additionalContext"])[0, 60]}, not a string" unless inner["additionalContext"].is_a?(String)
  end

  # The map as text, for a failure: easier to read than the JSON string the transcript shows.
  def map_shown
    "\n\nThe map, #{map.bytesize} bytes:\n#{map.gsub(/^/, "    ")}\n\n#{finished}"
  end

# What Claude Code gives the Stop hook, padded to input_bytes with a long last_assistant_message.
def stop_input(id, from, permission_mode, input_bytes)
  keys = { "stop_hook_active" => false, "permission_mode" => permission_mode }
  return hook_input(id, "Stop", from, keys) unless input_bytes
  room = input_bytes - hook_input(id, "Stop", from, keys.merge("last_assistant_message" => "")).bytesize
  raise ArgumentError, "check_session input_bytes: #{input_bytes} is shorter than the input itself" if room.negative?
  text = ("I moved the reminder to two days ahead and ran the tests; all pass. " * (room / 60 + 1))[0, room]
  note_situation "The Stop hook's input is exactly #{input_bytes} bytes, a last_assistant_message of #{room} bytes among it."
  hook_input(id, "Stop", from, keys.merge("last_assistant_message" => text))
end

# The commit a ref names, or nil when there is no such ref.
def resolved(ref)
  output, status = Open3.capture2(Workspace::GIT_ENV, "git", "rev-parse", "--verify", "--quiet", ref, chdir: dir)
  status.success? ? output.strip : nil
end

# A commit on top of base that adds a line to the file, made with Git's plumbing in an index of
# its own: HEAD, the index and the working tree are left as they are. Returns its hash.
def commit_beside_the_working_tree(path, line, message, base:, on:, identity:)
  content = git("show", "#{base}:#{path}") + line
  blob, status = Open3.capture2(Workspace::GIT_ENV, "git", "hash-object", "-w", "--stdin", stdin_data: content, chdir: dir)
  raise "commit_beside_the_working_tree: git hash-object failed" unless status.success?
  index = { "GIT_INDEX_FILE" => File.join(folder_outside_the_workspace("index"), "index") }
  git "read-tree", base, env: index
  git "update-index", "--cacheinfo", "100644,#{blob.strip},#{path}", env: index
  tree = git("write-tree", env: index).strip
  git("commit-tree", tree, "-p", base, "-m", message, env: identity.merge(commit_date(on))).strip
end

  # What Claude Code writes on a hook's stdin: one JSON object about the session and the event.
  def hook_input(id, event, from, more = {})
    JSON.generate({ "session_id" => id, "transcript_path" => "~/.claude/projects/-workspace-atelier/#{id}.jsonl",
                    "cwd" => from ? File.join(dir, from) : dir, "permission_mode" => "default",
                    "hook_event_name" => event }.merge(more))
  end

  # Where HEAD is, for a failure to show when a session started.
  def head_described
    output, status = Open3.capture2e(Workspace::GIT_ENV, "git", "log", "-1", "--format=%s", chdir: dir)
    return "\"#{output.chomp}\"" if status.success?
    File.directory?(File.join(dir, ".git")) ? "no commit yet" : "no Git repository"
  end

  def head_commit
    output, status = Open3.capture2(Workspace::GIT_ENV, "git", "rev-parse", "--verify", "--quiet", "HEAD", chdir: dir)
    status.success? ? output.strip : nil
  end
end
