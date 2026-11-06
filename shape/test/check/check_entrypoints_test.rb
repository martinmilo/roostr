class CheckEntrypointsTest < AcceptanceTest
  def setup = atelier_under_git # entrypoints.tsv covers http, job and mail and holds 7 entrypoints

  test "when the scan script finds entrypoints that entrypoints.tsv lacks and misses one it has, check lists them under ENTRYPOINTS with + and -, says to run roostr scan, exits 1 and leaves entrypoints.tsv as it was" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list and WaitingListJob, no longer finds BookingMailer#reminder

    check

    assert_exit 1
    assert_stdout "ENTRYPOINTS\n"
    assert_stdout "\n    + http POST /waiting_list\n    + job WaitingListJob\n    - mail BookingMailer#reminder\n"
    assert_stdout "roostr scan to update it."
    assert_nothing_written
  end

  test "a scan script that finds exactly what entrypoints.tsv holds, in another order, is no problem" do
    scan_prints fixture("scans/atelier.txt") # the 7 entrypoints of Atelier, in another order

    check

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a scan script that finds the same entrypoints with one of them in another file is reported, because entrypoints.tsv is not what roostr scan would write, with no line marked + or -" do
    scan_prints fixture("scans/atelier-booking-moved.txt") # the same 7; POST /bookings is now in checkout_controller.rb

    check

    assert_exit 1
    assert_stdout "roostr scan to update it."
    refute_stdout(/^    [+-] /)
  end

  test "a scan script that exits 3 is a finding that names the exit code and shows, inside the finding on stdout, what the script wrote to stderr" do
    scan_prints fixture("scans/atelier.txt"), stderr: "bin/rails: database atelier_development does not exist", exit: 3

    check

    assert_exit 1
    assert_stdout "exited with code 3"
    assert_stdout "\n    bin/rails: database atelier_development does not exist\n"
  end

  test "a scan script whose output has a row of two columns is a finding that gives the reason roostr scan gives, naming line 2" do
    scan_prints "# covers: http\nhttp\tGET /workshops\n"

    check

    assert_exit 1
    assert_stdout "line 2"
  end

  test "a scan script without the executable bit is a finding, exit 1, that gives chmod +x .roostr/scan" do
    write_file ".roostr/scan", fixture("scans/leaves-a-trace.sh") # a working script, without the executable bit

    check

    assert_exit 1
    assert_stdout "chmod +x .roostr/scan"
  end

  test "a scan script that writes a warning to stderr and succeeds passes, and the warning is shown nowhere" do
    scan_prints fixture("scans/atelier.txt"), stderr: "DEPRECATION WARNING: config.active_job.queue_adapter is not set"

    check

    assert_success
    refute_stdout "DEPRECATION WARNING"
    refute_stderr "DEPRECATION WARNING"
  end

  test "started in app/models, check runs the scan script in the project folder, the folder .roostr/ is in" do
    scan_script "lists-controllers.sh" # one http row per file under app/controllers, found by a path relative to the folder it starts in

    check from: "app/models"

    assert_exit 1
    assert_stdout "+ http GET /bookings"
    refute_stdout "which has no app/controllers"
  end
end
