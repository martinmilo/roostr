class CheckSessionBeforeTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "the user left booking_reminder_job.rb edited before the session started, and the session did nothing: the edit is not the session's work, check --session prints No problems found" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user left booking_reminder_job.rb edited before the session, and the session edited it again: the file is the session's work, check --session lists remind-attendees" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    write_file "app/jobs/booking_reminder_job.rb", "# The reminder now goes two days ahead.\n"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "booking_reminder_job.rb deleted before the session and still deleted at its end is not the session's work: No problems found" do
    delete_file "app/jobs/booking_reminder_job.rb"
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user's edit of remind-attendees.yml from before the session does not count as the session's update: after the session commits a change to booking_reminder_job.rb, remind-attendees is listed" do
    the_user_left_edited ".roostr/processes/remind-attendees.yml"
    session_start
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "the session edits remind-attendees.yml further, after the user's edit from before the session: that is the session's update, and after its commit to booking_reminder_job.rb check --session prints No problems found" do
    the_user_left_edited ".roostr/processes/remind-attendees.yml"
    session_start
    reminder_job_changed_and_committed
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml")

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the session commits the user's earlier edit of booking_reminder_job.rb unchanged, as git commit -a does: what the session commits is its work, remind-attendees listed" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    commit "Send the reminder two days ahead", on: "2026-10-10"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "git stash and git stash pop in the session put the user's edit back with the content it had: not the session's work, No problems found" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    agent_runs_git "stash", "--quiet"
    agent_runs_git "stash", "pop", "--quiet"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user's edited file saved again in the session with the same content, only its modification time changed: not the session's work, No problems found" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    saved_again_unchanged "app/jobs/booking_reminder_job.rb"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a formatter run in the session re-saves the user's edited file with a change of its own: the file is the session's work, remind-attendees listed" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    formatted "app/jobs/booking_reminder_job.rb"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a later session start for the same id, as after compacting, keeps what was uncommitted when the session first started: the agent's edit of the user's file, made before it, still counts" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start
    write_file "app/jobs/booking_reminder_job.rb", "# The reminder now goes two days ahead.\n"
    session_start source: "compact"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "the user left remind-attendees.yml with an unknown key, steps, and the session only wrote README.md: VALIDATE leaves out the finding in the user's file, No problems found" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-with-steps.yml") # by the user, before the session
    session_start
    write_file "README.md", "# Atelier\n\nBookings for a pottery studio's workshops.\n"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end
end
