class HelpAfterACommandTest < AcceptanceTest
  [["show", "--help"], ["show", "-h"], ["show", "book-workshop", "-h"], ["changes", "--since", "3d", "--help"],
   ["check", "--base", "HEAD", "--help"], ["validate", "-h"], ["scan", "--help"], ["init", "--help"],
   ["session", "start", "-h"], ["--json", "check", "--help"]].each do |args|
    test "roostr #{args.join(" ")}, in an empty folder, prints the usage on stdout and exits 0, with nothing on stderr and nothing written" do
      cli(*args)

      assert_success
      assert_stdout "Usage: roostr"
      assert_stderr_is ""
      assert_nothing_written
    end
  end

  test "an option no command takes, check --verbose, still exits 2 with the usage on stderr and nothing on stdout" do
    workspace "atelier"

    check "--verbose"

    assert_exit 2
    assert_stderr "Usage: roostr"
    assert_stdout_is ""
  end
end
