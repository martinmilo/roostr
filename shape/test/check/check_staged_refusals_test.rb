class CheckStagedRefusalsTest < AcceptanceTest
  test "--staged and --base do not go together: check exits 2 and says they cannot be used together, with nothing on stdout" do
    atelier_under_git

    check "--staged", "--base", "main"

    assert_exit 2
    assert_stderr "--staged"
    assert_stderr "--base"
    assert_stderr "together"
    assert_stdout_lines 0
  end

  test "--staged outside a Git repository cannot run, and check says this is not a Git repository" do
    workspace "atelier" # not under Git

    check "--staged"

    assert_exit 2
    assert_stderr(/git repository/i)
    assert_nothing_written
  end
end
