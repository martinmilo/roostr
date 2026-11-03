class ShowProcessTest < AcceptanceTest
  def setup = workspace("atelier") # cancel-booking has no try, outcomes or why; book-workshop has all three

  test "a process with no try, no outcomes and no why leaves all three out" do
    show "cancel-booking"

    assert_success
    assert_stdout "Cancel a booking and free the place"
    refute_stdout(/^  try /)
    refute_stdout "OUTCOMES"
    refute_stdout "WHY"
  end

  test "a process without a group has a heading of id and name only" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops.yml")

    show "import-workshops"

    assert_stdout(/\Aimport-workshops\s+Import workshops from the studio calendar\n/)
  end

  test "an entrypoint missing from entrypoints.tsv says so on the line where its file would be" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds, which the scan did not find

    show "refund-booking"

    assert_success
    assert_stdout(/entry\s+http POST \/refunds\n\s+not in entrypoints\.tsv/)
  end

  test "a long description is wrapped at 80 columns" do
    write_file ".roostr/processes/sync-calendar.yml", fixture("processes/sync-calendar.yml") # a description of 130 characters

    show "sync-calendar"

    assert_stdout "never out of date"
    assert_lines_within 80
  end

  test "try keeps the indentation and the runs of spaces of a code snippet exactly as written" do
    write_file ".roostr/processes/fix-double-booking.yml", fixture("processes/fix-double-booking.yml") # try holds a console snippet indented by two spaces, with a comment set off by three spaces

    show "fix-double-booking"

    assert_stdout "  try      In a Rails console:\n" \
                  "             booking = Booking.find_by(reference: \"AT-2041\")\n" \
                  "             booking.move_to!(Workshop.find(17))   # frees the double-booked place\n"
  end

  test "a try line that does not fit in 80 columns is printed whole, never wrapped" do
    write_file ".roostr/processes/fix-double-booking.yml", fixture("processes/fix-double-booking.yml") # the last line of try is 85 characters

    show "fix-double-booking"

    assert_stdout "           Then check on /workshops that the number of free places for that date went up by one.\n"
  end

  test "a record with a checked date shows it on a line of its own after try, labelled checked" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    show "remind-attendees"

    assert_success
    assert_stdout(/^  try      In a console, .*\n  checked  2026-10-08$/)
  end

  test "a record without a checked date has no checked line" do
    show "remind-attendees"

    assert_success
    refute_stdout "checked"
  end
end
