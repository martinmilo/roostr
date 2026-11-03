class InitTest < AcceptanceTest
  test "in an empty folder called pottery-studio, init creates .roostr/project.yml with name: pottery-studio and every other key as a commented example, and nothing else" do
    folder_named "pottery-studio"

    init

    assert_file ".roostr/project.yml", is: expected("starter_project.yml")
    assert_written_only ".roostr/project.yml"
    assert_success
  end

  test "init prints create and the file it created, then the line for AGENTS.md and the setup instruction" do
    init

    assert_stdout_is "create  .roostr/project.yml\n\n" + expected("init_guidance.txt")
    assert_success
  end

  test "a freshly initialised project validates with no problems" do
    init

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "init prints the line for AGENTS.md and never edits AGENTS.md itself" do
    write_file "AGENTS.md", "# Atelier\n\nRun bin/rails test before every commit.\n"

    init

    assert_success
    assert_unchanged "AGENTS.md"
  end

  test "run in a project that has .roostr/project.yml, init succeeds, prints exists instead of create, the same line and instruction, and overwrites nothing" do
    workspace "atelier"

    init

    assert_stdout_is "exists  .roostr/project.yml\n\n" + expected("init_guidance.txt")
    assert_nothing_written
    assert_success
  end
end
