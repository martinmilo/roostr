class CheckBaseTest < AcceptanceTest
  test "--base=HEAD, written with an equals sign, is the same as --base HEAD: old debt before HEAD is not reported" do
    atelier_under_git
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06" # remind-attendees is now behind, before HEAD
    edit_file "app/controllers/workshops_controller.rb" # the work after HEAD; listed by no record

    check "--base=HEAD"

    assert_success
    refute_stdout "remind-attendees"
  end

  test "a base Git cannot resolve, --base mian, cannot run: check names mian, says fetching more history may help, prints nothing on stdout and writes nothing" do
    atelier_under_git

    check "--base", "mian"

    assert_exit 2
    assert_stderr "mian"
    assert_stderr "fetch"
    assert_stdout_lines 0
    assert_nothing_written
  end

  test "in a shallow clone where origin/main and HEAD share no commit, --base origin/main cannot run, and check says to fetch more history" do
    atelier_under_git
    on_a_branch_after_main_moved_on
    shallow_clone # depth 1: the tips of main and trainer-page, without the commit they share

    check "--base", "origin/main"

    assert_exit 2
    assert_stderr "fetch"
    assert_stdout_lines 0
  end

  test "--base outside a Git repository cannot run, and check says this is not a Git repository" do
    workspace "atelier" # not under Git

    check "--base", "HEAD"

    assert_exit 2
    assert_stderr(/git repository/i)
    assert_nothing_written
  end

  test "--base with no value after it is refused as a mistake in the command" do
    atelier_under_git

    check "--base"

    assert_exit 2
    assert_stderr "--base"
    assert_nothing_written
  end

  test "an option check does not take, --fix, is refused as a mistake in the command, naming it" do
    atelier_under_git

    check "--fix"

    assert_exit 2
    assert_stderr "--fix"
  end

  test "without .roostr/project.yml, check says to run roostr init" do
    write_file "README.md", "# Atelier\n"
    start_repository

    check

    assert_exit 2
    assert_stderr "roostr init"
    assert_nothing_written
  end
end
