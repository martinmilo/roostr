class JsonNotTakenTest < AcceptanceTest
  test "scan does not take --json: it exits 2 naming --json, does not run the scan script and writes nothing" do
    workspace "atelier"
    scan_script "leaves-a-trace.sh" # creates scan-was-run whenever it runs

    scan "--json"

    assert_exit 2
    assert_stderr "--json"
    assert_nothing_written
  end

  test "init does not take --json: it exits 2 naming --json and creates nothing" do
    init "--json"

    assert_exit 2
    assert_stderr "--json"
    assert_nothing_written
  end
end
