class ScanRefusalsTest < AcceptanceTest
  def setup = workspace("atelier") # entrypoints.tsv covers http, job and mail and holds 7 entrypoints

  test "a script that exits 3 is refused even when its rows are well formed, giving the exit code and passing on what the script wrote to stderr" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt"), stderr: "bin/rails: database atelier_development does not exist", exit: 3

    scan

    assert_exit 1
    assert_stderr(/\b3\b/)
    assert_stderr "database atelier_development does not exist"
    assert_nothing_written
  end

  test "output whose first line is not a covers line is refused, and the reason names the covers line" do
    scan_prints "http\tGET /workshops\tapp/controllers/workshops_controller.rb\tWorkshops\n"

    scan

    assert_exit 1
    assert_stderr "# covers:"
    assert_nothing_written
  end

  test "a covers line naming a kind outside http, job, mail, schedule, event, cli is refused, naming the kind" do
    scan_prints "# covers: http webhook\n"

    scan

    assert_exit 1
    assert_stderr "webhook"
    assert_nothing_written
  end

  { "two" => "http\tGET /workshops",
    "five" => "http\tGET /workshops\tapp/controllers/workshops_controller.rb\tWorkshops\tpublic" }.each do |count, row|
    test "a row of #{count} columns is refused because a row has three or four, naming it by its line in the script's output, line 2" do
      scan_prints "# covers: http\n#{row}\n"

      scan

      assert_exit 1
      assert_stderr "line 2"
      assert_nothing_written
    end
  end

  test "a row with a kind outside the six kinds is refused, naming its line and the kind" do
    scan_prints "# covers: http\nhttp\tGET /workshops\tapp/controllers/workshops_controller.rb\nwebhook\tstripe.payment\tapp/controllers/stripe_controller.rb\n"

    scan

    assert_exit 1
    assert_stderr "line 3"
    assert_stderr "webhook"
    assert_nothing_written
  end

  test "a row with a kind the covers line does not name is refused, naming its line and the kind" do
    scan_prints "# covers: http\njob\tBookingReminderJob\tapp/jobs/booking_reminder_job.rb\n"

    scan

    assert_exit 1
    assert_stderr "line 2"
    assert_stderr "job"
    assert_nothing_written
  end

  test "a row whose file column is empty is refused, naming its line and the file column" do
    scan_prints "# covers: http\nhttp\tGET /health\t\n"

    scan

    assert_exit 1
    assert_stderr "line 2"
    assert_stderr "file"
    assert_nothing_written
  end

  test "two rows with the same kind and name are refused, naming the line of the second and the entrypoint as kind and name" do
    scan_prints "# covers: mail\nmail\tBookingMailer#reminder\tapp/mailers/booking_mailer.rb\nmail\tBookingMailer#reminder\tapp/mailers/reminder_mailer.rb\n"

    scan

    assert_exit 1
    assert_stderr "line 3"
    assert_stderr "mail BookingMailer#reminder"
    assert_nothing_written
  end

  test "two malformed rows are both reported in one run, each with its line" do
    scan_prints "# covers: http\nhttp\tGET /workshops\nhttp\tPOST /bookings\tapp/controllers/bookings_controller.rb\nhttp\tGET /bookings/:id\n"

    scan

    assert_exit 1
    assert_stderr "line 2"
    assert_stderr "line 4"
    assert_nothing_written
  end

  test "a refused first scan creates no entrypoints.tsv" do
    delete_file ".roostr/entrypoints.tsv"
    scan_prints "# covers: http\nhttp\tGET /workshops\n" # a row of two columns

    scan

    assert_exit 1
    refute_file ".roostr/entrypoints.tsv"
  end
end
