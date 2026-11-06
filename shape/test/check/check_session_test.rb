class CheckSessionTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "a session starts, then the agent commits a change to booking_reminder_job.rb without its record: check --session lists remind-attendees under PROCESSES, exit 1" do
    session_start
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_stdout "PROCESSES\n"
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "nothing changed since the session started: check --session prints No problems found, exit 0" do
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a change committed before the session started is not the session's work: check --session prints No problems found" do
    reminder_job_changed_and_committed
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a change made in the session and not committed is the session's work too: check --session lists remind-attendees" do
    session_start
    edit_file "app/jobs/booking_reminder_job.rb"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a second session start with the same id, as when the session is resumed, keeps the first: a change committed between the two is listed" do
    session_start
    reminder_job_changed_and_committed
    session_start source: "resume"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "two sessions in one repository: for session-one, which started before the change was committed, check --session lists remind-attendees" do
    session_start "session-one"
    reminder_job_changed_and_committed
    session_start "session-two"

    check_session id: "session-one"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "two sessions in one repository: for session-two, which started after the change was committed, as a new id does after /clear, check --session prints No problems found" do
    session_start "session-one"
    reminder_job_changed_and_committed
    session_start "session-two"

    check_session id: "session-two"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with no record for the session, as when the hooks were added mid-session, check --session judges as --base HEAD and says so in one line on stderr: the committed change is not reported, No problems found" do
    reminder_job_changed_and_committed

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
    assert_stderr_is "roostr check --session: no usable record of session 5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f; judged as --base HEAD\n"
  end

  test "with no record for the session, an uncommitted change is listed, as check --base HEAD lists it" do
    edit_file "app/jobs/booking_reminder_job.rb"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  { "nothing" => "", "a line of text" => "Stop\n", "a JSON object without session_id" => "{}",
    "../../hooks/pre-commit as the session_id" => "{\"session_id\": \"../../hooks/pre-commit\", \"hook_event_name\": \"Stop\"}" }.each do |said, input|
    test "check --session with #{said} on stdin, no usable session_id, judges as --base HEAD without a word: a change committed in a session is not reported, No problems found, nothing on stderr" do
      session_start
      reminder_job_changed_and_committed

      check_session stdin: input

      assert_success
      assert_stdout_is "No problems found\n"
      assert_stderr_is ""
    end
  end

  test "when the commit the session started at no longer exists (reset, then removed by git gc), check --session judges as --base HEAD, No problems found, and stderr says it has no usable record of the session" do
    commit "Try the reminder three days ahead"
    session_start
    remove_the_last_commit_for_good
    reminder_job_changed_and_committed

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
    assert_stderr "no usable record of session 5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f; judged as --base HEAD"
  end

  test "a session that started 31 days before another session starts loses its record then: check --session for it judges as --base HEAD, No problems found" do
    session_start "session-one", today: "2026-09-09"
    reminder_job_changed_and_committed
    session_start "session-two" # on 2026-10-10

    check_session id: "session-one"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a session that started 30 days before another session starts keeps its record: check --session for it lists remind-attendees" do
    session_start "session-one", today: "2026-09-10"
    reminder_job_changed_and_committed
    session_start "session-two" # on 2026-10-10

    check_session id: "session-one"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "the Stop hook's input written and stdin then kept open, as by a runner that never closes it: check --session still judges the session, lists remind-attendees and exits within 5 s" do
    session_start
    reminder_job_changed_and_committed

    check_session stdin_held_open: true

    assert_finished_within 5
    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a session starts in the main checkout, and the agent adds a linked worktree and commits a change to booking_reminder_job.rb there: check --session run in the worktree finds the session's record and lists remind-attendees" do
    session_start
    linked_worktree "second-checkout"
    edit_file "second-checkout/app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-10", from: "second-checkout"

    check_session from: "second-checkout"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a Stop hook input of exactly 16 MiB, most of it a long last_assistant_message, is read whole: the session is judged, remind-attendees listed" do
    session_start
    reminder_job_changed_and_committed

    check_session input_bytes: 16_777_216

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a Stop hook input of 16 MiB and one byte is no input: check --session judges as --base HEAD, No problems found, and says so in one line on stderr that names 16 MiB" do
    session_start
    reminder_job_changed_and_committed

    check_session input_bytes: 16_777_217

    assert_success
    assert_stdout_is "No problems found\n"
    assert_stderr_is "roostr check --session: the input on stdin is over 16 MiB; judged as --base HEAD\n"
  end

  test "NOT DESCRIBED under --session says the entrypoints were new or listed when the session started, and names no commit hash" do
    session_start
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # http POST /waiting_list and job WaitingListJob, which no record lists

    check_session

    assert_exit 1
    assert_stdout "No process lists these entrypoints, though when the session started each was\n  either new or listed by a process."
    refute_stdout(/\h{40}/)
  end

  test "HEAD on an orphan branch, sharing no commit with the commit the session started at: check --session cannot judge, exit 2, and stderr says so without naming --base or a shallow clone" do
    session_start
    git "checkout", "--quiet", "--orphan", "start-again"
    commit "Start Atelier again", on: "2026-10-10"

    check_session

    assert_exit 2
    assert_stderr "Cannot judge the session: HEAD shares no commit with the commit it started at"
    refute_stderr "--base"
    refute_stderr "shallow"
  end

  test "run in a folder that has been deleted, check --session cannot run: exit 2, which the Stop hook does not hold on, and stderr says the current folder no longer exists" do
    session_start
    reminder_job_changed_and_committed

    check_session from_a_deleted_folder: true

    assert_exit 2
    assert_stderr "the current folder no longer exists"
  end

  [["--base", "HEAD"], ["--staged"]].each do |other|
    test "check --session #{other.join(" ")} cannot run: exit 2, and stderr says the two cannot be used together" do
      check_session(*other)

      assert_exit 2
      assert_stderr "together"
    end
  end
end
