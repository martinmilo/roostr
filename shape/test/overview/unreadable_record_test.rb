class UnreadableRecordTest < AcceptanceTest
  def setup = workspace("atelier") # book-workshop, cancel-booking and remind-attendees, all readable

  test "a process record that does not parse is listed under NEEDS ATTENTION and the rest of the overview is still printed" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    overview

    assert_success
    assert_stdout "book-workshop"
    assert_stdout(/NEEDS ATTENTION\n.*cancel-booking\.yml.*run roostr validate/)
  end
end
