class ShowOptionsTest < AcceptanceTest
  def setup = workspace("atelier") # the ids are book-workshop, cancel-booking and remind-attendees

  ["--verbose", "-q"].each do |option|
    test "show #{option} is an option show does not take: exit 2, as for any command, with the option named on stderr and nothing on stdout" do
      show option

      assert_exit 2
      assert_stderr option
      assert_stdout_is ""
    end
  end

  test "a double dash ends the options: show -- --verbose looks for a process with the id --verbose, finds none, and exits 1" do
    show "--", "--verbose"

    assert_exit 1
    assert_stderr "No process with id \"--verbose\" exists."
  end

  test "after a double dash --help is an id too: show -- --help looks for a process with the id --help, finds none, and exits 1" do
    show "--", "--help"

    assert_exit 1
    assert_stderr "No process with id \"--help\" exists."
  end

  test "show cancel-booking.yml, the record's file name, suggests the id cancel-booking" do
    show "cancel-booking.yml"

    assert_exit 1
    assert_stderr "Did you mean: cancel-booking?"
  end

  test "show .roostr/processes/cancel-booking.yml, the path validate and check print, suggests the id cancel-booking" do
    show ".roostr/processes/cancel-booking.yml"

    assert_exit 1
    assert_stderr "Did you mean: cancel-booking?"
  end
end
