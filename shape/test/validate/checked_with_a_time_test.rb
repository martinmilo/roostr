class CheckedWithATimeTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, 7 entrypoints, every listed file present

  test "a checked date with a time, written YYYY-MM-DD HH:MM, is accepted" do
    write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ndescription: Move one of two attendees booked on the same place\nchecked: 2026-10-10 14:30\n"

    validate

    assert_success
    assert_stdout "No problems found"
  end

  ["2026-10-10T14:30", "2026-10-10 24:00", "2026-10-10 14:30:15", "2026-10-10 9:30"].each do |value|
    test "checked: #{value} is in neither form: a finding names the record and the key checked, and gives both forms, YYYY-MM-DD and YYYY-MM-DD HH:MM" do
      write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ndescription: Move one of two attendees booked on the same place\nchecked: #{value}\n"

      validate

      assert_exit 1
      assert_stdout "fix-double-booking.yml: checked"
      assert_stdout "YYYY-MM-DD HH:MM"
    end
  end
end
