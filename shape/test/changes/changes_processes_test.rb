class ChangesProcessesTest < AcceptanceTest
  def setup = atelier_under_git # committed once on 2026-01-15; booking.rb is listed by book-workshop and cancel-booking, booking_reminder_job.rb by remind-attendees; today is 2026-10-10

  test "a process whose record was committed in the period is listed with the subject of that commit" do
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Describe the reminder as it works now", on: "2026-10-06"

    changes

    assert_stdout "PROCESSES\n  remind-attendees\n    Describe the reminder as it works now\n"
    assert_stdout "processes: 1 changed"
  end

  test "a process is listed with the subject of a commit that touched a file it lists, and a process that does not list that file is not" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06"

    changes

    assert_stdout "PROCESSES\n" \
                  "  book-workshop\n    Hold the place while the payment is pending\n" \
                  "  cancel-booking\n    Hold the place while the payment is pending\n"
    refute_stdout "remind-attendees"
  end

  test "a commit that touched both a record and a file it lists is given once under that process" do
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Send the reminder two days ahead", on: "2026-10-06"

    changes

    assert_stdout "PROCESSES\n  remind-attendees\n    Send the reminder two days ahead\n\nentrypoints: 0 added, 0 removed   processes: 1 changed\n"
  end

  test "three commits in the period that touched one process count as one changed process" do
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-05"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Skip the reminder for cancelled bookings", on: "2026-10-06"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Describe the reminder as it works now", on: "2026-10-07"

    changes

    assert_stdout "entrypoints: 0 added, 0 removed   processes: 1 changed"
  end

  test "a commit that touched only files no process lists is not reported" do
    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-06"

    changes

    refute_stdout "Show the trainer on the workshop page"
    assert_stdout "processes: 0 changed"
  end

  test "when nothing changed in the period, the report is the first day of the period, a blank line and a count line of zeros" do
    changes

    assert_success
    assert_stdout_is "CHANGES SINCE 2026-10-03\n\nentrypoints: 0 added, 0 removed   processes: 0 changed\n"
  end
end
