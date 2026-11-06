class UnreadableRecordTest < AcceptanceTest
  def setup = workspace("atelier") # book-workshop, cancel-booking and remind-attendees, all readable

  test "a process record that does not parse is listed under NEEDS ATTENTION and the rest of the overview is still printed" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    overview

    assert_success
    assert_stdout "book-workshop"
    assert_stdout(/NEEDS ATTENTION\n.*cancel-booking\.yml.*run roostr validate/)
  end

  test "a record with an unknown key, outcome: written for outcomes:, is listed under NEEDS ATTENTION as needing checking and left out of FUNCTIONALITY" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ngroup: Bookings\ndescription: Refund a cancelled booking\n" \
                                                       "outcome:\n  refunded:\n    when: the booking was cancelled\n    then: the card is refunded\n"

    overview

    assert_success
    assert_stdout(/NEEDS ATTENTION\n.*refund-booking\.yml.*run roostr validate/)
    refute_stdout "Refund a cancelled booking"
  end
end
