class CheckSessionPlanModeTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "in plan mode, where the agent cannot change files, check --session does not judge: it prints that the session is in plan mode and exits 0, though a change was committed without its record" do
    session_start
    reminder_job_changed_and_committed

    check_session permission_mode: "plan"

    assert_success
    assert_stdout_is "Not judged: the session is in plan mode, where the agent cannot change files.\n"
  end

  test "Claude Code runs the Stop hook init prints while the session is in plan mode: the hook exits 0 and does not hold the agent" do
    claude_code_runs "SessionStart", "roostr session start"
    reminder_job_changed_and_committed

    claude_code_runs "Stop", "roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }", permission_mode: "plan"

    assert_success
  end

  %w[default acceptEdits auto dontAsk bypassPermissions].each do |mode|
    test "in permission mode #{mode} check --session judges the session as ever: remind-attendees listed, exit 1" do
      session_start
      reminder_job_changed_and_committed

      check_session permission_mode: mode

      assert_exit 1
      assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
    end
  end

  test "check --session --json in plan mode gives problems null, since it did not judge, and the session's id, exit 0" do
    session_start
    reminder_job_changed_and_committed

    check_session "--json", permission_mode: "plan"

    assert_success
    assert_json "problems", nil
    assert_json "session", "5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f"
  end
end
