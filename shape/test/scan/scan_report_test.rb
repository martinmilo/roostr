class ScanReportTest < AcceptanceTest
  def setup = workspace("atelier") # entrypoints.tsv covers http, job and mail and holds 7 entrypoints

  test "scan lists each added entrypoint after a plus sign, each removed one after a minus sign, then a blank line and the totals" do
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list and WaitingListJob, no longer finds BookingMailer#reminder

    scan

    assert_stdout_is expected("scan_report_with_waiting_list.txt")
  end

  test "a scan that finds exactly what entrypoints.tsv holds prints only 7 entrypoints, 0 added, 0 removed and leaves the file as it was" do
    scan_prints fixture("scans/atelier.txt") # the 7 entrypoints of Atelier, in another order

    scan

    assert_success
    assert_stdout_is "7 entrypoints, 0 added, 0 removed\n"
    assert_nothing_written
  end

  test "an entrypoint whose file changed is neither added nor removed, and its row is updated" do
    scan_prints fixture("scans/atelier-booking-moved.txt") # the same 7; POST /bookings is now in checkout_controller.rb

    scan

    assert_stdout_is "7 entrypoints, 0 added, 0 removed\n"
    assert_file ".roostr/entrypoints.tsv", includes: "http\tPOST /bookings\tapp/controllers/checkout_controller.rb\tBookings\n"
  end
end
