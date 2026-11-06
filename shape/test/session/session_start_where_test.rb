class SessionStartWhereTest < AcceptanceTest
  test "run in app/models, as when Claude Code's current folder is there, session start still writes only in the repository's .git/roostr/sessions/" do
    atelier_under_git

    session_start from: "app/models"

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "in a linked worktree, second-checkout, session start writes only in the repository's common Git folder, .git/roostr/sessions/, where every worktree of it finds the record" do
    atelier_under_git
    linked_worktree "second-checkout"

    session_start from: "second-checkout"

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "with its stdout a full disk, /dev/full, so that the map cannot be written, session start still records the session and exits 0" do
    atelier_under_git

    session_start stdout: :full

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "run in a folder that has been deleted, session start exits 0, prints nothing, records nothing, and says in one line on stderr that the current folder no longer exists" do
    atelier_under_git

    session_start from_a_deleted_folder: true

    assert_success
    assert_stdout_is ""
    assert_stderr "roostr session start: the current folder no longer exists; nothing recorded"
    assert_nothing_written git_folder: true
  end

  test "session start needs no .roostr/: at the top of a repository whose project is in apps/atelier, it records the session in .git/roostr/sessions/" do
    atelier_inside_a_larger_repository

    session_start

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "outside a Git repository session start records nothing: exit 0, stderr says it is not in a Git repository, and the map is printed all the same, byte for byte as in Git" do
    workspace "atelier"

    session_start

    assert_success
    assert_map_is expected("session_map_atelier.txt")
    assert_stderr "not in a Git repository"
    assert_nothing_written
  end

  test "with no git on the PATH session start records nothing: exit 0, stderr says git was not found, and the map is printed all the same, byte for byte as with git" do
    atelier_under_git
    git_not_on_the_path

    session_start

    assert_success
    assert_map_is expected("session_map_atelier.txt")
    assert_stderr "git was not found on the PATH"
  end

  test "in a repository Git refuses, session start records nothing: exit 0, stderr gives what Git said, the safe.directory command among it, and the map is printed all the same" do
    atelier_under_git
    git_refuses_the_repository

    session_start

    assert_success
    assert_map "roostr project map of Atelier"
    assert_stderr "git config --global --add safe.directory"
  end
end
