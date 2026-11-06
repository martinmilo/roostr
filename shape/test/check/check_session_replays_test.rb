class CheckSessionReplaysTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb; book-workshop and cancel-booking list app/models/booking.rb

  test "git rebase origin/main in the session, as git pull --rebase runs it, replays the user's unpushed commit onto a teammate's: neither the replay nor the teammate's commit is the session's work, No problems found" do
    teammate_pushed_a_change_to "app/models/booking.rb"
    reminder_job_changed_and_committed # before the session: the user's own commit, not pushed
    session_start
    agent_runs_git "rebase", "--quiet", "origin/main"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the session rewords the commit it started at, git commit --amend with only a new message: the change that commit holds is not the session's work, No problems found" do
    reminder_job_changed_and_committed # before the session
    session_start
    agent_runs_git "commit", "--quiet", "--amend", "--message", "Send the reminder 48 hours ahead"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a replay in which the agent resolved a conflict is the session's work: after git rebase origin/main stops on booking_reminder_job.rb and the agent resolves it, remind-attendees is listed" do
    teammate_pushed_a_change_to "app/jobs/booking_reminder_job.rb"
    reminder_job_changed_and_committed # before the session: the user's own commit, not pushed
    session_start
    rebase_onto_origin_main_resolving_a_conflict_in "app/jobs/booking_reminder_job.rb"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "the session amends its own commit: the amended commit is still its work, remind-attendees listed" do
    session_start
    reminder_job_changed_and_committed
    agent_runs_git "commit", "--quiet", "--amend", "--message", "Send the reminder 48 hours ahead"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end
end
