class CheckCheckedSuggestionTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb; book-workshop and cancel-booking list app/models/booking.rb; no record has checked

  test "when the record's checked already says today, the paragraph suggests today's date with the time: checked: 2026-10-10 14:30" do
    record_checked "remind-attendees", "2026-10-10"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10 14:30"

    assert_exit 1
    assert_stdout "write checked: 2026-10-10 14:30 in it instead."
  end

  test "when the record's checked says today at an earlier time, 09:15, the paragraph suggests the time now: checked: 2026-10-10 14:30" do
    record_checked "remind-attendees", "2026-10-10 09:15"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10 14:30"

    assert_stdout "write checked: 2026-10-10 14:30 in it instead."
  end

  test "when the record's checked already holds this very minute, the paragraph suggests the next minute: checked: 2026-10-10 14:31" do
    record_checked "remind-attendees", "2026-10-10 14:30"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10 14:30"

    assert_stdout "write checked: 2026-10-10 14:31 in it instead."
  end

  test "at 23:59, when the record's checked holds 23:59, the next minute is on the next day: checked: 2026-10-11 00:00" do
    record_checked "remind-attendees", "2026-10-10 23:59"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10 23:59"

    assert_stdout "write checked: 2026-10-11 00:00 in it instead."
  end

  test "when the record's checked holds another day, the paragraph suggests the date alone, checked: 2026-10-10, though the clock has a time" do
    record_checked "remind-attendees", "2026-10-08"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10 14:30"

    assert_stdout "write checked: 2026-10-10 in it instead."
  end

  test "with two processes listed, one record checked today and the other on 2026-10-08, the one value suggested differs from both: checked: 2026-10-10 14:30" do
    record_checked "cancel-booking", "2026-10-10"
    record_checked "book-workshop", "2026-10-08"
    edit_file "app/models/booking.rb"

    check today: "2026-10-10 14:30"

    assert_stdout "write checked: 2026-10-10 14:30 in it instead."
  end

  test "with ROOSTR_TODAY a date alone, the time is 00:00: a record checked today gets checked: 2026-10-10 00:00 suggested" do
    record_checked "remind-attendees", "2026-10-10"
    edit_file "app/jobs/booking_reminder_job.rb"

    check today: "2026-10-10"

    assert_stdout "write checked: 2026-10-10 00:00 in it instead."
  end
end
