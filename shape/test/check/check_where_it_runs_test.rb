class CheckWhereItRunsTest < AcceptanceTest
  test "started in app/models, check finds the project and reports a changed listed file by its path from the project folder" do
    atelier_under_git
    edit_file "app/jobs/booking_reminder_job.rb" # listed by remind-attendees

    check from: "app/models"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "in a project whose .roostr/ is in apps/atelier of a larger Git repository, check --base HEAD reports a changed listed file by its path from apps/atelier" do
    atelier_inside_a_larger_repository
    edit_file "apps/atelier/app/jobs/booking_reminder_job.rb"

    check "--base", "HEAD", from: "apps/atelier"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "in a larger Git repository, a change outside the project folder is no change to the project: check --base HEAD does not run its scan script" do
    atelier_inside_a_larger_repository
    write_executable "apps/atelier/.roostr/scan", fixture("scans/leaves-a-trace.sh") # creates scan-was-run in apps/atelier whenever it runs
    commit "Add the scan script"
    edit_file "apps/storefront/README.md"

    check "--base", "HEAD", from: "apps/atelier"

    assert_success
    assert_stdout_is "No problems found\n"
    refute_file "apps/atelier/scan-was-run"
  end
end
