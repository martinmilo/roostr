class SessionMapCharactersTest < AcceptanceTest
  def setup = atelier_under_git

  test "characters that do not show are dropped from the map: a soft hyphen, a zero-width space, DEL, U+202E, NEL, tag characters, a byte order mark, U+2066, ESC and BEL" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking-with-hidden-characters.yml")

    session_start

    assert_map "\n- refund-booking: Refund a cancelled booking to the original card [app/models/booking.rb]\n"
  end

  test "in the map a tab is a space, and U+2028 and U+2029, line breaks, are a space too, so that the item stays on one line" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking-with-a-tab-and-line-separators.yml")

    session_start

    assert_map "\n- refund-booking: Refund a cancelled booking to the original card within five days. [app/models/booking.rb]\n"
  end

  test "the map drops the characters that show as nothing outside the format characters as well: the Hangul fillers U+3164, U+115F, U+1160 and U+FFA0, the Khmer vowels U+17B4 and U+17B5, U+034F and a Mongolian variation selector; U+2800, the braille pattern blank, stays" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking-with-fillers.yml")

    session_start

    assert_map "\n- refund-booking: Refund a\u{2800}cancelled booking to the original card [app/models/booking.rb]\n"
  end

  test "a record whose file name holds U+202E, or a zero-width space, is left out of the map: the id it would print is one roostr show does not know. The map is Atelier's" do
    write_file ".roostr/processes/refund\u{202E}redro.yml", fixture("processes/refund-booking.yml")
    write_file ".roostr/processes/refund\u{200B}-booking.yml", fixture("processes/refund-booking.yml")

    session_start

    assert_map_is expected("session_map_atelier.txt")
  end

  test "the flag of Scotland is U+1F3F4 alone in the map, a black flag: its tag characters are dropped, though validate allows them" do
    write_file ".roostr/project.yml", "name: Atelier\nwatch_out:\n  - \"Glasgow workshops (\\U0001F3F4\\U000E0067\\U000E0062\\U000E0073\\U000E0063\\U000E0074\\U000E007F) close at 17:00.\"\n"

    session_start

    assert_map "\nWatch out:\n- Glasgow workshops (\u{1F3F4}) close at 17:00.\n"
  end

  test "the map drops what validate allows as well: the variation selector of ⚠️ and the zero-width joiner of 👩‍💻" do
    write_file ".roostr/project.yml", fixture("project/watch-out-with-emoji.yml")

    session_start

    assert_map "\nWatch out:\n- \u{26A0} The kiln room door locks itself at 22:00.\n- Ask the \u{1F469}\u{1F4BB} on call before you change the payment settings.\n"
  end

  test "in project.yml, values that are empty once the characters that do not show are dropped count as absent: no Stack, Description or Watch out, no note after an operation, and an operation with no name or no command left out" do
    write_file ".roostr/project.yml", fixture("project/values-of-hidden-characters-only.yml")

    session_start

    refute_map "Stack:"
    refute_map "Description:"
    refute_map "Watch out:"
    assert_map "prints the overview and recent changes.\n\nOperations:\n- start: `bin/dev`\n\nProcesses by group"
  end
end
