class NeedsAttentionTest < AcceptanceTest
  def setup = atelier_under_git # committed once; booking.rb is listed by book-workshop and cancel-booking, book_workshop.rb by book-workshop, booking_reminder_job.rb by remind-attendees

  test "when a file was committed after the records that list it, NEEDS ATTENTION names each process that lists it, followed by the file and: changed since this record was updated" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06"

    overview

    assert_stdout "NEEDS ATTENTION\n" \
                  "  book-workshop    app/models/booking.rb changed since this record was updated\n" \
                  "  cancel-booking   app/models/booking.rb changed since this record was updated\n"
    refute_stdout(/remind-attendees .* changed since/)
  end

  test "a record committed in the same commit as a file it lists is up to date and is not listed" do
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Send the reminder two days ahead", on: "2026-10-06"

    overview

    assert_success
    refute_stdout "NEEDS ATTENTION"
  end

  test "once a record is committed after the file, here with a new checked date, the process is no longer listed" do
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08
    commit "The reminder record still holds", on: "2026-10-08"

    overview

    assert_success
    refute_stdout "NEEDS ATTENTION"
  end

  test "newer means later in history, not a later date: a file committed after its record with the very same commit date is listed" do
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Describe the reminder as it works now", on: "2026-10-06"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"

    overview

    assert_stdout(/NEEDS ATTENTION\n  remind-attendees\s+app\/jobs\/booking_reminder_job\.rb changed since/)
  end

  test "a file changed on a branch that was merged after the record's last commit is newer than the record: its commit is reachable from HEAD and not from the record's last commit, and the process is listed" do
    branch "longer-reminders"
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-05"
    switch_to "main"
    edit_file ".roostr/processes/remind-attendees.yml"
    commit "Describe the reminder as it works now", on: "2026-10-06"
    merge "longer-reminders", on: "2026-10-07"

    overview

    assert_stdout(/NEEDS ATTENTION\n  remind-attendees\s+app\/jobs\/booking_reminder_job\.rb changed since/)
  end

  test "a listed file with an edit that is not committed names its process" do
    edit_file "app/services/book_workshop.rb"

    overview

    assert_stdout(/NEEDS ATTENTION\n  book-workshop\s+app\/services\/book_workshop\.rb changed since/)
  end

  test "a listed file with a staged edit still names its process" do
    edit_file "app/services/book_workshop.rb"
    git "add", "app/services/book_workshop.rb"

    overview

    assert_stdout(/NEEDS ATTENTION\n  book-workshop\s+app\/services\/book_workshop\.rb changed since/)
  end

  test "a newer commit to a file that no process lists adds nothing, even when entrypoints.tsv names the file: NEEDS ATTENTION is left out" do
    edit_file "app/controllers/workshops_controller.rb"
    commit "Show the trainer on the workshop page", on: "2026-10-06"

    overview

    assert_success
    refute_stdout "NEEDS ATTENTION"
  end

  test "a record that has never been committed is skipped, while a committed record that lists the same changed file is named" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists app/models/booking.rb; never committed
    edit_file "app/models/booking.rb"

    overview

    assert_stdout(/cancel-booking\s+app\/models\/booking\.rb changed since/)
    refute_stdout(/refund-booking\s+app\/models\/booking\.rb/)
  end

  test "a record with an edit of its own that is not committed yet, here a new checked date, counts as updated and is not listed" do
    edit_file "app/jobs/booking_reminder_job.rb"
    commit "Send the reminder two days ahead", on: "2026-10-06"
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    overview

    assert_success
    refute_stdout "NEEDS ATTENTION"
  end

  test "a record that cannot be read is listed before the processes that need attention" do
    write_file ".roostr/processes/refund-booking.yml", "name: [unclosed\n"
    edit_file "app/jobs/booking_reminder_job.rb"

    overview

    assert_stdout(/NEEDS ATTENTION\n.*refund-booking\.yml.*run roostr validate\n  remind-attendees\s+app\/jobs\/booking_reminder_job\.rb changed since/)
  end
end
