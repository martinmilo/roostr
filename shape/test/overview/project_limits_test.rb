class ProjectLimitsTest < AcceptanceTest
  def setup = workspace("atelier")

  test "an environment value of 161 characters, over the limit, does not stop the overview: it prints, exit 0, and validate reports the value" do
    write_file ".roostr/project.yml", fixture("project/environment-161.yml") # staging's value has 161 characters and mentions Falkenstein

    overview

    assert_success
    assert_stdout "Falkenstein"

    validate

    assert_exit 1
    assert_stdout "environments.staging"
  end

  test "an operation note of 161 characters, over the limit, does not stop show: the process is shown, exit 0" do
    write_file ".roostr/project.yml", fixture("project/note-161.yml") # the note of the operation test has 161 characters

    show "book-workshop"

    assert_success
    assert_stdout "Book a place on a workshop"
  end

  test "watch_out written as one string instead of a list still stops the overview, exit 2, pointing at validate: a wrong shape cannot be read, a long value can" do
    write_file ".roostr/project.yml", "name: Atelier\nwatch_out: Stripe runs in test mode on staging.\n"

    overview

    assert_exit 2
    assert_stderr "project.yml"
    assert_stderr "roostr validate"
  end
end
