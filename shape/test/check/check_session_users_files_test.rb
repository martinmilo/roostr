class CheckSessionUsersFilesTest < AcceptanceTest
  test "an agent's or the user's work halfway through the waiting list, left uncommitted before the session started, and nothing done in the session: check --session prints No problems found" do
    atelier_halfway_through_the_waiting_list
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user left a new route and a new job, not scanned yet, and the session only wrote README.md: the scan runs, and ENTRYPOINTS leaves out the rows of the user's files, No problems found" do
    atelier_halfway_through_the_waiting_list scanned: false
    session_start
    write_file "README.md", "# Atelier\n\nBookings for a pottery studio's workshops.\n"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user left the waiting list unscanned, and the session edited waiting_list_controller.rb: ENTRYPOINTS lists + http POST /waiting_list, the row of that file, and not the rows of files only the user changed" do
    atelier_halfway_through_the_waiting_list scanned: false
    session_start
    write_file "app/controllers/waiting_list_controller.rb", "# The waiting list's controller, written in the session.\n"

    check_session

    assert_exit 1
    assert_stdout "ENTRYPOINTS\n"
    assert_stdout "\n    + http POST /waiting_list\n"
    refute_stdout "WaitingListJob"
    refute_stdout "BookingMailer#reminder"
  end
end
