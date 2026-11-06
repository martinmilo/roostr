class CheckJsonEntrypointsTest < AcceptanceTest
  def setup = atelier_under_git # entrypoints.tsv covers http, job and mail and holds 7 entrypoints

  test "a scan that finds entrypoints entrypoints.tsv lacks, and misses one it has, gives one problem of kind entrypoints_behind with added and removed, and exits 1 with the JSON on stdout" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list and WaitingListJob, no longer finds BookingMailer#reminder

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "entrypoints_behind", "added" => ["http POST /waiting_list", "job WaitingListJob"],
                               "removed" => ["mail BookingMailer#reminder"] }]
  end

  test "a scan that finds the same entrypoints with one of them in another file gives entrypoints_behind with added and removed both empty" do
    scan_prints fixture("scans/atelier-booking-moved.txt") # the same 7; POST /bookings is now in checkout_controller.rb

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "entrypoints_behind", "added" => [], "removed" => [] }]
  end

  test "a scan script without the executable bit gives one problem of kind scan_not_executable" do
    write_file ".roostr/scan", fixture("scans/leaves-a-trace.sh") # a working script, without the executable bit

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "scan_not_executable" }]
  end

  test "a scan script that exits 3 gives scan_failed with exit_code 3 and stderr exactly the two lines it wrote, the final newline included" do
    scan_prints fixture("scans/atelier.txt"), stderr: "bin/rails: database atelier_development does not exist\nRun bin/rails db:create first.", exit: 3

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "scan_failed", "exit_code" => 3,
                               "stderr" => "bin/rails: database atelier_development does not exist\nRun bin/rails db:create first.\n" }]
  end

  test "a scan script whose output has a row of two columns gives scan_malformed with one reason, on line 2, and its message" do
    scan_prints "# covers: http\nhttp\tGET /workshops\n"

    check "--json"

    assert_exit 1
    assert_json "problems.*.kind", ["scan_malformed"]
    assert_json "problems.1.reasons.*.line", [2]
    assert_json "problems.1.reasons.1.message", String
  end
end
