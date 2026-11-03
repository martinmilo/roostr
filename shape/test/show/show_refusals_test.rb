class ShowRefusalsTest < AcceptanceTest
  def setup = workspace("atelier") # book-workshop, cancel-booking and remind-attendees

  test "an id that does not exist is refused by name" do
    show "ship-to-mars"

    assert_exit 1
    assert_stderr "ship-to-mars"
    assert_nothing_written
  end

  test "show without an id is a mistake in the command" do
    show

    assert_exit 2
    assert_nothing_written
  end

  test "a project.yml that does not parse stops show and points at validate" do
    write_file ".roostr/project.yml", "name: [unclosed\n"

    show "book-workshop"

    assert_exit 2
    assert_stderr "project.yml"
    assert_stderr "roostr validate"
  end

  test "a process record that does not parse cannot be shown and points at validate" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    show "cancel-booking"

    assert_exit 2
    assert_stderr "cancel-booking.yml"
    assert_stderr "roostr validate"
  end
end
