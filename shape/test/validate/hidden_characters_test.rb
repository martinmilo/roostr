class HiddenCharactersTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; refund-booking.yml is each test's own

  { "a zero-width space" => ["\\u200B", "U+200B"], "a right-to-left override" => ["\\u202E", "U+202E"],
    "a left-to-right isolate" => ["\\u2066", "U+2066"], "a tag character, an invisible A" => ["\\U000E0041", "U+E0041"],
    "ESC" => ["\\u001B", "U+001B"], "BEL" => ["\\u0007", "U+0007"], "a backspace" => ["\\u0008", "U+0008"],
    "DEL" => ["\\u007F", "U+007F"], "NEL, a C1 control" => ["\\u0085", "U+0085"], "a line separator" => ["\\u2028", "U+2028"],
    "a byte order mark inside the text" => ["\\uFEFF", "U+FEFF"],
    "a second variation selector after the one of ⚠️" => ["\\u26A0\\uFE0F\\uFE0F", "U+FE0F"] }.each do |said, (escape, code)|
    test "a record's description holding #{said}, #{code}, is a finding on description that names #{code}, the record's only finding: exit 1" do
      write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a#{escape} cancelled booking\"\n"

      validate

      assert_exit 1
      assert_stdout_is ".roostr/processes/refund-booking.yml: description: holds #{code}, a character that is invisible or changes how text shows: remove it\n"
    end
  end

  test "a description holding two such characters gets one finding that names both, in the order they first appear, each once" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Re\\u202Efund a\\u200B can\\u202Ecelled booking\"\n"

    validate

    assert_exit 1
    assert_stdout_is ".roostr/processes/refund-booking.yml: description: holds U+202E and U+200B, characters that are invisible or change how text shows: remove them\n"
  end

  test "in project.yml's second watch_out item, the finding is on watch_out.2" do
    write_file ".roostr/project.yml", "name: Atelier\nwatch_out:\n  - Prices are stored in cents.\n  - \"Refunds take\\u202E five days.\"\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/project.yml: watch_out.2: holds U+202E, a character that is invisible or changes how text shows: remove it\n"
  end

  test "U+202E written as itself in a plain value, not as an escape, is found as well" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a\u{202E} cancelled booking\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yml: description: holds U+202E, a character that is invisible or changes how text shows: remove it\n"
  end

  test "in try, which the map leaves out and roostr show prints, the character is found as well" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\ntry: \"Open /refunds\\u202E and refund AT-2041.\"\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/refund-booking.yml: try: holds U+202E, a character that is invisible or changes how text shows: remove it\n"
  end

  { "⚠️, a warning sign and its variation selector" => "\\u26A0\\uFE0F Refund a cancelled booking",
    "👩‍💻, two emoji joined by a zero-width joiner" => "Ask the \\U0001F469\\u200D\\U0001F4BB on call to refund a cancelled booking",
    "Persian with a zero-width non-joiner inside a word" => "\\u0645\\u06CC\\u200C\\u062E\\u0648\\u0627\\u0647\\u0645 a refund",
    "Hebrew with a right-to-left mark" => "Refund \\u05D4\\u05D6\\u05DE\\u05E0\\u05D4\\u200F AT-2041",
    "a soft hyphen" => "Re\\u00ADfund a cancelled booking", "a tab" => "Refund\\ta cancelled booking" }.each do |said, text|
    test "text written on purpose is not a finding: a description with #{said}" do
      write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"#{text}\"\n"

      validate

      assert_success
      assert_stdout_is "No problems found\n"
    end
  end

  test "a record holding U+202E is still readable: roostr show prints it, exit 0" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a\\u202E cancelled booking\"\n"

    show "refund-booking"

    assert_success
    assert_stdout "Refund a booking"
  end

  test "validate --json gives the finding as kind invalid, key description, line null, and a message that names U+202E" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a\\u202E cancelled booking\"\n"

    validate "--json"

    assert_exit 1
    assert_json "problems.1.kind", "invalid"
    assert_json "problems.1.key", "description"
    assert_json "problems.1.line", nil
    assert_json "problems.1.message", /U\+202E/
  end
end
