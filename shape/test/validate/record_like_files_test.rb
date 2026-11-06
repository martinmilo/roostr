class RecordLikeFilesTest < AcceptanceTest
  def setup = atelier_under_git # the processes are book-workshop, cancel-booking and remind-attendees; validate finds nothing

  test "refund-booking.yaml in .roostr/processes/ is a finding that names it, says a record is a file named <id>.yml directly in .roostr/processes/, and names refund-booking.yml to rename it to" do
    write_file ".roostr/processes/refund-booking.yaml", fixture("processes/refund-booking.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yaml: "
    assert_stdout "<id>.yml directly in .roostr/processes/"
    assert_stdout ".roostr/processes/refund-booking.yml"
  end

  test "Refund-Booking.YML, its extension in capitals, is a finding that names Refund-Booking.yml to rename it to" do
    write_file ".roostr/processes/Refund-Booking.YML", fixture("processes/refund-booking.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/Refund-Booking.YML: "
    assert_stdout ".roostr/processes/Refund-Booking.yml"
  end

  test "bookings/refund-booking.yml, in a folder below .roostr/processes/, is a finding that names .roostr/processes/refund-booking.yml to move it to" do
    write_file ".roostr/processes/bookings/refund-booking.yml", fixture("processes/refund-booking.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/bookings/refund-booking.yml: "
    assert_stdout ".roostr/processes/refund-booking.yml"
  end

  test "cancel-booking.yaml beside the record cancel-booking.yml is a finding that names no file to rename it to, since that name is taken" do
    write_file ".roostr/processes/cancel-booking.yaml", fixture("processes/refund-booking.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/cancel-booking.yaml: "
    refute_stdout ".roostr/processes/cancel-booking.yml"
  end

  test "other files and hidden ones are ignored, README.md, notes.txt, .refund-booking.yaml and .drafts/refund-booking.yml: No problems found" do
    write_file ".roostr/processes/README.md", "One file per process, named after its id.\n"
    write_file ".roostr/processes/notes.txt", "Describe the waiting list next.\n"
    write_file ".roostr/processes/.refund-booking.yaml", fixture("processes/refund-booking.yml")
    write_file ".roostr/processes/.drafts/refund-booking.yml", fixture("processes/refund-booking.yml")

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "check --base HEAD reports an uncommitted refund-booking.yaml under VALIDATE, exit 1" do
    write_file ".roostr/processes/refund-booking.yaml", fixture("processes/refund-booking.yml")

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "VALIDATE\n"
    assert_stdout ".roostr/processes/refund-booking.yaml: "
  end

  test "the overview is unaffected by refund-booking.yaml: it exits 0 and does not mention it" do
    write_file ".roostr/processes/refund-booking.yaml", fixture("processes/refund-booking.yml")

    overview

    assert_success
    refute_stdout "refund-booking"
  end
end
