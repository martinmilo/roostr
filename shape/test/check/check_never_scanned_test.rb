class CheckNeverScannedTest < AcceptanceTest
  test "a project with a scan script and no entrypoints.tsv yet, never scanned, is a finding under ENTRYPOINTS that says to run roostr scan" do
    init # a new project: no entrypoints.tsv, no records
    scan_prints fixture("scans/atelier.txt")

    check

    assert_exit 1
    assert_stdout "ENTRYPOINTS\n"
    assert_stdout "roostr scan"
  end
end
