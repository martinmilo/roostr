class ChangesCannotRunTest < AcceptanceTest
  test "outside a Git repository, changes cannot run and says that this is not a Git repository" do
    workspace "atelier" # initialised, and not under Git

    changes

    assert_exit 2
    assert_stderr(/git repository/i)
    assert_nothing_written
  end

  ["yesterday", "14", "2026-02-30"].each do |value|
    test "--since #{value} is neither a number of days like 7d nor a date like 2026-10-03: changes refuses the command and names #{value}" do
      workspace "atelier", git: true

      changes "--since", value

      assert_exit 2
      assert_stderr value
      assert_nothing_written
    end
  end

  test "--since with no value after it is refused as a mistake in the command" do
    workspace "atelier", git: true

    changes "--since"

    assert_exit 2
    assert_stderr "--since"
    assert_nothing_written
  end

  test "without .roostr/project.yml, changes says to run roostr init" do
    write_file "README.md", "# Atelier\n"
    start_repository

    changes

    assert_exit 2
    assert_stderr "roostr init"
    assert_nothing_written
  end
end
