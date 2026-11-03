class FunctionalitySectionTest < AcceptanceTest
  def setup = workspace("atelier") # 3 processes in groups Bookings and Notifications; 7 entrypoints in areas Bookings, Notifications, Workshops and 2 without an area; 5 described

  test "the FUNCTIONALITY heading counts the entrypoints and how many a process describes" do
    overview

    assert_success
    assert_stdout "7 entrypoints, 5 described"
  end

  test "an area with entrypoints but no process is listed with its counts" do
    overview

    assert_stdout(/^  Workshops\s+http 1$/)
  end

  test "a process whose group matches no area is listed under its own heading with no counts" do
    write_file ".roostr/processes/sync-calendar.yml", fixture("processes/sync-calendar.yml") # group Imports, which no entrypoint has as its area

    overview

    assert_stdout(/^  Imports\n\s+sync-calendar /)
  end

  test "a process without a group is listed under other" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops.yml")

    overview

    assert_stdout(/^  other .*\n\s+import-workshops /)
  end

  test "a long description wraps under its column and is never cut" do
    write_file ".roostr/processes/sync-calendar.yml", fixture("processes/sync-calendar.yml") # a description of 130 characters

    overview

    assert_stdout "    sync-calendar      Imports the studio calendar every night and creates or\n" \
                  "                       updates workshops, trainers and rooms so that the booking\n" \
                  "                       pages are never out of date.\n"
    refute_stdout "..."
  end

  test "with entrypoints but no processes, FUNCTIONALITY shows the areas and their counts" do
    delete_file ".roostr/processes/book-workshop.yml"
    delete_file ".roostr/processes/cancel-booking.yml"
    delete_file ".roostr/processes/remind-attendees.yml"

    overview

    assert_stdout "7 entrypoints, 0 described"
    assert_stdout(/^  Bookings\s+http 3$/)
    refute_stdout "book-workshop"
  end
end
