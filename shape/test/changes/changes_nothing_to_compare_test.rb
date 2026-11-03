class ChangesNothingToCompareTest < AcceptanceTest
  test "when no entrypoints.tsv was committed before the period started, entrypoints are not compared: none is listed as added and the count line says not compared" do
    workspace "atelier"
    delete_file ".roostr/entrypoints.tsv"
    start_repository # committed on 2026-01-15 without entrypoints.tsv
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/all-areas.tsv") # 7 entrypoints
    commit "Scan the entrypoints for the first time", on: "2026-10-06"

    changes

    assert_stdout "entrypoints: not compared   processes: 0 changed"
    refute_stdout "+ http"
  end

  test "in a Git repository with no commits yet, changes succeeds: entrypoints are not compared and no process changed" do
    workspace "atelier"
    git "init", "--quiet"

    changes

    assert_success
    assert_stdout "entrypoints: not compared   processes: 0 changed"
  end
end
