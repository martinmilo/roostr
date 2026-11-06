class ChangesGitCannotBeReadTest < AcceptanceTest
  def setup = atelier_under_git # committed once

  test "with git not on the PATH, changes cannot run: exit 2, nothing on stdout, and stderr says that git was not found on the PATH" do
    git_not_on_the_path

    changes

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "git was not found on the PATH"
  end

  test "when Git refuses the repository, changes exits 2 and passes on all that Git said, the safe.directory command included" do
    git_refuses_the_repository

    changes

    assert_exit 2
    assert_stderr "fatal: detected dubious ownership in repository at '/workspace/atelier'"
    assert_stderr "git config --global --add safe.directory /workspace/atelier"
  end

  test "with a Git older than 2.25, changes exits 2 and says that roostr needs Git 2.25 or later and that this one is 2.20.0" do
    git_older_than_supported

    changes

    assert_exit 2
    assert_stderr "2.25 or later"
    assert_stderr "2.20.0"
  end
end
