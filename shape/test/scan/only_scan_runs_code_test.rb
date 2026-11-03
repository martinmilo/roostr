class OnlyScanRunsCodeTest < AcceptanceTest
  def setup = workspace("atelier") # initialised; each test adds a .roostr/scan that creates the file scan-was-run when it is run

  test "the overview never runs the scan script" do
    scan_script "leaves-a-trace.sh"

    overview

    assert_success
    refute_file "scan-was-run"
  end

  test "show never runs the scan script" do
    scan_script "leaves-a-trace.sh"

    show "book-workshop"

    assert_success
    refute_file "scan-was-run"
  end

  test "validate never runs the scan script" do
    scan_script "leaves-a-trace.sh"

    validate

    assert_success
    refute_file "scan-was-run"
  end

  test "init never runs the scan script" do
    scan_script "leaves-a-trace.sh"

    init

    assert_success
    refute_file "scan-was-run"
  end

  test "changes never runs the scan script" do
    scan_script "leaves-a-trace.sh"
    start_repository

    changes

    assert_success
    refute_file "scan-was-run"
  end
end
