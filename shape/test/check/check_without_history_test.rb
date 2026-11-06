class CheckWithoutHistoryTest < AcceptanceTest
  test "outside a Git repository, check still compares entrypoints.tsv with the scan script and reports it behind, and has no PROCESSES block" do
    workspace "atelier" # not under Git
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list, which entrypoints.tsv lacks

    check

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
    refute_stdout "PROCESSES"
  end

  test "in a Git repository with no commits yet, check without --base passes a valid project: no record has been committed, so none is behind" do
    workspace "atelier"
    git "init", "--quiet", "--initial-branch=main"

    check

    assert_success
    assert_stdout_is "No problems found\n"
  end
end
