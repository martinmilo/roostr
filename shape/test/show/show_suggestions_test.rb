class ShowSuggestionsTest < AcceptanceTest
  def setup = workspace("atelier") # the ids are book-workshop, cancel-booking and remind-attendees

  test "show booking, an id that does not exist, is refused as before, exit 1, and the line after it suggests the id that contains it: Did you mean: cancel-booking?" do
    show "booking"

    assert_exit 1
    assert_stderr "No process with id \"booking\" exists.\nDid you mean: cancel-booking?\n"
  end

  ["Book_Workshop", "bookworkshop", "BOOK-WORKSHOP"].each do |given|
    test "show #{given} suggests book-workshop, which it matches once case, - and _ are ignored" do
      show given

      assert_exit 1
      assert_stderr "Did you mean: book-workshop?"
    end
  end

  test "a part of an id in another case is found as well: show Booking suggests cancel-booking" do
    show "Booking"

    assert_exit 1
    assert_stderr "Did you mean: cancel-booking?"
  end

  test "several ids that contain the text are suggested in alphabetical order, separated by commas: show book suggests book-workshop, cancel-booking" do
    show "book"

    assert_exit 1
    assert_stderr "Did you mean: book-workshop, cancel-booking?"
  end

  test "at most five ids are suggested, the first five in alphabetical order" do
    write_file ".roostr/processes/export-bookings.yml", fixture("processes/export-bookings.yml")
    write_file ".roostr/processes/fix-double-booking.yml", fixture("processes/fix-double-booking.yml")
    write_file ".roostr/processes/rebook-workshop.yml", "name: Rebook a workshop\ndescription: Move a booking to another date of the same workshop.\n"
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # now six ids contain book; refund-booking is the sixth

    show "book"

    assert_stderr "Did you mean: book-workshop, cancel-booking, export-bookings, fix-double-booking, rebook-workshop?\n"
  end

  test "when no id is close, show prints only the line it printed before, with no suggestion" do
    show "ship-to-mars"

    assert_exit 1
    assert_stderr "No process with id \"ship-to-mars\" exists."
    refute_stderr "Did you mean"
  end

  test "with --json, stdout stays empty and the suggestion goes to stderr with the rest" do
    show "book", "--json"

    assert_exit 1
    assert_stdout_is ""
    assert_stderr "Did you mean: book-workshop, cancel-booking?"
  end
end
