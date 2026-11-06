class ScanHelperTest < AcceptanceTest
  def setup = workspace("atelier")

  test "a scan script that leaves a helper running for 5 seconds with its stderr open does not hold roostr scan: it returns within 3 seconds" do
    scan_script "leaves-a-helper-holding-stderr.sh"

    scan

    assert_finished_within 3
    assert_success
  end
end
