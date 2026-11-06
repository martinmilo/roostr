class ValidateTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, 7 entrypoints, every listed file present

  test "a project with no problems validates" do
    validate

    assert_success
    assert_stdout "No problems found"
    assert_nothing_written
  end

  test "a listed file that does not exist is a finding naming the record and the file" do
    write_file ".roostr/processes/export-bookings.yml", fixture("processes/export-bookings.yml") # lists app/services/export_bookings.rb, which is not there

    validate

    assert_exit 1
    assert_stdout "export-bookings.yml"
    assert_stdout "app/services/export_bookings.rb"
  end

  test "a listed entrypoint missing from entrypoints.tsv is a finding naming it" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds

    validate

    assert_exit 1
    assert_stdout "refund-booking.yml"
    assert_stdout "http POST /refunds"
  end

  test "an entrypoint with a kind outside the closed list is a finding" do
    write_file ".roostr/processes/receive-payment.yml", fixture("processes/receive-payment.yml") # kind webhook

    validate

    assert_exit 1
    assert_stdout "receive-payment.yml"
    assert_stdout "webhook"
  end

  test "a record that does not parse is a finding and the other records are still checked" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds

    validate

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    assert_stdout "refund-booking.yml"
  end

  test "a record without a description is a finding" do
    write_file ".roostr/processes/export-bookings.yml", "name: Export bookings\ngroup: Reports\n"

    validate

    assert_exit 1
    assert_stdout "export-bookings.yml"
    assert_stdout "description"
  end

  test "records that list entrypoints while entrypoints.tsv is missing give one finding that says to run roostr scan" do
    delete_file ".roostr/entrypoints.tsv"

    validate

    assert_exit 1
    assert_stdout "roostr scan"
    assert_stdout_lines 1
  end

  test "the finding for a missing entrypoints.tsv starts, as every finding does, with the file it is about and a colon, as .roostr/entrypoints.tsv: followed by the message" do
    delete_file ".roostr/entrypoints.tsv"

    validate

    assert_stdout(/\A\.roostr\/entrypoints\.tsv: \S/)
  end

  test "an entrypoints.tsv row with an unknown kind is a finding" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/unknown-kind.tsv") # the Atelier rows plus one of kind webhook

    validate

    assert_exit 1
    assert_stdout "entrypoints.tsv"
    assert_stdout "webhook"
  end

  test "a project.yml that does not parse is a finding and the process records are still checked" do
    write_file ".roostr/project.yml", "name: [unclosed\n"
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "refund-booking.yml"
  end

  test "why given as a single string instead of a list is a finding naming the record and the key" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\nwhy: Cards are refunded, never vouchers.\n"

    validate

    assert_exit 1
    assert_stdout "refund-booking.yml"
    assert_stdout "why"
  end

  test "watch_out given as a string instead of a list is a finding naming project.yml and the key" do
    write_file ".roostr/project.yml", "name: Atelier\nstack: Rails 8\nwatch_out: Stripe runs in test mode on staging.\n"

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "watch_out"
  end

  test "two broken records give two findings in one run" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"
    write_file ".roostr/processes/remind-attendees.yml", "name: [unclosed\n"

    validate

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    assert_stdout "remind-attendees.yml"
  end

  test "validate writes nothing when it finds problems" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    validate

    assert_exit 1
    assert_nothing_written
  end
  test "a watch_out item of 160 characters is accepted" do
    write_file ".roostr/project.yml", fixture("project/watch-out-160.yml") # one item of exactly 160 characters

    validate

    assert_success
  end

  test "a watch_out item of 161 characters is a finding naming project.yml, watch_out and the limit of 160" do
    write_file ".roostr/project.yml", fixture("project/watch-out-161.yml") # one item of 161 characters

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "watch_out"
    assert_stdout "160"
  end

  test "seven watch_out items are accepted" do
    write_file ".roostr/project.yml", fixture("project/watch-out-7-items.yml")

    validate

    assert_success
  end

  test "eight watch_out items are a finding naming project.yml, watch_out and the limit of 7" do
    write_file ".roostr/project.yml", fixture("project/watch-out-8-items.yml")

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "watch_out"
    assert_stdout "7"
  end

  test "a watch_out item of 160 characters and 164 bytes is accepted: the limit counts characters, not bytes" do
    write_file ".roostr/project.yml", fixture("project/watch-out-160-non-ascii.yml") # one item of 160 characters, with é and € in it, 164 bytes

    validate

    assert_success
  end

  test "an operation note of 160 characters is accepted" do
    write_file ".roostr/project.yml", fixture("project/note-160.yml") # the note on test has exactly 160 characters

    validate

    assert_success
  end

  test "an operation note of 161 characters is a finding naming project.yml, operations.test.note and the limit of 160" do
    write_file ".roostr/project.yml", fixture("project/note-161.yml") # the note on test has 161 characters

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "operations.test.note"
    assert_stdout "160"
  end

  test "an operation note of 160 characters and 163 bytes is accepted: the limit counts characters, not bytes" do
    write_file ".roostr/project.yml", fixture("project/note-160-non-ascii.yml") # the note on test has 160 characters, with é and € in it, 163 bytes

    validate

    assert_success
  end

  test "an environment value of 160 characters is accepted" do
    write_file ".roostr/project.yml", fixture("project/environment-160.yml") # staging has exactly 160 characters

    validate

    assert_success
  end

  test "an environment value of 161 characters is a finding naming project.yml, environments.staging and the limit of 160" do
    write_file ".roostr/project.yml", fixture("project/environment-161.yml") # staging has 161 characters

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "environments.staging"
    assert_stdout "160"
  end

  test "a note and an environment value that are both too long are two findings in one run, one naming operations.test.note and one naming environments.staging" do
    write_file ".roostr/project.yml", fixture("project/note-and-environment-161.yml") # the note on test and staging have 161 characters each

    validate

    assert_exit 1
    assert_stdout "operations.test.note"
    assert_stdout "environments.staging"
  end

  test "an unknown key in project.yml is a finding naming the file and the key" do
    write_file ".roostr/project.yml", "name: Atelier\nrepository: github.com/atelier/atelier\n"

    validate

    assert_exit 1
    assert_stdout "project.yml"
    assert_stdout "repository"
  end

  test "an unknown key in a process record is a finding naming the record and the key" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\nsteps:\n  - Find the booking in the admin\n"

    validate

    assert_exit 1
    assert_stdout "refund-booking.yml"
    assert_stdout "steps"
  end

  test "an outcome given as a string is a finding naming the key by its path outcomes.refunded, with no trailing dot" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\noutcomes:\n  refunded: the money goes back to the card within five days\n"

    validate

    assert_exit 1
    assert_stdout "outcomes.refunded"
    refute_stdout "outcomes.refunded."
  end

  test "an unknown key inside an outcome is a finding naming it by its path outcomes.refunded.because" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\noutcomes:\n  refunded:\n    when: the booking was paid by card\n    then: the money goes back to the card\n    because: Stripe only refunds to the source\n"

    validate

    assert_exit 1
    assert_stdout "outcomes.refunded.because"
  end

  test "the same entrypoint listed twice in one record is a finding naming the record, the entrypoint and the repeat by its position in the list, entrypoints.3" do
    write_file ".roostr/processes/cancel-booking.yml", fixture("processes/cancel-booking-listed-twice.yml") # http DELETE /bookings/:id is its first and its third entrypoint

    validate

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    assert_stdout "entrypoints.3"
    assert_stdout "http DELETE /bookings/:id"
  end

  test "the same entrypoint listed in two different records is not a finding" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\nentrypoints:\n  - http DELETE /bookings/:id\n" # cancel-booking lists it too

    validate

    assert_success
  end

  test "a finding about an item of a list names the item by its position counted from 1: a second watch_out item that is too long is watch_out.2" do
    write_file ".roostr/project.yml", fixture("project/watch-out-second-item-161.yml") # the first item is short, the second has 161 characters

    validate

    assert_exit 1
    assert_stdout "watch_out.2"
  end

  test "a checked date written as YYYY-MM-DD is accepted" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    validate

    assert_success
    assert_stdout "No problems found"
  end

  ["yesterday", "2026-02-30"].each do |value|
    test "checked: #{value} is not a date: a finding names the record, the key checked and the form YYYY-MM-DD" do
      write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\nchecked: #{value}\n"

      validate

      assert_exit 1
      assert_stdout "refund-booking.yml"
      assert_stdout "checked"
      assert_stdout "YYYY-MM-DD"
    end
  end

  test "a row of entrypoints.tsv out of byte order is a finding on its line: the rows are sorted by their whole line" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/row-out-of-order.tsv") # Atelier's header and 7 rows, then http GET /gift_cards on line 9

    validate

    assert_exit 1
    assert_stdout ".roostr/entrypoints.tsv: line 9: "
    assert_stdout "sorted"
  end
end
