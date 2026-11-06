class CheckSessionHoldsTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "held 3 times in a row with the same problems and no change to the session's work, the agent is let through the fourth time: exit 0 and one line saying the problems remain for the pre-commit hook and CI" do
    session_start
    reminder_job_changed_and_committed
    session_held 3

    check_session

    assert_success
    assert_stdout_is "Not held again: 3 holds in a row found the same problems and no change to the session's work. The problems remain; the pre-commit hook and CI will report them.\n"
  end

  test "held 3 times, then the agent changes its work: the count starts again, and check --session holds, remind-attendees listed" do
    session_start
    reminder_job_changed_and_committed
    session_held 3
    edit_file "app/controllers/workshops_controller.rb" # listed by no record

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "check --session keeps its count of holds beside the session's record: it writes in .git/roostr/sessions/ and nowhere else" do
    session_start
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "check --session --json is never let through: after 3 holds in a row with nothing changed, it still exits 1 and lists the problem" do
    session_start
    reminder_job_changed_and_committed
    session_held 3

    check_session "--json"

    assert_exit 1
    assert_json "problems.1.process", "remind-attendees"
  end
end
