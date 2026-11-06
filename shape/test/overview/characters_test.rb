class OverviewCharactersTest < AcceptanceTest
  def setup = atelier_under_git # 3 valid processes; each test adds refund-booking

  test "the overview writes U+202E and ESC in a description as <U+202E> and <U+001B>, so that they neither reorder nor colour the terminal" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ngroup: Bookings\ndescription: \"Refund\\u202E a cancelled\\e[31m booking\"\n"

    overview

    assert_success
    assert_stdout "Refund<U+202E> a cancelled<U+001B>[31m booking"
    refute_stdout "\e"
  end

  test "the overview writes an id holding U+202E, from the file's name, as refund<U+202E>redro" do
    write_file ".roostr/processes/refund\u{202E}redro.yml", "name: Refund a booking\ngroup: Bookings\ndescription: Refund a cancelled booking\n"

    overview

    assert_success
    assert_stdout "refund<U+202E>redro"
    refute_stdout "\u{202E}"
  end
end
