class NotInitialisedTest < AcceptanceTest
  test "without .roostr/project.yml the overview says to run roostr init" do
    write_file "README.md", "# Atelier\n"

    overview

    assert_exit 2
    assert_stderr "roostr init"
    assert_nothing_written
  end
end
