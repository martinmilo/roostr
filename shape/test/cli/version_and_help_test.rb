class VersionAndHelpTest < AcceptanceTest
  test "roostr --version, in a folder with no .roostr/, prints one line, roostr and the version as X.Y.Z, and exits 0" do
    cli "--version"

    assert_success
    assert_stdout(/\Aroostr \d+\.\d+\.\d+\n\z/)
  end

  test "roostr -V is the same as roostr --version: one line, roostr and the version as X.Y.Z, exit 0" do
    cli "-V"

    assert_success
    assert_stdout(/\Aroostr \d+\.\d+\.\d+\n\z/)
  end

  test "roostr --help names the check command" do
    cli "--help"

    assert_success
    assert_stdout "roostr check"
  end

  test "roostr -h is the same as roostr --help: the usage on stdout, exit 0, nothing on stderr" do
    cli "-h"

    assert_success
    assert_stdout "Usage: roostr"
    assert_stderr_is ""
  end

  test "the usage names roostr session start and check --session" do
    cli "--help"

    assert_success
    assert_stdout "roostr session start"
    assert_stdout "--session"
  end
end
