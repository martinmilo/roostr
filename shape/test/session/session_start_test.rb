class SessionStartTest < AcceptanceTest
  def setup = atelier_under_git

  test "session start, given the SessionStart hook's input, prints the project map on stdout and nothing on stderr, exits 0, and writes only in .git/roostr/sessions/" do
    session_start

    assert_success
    assert_map "roostr project map of Atelier"
    assert_stderr_is ""
    assert_written_only_under ".git/roostr/sessions/"
  end

  ["5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f", "Session_2026.11-06", "s" * 128].each do |id| # 128 characters is the longest
    test "session id #{id[0, 36].inspect}#{"... (#{id.length} characters)" if id.length > 36}, ASCII letters, digits, -, _ and . and at most 128 characters, is recorded: session start writes in .git/roostr/sessions/" do
      session_start id

      assert_success
      assert_written_only_under ".git/roostr/sessions/"
    end
  end

  { "nothing" => "", "a line of text" => "Session started\n", "a JSON list" => "[\"5f0c8e2a\"]",
    "a JSON object without session_id" => "{\"hook_event_name\": \"SessionStart\"}", "a number as the session_id" => "{\"session_id\": 42}" }.each do |said, input|
    test "session start with #{said} on stdin, no usable session_id, records nothing: exit 0, stderr names session_id, and the map is printed all the same" do
      session_start stdin: input

      assert_success
      assert_map "roostr project map of Atelier"
      assert_stderr "session_id"
      assert_nothing_written git_folder: true
    end
  end

  ["", ".session", "../../hooks/pre-commit", "session one", "sessions/one", "s" * 129].each do |id| # 129 characters is one too many
    test "session id #{id[0, 30].inspect}#{" (#{id.length} characters)" if id.length > 30} is not one it would use as a file name: session start records nothing, exits 0, stderr names session_id, and the map is printed all the same" do
      session_start id

      assert_success
      assert_map "roostr project map of Atelier"
      assert_stderr "session_id"
      assert_nothing_written git_folder: true
    end
  end

  [["session"], ["session", "stop"], ["session", "start", "--json"]].each do |args|
    test "roostr #{args.join(" ")} is a mistake in the command: exit 2, the usage on stderr names roostr session start, nothing written" do
      cli(*args)

      assert_exit 2
      assert_stderr "roostr session start"
      assert_nothing_written git_folder: true
    end
  end
end
