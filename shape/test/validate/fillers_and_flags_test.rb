class FillersAndFlagsTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; refund-booking.yml is each test's own

  { "the Hangul filler U+3164" => ["\\u3164", "U+3164"], "the Hangul choseong filler U+115F" => ["\\u115F", "U+115F"],
    "the Hangul jungseong filler U+1160" => ["\\u1160", "U+1160"], "the halfwidth Hangul filler U+FFA0" => ["\\uFFA0", "U+FFA0"],
    "the Khmer vowel U+17B4" => ["\\u17B4", "U+17B4"], "the Khmer vowel U+17B5" => ["\\u17B5", "U+17B5"],
    "U+2065, unassigned and ignorable" => ["\\u2065", "U+2065"], "U+FFF0, unassigned and ignorable" => ["\\uFFF0", "U+FFF0"],
    "U+E0080, unassigned and ignorable" => ["\\U000E0080", "U+E0080"], "U+E01F0, unassigned and ignorable" => ["\\U000E01F0", "U+E01F0"] }.each do |said, (escape, code)|
    test "a record's description holding #{said}, which shows as nothing, is a finding on description that names #{code}: exit 1" do
      write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a#{escape} cancelled booking\"\n"

      validate

      assert_exit 1
      assert_stdout_is ".roostr/processes/refund-booking.yml: description: holds #{code}, a character that is invisible or changes how text shows: remove it\n"
    end
  end

  { "the flag of England" => "\\U0001F3F4\\U000E0067\\U000E0062\\U000E0065\\U000E006E\\U000E0067\\U000E007F",
    "the flag of Scotland" => "\\U0001F3F4\\U000E0067\\U000E0062\\U000E0073\\U000E0063\\U000E0074\\U000E007F",
    "the flag of Wales" => "\\U0001F3F4\\U000E0067\\U000E0062\\U000E0077\\U000E006C\\U000E0073\\U000E007F",
    "U+034F, the combining grapheme joiner" => "Re\\u034Ffund", "a Mongolian letter and its variation selector U+180B" => "\\u1820\\u180B",
    "U+2800, the braille pattern blank" => "Refund\\u2800a" }.each do |said, text|
    test "text written on purpose is not a finding: a description with #{said}" do
      write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"#{text} cancelled booking\"\n"

      validate

      assert_success
      assert_stdout_is "No problems found\n"
    end
  end

  test "a black flag followed by tag letters other than those of England, Scotland and Wales (usca, the flag of California) is a finding that names each tag character: tag letters spell text that does not show" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"\\U0001F3F4\\U000E0075\\U000E0073\\U000E0063\\U000E0061\\U000E007F Refund a cancelled booking\"\n"

    validate

    assert_exit 1
    assert_stdout_is ".roostr/processes/refund-booking.yml: description: holds U+E0075, U+E0073, U+E0063, U+E0061 and U+E007F, characters that are invisible or change how text shows: remove them\n"
  end
end
