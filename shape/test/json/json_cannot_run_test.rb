class JsonCannotRunTest < AcceptanceTest
  test "without .roostr/project.yml, roostr --json exits 2, prints nothing on stdout and says on stderr to run roostr init" do
    write_file "README.md", "# Atelier\n"

    overview "--json"

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "roostr init"
  end

  test "show --json with an id that does not exist exits 1, prints nothing on stdout and names the id on stderr" do
    workspace "atelier"

    show "refund-booking", "--json"

    assert_exit 1
    assert_stdout_is ""
    assert_stderr "refund-booking"
  end

  test "show --json on a record that does not parse exits 2, prints nothing on stdout and names the record on stderr" do
    workspace "atelier"
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    show "cancel-booking", "--json"

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "cancel-booking.yml"
  end

  test "check --json with a base that is not a commit exits 2, prints nothing on stdout and names the base on stderr" do
    atelier_under_git

    check "--base", "mian", "--json"

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "mian"
  end
end
