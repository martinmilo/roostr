class ChangesJsonTest < AcceptanceTest
  test "after the week of the waiting list, changes --json gives the first day, the entrypoints added and removed, and each changed process with its subjects newest first, exactly as laid out" do
    atelier_after_the_waiting_list_week # one commit in September, five from 5 to 9 October; today is 2026-10-10

    changes "--json"

    assert_success
    assert_json_is expected("changes_atelier.json")
  end

  test "when no entrypoints.tsv was committed before the period, entrypoints is null where the text says not compared" do
    workspace "atelier"
    delete_file ".roostr/entrypoints.tsv"
    start_repository # committed on 2026-01-15 without entrypoints.tsv
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/all-areas.tsv")
    commit "Scan the entrypoints for the first time", on: "2026-10-06"

    changes "--json"

    assert_json "entrypoints", nil
  end

  test "in a Git repository with no commits yet, changes --json gives entrypoints null and processes empty" do
    workspace "atelier"
    git "init", "--quiet"

    changes "--json"

    assert_success
    assert_json "entrypoints", nil
    assert_json "processes", []
  end

  test "when nothing changed in the period, entrypoints has added and removed empty, and processes is empty" do
    atelier_under_git # committed once, on 2026-01-15; the period starts on 2026-10-03

    changes "--json"

    assert_json "entrypoints", { "added" => [], "removed" => [] }
    assert_json "processes", []
  end
end
