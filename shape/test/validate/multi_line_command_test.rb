class MultiLineCommandTest < AcceptanceTest
  def setup = write_file(".roostr/project.yml", fixture("project/setup-in-two-lines.yml")) # setup: a | block of two lines, bin/setup and bin/rails db:seed; then start: bin/dev

  test "an operation whose command has two lines is a finding on operations.setup that says to put the steps in a script, exit 1" do
    validate

    assert_exit 1
    assert_stdout ".roostr/project.yml: operations.setup: "
    assert_stdout "script"
  end

  test "the overview still prints, exit 0, and shows the first line of the command followed by a space and three dots" do
    overview

    assert_success
    assert_stdout "  setup    bin/setup ...\n" \
                  "  start    bin/dev\n"
  end

  test "roostr --json gives the whole command: both lines and the newline at its end" do
    overview "--json"

    assert_success
    assert_json "project.operations.1", { "name" => "setup", "command" => "bin/setup\nbin/rails db:seed\n", "note" => nil }
  end

  test "a command written as a YAML literal block (setup: |) of one line is one line: the newline at its end is no finding" do
    write_file ".roostr/project.yml", fixture("project/setup-as-a-block-of-one-line.yml") # setup: a | block of one line, bin/setup

    validate

    assert_success
  end

  test "the overview shows a command written as a literal block of one line as that line alone" do
    write_file ".roostr/project.yml", fixture("project/setup-as-a-block-of-one-line.yml")

    overview

    assert_success
    assert_stdout "  setup    bin/setup\n"
  end
end
