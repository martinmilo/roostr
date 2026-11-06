class CheckSessionHooksTest < AcceptanceTest
  test "Claude Code runs the two hooks init prints: after the agent committed a change in the session, the Stop hook exits 2 with the report on stderr and nothing on stdout, so the agent cannot finish its turn" do
    atelier_under_git
    claude_code_runs "SessionStart", "roostr session start"
    reminder_job_changed_and_committed

    claude_code_runs "Stop", "roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }"

    assert_exit 2
    assert_stderr "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
    assert_stdout_is ""
  end

  test "Atelier in apps/atelier: the hooks init prints there, run by Claude Code in apps/storefront, hold the agent after it committed a change: the Stop hook exits 2 with the report on stderr" do
    atelier_inside_a_larger_repository
    claude_code_runs "SessionStart", 'cd "$(git rev-parse --show-toplevel)/apps/atelier" && roostr session start', from: "apps/storefront"
    edit_file "apps/atelier/app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-10"

    claude_code_runs "Stop", 'cd "$(git rev-parse --show-toplevel)/apps/atelier" && { roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }; }', from: "apps/storefront"

    assert_exit 2
    assert_stderr "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end
end
