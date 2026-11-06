class CheckFirstCommitTest < AcceptanceTest
  def setup = workspace("atelier") # each test makes it a Git repository with no commits yet

  test "in a repository with no commits yet, check --base HEAD passes a valid project: everything is work after the base, and the entrypoints no process lists are not compared" do
    git "init", "--quiet", "--initial-branch=main"

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "in a repository with no commits yet, check --base HEAD runs the scan script and reports entrypoints.tsv behind it" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list, which entrypoints.tsv lacks
    git "init", "--quiet", "--initial-branch=main"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end

  test "in a repository with no commits yet, --base main cannot run: there is no commit called main, and check names it" do
    git "init", "--quiet", "--initial-branch=main"

    check "--base", "main"

    assert_exit 2
    assert_stderr "main"
    assert_stdout_lines 0
  end

  test "in a repository with no commits yet, check --staged judges the first commit: a record left out of what is staged is listed under NOT STAGED, exit 1" do
    git "init", "--quiet", "--initial-branch=main"
    stage_everything_but ".roostr/processes/remind-attendees.yml"

    check "--staged"

    assert_exit 1
    assert_stdout "NOT STAGED\n"
    assert_stdout "\n    .roostr/processes/remind-attendees.yml\n"
  end

  test "a session that started before the repository's first commit: the commits made in it are its work, and check --session lists remind-attendees after a commit of Atelier and one of the change" do
    git "init", "--quiet", "--initial-branch=main"
    session_start
    commit "Start Atelier", on: "2026-10-10"
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end
end
