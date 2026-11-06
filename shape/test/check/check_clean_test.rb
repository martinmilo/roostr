class CheckCleanTest < AcceptanceTest
  test "in a project whose .roostr/ is current, check prints only No problems found, exits 0 and writes nothing" do
    atelier_under_git # committed once, nothing uncommitted, no scan script

    check

    assert_success
    assert_stdout_is "No problems found\n"
    assert_nothing_written
  end

  test "a freshly initialised project, with no scan script, no entrypoints.tsv and no records, passes check: roostr works without a scan" do
    init

    check

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with --base HEAD and nothing changed since the last commit, check prints No problems found and does not run the scan script" do
    atelier_under_git
    commit_scan_script "leaves-a-trace.sh" # creates scan-was-run whenever it runs

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
    refute_file "scan-was-run"
  end
end
