class ScanWritesTest < AcceptanceTest
  def setup = workspace("atelier") # entrypoints.tsv covers http, job and mail and holds 7 entrypoints

  test "scan writes the rows the scan script prints to entrypoints.tsv, the covers line first and the rows sorted by their whole line, and writes nothing else" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # 8 rows as a script finds them: jobs, mails, then routes

    scan

    assert_success
    assert_file ".roostr/entrypoints.tsv", is: expected("entrypoints_with_waiting_list.tsv")
    assert_written_only ".roostr/entrypoints.tsv"
  end

  test "a first scan, with no entrypoints.tsv yet, writes the file and prints only the line 7 entrypoints, first scan" do
    delete_file ".roostr/entrypoints.tsv"
    scan_prints fixture("scans/atelier.txt") # the 7 entrypoints of Atelier

    scan

    assert_success
    assert_file ".roostr/entrypoints.tsv", includes: "# covers: http job mail\nhttp\tDELETE /bookings/:id\t"
    assert_stdout_is "7 entrypoints, first scan\n"
  end

  test "a scan that finds no entrypoints writes the covers line alone" do
    scan_prints "# covers: http job mail\n"

    scan

    assert_success
    assert_file ".roostr/entrypoints.tsv", is: "# covers: http job mail\n"
    assert_stdout "0 entrypoints, 0 added, 7 removed"
  end

  test "the covers line is written with its kinds in the order http, job, mail, schedule, event, cli, whatever order the script names them in" do
    scan_prints "# covers: cli mail http\n"

    scan

    assert_file ".roostr/entrypoints.tsv", is: "# covers: http mail cli\n"
  end

  test "a row whose fourth column is empty is written with three columns and no trailing tab" do
    scan_prints "# covers: mail\nmail\tBookingMailer#reminder\tapp/mailers/booking_mailer.rb\t\n"

    scan

    assert_file ".roostr/entrypoints.tsv", is: "# covers: mail\nmail\tBookingMailer#reminder\tapp/mailers/booking_mailer.rb\n"
  end

  test "the scan script is started in the repository root, the folder the .roostr folder is in" do
    scan_script "lists-controllers.sh" # one http row per file under app/controllers, found by a path relative to the root

    scan

    assert_success
    assert_file ".roostr/entrypoints.tsv", includes: "http\tGET /workshops\tapp/controllers/workshops_controller.rb\n"
  end

  test "a script that writes to stderr and exits 0 succeeds, and what it wrote is passed on to stderr" do
    scan_prints fixture("scans/atelier.txt"), stderr: "DEPRECATION WARNING: config.active_job.queue_adapter is not set"

    scan

    assert_success
    assert_stderr "DEPRECATION WARNING: config.active_job.queue_adapter is not set"
    assert_stdout "7 entrypoints, 0 added, 0 removed"
  end
end
