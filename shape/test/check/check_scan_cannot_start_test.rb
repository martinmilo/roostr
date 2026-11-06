class CheckScanCannotStartTest < AcceptanceTest
  def setup = workspace("atelier") # not under Git; entrypoints.tsv covers http, job and mail

  test "an executable scan script saved with Windows line endings cannot be started: check reports, exit 1, that its shebang line (#!) names an interpreter that cannot be started and that the file has Windows line endings, and does not suggest chmod" do
    scan_prints fixture("scans/atelier.txt")
    saved_with_windows_line_endings ".roostr/scan"

    check

    assert_exit 1
    assert_stdout "#!"
    assert_stdout "Windows line endings"
    refute_stdout "chmod"
  end

  test "with --json, that script is one problem of kind scan_cannot_start, with interpreter the program its shebang line names, as written, carriage return included" do
    scan_prints fixture("scans/atelier.txt")
    saved_with_windows_line_endings ".roostr/scan"

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "scan_cannot_start", "interpreter" => "/bin/sh\r" }]
  end

  test "an executable scan script whose shebang line names an interpreter that is not installed cannot be started: check names the interpreter, and says nothing of line endings" do
    write_executable ".roostr/scan", "#!/opt/atelier/venv/bin/python3.13\nprint(\"# covers: http job mail\")\n"

    check

    assert_exit 1
    assert_stdout "/opt/atelier/venv/bin/python3.13"
    refute_stdout "Windows"
  end
end
