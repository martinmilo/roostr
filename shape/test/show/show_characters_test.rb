class ShowCharactersTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; refund-booking.yml is each test's own

  test "show writes U+202E and ESC in a description as <U+202E> and <U+001B>, so that they neither reorder nor colour the terminal" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund\\u202E a cancelled\\e[31m booking\"\n"

    show "refund-booking"

    assert_success
    assert_stdout "Refund<U+202E> a cancelled<U+001B>[31m booking"
    refute_stdout "\e"
  end

  test "show writes tag characters as <U+E0049> and the like, so that an agent reading it sees them" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a cancelled booking\\U000E0049\\U000E0047\"\n"

    show "refund-booking"

    assert_success
    assert_stdout "Refund a cancelled booking<U+E0049><U+E0047>"
  end

  test "show prints what people write on purpose as it is: the zero-width joiner of 👩‍💻 and a Hebrew right-to-left mark" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Ask the \\U0001F469\\u200D\\U0001F4BB on call to refund \\u05D4\\u05D6\\u05DE\\u05E0\\u05D4\\u200F AT-2041\"\n"

    show "refund-booking"

    assert_success
    assert_stdout "Ask the \u{1F469}\u{200D}\u{1F4BB} on call to refund \u{05D4}\u{05D6}\u{05DE}\u{05E0}\u{05D4}\u{200F} AT-2041"
  end

  test "show --json gives the description as it is, U+202E and ESC included" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund\\u202E a cancelled\\e[31m booking\"\n"

    show "refund-booking", "--json"

    assert_success
    assert_json "process.description", "Refund\u{202E} a cancelled\e[31m booking"
  end
end
