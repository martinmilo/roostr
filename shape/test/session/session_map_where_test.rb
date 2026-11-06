class SessionMapWhereTest < AcceptanceTest
  test "at the top of a repository whose project is in apps/atelier there is no .roostr/project.yml, so no map: nothing on stdout or stderr, exit 0" do
    atelier_inside_a_larger_repository

    session_start

    assert_success
    assert_stdout_is ""
    assert_stderr_is ""
  end

  test "in apps/atelier of a larger repository, session start prints Atelier's map" do
    atelier_inside_a_larger_repository

    session_start from: "apps/atelier"

    assert_success
    assert_map_is expected("session_map_atelier.txt")
  end

  test "in app/models of Atelier, session start finds .roostr/project.yml above it and prints Atelier's map" do
    atelier_under_git

    session_start from: "app/models"

    assert_map_is expected("session_map_atelier.txt")
  end

  { "does not parse" => "name: [Atelier\n", "has a key roostr does not know, owner" => "name: Atelier\nowner: Studio team\n",
    "has no name" => "stack: Rails 8, PostgreSQL, Solid Queue\n" }.each do |said, content|
    test "a project.yml that #{said} cannot be read, as by the overview: no map, exit 0, one line on stderr names .roostr/project.yml and roostr validate, and the session is still recorded" do
      atelier_under_git
      write_file ".roostr/project.yml", content

      session_start

      assert_success
      assert_stdout_is ""
      assert_stderr ".roostr/project.yml"
      assert_stderr "roostr validate"
      assert_stderr(/\A[^\n]*\n\z/) # one line
      assert_written_only_under ".git/roostr/sessions/"
    end
  end

  test "a project.yml with 8 watch_out items, a validate finding that leaves it readable, still gives a map, with every item" do
    atelier_under_git
    write_file ".roostr/project.yml", fixture("project/watch-out-8-items.yml") # the 8th: Prices are stored in cents.

    session_start

    assert_map "\n- Prices are stored in cents."
  end

  test "the map does not depend on the day: with ROOSTR_TODAY 2026-12-24, Atelier's map is the same, byte for byte" do
    atelier_under_git

    session_start today: "2026-12-24"

    assert_map_is expected("session_map_atelier.txt")
  end

  test "the map does not depend on the terminal's width: with COLUMNS 40, Atelier's map is the same, byte for byte, and its lines are not wrapped" do
    atelier_under_git

    session_start env: { "COLUMNS" => "40" }

    assert_map_is expected("session_map_atelier.txt")
  end

  test "a project whose project.yml has only a name, and no record, has the header alone as its map" do
    workspace "just-a-name", git: true # name: Pottery Studio

    session_start

    assert_map_is "roostr project map of Pottery Studio, from .roostr/, kept by people and agents working here. Records can lag the code; where they differ, the code is current. `roostr show` with a process id prints a process in full: files, entrypoints, how to try it, outcomes. `roostr` prints the overview and recent changes."
  end
end
