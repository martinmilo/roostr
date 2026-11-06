class CheckScanHelperTest < AcceptanceTest
  def setup = workspace("atelier") # not under Git; entrypoints.tsv holds Atelier's 7 entrypoints

  test "a scan script that leaves a helper running for 5 seconds with its stderr open does not hold check: it returns within 3 seconds, No problems found" do
    scan_script "leaves-a-helper-holding-stderr.sh"

    check

    assert_finished_within 3
    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "when that script fails, check returns within 3 seconds and shows what the script wrote to stderr before it exited, exit 1" do
    scan_script "fails-leaving-a-helper-holding-stderr.sh"

    check

    assert_finished_within 3
    assert_exit 1
    assert_stdout "config/routes.rb could not be loaded"
  end
end
