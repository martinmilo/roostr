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

  test "the line for AGENTS.md ends by telling agents to run roostr check --base HEAD before each commit and before they finish, and to fix everything it reports" do
    init

    assert_stdout "run `roostr scan`. Before each commit, and before you finish, run `roostr check --base HEAD` and fix everything it reports.\n"
  end

  test "the line for AGENTS.md tells agents to run roostr before they start when their context holds no roostr project map, as when no hook gave them one" do
    init

    assert_stdout "run `roostr` for the project overview; if your context holds no roostr project map, run it before you start. Use `roostr show <id>`"
  end

  test "init prints CHECKS between the paragraph for Claude Code and Gemini CLI, after the line for AGENTS.md, and SETUP: a line for .claude/settings.json with the SessionStart hook roostr session start, given 10 seconds, and the Stop hook roostr check --session, a Git pre-commit hook that runs roostr check --staged and the command for CI" do
    init

    assert_stdout(/or the line above itself\.\n\nCHECKS\n.*\n\nSETUP\n/m)
    assert_stdout '{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"roostr session start","timeout":10}]}],"Stop":[{"hooks":[{"type":"command","command":"roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }"}]}]}}'
    assert_stdout "\n#!/bin/sh\nroostr check --staged\n"
    assert_stdout "\nroostr check --base origin/main\n"
  end

  test "init prints the line for AGENTS.md and never edits AGENTS.md itself" do
    write_file "AGENTS.md", "# Atelier\n\nRun bin/rails test before every commit.\n"

    init

    assert_success
    assert_unchanged "AGENTS.md"
  end

  test "after the line for AGENTS.md, init says that Claude Code reads AGENTS.md only where there is no CLAUDE.md and Gemini CLI reads GEMINI.md, and that a line @AGENTS.md in that file brings it in" do
    init

    assert_stdout "and fix everything it reports.\n\n" \
                  "Claude Code reads AGENTS.md only where the project has no CLAUDE.md, and Gemini\n" \
                  "CLI reads GEMINI.md unless set otherwise. For them, CLAUDE.md or GEMINI.md holds\n" \
                  "a line @AGENTS.md, which brings AGENTS.md in, or the line above itself.\n\nCHECKS\n"
  end

  test "the setup instruction ends by saying not to commit and not to edit AGENTS.md, CLAUDE.md or GEMINI.md, which the owner adds the line to" do
    init

    assert_stdout "\nDo not commit, and do not edit AGENTS.md, CLAUDE.md or GEMINI.md. The owner\nreviews .roostr/ first and adds the line.\n"
  end

  test "init never edits CLAUDE.md or GEMINI.md either" do
    write_file "CLAUDE.md", "@AGENTS.md\n\nUse the kiln room calendar for every booking test.\n"
    write_file "GEMINI.md", "# Atelier\n\nRun bin/rails test before every commit.\n"

    init

    assert_success
    assert_unchanged "CLAUDE.md"
    assert_unchanged "GEMINI.md"
  end

  test "run in a project that has .roostr/project.yml, init succeeds, prints exists instead of create, the same line and instruction, and overwrites nothing" do
    workspace "atelier"

    init

    assert_stdout_is "exists  .roostr/project.yml\n\n" + expected("init_guidance.txt")
    assert_nothing_written
    assert_success
  end
end
