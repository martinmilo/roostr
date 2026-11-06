class CheckSessionBroughtInTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb; book-workshop and cancel-booking list app/models/booking.rb

  { "pulled, a fast-forward as git pull makes it" => ["merge", "--quiet", "--ff-only", "origin/main"],
    "checked out as a branch of its own" => ["switch", "--quiet", "--create", "dana-reminders", "origin/main"],
    "taken with git reset --hard origin/main" => ["reset", "--quiet", "--hard", "origin/main"] }.each do |how, command|
    test "a teammate's commit to booking_reminder_job.rb, made before the session and #{how} during it, is not the session's work: check --session prints No problems found" do
      teammate_pushed_a_change_to "app/jobs/booking_reminder_job.rb"
      session_start
      agent_runs_git(*command)

      check_session

      assert_success
      assert_stdout_is "No problems found\n"
    end
  end

  test "a pull request squash-merged on GitHub during the session and then pulled is not the session's work, though dated after it started: GitHub committed it, No problems found" do
    session_start
    pull_request_squash_merged_on_github "app/jobs/booking_reminder_job.rb"
    agent_runs_git "merge", "--quiet", "--ff-only", "origin/main"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a teammate's commit dated the day after the session started, as from a machine whose clock is ahead, pulled in the session: another committer made it, No problems found" do
    teammate_pushed_a_change_to "app/jobs/booking_reminder_job.rb", on: "2026-10-11"
    session_start
    agent_runs_git "merge", "--quiet", "--ff-only", "origin/main"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a teammate's commit dated during the session, pulled from a URL, so on no origin/ branch: the committer is not the session's, No problems found" do
    session_start
    teammate_pushed_a_change_to "app/jobs/booking_reminder_job.rb", on: "2026-10-10T10:00:00+00:00", fetched_from_a_url: true
    agent_runs_git "merge", "--quiet", "--ff-only", "FETCH_HEAD"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the session commits a change to booking_reminder_job.rb and then merges origin/main, where a teammate changed booking.rb: only remind-attendees is listed, not book-workshop or cancel-booking" do
    teammate_pushed_a_change_to "app/models/booking.rb"
    session_start
    reminder_job_changed_and_committed
    agent_runs_git "merge", "--quiet", "--no-edit", "origin/main"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
    refute_stdout "book-workshop"
    refute_stdout "cancel-booking"
  end

  test "the session rebases its own commit onto origin/main, where a teammate changed booking.rb: its replayed commit is still its work and listed, the teammate's is not" do
    teammate_pushed_a_change_to "app/models/booking.rb"
    session_start
    reminder_job_changed_and_committed
    agent_runs_git "rebase", "--quiet", "origin/main"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
    refute_stdout "book-workshop"
  end

  test "the session's commit on a side branch, merged with a merge commit, is its work: remind-attendees listed" do
    session_start
    branch "reminders"
    reminder_job_changed_and_committed
    switch_to "main"
    merge "reminders", on: "2026-10-10"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a commit of the session dated 23:55:00 the day before the session started at 00:00, as by a clock 5 minutes behind, is still its work: remind-attendees listed" do
    session_start
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-09T23:55:00+00:00"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "the user's own commit dated 23:54:59, 5 minutes and 1 second before the session started, on a branch the session merges with a fast-forward, is not the session's work: No problems found" do
    branch "reminders"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-09T23:54:59+00:00"
    switch_to "main"
    session_start
    agent_runs_git "merge", "--quiet", "--ff-only", "reminders"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end
end
