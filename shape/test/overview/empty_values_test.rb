class OverviewEmptyValuesTest < AcceptanceTest
  test "watch_out: uncommented in the file init writes, with its example item still commented out, is no list at all: the overview prints, exit 0, without WATCH OUT" do
    write_file ".roostr/project.yml", fixture("project/watch-out-uncommented.yml") # init's file for pottery-studio, with "# " removed in front of watch_out: only

    overview

    assert_success
    assert_stdout(/\Apottery-studio\n/)
    refute_stdout "WATCH OUT"
  end

  test "project.yml whose optional keys have nothing after them prints, exit 0: the name alone on the heading line, and no WATCH OUT, WORK WITH THIS PROJECT or ENVIRONMENTS" do
    write_file ".roostr/project.yml", fixture("project/keys-left-empty.yml") # stack:, watch_out:, operations:, environments: with nothing after them, description: ""

    overview

    assert_success
    assert_stdout(/\AAtelier\n/)
    refute_stdout "WATCH OUT"
    refute_stdout "WORK WITH THIS PROJECT"
    refute_stdout "ENVIRONMENTS"
  end

  test "a record whose group has nothing after it is listed under other, as one without a group" do
    workspace "atelier"
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml")

    overview

    assert_success
    assert_stdout(/^  other .*\n\s+import-workshops /)
  end

  test "an operation with no command is left out of WORK WITH THIS PROJECT, and the overview still prints, exit 0" do
    write_file ".roostr/project.yml", fixture("project/deploy-without-command.yml") # start, test, and deploy: with its command line still commented out

    overview

    assert_success
    assert_stdout(/^  test +bin\/rails test\n/)
    refute_stdout "deploy"
  end

  test "an environment with nothing after its name is left out of ENVIRONMENTS, and the overview still prints, exit 0" do
    write_file ".roostr/project.yml", fixture("project/staging-left-empty.yml") # staging: with nothing after it, production: atelier.example (Hetzner)

    overview

    assert_success
    assert_stdout(/^  production +atelier\.example \(Hetzner\)\n/)
    refute_stdout "staging"
  end
end
