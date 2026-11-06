class CheckStagedHookInASubfolderTest < AcceptanceTest
  def setup = atelier_inside_a_larger_repository # .roostr/ is in apps/atelier; Git is at the top

  test "a plain git commit of everything staged makes the commit: Git names the index .git/index, a path relative to the top of the repository, and the hook runs in apps/atelier" do
    pre_commit_hook "cd apps/atelier && roostr check --staged"
    edit_file "apps/atelier/app/jobs/booking_reminder_job.rb"
    edit_file "apps/atelier/.roostr/processes/remind-attendees.yml"
    git "add", "--all"

    git_commit "-m", "Send the reminder two days ahead"

    assert_success
    assert_commit_made
  end

  test "git commit -a while a new record is untracked is refused, and the hook lists it by its path in the project" do
    pre_commit_hook "cd apps/atelier && roostr check --staged"
    edit_file "apps/atelier/app/jobs/booking_reminder_job.rb"
    write_file "apps/atelier/.roostr/processes/fix-double-booking.yml", fixture("processes/fix-double-booking.yml")

    git_commit "-a", "-m", "Send the reminder two days ahead"

    assert_exit 1
    assert_stderr "\n    .roostr/processes/fix-double-booking.yml\n"
    refute_commit_made
  end

  test "git commit with paths, run in apps/atelier and naming a listed file and its record, makes the commit" do
    pre_commit_hook "cd apps/atelier && roostr check --staged"
    edit_file "apps/atelier/app/jobs/booking_reminder_job.rb"
    edit_file "apps/atelier/.roostr/processes/remind-attendees.yml"

    git_commit "-m", "Send the reminder two days ahead", "app/jobs/booking_reminder_job.rb", ".roostr/processes/remind-attendees.yml", from: "apps/atelier"

    assert_success
    assert_commit_made
  end
end
