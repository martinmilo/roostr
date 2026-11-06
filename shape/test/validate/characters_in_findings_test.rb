class CharactersInFindingsTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; refund-booking.yml is each test's own

  test "a key holding BEL is written bell<U+0007> in its findings, so that no raw control character reaches the terminal" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n\"bell\\a\": rings\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yml: bell<U+0007>: holds U+0007, a character that is invisible or changes how text shows: remove it\n"
    refute_stdout "\a"
  end

  test "a key holding U+202E is written rl<U+202E>o in its findings, so that it does not reorder the line it is on" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n\"rl\\u202Eo\": turns\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yml: rl<U+202E>o: holds U+202E, a character that is invisible or changes how text shows: remove it\n"
    refute_stdout "\u{202E}"
  end

  test "a key holding a newline, which a value may hold, is written steps<U+000A>try in its finding, so that the finding stays on one line" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n\"steps\\ntry\": open the admin\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yml: steps<U+000A>try: "
    assert_equal 1, stdout.lines.size, "Expected one finding on one line.\n\n#{finished}"
  end
end
