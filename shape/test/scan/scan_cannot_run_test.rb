class ScanCannotRunTest < AcceptanceTest
  test "with no scan script, scan cannot run and says to add an executable file at .roostr/scan" do
    workspace "atelier" # initialised, with no .roostr/scan

    scan

    assert_exit 2
    assert_stderr ".roostr/scan"
    assert_stderr "executable"
    assert_nothing_written
  end

  test "a scan script without the executable bit is not run, and scan says how to set the bit with chmod" do
    workspace "atelier"
    write_file ".roostr/scan", fixture("scans/leaves-a-trace.sh") # a working script, without the executable bit

    scan

    assert_exit 2
    assert_stderr "chmod +x .roostr/scan"
    assert_nothing_written
  end

  test "without .roostr/project.yml, scan says to run roostr init" do
    write_file "README.md", "# Atelier\n"

    scan

    assert_exit 2
    assert_stderr "roostr init"
    assert_nothing_written
  end
end
