class CheckDescribedNoLongerTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists job BookingReminderJob and mail BookingMailer#reminder, which no other record lists

  test "with remind-attendees.yml deleted, check --base HEAD lists its two entrypoints under NOT DESCRIBED, and the paragraph says to restore the record, exit 1" do
    delete_file ".roostr/processes/remind-attendees.yml"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "NOT DESCRIBED\n"
    assert_stdout "\n    job BookingReminderJob\n    mail BookingMailer#reminder\n"
    assert_stdout "restore that record or its entry instead."
  end

  test "with mail BookingMailer#reminder dropped from remind-attendees' list, NOT DESCRIBED lists that entrypoint alone" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-without-the-reminder-mail.yml")

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "NOT DESCRIBED\n"
    assert_stdout "\n    mail BookingMailer#reminder\n"
    refute_stdout "job BookingReminderJob"
  end

  test "an entrypoint moved from remind-attendees to book-workshop is still described and is not reported" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-without-the-reminder-mail.yml")
    write_file ".roostr/processes/book-workshop.yml", fixture("processes/book-workshop-with-the-reminder-mail.yml")

    check "--base", "HEAD"

    assert_success
    refute_stdout "NOT DESCRIBED"
  end

  test "a feature removed whole, its record, its job and its two rows of entrypoints.tsv, is not reported" do
    delete_file ".roostr/processes/remind-attendees.yml"
    delete_file "app/jobs/booking_reminder_job.rb"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/reminders-dropped.tsv") # Atelier's rows without the job and the reminder mail

    check "--base", "HEAD"

    assert_success
    refute_stdout "NOT DESCRIBED"
  end

  test "on a branch that deleted remind-attendees.yml in a commit, check --base main lists its two entrypoints under NOT DESCRIBED" do
    branch "quiet-reminders"
    delete_file ".roostr/processes/remind-attendees.yml"
    commit "Stop describing the reminders", on: "2026-10-06"

    check "--base", "main"

    assert_exit 1
    assert_stdout "\n    job BookingReminderJob\n    mail BookingMailer#reminder\n"
  end

  test "an entrypoint main described after the branch left is old debt on the branch: the records are read where the branch left main, and GET /workshops is not reported" do
    main_described_the_workshop_list_after_the_branch_left

    check "--base", "main"

    assert_success
    refute_stdout "GET /workshops"
  end

  test "a record that no longer parses is reported once, under VALIDATE: its entrypoints are taken as it listed them at the base and are not listed under NOT DESCRIBED" do
    write_file ".roostr/processes/remind-attendees.yml", "name: [Remind attendees\n"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "VALIDATE\n"
    refute_stdout "NOT DESCRIBED"
  end
end
