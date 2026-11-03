class SectionsLeftOutTest < AcceptanceTest
  test "with no operations, WORK WITH THIS PROJECT is left out and ENVIRONMENTS is still shown" do
    workspace "atelier"
    write_file ".roostr/project.yml", "name: Atelier\nenvironments:\n  production: atelier.example (Hetzner)\n"

    overview

    assert_stdout(/^ENVIRONMENTS\n  production\s+atelier\.example \(Hetzner\)$/)
    refute_stdout "WORK WITH THIS PROJECT"
  end

  test "with no environments, ENVIRONMENTS is left out and WORK WITH THIS PROJECT is still shown" do
    workspace "atelier"
    write_file ".roostr/project.yml", "name: Atelier\noperations:\n  start: bin/dev\n"

    overview

    assert_stdout(/^WORK WITH THIS PROJECT\n  start\s+bin\/dev$/)
    refute_stdout "ENVIRONMENTS"
  end

  test "with no watch_out, WATCH OUT is left out" do
    workspace "atelier"
    write_file ".roostr/project.yml", "name: Atelier\nstack: Rails 8, PostgreSQL, Solid Queue\n"

    overview

    assert_stdout "Atelier"
    refute_stdout "WATCH OUT"
  end

  test "a project with only project.yml prints the heading and FUNCTIONALITY reading not scanned yet" do
    write_file ".roostr/project.yml", "name: Atelier\nstack: Rails 8, PostgreSQL, Solid Queue\n"

    overview

    assert_success
    assert_stdout(/\AAtelier\s+Rails 8, PostgreSQL, Solid Queue\n/)
    assert_stdout(/FUNCTIONALITY\s+not scanned yet/)
    refute_stdout "WORK WITH THIS PROJECT"
    refute_stdout "ENVIRONMENTS"
  end

  test "in a folder that is not a Git repository, LAST 7 DAYS is left out, and NEEDS ATTENTION is left out when no record is unreadable" do
    workspace "atelier"

    overview

    assert_stdout "Atelier"
    refute_stdout "LAST 7 DAYS"
    refute_stdout "NEEDS ATTENTION"
  end
end
