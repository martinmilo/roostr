class SessionMapSizeTest < AcceptanceTest
  def setup = atelier_under_git

  test "a map of exactly 8,000 bytes is printed whole, with every description and no closing line" do
    write_file ".roostr/processes/archive-workshops.yml", fixture("processes/archive-workshops-for-a-map-of-8000-bytes.yml")

    session_start

    assert_map_is expected("session_map_of_8000_bytes.txt")
  end

  test "a map one byte over 8,000 is shrunk: each process as - id [first file], under the legend Processes by group, each as id [first file]:, and a closing line that says what was left out" do
    write_file ".roostr/processes/archive-workshops.yml", fixture("processes/archive-workshops-for-a-map-of-8001-bytes.yml")

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_shrunk_to_ids_and_first_files.txt")
  end

  test "with 200 more processes in six groups, even - id [first file] is over 8,000 bytes: each group is one line of its ids, under the legend Process ids by group:, and a closing line" do
    processes_written 200, groups: 6

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_shrunk_to_ids.txt")
  end

  test "with 300 more processes with long ids, even one line of ids per group is over 8,000 bytes: ids are cut from the end, the last group line kept ends in , ... after the last id it keeps, and the closing line says 130 of the 303 process ids were left out" do
    processes_written 300, groups: 6, id: "nightly-import-of-partner-workshops"

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_cut.txt")
  end

  test "with 80 watch_out items the cut reaches into Watch out: every process and operation goes first, then the last items, and the closing line says every process, every operation, and 11 of the 80 watch-out items; the header, Stack and Description stay" do
    write_file ".roostr/project.yml", fixture("project/watch-out-80-items.yml") # Atelier's, with 80 items of 103 characters

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_cut_in_watch_out.txt")
  end

  test "with 350 more processes and no group, the one line other: is over 8,000 bytes on its own: it keeps the ids that fit, ending in , ..., after Bookings and Notifications, and the closing line says how many of the ids were left out" do
    processes_written 350, groups: nil, id: "nightly-import-of-workshops"

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_cut_in_other.txt")
  end

  test "a project name of 9,007 characters is cut in the header to its first 200 characters, followed by ..., so that the map stays within 8,000 bytes; the rest of the map is whole" do
    write_file ".roostr/project.yml", fixture("project/name-of-9007-characters.yml") # Atelier's, named "Café Pottery Studio, " 428 times and once more

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_with_a_name_of_9007_characters.txt")
  end

  test "with a project description of 8,279 characters the cut reaches the Description line: every process, operation and watch-out item goes, then that line; the header and Stack stay, and the closing line names each part cut" do
    write_file ".roostr/project.yml", fixture("project/description-of-8279-characters.yml") # Atelier's, with a description of 8,279 characters

    session_start

    assert_map_at_most 8000
    assert_map_is expected("session_map_cut_to_the_stack.txt")
  end
end
